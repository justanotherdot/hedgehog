use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use syn::{Item, ItemFn};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct PropertyTest {
    pub name: String,
    pub file_path: PathBuf,
    pub line_number: usize,
    pub is_async: bool,
}

pub struct TestDiscovery {
    root_dir: PathBuf,
}

impl TestDiscovery {
    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }

    pub fn discover_properties(&self) -> Result<Vec<PropertyTest>> {
        let mut properties = Vec::new();

        for entry in WalkDir::new(&self.root_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
        {
            let file_properties = self.scan_file(entry.path())?;
            properties.extend(file_properties);
        }

        Ok(properties)
    }

    fn scan_file(&self, path: &Path) -> Result<Vec<PropertyTest>> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        let parsed = syn::parse_file(&content)
            .with_context(|| format!("Failed to parse file: {}", path.display()))?;

        let mut properties = Vec::new();

        for item in &parsed.items {
            if let Some(prop) = self.extract_property_test(item, path)? {
                properties.push(prop);
            }
        }

        Ok(properties)
    }

    fn extract_property_test(&self, item: &Item, file_path: &Path) -> Result<Option<PropertyTest>> {
        match item {
            Item::Fn(item_fn) => {
                if self.is_property_test(item_fn) {
                    Ok(Some(PropertyTest {
                        name: item_fn.sig.ident.to_string(),
                        file_path: file_path.to_path_buf(),
                        line_number: self.get_line_number(item_fn),
                        is_async: item_fn.sig.asyncness.is_some(),
                    }))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    fn is_property_test(&self, item_fn: &ItemFn) -> bool {
        // Check if function has #[test] attribute
        let has_test_attr = item_fn
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("test"));

        if !has_test_attr {
            return false;
        }

        // Check if function body contains hedgehog constructs
        let body_str = quote::quote!(#item_fn).to_string();

        // Look for hedgehog-specific patterns
        body_str.contains("for_all")
            || body_str.contains("Property")
            || body_str.contains("Gen::")
            || body_str.contains("TestResult::")
    }

    fn get_line_number(&self, _item_fn: &ItemFn) -> usize {
        // syn doesn't provide line numbers directly in all cases
        // This is a simplified implementation
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_discover_properties_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        let discovery = TestDiscovery::new(temp_dir.path().to_path_buf());

        let properties = discovery.discover_properties().unwrap();
        assert!(properties.is_empty());
    }

    #[test]
    fn test_discover_properties_with_hedgehog_test() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        fs::write(
            &test_file,
            r#"
use hedgehog::*;

#[test]
fn test_property_addition() {
    let prop = for_all(Gen::int_range(0, 100), |&x| x + 0 == x);
    assert!(prop.run(&Config::default()).is_pass());
}

#[test]
fn regular_test() {
    assert_eq!(2 + 2, 4);
}
"#,
        )
        .unwrap();

        let discovery = TestDiscovery::new(temp_dir.path().to_path_buf());
        let properties = discovery.discover_properties().unwrap();

        assert_eq!(properties.len(), 1);
        assert_eq!(properties[0].name, "test_property_addition");
        assert_eq!(properties[0].file_path, test_file);
        assert!(!properties[0].is_async);
    }

    #[test]
    fn test_discover_properties_with_async() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("async_test.rs");

        fs::write(
            &test_file,
            r#"
use hedgehog::*;

#[test]
async fn async_property_test() {
    let prop = for_all(Gen::int_range(0, 100), |&x| x >= 0);
    assert!(prop.run(&Config::default()).is_pass());
}
"#,
        )
        .unwrap();

        let discovery = TestDiscovery::new(temp_dir.path().to_path_buf());
        let properties = discovery.discover_properties().unwrap();

        assert_eq!(properties.len(), 1);
        assert_eq!(properties[0].name, "async_property_test");
        assert!(properties[0].is_async);
    }

    #[test]
    fn test_discover_properties_multiple_files() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(
            temp_dir.path().join("test1.rs"),
            r#"
use hedgehog::*;

#[test]
fn prop_test_1() {
    let prop = for_all(Gen::bool(), |&b| b || !b);
    assert!(prop.run(&Config::default()).is_pass());
}
"#,
        )
        .unwrap();

        fs::write(
            temp_dir.path().join("test2.rs"),
            r#"
use hedgehog::*;

#[test]
fn prop_test_2() {
    let gen = Gen::int_range(1, 10);
    let prop = for_all(gen, |&x| x > 0 && x <= 10);
    assert!(prop.run(&Config::default()).is_pass());
}
"#,
        )
        .unwrap();

        let discovery = TestDiscovery::new(temp_dir.path().to_path_buf());
        let properties = discovery.discover_properties().unwrap();

        assert_eq!(properties.len(), 2);

        let names: Vec<_> = properties.iter().map(|p| &p.name).collect();
        assert!(names.contains(&&"prop_test_1".to_string()));
        assert!(names.contains(&&"prop_test_2".to_string()));
    }

    #[test]
    fn test_non_property_test_ignored() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("regular_test.rs");

        fs::write(
            &test_file,
            r#"
#[test]
fn regular_unit_test() {
    assert_eq!(1 + 1, 2);
}

fn helper_function() {
    // This should be ignored
}
"#,
        )
        .unwrap();

        let discovery = TestDiscovery::new(temp_dir.path().to_path_buf());
        let properties = discovery.discover_properties().unwrap();

        assert!(properties.is_empty());
    }
}
