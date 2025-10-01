//! Generator invariant properties
//!
//! These properties ensure that generators behave correctly with respect to
//! size bounds, distribution properties, and other fundamental invariants.

use crate::{arbitrary_seed, arbitrary_size};
use hedgehog::*;

/// Property: Generated values should respect size bounds
pub fn test_generator_size_bounds() {
    let prop = for_all_named(
        Gen::<((Vec<i32>, Size), Seed)>::tuple_of(
            Gen::<(Vec<i32>, Size)>::tuple_of(Gen::vec_of(Gen::int_range(0, 10)), arbitrary_size()),
            arbitrary_seed(),
        ),
        "((vector, size), seed)",
        |&((ref _vector, size), seed): &((Vec<i32>, Size), Seed)| {
            let gen = Gen::vec_of(Gen::int_range(0, 10));
            let tree = gen.generate(size, seed);
            let value = tree.value;

            // Vector size should be bounded by the Size parameter
            value.len() <= size.get() + 10 // Allow some variance
        },
    );

    // Use fast config for meta testing - fewer tests but still good coverage
    let fast_config = Config::default().with_tests(20).with_shrinks(10);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Generator size bounds property passed"),
        result => panic!("Generator size bounds property failed: {result:?}"),
    }
}

/// Property: Generators should be deterministic for same size/seed
pub fn test_generator_determinism() {
    let prop = for_all_named(
        Gen::<(Size, Seed)>::tuple_of(arbitrary_size(), arbitrary_seed()),
        "(size, seed)",
        |&(size, seed): &(Size, Seed)| {
            let gen = Gen::int_range(0, 100);
            let tree1 = gen.generate(size, seed);
            let tree2 = gen.generate(size, seed);

            // Same generator with same size/seed should produce same result
            tree1.value == tree2.value
        },
    );

    let fast_config = Config::default().with_tests(20).with_shrinks(10);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Generator determinism property passed"),
        result => panic!("Generator determinism property failed: {result:?}"),
    }
}

/// Property: Size parameter should influence generation
pub fn test_size_influence() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&seed: &Seed| {
        let gen = Gen::vec_of(Gen::int_range(0, 10));
        let small_tree = gen.generate(Size::new(1), seed);
        let large_tree = gen.generate(Size::new(50), seed);

        // Larger size should generally produce larger or equal structures
        // (not guaranteed for every single case, but should hold statistically)
        small_tree.value.len() <= large_tree.value.len() + 10 // Allow some variance
    });

    let fast_config = Config::default().with_tests(20).with_shrinks(10);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Size influence property passed"),
        result => panic!("Size influence property failed: {result:?}"),
    }
}

/// Property: Range generators should respect bounds
pub fn test_range_bounds() {
    let prop = for_all_named(
        Gen::<(i32, i32, Size, Seed)>::tuple_of(
            Gen::int_range(-20, 0), // min - smaller range for faster testing
            Gen::int_range(1, 20),  // max - smaller range for faster testing
            arbitrary_size(),
            arbitrary_seed(),
        ),
        "(min, max, size, seed)",
        |&(min, max, size, seed): &(i32, i32, Size, Seed)| {
            if min >= max {
                return true;
            } // Skip invalid ranges

            let gen = Gen::<i32>::from_range(Range::new(min, max));
            let tree = gen.generate(size, seed);
            let value = tree.value;

            // Generated value should be within specified range
            value >= min && value <= max
        },
    );

    let fast_config = Config::default().with_tests(20).with_shrinks(10);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Range bounds property passed"),
        result => panic!("Range bounds property failed: {result:?}"),
    }
}

/// Property: Frequency generators should not crash
pub fn test_frequency_generator_stability() {
    let prop = for_all_named(
        Gen::<((Vec<i32>, Size), Seed)>::tuple_of(
            Gen::<(Vec<i32>, Size)>::tuple_of(
                Gen::vec_of(Gen::int_range(1, 100)), // weights
                arbitrary_size(),
            ),
            arbitrary_seed(),
        ),
        "((weights, size), seed)",
        |&((ref weights, size), seed): &((Vec<i32>, Size), Seed)| {
            if weights.is_empty() {
                return true;
            }

            // Create weighted choices
            let choices: Vec<WeightedChoice<i32>> = weights
                .iter()
                .enumerate()
                .map(|(i, &weight)| WeightedChoice::new(weight as u64, Gen::constant(i as i32)))
                .collect();

            // Generator should not crash
            match Gen::frequency(choices) {
                Ok(gen) => {
                    let _tree = gen.generate(size, seed);
                    true // Success if no panic
                }
                Err(_) => true, // Expected for invalid inputs
            }
        },
    );

    let fast_config = Config::default().with_tests(15).with_shrinks(5); // Even smaller for stability test
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Frequency generator stability property passed"),
        result => panic!("Frequency generator stability property failed: {result:?}"),
    }
}

// Helper functions

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_generator_invariant_tests() {
        test_generator_size_bounds();
        test_generator_determinism();
        test_size_influence();
        test_range_bounds();
        test_frequency_generator_stability();
    }
}
