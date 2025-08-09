//! Examples demonstrating targeted property-based testing with Hedgehog.
//!
//! This shows how to use search-guided generation to find inputs that maximize
//! or minimize specific utility functions, making it easier to discover edge cases
//! and explore specific behaviors.

use hedgehog_core::{
    data::*, 
    gen::Gen,
    targeted::*,
};

/// Example function that we want to test - has interesting behavior for large inputs
fn expensive_computation(n: i32) -> i32 {
    if n < 0 {
        return 0;
    }
    
    let mut result: i32 = 1;
    for i in 1..=(n.min(20)) {
        result = result.saturating_mul(i);
    }
    
    // Simulate expensive computation that gets slower with larger inputs
    std::thread::sleep(std::time::Duration::from_micros((n.abs() as u64).min(1000)));
    
    result
}

/// Example 1: Find inputs that maximize computation time
fn example_maximize_computation_time() {
    println!("=== Example 1: Finding inputs that maximize computation time ===");
    
    let generator = Gen::<i32>::from_range(Range::new(-50, 50));
    
    // Utility function that measures computation time
    let utility_function = |input: &i32, _result: &TargetedResult| -> f64 {
        let start = std::time::Instant::now();
        let _ = expensive_computation(*input);
        start.elapsed().as_micros() as f64
    };
    
    // Test function that always passes but measures the utility
    let test_function = |input: &i32| -> TargetedResult {
        let result = expensive_computation(*input);
        
        // Property: result should be positive for positive inputs
        if *input > 0 && result <= 0 {
            TargetedResult::Fail {
                counterexample: format!("input: {}, result: {}", input, result),
                tests_run: 1,
                shrinks_performed: 0,
                property_name: Some("positive_input_positive_result".to_string()),
                module_path: Some("targeted_testing::example_1".to_string()),
                assertion_type: Some("computation_correctness".to_string()),
                shrink_steps: vec![],
                utility: 0.0, // Will be overwritten
            }
        } else {
            TargetedResult::Pass {
                tests_run: 1,
                property_name: Some("positive_input_positive_result".to_string()),
                module_path: Some("targeted_testing::example_1".to_string()),
                utility: 0.0, // Will be overwritten
            }
        }
    };
    
    let neighborhood = IntegerNeighborhood::new(5);
    
    let config = TargetedConfig {
        objective: SearchObjective::Maximize,
        search_steps: 100,
        initial_temperature: 50.0,
        cooling_rate: 0.90,
        initial_samples: 20,
        max_search_time: Some(std::time::Duration::from_secs(5)),
        ..Default::default()
    };
    
    let search = for_all_targeted_with_config(
        generator,
        utility_function,
        test_function,
        neighborhood,
        config,
    );
    
    let test_config = Config::default();
    let (result, stats) = search.search(&test_config);
    
    println!("Search completed!");
    println!("  Evaluations: {}", stats.evaluations);
    println!("  Accepted moves: {}", stats.accepted_moves);
    println!("  Best utility (computation time): {:.2} μs", stats.best_utility);
    println!("  Final temperature: {:.4}", stats.final_temperature);
    println!("  Search time: {:?}", stats.search_time);
    println!("  Converged: {}", stats.converged);
    
    match result {
        TargetedResult::Pass { utility, .. } => {
            println!("  Result: PASS with utility {:.2}", utility);
        }
        TargetedResult::Fail { counterexample, utility, .. } => {
            println!("  Result: FAIL with utility {:.2}", utility);
            println!("  Counterexample: {}", counterexample);
        }
        TargetedResult::Discard { .. } => {
            println!("  Result: DISCARDED");
        }
    }
    
    println!();
}

/// Example 2: Find strings that cause parsing errors
fn example_find_parsing_errors() {
    println!("=== Example 2: Finding strings that cause parsing errors ===");
    
    // Simple parser that fails on certain patterns
    fn simple_parser(s: &str) -> Result<i32, String> {
        if s.contains("null") {
            return Err("null pointer detected".to_string());
        }
        if s.len() > 15 {
            return Err("input too long".to_string());
        }
        if s.chars().any(|c| c.is_ascii_control()) {
            return Err("control character detected".to_string());
        }
        
        // Try to parse as number
        s.parse().map_err(|e| format!("parse error: {}", e))
    }
    
    let chars = "abcdefghijklmnopqrstuvwxyz0123456789null\x00\x01\x02".chars().collect::<Vec<_>>();
    let char_gen = Gen::<char>::from_elements(chars).expect("chars should not be empty");
    let generator = Gen::string_of(char_gen);
    
    // Utility function that rewards finding parse errors
    let utility_function = |input: &String, _result: &TargetedResult| -> f64 {
        match simple_parser(input) {
            Ok(_) => 0.0, // No error found = low utility
            Err(error_msg) => {
                // Different error types get different rewards
                if error_msg.contains("null pointer") {
                    100.0
                } else if error_msg.contains("too long") {
                    80.0
                } else if error_msg.contains("control character") {
                    90.0
                } else {
                    50.0 // Generic parse errors
                }
            }
        }
    };
    
    // Test function that passes but tracks parsing results
    let test_function = |input: &String| -> TargetedResult {
        match simple_parser(input) {
            Ok(_value) => TargetedResult::Pass {
                tests_run: 1,
                property_name: Some("parser_robustness".to_string()),
                module_path: Some("targeted_testing::example_2".to_string()),
                utility: 0.0,
            },
            Err(_error) => {
                // Found an error - this is what we're looking for!
                TargetedResult::Pass {
                    tests_run: 1,
                    property_name: Some("parser_robustness".to_string()),
                    module_path: Some("targeted_testing::example_2".to_string()),
                    utility: 0.0,
                }
            }
        }
    };
    
    let neighborhood = StringNeighborhood::new(
        "abcdefghijklmnopqrstuvwxyz0123456789null\x00\x01\x02".chars().collect()
    );
    
    let config = TargetedConfig {
        objective: SearchObjective::Maximize,
        search_steps: 200,
        initial_temperature: 100.0,
        cooling_rate: 0.95,
        initial_samples: 50,
        max_search_time: Some(std::time::Duration::from_secs(10)),
        ..Default::default()
    };
    
    let search = for_all_targeted_with_config(
        generator,
        utility_function,
        test_function,
        neighborhood,
        config,
    );
    
    let test_config = Config::default();
    let (_result, stats) = search.search(&test_config);
    
    println!("Search completed!");
    println!("  Evaluations: {}", stats.evaluations);
    println!("  Accepted moves: {}", stats.accepted_moves);
    println!("  Best utility (error severity): {:.2}", stats.best_utility);
    println!("  Final temperature: {:.4}", stats.final_temperature);
    println!("  Search time: {:?}", stats.search_time);
    println!("  Converged: {}", stats.converged);
    
    // Show some interesting inputs found during search
    println!("  Utility progression (first 10): {:?}", 
             stats.utility_history.iter().take(10).collect::<Vec<_>>());
    
    println!();
}

/// Example 3: Find vectors that minimize a cost function  
fn example_minimize_cost_function() {
    println!("=== Example 3: Finding vectors that minimize a cost function ===");
    
    // Cost function with multiple local minima
    fn cost_function(vec: &[i32]) -> f64 {
        if vec.is_empty() {
            return 1000.0;
        }
        
        let sum: i32 = vec.iter().sum();
        let mean = sum as f64 / vec.len() as f64;
        
        // Cost is distance from target mean (42) plus variance penalty
        let target_distance = (mean - 42.0).abs();
        let variance = vec.iter()
            .map(|&x| (x as f64 - mean).powi(2))
            .sum::<f64>() / vec.len() as f64;
        
        target_distance + variance / 10.0
    }
    
    let element_gen = Gen::<i32>::from_range(Range::new(0, 100));
    let generator = Gen::vec_of(element_gen);
    
    // Utility function that returns the cost (to be minimized)
    let utility_function = |input: &Vec<i32>, _result: &TargetedResult| -> f64 {
        cost_function(input)
    };
    
    // Test function that always passes
    let test_function = |_input: &Vec<i32>| -> TargetedResult {
        TargetedResult::Pass {
            tests_run: 1,
            property_name: Some("cost_minimization".to_string()),
            module_path: Some("targeted_testing::example_3".to_string()),
            utility: 0.0,
        }
    };
    
    let element_neighborhood = IntegerNeighborhood::new(10);
    let neighborhood = VecNeighborhood::new(element_neighborhood, 0.3);
    
    let config = TargetedConfig {
        objective: SearchObjective::Minimize,
        search_steps: 300,
        initial_temperature: 200.0,
        cooling_rate: 0.98,
        initial_samples: 100,
        max_search_time: Some(std::time::Duration::from_secs(15)),
        ..Default::default()
    };
    
    let search = for_all_targeted_with_config(
        generator,
        utility_function,
        test_function,
        neighborhood,
        config,
    );
    
    let test_config = Config::default();
    let (_result, stats) = search.search(&test_config);
    
    println!("Search completed!");
    println!("  Evaluations: {}", stats.evaluations);
    println!("  Accepted moves: {}", stats.accepted_moves);
    println!("  Best utility (minimum cost): {:.2}", stats.best_utility);
    println!("  Final temperature: {:.4}", stats.final_temperature);
    println!("  Search time: {:?}", stats.search_time);
    println!("  Converged: {}", stats.converged);
    
    println!("  Cost progression (last 10): {:?}",
             stats.utility_history.iter().rev().take(10).rev().collect::<Vec<_>>());
    
    println!();
}

/// Example 4: Compare targeted vs random testing effectiveness
fn example_compare_targeted_vs_random() {
    println!("=== Example 4: Comparing targeted vs random testing ===");
    
    // Function with a rare edge case
    fn tricky_function(n: i32) -> i32 {
        if n == 12345 {
            panic!("Found the magic number!");
        }
        if n > 10000 && n < 15000 {
            return n * 2; // Different behavior in this range
        }
        n
    }
    
    // Utility function that rewards finding numbers close to the magic number
    let utility_function = |input: &i32, _result: &TargetedResult| -> f64 {
        let distance_from_magic = (input - 12345).abs();
        1000.0 - (distance_from_magic as f64).min(1000.0)
    };
    
    let test_function = |input: &i32| -> TargetedResult {
        let result = std::panic::catch_unwind(|| tricky_function(*input));
        
        match result {
            Ok(_) => TargetedResult::Pass {
                tests_run: 1,
                property_name: Some("no_panic".to_string()),
                module_path: Some("targeted_testing::example_4".to_string()),
                utility: 0.0,
            },
            Err(_) => TargetedResult::Fail {
                counterexample: format!("input: {} caused panic", input),
                tests_run: 1,
                shrinks_performed: 0,
                property_name: Some("no_panic".to_string()),
                module_path: Some("targeted_testing::example_4".to_string()),
                assertion_type: Some("panic_detected".to_string()),
                shrink_steps: vec![],
                utility: 0.0,
            }
        }
    };
    
    let generator = Gen::<i32>::from_range(Range::new(0, 20000));
    let neighborhood = IntegerNeighborhood::new(100);
    
    let config = TargetedConfig {
        objective: SearchObjective::Maximize,
        search_steps: 200,
        initial_temperature: 1000.0,
        cooling_rate: 0.95,
        initial_samples: 20,
        max_search_time: Some(std::time::Duration::from_secs(5)),
        ..Default::default()
    };
    
    let search = for_all_targeted_with_config(
        generator,
        utility_function,
        test_function,
        neighborhood,
        config,
    );
    
    let test_config = Config::default();
    let (result, stats) = search.search(&test_config);
    
    println!("Targeted search completed!");
    println!("  Evaluations: {}", stats.evaluations);
    println!("  Best utility: {:.2}", stats.best_utility);
    
    match result {
        TargetedResult::Fail { counterexample, .. } => {
            println!("  SUCCESS: Found the edge case! {}", counterexample);
        }
        _ => {
            println!("  Did not find the magic number, best utility: {:.2}", stats.best_utility);
        }
    }
    
    println!();
}

fn main() {
    println!("Hedgehog Targeted Property Testing Examples");
    println!("==========================================");
    println!();
    
    example_maximize_computation_time();
    example_find_parsing_errors();
    example_minimize_cost_function();
    example_compare_targeted_vs_random();
    
    println!("All examples completed!");
}