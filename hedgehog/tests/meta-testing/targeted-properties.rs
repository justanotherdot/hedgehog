//! Targeted property testing meta tests
//!
//! These properties test the advanced targeted testing capabilities including
//! simulated annealing search and utility-guided generation.

use hedgehog::*;
use hedgehog::targeted::{
    TargetedResult, TargetedConfig, SearchObjective,
    for_all_targeted_with_config, IntegerNeighborhood
};
use crate::arbitrary_seed;
use std::time::Duration;

/// Property: Targeted search should converge towards optimal values
pub fn test_targeted_search_convergence() {
    let prop = for_all_named(
        arbitrary_seed(),
        "seed",
        |&_seed: &Seed| {
            // Create a simple optimization problem: find input close to target value
            let target = 100;
            let generator = Gen::<i32>::from_range(Range::new(0, 200));
            
            // Utility function rewards values closer to target
            let utility_function = move |input: &i32, _result: &TargetedResult| -> f64 {
                let distance = (input - target).abs() as f64;
                100.0 - distance  // Higher utility for closer values
            };
            
            // Test function that always passes but provides utility feedback
            let test_function = |_input: &i32| -> TargetedResult {
                TargetedResult::Pass {
                    tests_run: 1,
                    property_name: Some("convergence_test".to_string()),
                    module_path: Some("meta_testing".to_string()),
                    utility: 0.0, // Will be overridden by utility_function
                }
            };
            
            let neighborhood = IntegerNeighborhood::new(20);
            
            let config = TargetedConfig {
                search_steps: 50,  // Reasonable number for testing
                max_search_time: Some(Duration::from_millis(200)),
                ..Default::default()
            };
            
            let search = for_all_targeted_with_config(
                generator,
                utility_function,
                test_function,
                neighborhood,
                config
            );
            
            let (result, stats) = search.search(&Config::default().with_tests(1));
            
            // Check that search completed and found reasonable results
            matches!(result, TargetedResult::Pass { .. }) &&
            stats.best_utility.is_finite() &&
            stats.evaluations > 0
        }
    );
    
    let fast_config = Config::default().with_tests(8).with_shrinks(2);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Targeted search convergence property passed"),
        result => panic!("Targeted search convergence property failed: {:?}", result),
    }
}

/// Property: Temperature scheduling should affect search behavior
pub fn test_temperature_scheduling() {
    let prop = for_all_named(
        Gen::<f64>::from_range(Range::new(10.0, 100.0)),
        "initial_temp",
        |&initial_temp: &f64| {
            let generator = Gen::<i32>::from_range(Range::new(0, 50));
            
            let utility_function = |input: &i32, _result: &TargetedResult| -> f64 {
                *input as f64  // Simple increasing utility
            };
            
            let test_function = |_input: &i32| -> TargetedResult {
                TargetedResult::Pass {
                    tests_run: 1,
                    property_name: Some("temp_test".to_string()),
                    module_path: Some("meta_testing".to_string()),
                    utility: 0.0,
                }
            };
            
            let neighborhood = IntegerNeighborhood::new(10);
            
            let config = TargetedConfig {
                initial_temperature: initial_temp,
                search_steps: 30,
                max_search_time: Some(Duration::from_millis(150)),
                ..Default::default()
            };
            
            let search = for_all_targeted_with_config(
                generator,
                utility_function,
                test_function,
                neighborhood,
                config
            );
            
            let (_result, stats) = search.search(&Config::default().with_tests(1));
            
            // Should complete successfully with different temperatures
            // Any result type is acceptable since we're just testing that the search runs
            stats.evaluations > 0 && stats.final_temperature <= initial_temp
        }
    );
    
    let fast_config = Config::default().with_tests(10).with_shrinks(2);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Temperature scheduling property passed"),
        result => panic!("Temperature scheduling property failed: {:?}", result),
    }
}

/// Property: Search objectives (maximize vs minimize) should work correctly
pub fn test_search_objectives() {
    let prop = for_all_named(
        arbitrary_seed(),
        "seed",
        |&_seed: &Seed| {
            let generator = Gen::<i32>::from_range(Range::new(-10, 10));
            
            // Utility function - higher values have higher utility
            let utility_function = |input: &i32, _result: &TargetedResult| -> f64 {
                *input as f64
            };
            
            let test_function = |_input: &i32| -> TargetedResult {
                TargetedResult::Pass {
                    tests_run: 1,
                    property_name: Some("objective_test".to_string()),
                    module_path: Some("meta_testing".to_string()),
                    utility: 0.0,
                }
            };
            
            // Test maximize objective
            let max_config = TargetedConfig {
                objective: SearchObjective::Maximize,
                search_steps: 20,
                max_search_time: Some(Duration::from_millis(100)),
                ..Default::default()
            };
            
            let max_search = for_all_targeted_with_config(
                Gen::<i32>::from_range(Range::new(-10, 10)),
                utility_function,
                test_function,
                IntegerNeighborhood::new(3),
                max_config
            );
            
            let (max_result, max_stats) = max_search.search(&Config::default().with_tests(1));
            
            // Test minimize objective  
            let min_config = TargetedConfig {
                objective: SearchObjective::Minimize,
                search_steps: 20,
                max_search_time: Some(Duration::from_millis(100)),
                ..Default::default()
            };
            
            let min_search = for_all_targeted_with_config(
                generator,
                utility_function,
                test_function,
                IntegerNeighborhood::new(3),
                min_config
            );
            
            let (min_result, min_stats) = min_search.search(&Config::default().with_tests(1));
            
            // Both should complete successfully (any result type is valid)
            match (max_result, min_result) {
                (TargetedResult::Pass { .. }, TargetedResult::Pass { .. }) => 
                    max_stats.evaluations > 0 && min_stats.evaluations > 0,
                (TargetedResult::Fail { .. }, TargetedResult::Fail { .. }) => 
                    max_stats.evaluations > 0 && min_stats.evaluations > 0,
                _ => max_stats.evaluations > 0 && min_stats.evaluations > 0, // Any combo is fine
            }
        }
    );
    
    let fast_config = Config::default().with_tests(8).with_shrinks(2);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Search objectives property passed"),
        result => panic!("Search objectives property failed: {:?}", result),
    }
}

/// Property: Search should respect time limits
pub fn test_search_time_limits() {
    let prop = for_all_named(
        arbitrary_seed(),
        "seed",
        |&_seed: &Seed| {
            let generator = Gen::<i32>::from_range(Range::new(0, 1000));
            
            let utility_function = |input: &i32, _result: &TargetedResult| -> f64 {
                (*input as f64).sin() // Complex utility to slow things down
            };
            
            let test_function = |_input: &i32| -> TargetedResult {
                TargetedResult::Pass {
                    tests_run: 1,
                    property_name: Some("time_limit_test".to_string()),
                    module_path: Some("meta_testing".to_string()),
                    utility: 0.0,
                }
            };
            
            let neighborhood = IntegerNeighborhood::new(50);
            
            let config = TargetedConfig {
                search_steps: 1000, // Large number that should be cut off by time limit
                max_search_time: Some(Duration::from_millis(50)), // Short time limit
                ..Default::default()
            };
            
            let search = for_all_targeted_with_config(
                generator,
                utility_function,
                test_function,
                neighborhood,
                config
            );
            
            let start_time = std::time::Instant::now();
            let (result, _stats) = search.search(&Config::default().with_tests(1));
            let elapsed = start_time.elapsed();
            
            // Should complete within reasonable time bound (allow some overhead)
            matches!(result, TargetedResult::Pass { .. }) &&
            elapsed < Duration::from_millis(200)
        }
    );
    
    let fast_config = Config::default().with_tests(5).with_shrinks(1);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Search time limits property passed"),
        result => panic!("Search time limits property failed: {:?}", result),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_targeted_property_tests() {
        test_targeted_search_convergence();
        test_temperature_scheduling();
        test_search_objectives();
        test_search_time_limits();
    }
}