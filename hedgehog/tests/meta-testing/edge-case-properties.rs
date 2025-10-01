//! Edge case and boundary condition properties
//!
//! These properties test how Hedgehog handles edge cases like empty inputs,
//! extreme values, and boundary conditions.

use crate::arbitrary_seed;
use hedgehog::*;

/// Property: Empty vectors should be handled correctly
pub fn test_empty_vector_generation() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&seed: &Seed| {
        let size = Size::new(0); // Force small size
        let gen = Gen::vec_of(Gen::int_range(0, 100));
        let tree = gen.generate(size, seed);

        // Should be able to generate small vectors (including empty)
        tree.value.len() <= 5 // Allow some flexibility
    });

    let fast_config = Config::default().with_tests(20).with_shrinks(5);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Empty vector generation property passed"),
        result => panic!("Empty vector generation property failed: {result:?}"),
    }
}

/// Property: Single element ranges should work
pub fn test_single_element_ranges() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&seed: &Seed| {
        let size = Size::new(10);
        let gen = Gen::<i32>::from_range(Range::new(42, 42)); // Single element range
        let tree = gen.generate(size, seed);

        // Should always generate the single value
        tree.value == 42
    });

    let fast_config = Config::default().with_tests(15).with_shrinks(3);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Single element range property passed"),
        result => panic!("Single element range property failed: {result:?}"),
    }
}

/// Property: Extreme integer ranges should not overflow
pub fn test_extreme_integer_ranges() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&seed: &Seed| {
        let size = Size::new(10);

        // Test extreme but safe ranges
        let gen1 = Gen::<i32>::from_range(Range::new(i32::MIN + 1000, i32::MIN + 2000));
        let gen2 = Gen::<i32>::from_range(Range::new(i32::MAX - 2000, i32::MAX - 1000));

        let result1 = gen1.generate(size, seed);
        let result2 = gen2.generate(size, seed);

        // Values should be within expected ranges
        let valid1 = result1.value >= (i32::MIN + 1000) && result1.value <= (i32::MIN + 2000);
        let valid2 = result2.value >= (i32::MAX - 2000) && result2.value <= (i32::MAX - 1000);

        valid1 && valid2
    });

    let fast_config = Config::default().with_tests(15).with_shrinks(3);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Extreme integer range property passed"),
        result => panic!("Extreme integer range property failed: {result:?}"),
    }
}

/// Property: Zero-sized structures should be handled
pub fn test_zero_size_generation() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&seed: &Seed| {
        let size = Size::new(0);

        // Various generators with zero size
        let vec_gen = Gen::vec_of(Gen::int_range(0, 10));
        let string_gen = Gen::<String>::ascii_alpha();

        let vec_result = vec_gen.generate(size, seed);
        let string_result = string_gen.generate(size, seed);

        // Zero size should generate minimal structures
        vec_result.value.len() <= 3 && string_result.value.len() <= 10
    });

    let fast_config = Config::default().with_tests(15).with_shrinks(3);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Zero size generation property passed"),
        result => panic!("Zero size generation property failed: {result:?}"),
    }
}

/// Property: Maximum size should not cause issues
pub fn test_maximum_size_generation() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&seed: &Seed| {
        let size = Size::new(100); // Maximum reasonable size
        let gen = Gen::vec_of(Gen::int_range(0, 10));
        let tree = gen.generate(size, seed);

        // Should generate reasonable-sized structures (not infinite)
        tree.value.len() <= 200 // Allow some overhead but not unlimited
    });

    let fast_config = Config::default().with_tests(10).with_shrinks(3);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Maximum size generation property passed"),
        result => panic!("Maximum size generation property failed: {result:?}"),
    }
}

/// Property: Negative ranges should work correctly
pub fn test_negative_ranges() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&seed: &Seed| {
        let size = Size::new(10);
        let gen = Gen::<i32>::from_range(Range::new(-100, -10));
        let tree = gen.generate(size, seed);

        // Value should be in negative range
        tree.value >= -100 && tree.value <= -10
    });

    let fast_config = Config::default().with_tests(15).with_shrinks(3);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Negative range property passed"),
        result => panic!("Negative range property failed: {result:?}"),
    }
}

/// Property: Frequency with zero weights should be handled
pub fn test_frequency_edge_cases() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&seed: &Seed| {
        let size = Size::new(10);

        // Test frequency with some zero weights
        let choices = vec![
            WeightedChoice::new(0, Gen::constant(1)),  // Zero weight
            WeightedChoice::new(10, Gen::constant(2)), // Normal weight
            WeightedChoice::new(1, Gen::constant(3)),  // Small weight
        ];

        match Gen::frequency(choices) {
            Ok(gen) => {
                let tree = gen.generate(size, seed);
                // Should only generate values 2 or 3 (not 1, which has zero weight)
                tree.value == 2 || tree.value == 3
            }
            Err(_) => true, // Expected behavior if all weights are zero
        }
    });

    let fast_config = Config::default().with_tests(15).with_shrinks(3);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Frequency edge cases property passed"),
        result => panic!("Frequency edge cases property failed: {result:?}"),
    }
}

/// Property: One-of with single generator should work
pub fn test_one_of_single_generator() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&seed: &Seed| {
        let size = Size::new(10);
        let single_gen = vec![Gen::constant(42)];

        match Gen::one_of(single_gen) {
            Ok(gen) => {
                let tree = gen.generate(size, seed);
                tree.value == 42 // Should always generate 42
            }
            Err(_) => false, // Should not fail with valid input
        }
    });

    let fast_config = Config::default().with_tests(15).with_shrinks(3);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ One-of single generator property passed"),
        result => panic!("One-of single generator property failed: {result:?}"),
    }
}

/// Property: String generators should handle empty strings
pub fn test_empty_string_handling() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&seed: &Seed| {
        let size = Size::new(0);
        let gen = Gen::<String>::ascii_alpha();
        let tree = gen.generate(size, seed);

        // Should be able to generate empty or very short strings
        tree.value.len() <= 5
    });

    let fast_config = Config::default().with_tests(15).with_shrinks(3);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Empty string handling property passed"),
        result => panic!("Empty string handling property failed: {result:?}"),
    }
}

/// Property: Nested structures should not cause stack overflow
pub fn test_nested_structure_limits() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&seed: &Seed| {
        let size = Size::new(5); // Small size to control depth
        let gen = Gen::vec_of(Gen::vec_of(Gen::int_range(0, 10)));
        let tree = gen.generate(size, seed);

        // Should generate without crashing
        let total_elements: usize = tree.value.iter().map(|v| v.len()).sum();
        total_elements <= 50 // Reasonable limit
    });

    let fast_config = Config::default().with_tests(10).with_shrinks(3);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Nested structure limits property passed"),
        result => panic!("Nested structure limits property failed: {result:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_edge_case_property_tests() {
        test_empty_vector_generation();
        test_single_element_ranges();
        test_extreme_integer_ranges();
        test_zero_size_generation();
        test_maximum_size_generation();
        test_negative_ranges();
        test_frequency_edge_cases();
        test_one_of_single_generator();
        test_empty_string_handling();
        test_nested_structure_limits();
    }
}
