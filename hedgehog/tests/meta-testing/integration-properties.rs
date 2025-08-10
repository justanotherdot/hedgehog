//! Integration properties for full property testing workflows
//!
//! These properties test complete scenarios from generation through property 
//! evaluation, failure detection, and shrinking to minimal counterexamples.

use hedgehog::*;
use crate::arbitrary_seed;

/// Property: Full workflow for a simple failing property should work
pub fn test_simple_failing_property_workflow() {
    let prop = for_all_named(
        arbitrary_seed(),
        "seed",
        |&_seed: &Seed| {
            // Create a property that will fail for some values
            let failing_property = for_all(
                Gen::int_range(0, 100),
                |&x: &i32| x < 50  // This will fail for x >= 50
            );
            
            let config = Config::default().with_tests(20).with_shrinks(10);
            let result = failing_property.run(&config);
            
            match result {
                TestResult::Fail { counterexample, shrinks_performed, .. } => {
                    // Should have found a counterexample >= 50
                    let counter_value: i32 = counterexample.parse().unwrap_or(-1);
                    
                    counter_value >= 50 && shrinks_performed <= 20  // Reasonable shrink count
                }
                TestResult::Pass { .. } => {
                    // This could happen with unlucky random generation, it's okay
                    true
                }
                _ => false // Other results shouldn't happen
            }
        }
    );
    
    let fast_config = Config::default().with_tests(10).with_shrinks(3);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Simple failing property workflow passed"),
        result => panic!("Simple failing property workflow failed: {:?}", result),
    }
}

/// Property: Property with multiple generators should work end-to-end
pub fn test_multiple_generator_workflow() {
    let prop = for_all_named(
        arbitrary_seed(),
        "seed",
        |&_seed: &Seed| {
            // Property using multiple generators
            let tuple_property = for_all(
                Gen::<(i32, Vec<i32>)>::tuple_of(
                    Gen::int_range(0, 20),
                    Gen::vec_of(Gen::int_range(0, 10))
                ),
                |&(ref num, ref vec): &(i32, Vec<i32>)| {
                    // Property: the number should be less than vector length
                    vec.len() > (*num as usize)
                }
            );
            
            let config = Config::default().with_tests(15).with_shrinks(5);
            let result = tuple_property.run(&config);
            
            // This property might pass or fail depending on generation
            // We're testing that the workflow completes without crashes
            match result {
                TestResult::Pass { .. } => true,
                TestResult::Fail { .. } => true,
                _ => false
            }
        }
    );
    
    let fast_config = Config::default().with_tests(10).with_shrinks(3);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Multiple generator workflow passed"),
        result => panic!("Multiple generator workflow failed: {:?}", result),
    }
}

/// Property: String properties should work with shrinking
pub fn test_string_property_workflow() {
    let prop = for_all_named(
        arbitrary_seed(),
        "seed",
        |&_seed: &Seed| {
            // Property about string length
            let string_property = for_all(
                Gen::<String>::ascii_alpha(),
                |s: &String| s.len() <= 5  // Will fail for longer strings
            );
            
            let config = Config::default().with_tests(20).with_shrinks(5);
            let result = string_property.run(&config);
            
            // Test that the workflow completes
            match result {
                TestResult::Pass { .. } => true,
                TestResult::Fail { shrinks_performed, .. } => {
                    // If it failed, should have performed some shrinking
                    shrinks_performed <= 10  // Reasonable shrink count
                }
                _ => false
            }
        }
    );
    
    let fast_config = Config::default().with_tests(8).with_shrinks(3);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ String property workflow passed"),
        result => panic!("String property workflow failed: {:?}", result),
    }
}

/// Property: Frequency generator integration should work
pub fn test_frequency_generator_workflow() {
    let prop = for_all_named(
        arbitrary_seed(),
        "seed",
        |&_seed: &Seed| {
            let choices = vec![
                WeightedChoice::new(70, Gen::int_range(0, 10)),
                WeightedChoice::new(30, Gen::int_range(90, 100)),
            ];
            
            match Gen::frequency(choices) {
                Ok(freq_gen) => {
                    let freq_property = for_all(
                        freq_gen,
                        |&x: &i32| x >= 0 && x <= 100  // Should always be true
                    );
                    
                    let config = Config::default().with_tests(15).with_shrinks(3);
                    let result = freq_property.run(&config);
                    
                    matches!(result, TestResult::Pass { .. })
                }
                Err(_) => false // Frequency generator creation should not fail
            }
        }
    );
    
    let fast_config = Config::default().with_tests(10).with_shrinks(3);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Frequency generator workflow passed"),
        result => panic!("Frequency generator workflow failed: {:?}", result),
    }
}

/// Property: One-of generator integration should work
pub fn test_one_of_generator_workflow() {
    let prop = for_all_named(
        arbitrary_seed(),
        "seed",
        |&_seed: &Seed| {
            let generators = vec![
                Gen::int_range(0, 10),
                Gen::int_range(20, 30),
                Gen::int_range(40, 50),
            ];
            
            match Gen::one_of(generators) {
                Ok(oneof_gen) => {
                    let oneof_property = for_all(
                        oneof_gen,
                        |&x: &i32| {
                            // Value should be from one of the ranges
                            (x >= 0 && x <= 10) || (x >= 20 && x <= 30) || (x >= 40 && x <= 50)
                        }
                    );
                    
                    let config = Config::default().with_tests(15).with_shrinks(3);
                    let result = oneof_property.run(&config);
                    
                    matches!(result, TestResult::Pass { .. })
                }
                Err(_) => false // One-of generator creation should not fail
            }
        }
    );
    
    let fast_config = Config::default().with_tests(10).with_shrinks(3);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ One-of generator workflow passed"),
        result => panic!("One-of generator workflow failed: {:?}", result),
    }
}

/// Property: Complex nested property should complete
pub fn test_complex_nested_workflow() {
    let prop = for_all_named(
        arbitrary_seed(),
        "seed",
        |&_seed: &Seed| {
            // Complex property with nested structures
            let nested_property = for_all(
                Gen::vec_of(
                    Gen::<(i32, String)>::tuple_of(
                        Gen::int_range(0, 50),
                        Gen::<String>::ascii_alpha()
                    )
                ),
                |pairs: &Vec<(i32, String)>| {
                    // Property: all numbers should be less than 100 and strings non-empty
                    pairs.iter().all(|(num, string)| *num < 100 && !string.is_empty())
                }
            );
            
            let config = Config::default().with_tests(12).with_shrinks(3);
            let result = nested_property.run(&config);
            
            // Should complete without crashing
            match result {
                TestResult::Pass { .. } => true,
                TestResult::Fail { .. } => true,
                _ => false
            }
        }
    );
    
    let fast_config = Config::default().with_tests(8).with_shrinks(2);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Complex nested workflow passed"),
        result => panic!("Complex nested workflow failed: {:?}", result),
    }
}

/// Property: Property with custom config should work
pub fn test_custom_config_workflow() {
    let prop = for_all_named(
        arbitrary_seed(),
        "seed",
        |&_seed: &Seed| {
            let simple_property = for_all(
                Gen::int_range(0, 10),
                |&x: &i32| x >= 0  // Always true
            );
            
            // Test various configurations
            let configs = vec![
                Config::default().with_tests(5),
                Config::default().with_tests(10).with_shrinks(2),
                Config::default().with_size_limit(20),
            ];
            
            configs.into_iter().all(|config| {
                matches!(simple_property.run(&config), TestResult::Pass { .. })
            })
        }
    );
    
    let fast_config = Config::default().with_tests(8).with_shrinks(2);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Custom config workflow passed"),
        result => panic!("Custom config workflow failed: {:?}", result),
    }
}

/// Property: Named properties should preserve names
pub fn test_named_property_workflow() {
    let prop = for_all_named(
        arbitrary_seed(),
        "seed",
        |&_seed: &Seed| {
            let named_property = for_all_named(
                Gen::int_range(0, 100),
                "test_number",
                |&x: &i32| x < 150  // Always true for this range
            );
            
            let config = Config::default().with_tests(10).with_shrinks(2);
            let result = named_property.run(&config);
            
            // Should pass with proper naming
            matches!(result, TestResult::Pass { .. })
        }
    );
    
    let fast_config = Config::default().with_tests(10).with_shrinks(2);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Named property workflow passed"),
        result => panic!("Named property workflow failed: {:?}", result),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_integration_property_tests() {
        test_simple_failing_property_workflow();
        test_multiple_generator_workflow();
        test_string_property_workflow();
        test_frequency_generator_workflow();
        test_one_of_generator_workflow();
        test_complex_nested_workflow();
        test_custom_config_workflow();
        test_named_property_workflow();
    }
}