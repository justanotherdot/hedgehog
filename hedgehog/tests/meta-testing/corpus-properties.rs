//! Practical examples using curated corpus data
//!
//! These examples demonstrate how the curated corpus collections provide
//! realistic test data for common testing scenarios, especially for:
//! - Unicode/internationalization testing 
//! - Text processing with realistic inputs
//! - Domain-specific test data generation

use hedgehog::*;
use hedgehog::corpus;

/// Example: Testing a web application's input validation with realistic data
/// This shows how corpus data provides better test coverage than purely random strings
pub fn test_web_input_validation_with_corpus() {
    // Simulate a web form that accepts animal names, colors, and user comments
    fn validate_animal_form(animal: &str, color: &str, comment: &str) -> Result<String, &'static str> {
        if animal.is_empty() || animal.len() > 50 {
            return Err("Invalid animal name");
        }
        if color.is_empty() || color.len() > 20 {
            return Err("Invalid color");
        }
        if comment.len() > 500 {
            return Err("Comment too long");
        }
        Ok(format!("Registered: {} {} - {}", color, animal, comment))
    }

    let prop = for_all(
        Gen::<(String, String, String)>::tuple_of(
            Gen::<&str>::animal().map(|s| s.to_string()),
            Gen::<&str>::colour().map(|s| s.to_string()), 
            Gen::<&str>::glass().map(|s| {
                // Use glass text as comments to test unicode handling
                if s.len() > 400 { &s[..400] } else { s }
            }).map(|s| s.to_string())
        ),
        |(animal, color, comment)| {
            // Property: Valid corpus data should always pass basic validation
            match validate_animal_form(animal, color, comment) {
                Ok(result) => {
                    result.contains(animal) && 
                    result.contains(color) &&
                    result.starts_with("Registered:")
                }
                Err(_) => {
                    // Should only fail if comment is too long
                    comment.len() > 500
                }
            }
        }
    );
    
    let config = Config::default().with_tests(50);
    match prop.run(&config) {
        TestResult::Pass { .. } => println!("✓ Web input validation with realistic corpus data passed"),
        result => panic!("Web input validation test failed: {:?}", result),
    }
}

/// Example: Testing internationalization/localization with the glass collection  
/// The glass collection contains "I can eat glass" in ~100+ languages and scripts
pub fn test_i18n_text_processing_with_glass() {
    // Simulate text processing functions that need to handle international text
    fn process_international_text(text: &str) -> Result<(usize, usize, bool), &'static str> {
        if text.is_empty() {
            return Err("Empty text");
        }
        
        let char_count = text.chars().count();
        let byte_count = text.bytes().len();
        let has_non_ascii = !text.is_ascii();
        
        // Basic processing: count characters and bytes, detect non-ASCII
        Ok((char_count, byte_count, has_non_ascii))
    }
    
    fn normalize_for_search(text: &str) -> String {
        // Simulate search normalization (common i18n requirement)
        text.to_lowercase().chars().filter(|c| c.is_alphanumeric() || c.is_whitespace()).collect()
    }

    let prop = for_all(
        Gen::<&str>::glass(),
        |&text| {
            // Property: Text processing should handle all languages gracefully
            match process_international_text(text) {
                Ok((char_count, byte_count, has_non_ascii)) => {
                    // Characters <= bytes (multi-byte unicode)
                    char_count <= byte_count &&
                    char_count > 0 &&
                    
                    // Search normalization should not crash
                    normalize_for_search(text).len() <= text.len() &&
                    
                    // Most glass entries have non-ASCII text (testing unicode paths)
                    (has_non_ascii || text.is_ascii()) // Either is valid
                }
                Err(_) => false, // Should never fail with our corpus data
            }
        }
    );
    
    let config = Config::default().with_tests(30);
    match prop.run(&config) {
        TestResult::Pass { .. } => println!("✓ I18n text processing with glass collection passed"),  
        result => panic!("I18n text processing test failed: {:?}", result),
    }
}

/// Example: Testing a game inventory system with realistic item names
pub fn test_game_inventory_with_corpus() {
    #[derive(Debug, Clone)]
    struct Item {
        name: String,
        category: String,
        value: u32,
    }
    
    fn create_inventory_item(name: &str, category: &str, base_value: u32) -> Item {
        let value = if name.len() > 8 { base_value * 2 } else { base_value };
        Item {
            name: name.to_string(),
            category: category.to_string(), 
            value,
        }
    }
    
    fn inventory_system_constraints(items: &[Item]) -> bool {
        // Game constraints: no duplicate names, reasonable total value
        let mut names = std::collections::HashSet::new();
        let mut total_value = 0u64;
        
        for item in items {
            if !names.insert(&item.name) {
                return false; // Duplicate name
            }
            total_value += item.value as u64;
            if total_value > 1_000_000 {
                return false; // Economy balance
            }
        }
        true
    }

    let prop = for_all(
        Gen::<Vec<(String, String, u32)>>::vec_of(
            Gen::<(String, String, u32)>::tuple_of(
                Gen::frequency(vec![
                    WeightedChoice::new(3, Gen::<&str>::animal().map(|s| s.to_string())),
                    WeightedChoice::new(2, Gen::<&str>::fruit().map(|s| s.to_string())),
                    WeightedChoice::new(1, Gen::<&str>::muppet().map(|s| s.to_string())),
                ]).unwrap(),
                Gen::<&str>::one_of_slice(&["weapon", "food", "treasure", "tool"]).map(|s| s.to_string()),
                Gen::<u32>::from_range(Range::new(1, 1000)),
            )
        ).with_range(Range::new(0, 20)),
        |item_specs| {
            let items: Vec<Item> = item_specs.iter()
                .map(|(name, category, value)| create_inventory_item(name, category, *value))
                .collect();
                
            // Property: Realistic item names should create valid inventory
            inventory_system_constraints(&items) &&
            items.iter().all(|item| {
                !item.name.is_empty() && 
                !item.category.is_empty() &&
                item.value > 0
            })
        }
    );
    
    let config = Config::default().with_tests(25);
    match prop.run(&config) {
        TestResult::Pass { .. } => println!("✓ Game inventory system with realistic names passed"),
        result => panic!("Game inventory system test failed: {:?}", result),
    }
}

/// Example: Testing a search/autocomplete system with metasyntactic variables
/// This shows how the metasyntactic corpus helps test code-related features
pub fn test_code_autocomplete_with_metasyntactic() {
    fn autocomplete_suggestions(prefix: &str, candidates: &[&str]) -> Vec<String> {
        candidates.iter()
            .filter(|candidate| candidate.starts_with(prefix))
            .map(|s| s.to_string())
            .collect()
    }
    
    fn is_valid_variable_name(name: &str) -> bool {
        !name.is_empty() && 
        name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') &&
        name.chars().next().map_or(false, |c| c.is_ascii_alphabetic() || c == '_')
    }

    let prop = for_all(
        Gen::<&str>::metasyntactic(),
        |&var_name| {
            // Property: All metasyntactic variables should be valid identifiers
            let is_valid = is_valid_variable_name(var_name);
            
            // Test autocomplete with various prefixes
            let suggestions_1 = autocomplete_suggestions(&var_name[..1], corpus::METASYNTACTIC);
            let suggestions_2 = autocomplete_suggestions(&var_name[..2.min(var_name.len())], corpus::METASYNTACTIC);
            let suggestions_full = autocomplete_suggestions(var_name, corpus::METASYNTACTIC);
            
            is_valid &&
            !suggestions_1.is_empty() && // Single char should match something
            suggestions_2.len() <= suggestions_1.len() && // More specific = fewer matches
            suggestions_full.contains(&var_name.to_string()) // Full name matches itself
        }
    );
    
    let config = Config::default().with_tests(20);
    match prop.run(&config) {
        TestResult::Pass { .. } => println!("✓ Code autocomplete with metasyntactic variables passed"),
        result => panic!("Code autocomplete test failed: {:?}", result),
    }
}

/// Example: Database/search testing with realistic corpus combinations
/// Shows how multiple corpus collections can create realistic test scenarios
pub fn test_database_search_with_mixed_corpus() {
    #[derive(Debug, Clone)]
    struct SearchRecord {
        id: u32,
        title: String,
        tags: Vec<String>,
        content: String,
    }
    
    fn search_records(records: &[SearchRecord], query: &str) -> Vec<u32> {
        records.iter()
            .filter(|record| {
                record.title.to_lowercase().contains(&query.to_lowercase()) ||
                record.content.to_lowercase().contains(&query.to_lowercase()) ||
                record.tags.iter().any(|tag| tag.to_lowercase().contains(&query.to_lowercase()))
            })
            .map(|record| record.id)
            .collect()
    }

    let prop = for_all(
        Gen::<Vec<SearchRecord>>::vec_of(
            Gen::<SearchRecord>::new(|size, seed| {
                let (id_seed, rest) = seed.split();
                let (title_seed, rest) = rest.split();
                let (tags_seed, content_seed) = rest.split();
                
                let id = id_seed.next_bounded(10000).0 as u32;
                let title = Gen::<String>::frequency(vec![
                    WeightedChoice::new(2, Gen::<&str>::animal().map(|s| format!("About {}", s))),
                    WeightedChoice::new(1, Gen::<&str>::muppet().map(|s| format!("{} Adventures", s))),
                    WeightedChoice::new(1, Gen::<&str>::fruit().map(|s| format!("Growing {}", s))),
                ]).unwrap().generate(size, title_seed).value;
                
                let tags = Gen::<Vec<String>>::vec_of(
                    Gen::frequency(vec![
                        WeightedChoice::new(3, Gen::<&str>::colour().map(|s| s.to_string())),
                        WeightedChoice::new(2, Gen::<&str>::weather().map(|s| s.to_string())),
                        WeightedChoice::new(1, Gen::<&str>::cooking().map(|s| s.to_string())),
                    ]).unwrap()
                ).with_range(Range::new(1, 5)).generate(size, tags_seed).value;
                
                let content = Gen::<&str>::water().map(|w| format!("This is about {}", w))
                    .generate(size, content_seed).value;
                    
                Tree::singleton(SearchRecord { id, title, tags, content })
            })
        ).with_range(Range::new(1, 10)),
        |records| {
            // Property: Search should find records that contain query terms
            let all_animals: Vec<&str> = corpus::ANIMALS.iter().take(5).copied().collect();
            
            all_animals.iter().all(|&animal| {
                let results = search_records(records, animal);
                let expected_matches = records.iter()
                    .filter(|r| r.title.to_lowercase().contains(&animal.to_lowercase()))
                    .count();
                    
                results.len() == expected_matches &&
                results.iter().all(|&id| records.iter().any(|r| r.id == id))
            })
        }
    );
    
    let config = Config::default().with_tests(15);
    match prop.run(&config) {
        TestResult::Pass { .. } => println!("✓ Database search with mixed corpus data passed"),
        result => panic!("Database search test failed: {:?}", result),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_practical_corpus_examples() {
        test_web_input_validation_with_corpus();
        test_i18n_text_processing_with_glass();
        test_game_inventory_with_corpus();
        test_code_autocomplete_with_metasyntactic();
        test_database_search_with_mixed_corpus();
    }
}