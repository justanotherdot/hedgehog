//! Tree rendering functionality for debugging and visualization.

use super::Tree;

impl<T> Tree<T>
where
    T: std::fmt::Display,
{
    /// Render the tree structure as a string for debugging.
    pub fn render(&self) -> String {
        let mut result = String::new();
        self.render_recursive(&mut result, "", true);
        result
    }

    fn render_recursive(&self, result: &mut String, prefix: &str, is_last: bool) {
        result.push_str(prefix);
        if is_last {
            result.push_str("└── ");
        } else {
            result.push_str("├── ");
        }
        result.push_str(&format!("{}\n", self.value));

        let child_prefix = if is_last {
            format!("{prefix}    ")
        } else {
            format!("{prefix}│   ")
        };

        for (i, child) in self.children.iter().enumerate() {
            let child_is_last = i == self.children.len() - 1;
            child.render_recursive(result, &child_prefix, child_is_last);
        }
    }

    /// Render the tree structure compactly, showing only values.
    pub fn render_compact(&self) -> String {
        if self.children.is_empty() {
            format!("{}", self.value)
        } else {
            let children_str: Vec<String> = self
                .children
                .iter()
                .map(|child| child.render_compact())
                .collect();
            format!("{}[{}]", self.value, children_str.join(", "))
        }
    }

    /// Render the tree showing only the shrink sequence.
    pub fn render_shrinks(&self) -> String {
        let shrinks = self.shrinks();
        if shrinks.is_empty() {
            format!("{} (no shrinks)", self.value)
        } else {
            let shrink_strs: Vec<String> = shrinks.iter().map(|v| format!("{v}")).collect();
            format!("{} → [{}]", self.value, shrink_strs.join(", "))
        }
    }

    /// Render the tree with numbered shrinks for easier debugging.
    pub fn render_numbered(&self) -> String {
        let shrinks = self.shrinks();
        if shrinks.is_empty() {
            format!("{} (no shrinks)", self.value)
        } else {
            let mut result = format!("Original: {}\nShrinks:\n", self.value);
            for (i, shrink) in shrinks.iter().enumerate() {
                result.push_str(&format!("  {}: {}\n", i + 1, shrink));
            }
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Tree;
    use crate::data::{Seed, Size};
    use crate::gen::Gen;

    #[test]
    fn test_tree_rendering() {
        let tree = Tree::with_children(
            10,
            vec![
                Tree::with_children(5, vec![Tree::singleton(2)]),
                Tree::singleton(0),
            ],
        );

        // Test full rendering
        let rendered = tree.render();
        assert!(rendered.contains("└── 10"));
        assert!(rendered.contains("├── 5"));
        assert!(rendered.contains("└── 0"));
        assert!(rendered.contains("└── 2"));

        // Test compact rendering
        let compact = tree.render_compact();
        assert_eq!(compact, "10[5[2], 0]");

        // Test shrink rendering
        let shrinks = tree.render_shrinks();
        assert_eq!(shrinks, "10 → [5, 0, 2]");

        // Test singleton rendering
        let singleton = Tree::singleton(42);
        assert_eq!(singleton.render_compact(), "42");
        assert_eq!(singleton.render_shrinks(), "42 (no shrinks)");
    }

    #[test]
    fn test_numbered_rendering() {
        let tree = Tree::with_children(100, vec![Tree::singleton(50), Tree::singleton(0)]);

        let numbered = tree.render_numbered();
        assert!(numbered.contains("Original: 100"));
        assert!(numbered.contains("1: 50"));
        assert!(numbered.contains("2: 0"));

        // Test singleton
        let singleton = Tree::singleton(42);
        let numbered_single = singleton.render_numbered();
        assert!(numbered_single.contains("42 (no shrinks)"));
    }

    // Snapshot tests for tree rendering output
    #[test]
    fn snapshot_integer_tree_rendering() {
        let gen = Gen::int_range(1, 20);
        let seed = Seed::from_u64(42);
        let tree = gen.generate(Size::new(10), seed);

        // Snapshot tests using archetype
        archetype::snap("integer_tree_render", tree.render());
        archetype::snap("integer_tree_render_compact", tree.render_compact());
        archetype::snap("integer_tree_render_shrinks", tree.render_shrinks());
        archetype::snap("integer_tree_render_numbered", tree.render_numbered());
    }

    #[test]
    fn snapshot_string_tree_rendering() {
        let gen = Gen::<String>::ascii_alpha();
        let seed = Seed::from_u64(1);
        let tree = gen.generate(Size::new(4), seed);

        archetype::snap("string_tree_render", tree.render());
        archetype::snap("string_tree_render_compact", tree.render_compact());
        archetype::snap("string_tree_render_shrinks", tree.render_shrinks());
        archetype::snap("string_tree_render_numbered", tree.render_numbered());
    }

    #[test]
    fn snapshot_boolean_tree_rendering() {
        let gen = Gen::bool();
        let seed = Seed::from_u64(123);
        let tree = gen.generate(Size::new(10), seed);

        archetype::snap("boolean_tree_render_compact", tree.render_compact());
        archetype::snap("boolean_tree_render_shrinks", tree.render_shrinks());
    }

    #[test]
    fn snapshot_float_tree_rendering() {
        let gen = Gen::f64_range(-2.0, 2.0);
        let seed = Seed::from_u64(789);
        let tree = gen.generate(Size::new(10), seed);

        archetype::snap("float_tree_render_compact", tree.render_compact());
        archetype::snap("float_tree_render_shrinks", tree.render_shrinks());
    }

    #[test]
    fn snapshot_character_tree_rendering() {
        let gen = Gen::<char>::ascii_alphanumeric();
        let seed = Seed::from_u64(456);
        let tree = gen.generate(Size::new(10), seed);

        archetype::snap("character_tree_render_compact", tree.render_compact());
        archetype::snap("character_tree_render_shrinks", tree.render_shrinks());
    }
}
