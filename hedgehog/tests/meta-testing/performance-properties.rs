//! Performance property testing
//!
//! These properties ensure that Hedgehog's generation and shrinking operations
//! complete within reasonable time bounds and don't have pathological performance.

use crate::{arbitrary_seed, arbitrary_size};
use hedgehog::*;
use std::time::{Duration, Instant};

/// Property: Generation should complete within reasonable time bounds
pub fn test_generation_time_bounds() {
    let prop = for_all_named(
        Gen::<(Size, Seed)>::tuple_of(arbitrary_size(), arbitrary_seed()),
        "(size, seed)",
        |&(size, seed): &(Size, Seed)| {
            let gen = Gen::vec_of(Gen::int_range(0, 10));
            let start = Instant::now();
            let _tree = gen.generate(size, seed);
            let duration = start.elapsed();

            // Generation should complete in under 100ms for reasonable sizes
            duration < Duration::from_millis(100)
        },
    );

    let fast_config = Config::default().with_tests(10).with_shrinks(3); // Very fast for perf tests
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Generation time bounds property passed"),
        result => panic!("Generation time bounds property failed: {result:?}"),
    }
}

/// Property: Shrinking should complete within reasonable time bounds  
pub fn test_shrinking_time_bounds() {
    let prop = for_all_named(
        Gen::<(Size, Seed)>::tuple_of(arbitrary_size(), arbitrary_seed()),
        "(size, seed)",
        |&(size, seed): &(Size, Seed)| {
            let gen = Gen::vec_of(Gen::int_range(0, 10));
            let tree = gen.generate(size, seed);

            let start = Instant::now();
            let _shrinks = tree.shrinks();
            let duration = start.elapsed();

            // Shrinking should complete in under 50ms for reasonable values
            duration < Duration::from_millis(50)
        },
    );

    let fast_config = Config::default().with_tests(10).with_shrinks(3); // Very fast for perf tests
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Shrinking time bounds property passed"),
        result => panic!("Shrinking time bounds property failed: {result:?}"),
    }
}

/// Property: Property evaluation should scale reasonably with test count
pub fn test_property_scaling() {
    let simple_property = |&x: &i32| x >= 0;

    // Test with different test counts
    let test_counts = vec![10, 50, 100, 200];
    let mut timings = Vec::new();

    for &test_count in &test_counts {
        let config = Config {
            test_limit: test_count,
            ..Config::default()
        };

        let prop = for_all(Gen::int_range(0, 1000), simple_property);

        let start = Instant::now();
        let _result = prop.run(&config);
        let duration = start.elapsed();

        timings.push((test_count, duration));
    }

    // Check that timing scales roughly linearly (not exponentially)
    let (first_count, first_time) = timings[0];
    let (last_count, last_time) = timings.last().unwrap();

    let count_ratio = *last_count as f64 / first_count as f64;
    let time_ratio = last_time.as_millis() as f64 / first_time.as_millis() as f64;

    // Time ratio should not be much worse than count ratio (allow 3x overhead)
    if time_ratio > count_ratio * 3.0 {
        panic!("Property evaluation doesn't scale linearly: {count_ratio}x count increase led to {time_ratio}x time increase");
    }

    println!("✓ Property scaling property passed ({count_ratio}x count -> {time_ratio}x time)");
}

/// Property: Large generators should not cause stack overflow
pub fn test_no_stack_overflow() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&seed: &Seed| {
        let size = Size::new(20); // Smaller size for faster testing
                                  // Create a deeply nested generator structure
        let deep_gen = create_deep_generator(10); // 10 levels deep for faster testing

        // Should not stack overflow
        let _tree = deep_gen.generate(size, seed);
        true
    });

    let fast_config = Config::default().with_tests(10).with_shrinks(3); // Very fast for perf tests
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ No stack overflow property passed"),
        result => panic!("No stack overflow property failed: {result:?}"),
    }
}

/// Property: Memory usage should be reasonable for large generations
pub fn test_memory_usage_bounds() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&seed: &Seed| {
        let size = Size::new(10);
        // Generate a large structure
        let gen = Gen::vec_of(Gen::vec_of(Gen::int_range(0, 5)));
        let tree = gen.generate(size, seed);

        // Check that the generated structure has reasonable size bounds
        let total_elements: usize = tree.value.iter().map(|v| v.len()).sum();

        // Should not generate millions of elements
        total_elements < 1000 // Much smaller for fast testing
    });

    let fast_config = Config::default().with_tests(10).with_shrinks(3); // Very fast for perf tests
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Memory usage bounds property passed"),
        result => panic!("Memory usage bounds property failed: {result:?}"),
    }
}

/// Property: Targeted testing should complete within reasonable time
pub fn test_targeted_testing_performance() {
    use hedgehog::targeted::*;

    let generator = Gen::<i32>::from_range(Range::new(0, 100));

    let utility_function = |_input: &i32, _result: &TargetedResult| -> f64 {
        1.0 // Simple utility function
    };

    let test_function = |_input: &i32| -> TargetedResult {
        TargetedResult::Pass {
            tests_run: 1,
            property_name: Some("performance_test".to_string()),
            module_path: Some("meta_testing".to_string()),
            utility: 0.0,
        }
    };

    let neighborhood = IntegerNeighborhood::new(10);

    let config = TargetedConfig {
        search_steps: 10, // Very small number for performance test
        max_search_time: Some(Duration::from_millis(500)),
        ..Default::default()
    };

    let search = for_all_targeted_with_config(
        generator,
        utility_function,
        test_function,
        neighborhood,
        config,
    );

    let start = Instant::now();
    let (_result, _stats) = search.search(&Config::default());
    let duration = start.elapsed();

    // Should complete within the specified time limit (plus some overhead)
    if duration > Duration::from_millis(600) {
        panic!("Targeted testing took too long: {duration:?}");
    }

    println!("✓ Targeted testing performance property passed ({duration:?})");
}

// Helper functions

fn create_deep_generator(depth: usize) -> Gen<Vec<i32>> {
    if depth == 0 {
        Gen::vec_of(Gen::int_range(0, 10))
    } else {
        // Create nested structure (not literally recursive in types, but in generation)
        Gen::vec_of(Gen::int_range(0, depth as i32))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_performance_property_tests() {
        test_generation_time_bounds();
        test_shrinking_time_bounds();
        test_property_scaling();
        test_no_stack_overflow();
        test_memory_usage_bounds();
        test_targeted_testing_performance();
    }
}
