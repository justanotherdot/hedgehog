//! Generator combinator properties
//!
//! These properties ensure that map, bind, filter and other generator
//! combinators behave correctly and maintain mathematical laws.

use crate::{arbitrary_seed, arbitrary_size};
use hedgehog::*;

/// Property: Map should preserve generator determinism
pub fn test_map_determinism() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&seed: &Seed| {
        let size = Size::new(10);
        let result1 = Gen::int_range(0, 100).map(|x| x * 2).generate(size, seed);
        let result2 = Gen::int_range(0, 100).map(|x| x * 2).generate(size, seed);

        // Same inputs should produce same outputs
        result1.value == result2.value
    });

    let fast_config = Config::default().with_tests(15).with_shrinks(5);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Map determinism property passed"),
        result => panic!("Map determinism property failed: {result:?}"),
    }
}

/// Property: Map should apply the function correctly
pub fn test_map_function_application() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&seed: &Seed| {
        let size = Size::new(10);
        let base_result = Gen::int_range(0, 50).generate(size, seed);
        let mapped_result = Gen::int_range(0, 50).map(|x| x + 10).generate(size, seed);

        // Mapped result should be base result + 10
        mapped_result.value == base_result.value + 10
    });

    let fast_config = Config::default().with_tests(15).with_shrinks(5);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Map function application property passed"),
        result => panic!("Map function application property failed: {result:?}"),
    }
}

/// Property: Map composition law (map f . map g = map (f . g))
pub fn test_map_composition() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&seed: &Seed| {
        let size = Size::new(10);
        // Two separate maps
        let double_result = Gen::int_range(0, 20)
            .map(|x| x + 1)
            .map(|x| x * 2)
            .generate(size, seed);

        // Single composed map
        let composed_result = Gen::int_range(0, 20)
            .map(|x| (x + 1) * 2)
            .generate(size, seed);

        // Results should be the same
        double_result.value == composed_result.value
    });

    let fast_config = Config::default().with_tests(15).with_shrinks(5);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Map composition property passed"),
        result => panic!("Map composition property failed: {result:?}"),
    }
}

/// Property: Bind should preserve generator determinism
pub fn test_bind_determinism() {
    let prop = for_all_named(
        Gen::<(Size, Seed)>::tuple_of(arbitrary_size(), arbitrary_seed()),
        "(size, seed)",
        |&(size, seed): &(Size, Seed)| {
            let bound_gen1 = Gen::int_range(1, 10).bind(|x| Gen::int_range(0, x));
            let bound_gen2 = Gen::int_range(1, 10).bind(|x| Gen::int_range(0, x));

            // Same inputs should produce same outputs
            let result1 = bound_gen1.generate(size, seed);
            let result2 = bound_gen2.generate(size, seed);

            result1.value == result2.value
        },
    );

    let fast_config = Config::default().with_tests(15).with_shrinks(5);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Bind determinism property passed"),
        result => panic!("Bind determinism property failed: {result:?}"),
    }
}

/// Property: Bind should respect bounds from dependent generation
pub fn test_bind_dependent_bounds() {
    let prop = for_all_named(
        Gen::<(Size, Seed)>::tuple_of(arbitrary_size(), arbitrary_seed()),
        "(size, seed)",
        |&(size, seed): &(Size, Seed)| {
            let base_gen = Gen::int_range(5, 20);
            let bound_gen = base_gen.bind(|x| Gen::int_range(0, x));

            let result = bound_gen.generate(size, seed);

            // Result should be between 0 and 20 (max possible from base generator)
            result.value >= 0 && result.value <= 20
        },
    );

    let fast_config = Config::default().with_tests(15).with_shrinks(5);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Bind dependent bounds property passed"),
        result => panic!("Bind dependent bounds property failed: {result:?}"),
    }
}

/// Property: Filter should only produce values matching the predicate  
pub fn test_filter_predicate_correctness() {
    // Test filter directly without meta-property to avoid issues
    let size = Size::new(10);

    for i in 0..10 {
        let seed = Seed::from_u64(i);
        let filtered_gen = Gen::int_range(0, 20).filter(|&x| x < 15);
        let tree = filtered_gen.generate(size, seed);
        let value = tree.value;

        if value >= 15 {
            panic!("Filter failed: generated {value} which is not < 15");
        }
    }

    println!("✓ Filter predicate correctness property passed");
}

/// Property: Filter with always-true predicate should be identity
pub fn test_filter_identity() {
    let prop = for_all_named(
        Gen::<(Size, Seed)>::tuple_of(arbitrary_size(), arbitrary_seed()),
        "(size, seed)",
        |&(size, seed): &(Size, Seed)| {
            let base_gen = Gen::int_range(0, 50);
            let filtered_gen = Gen::int_range(0, 50).filter(|_| true);

            let base_result = base_gen.generate(size, seed);
            let filtered_result = filtered_gen.generate(size, seed);

            // Should produce the same result
            base_result.value == filtered_result.value
        },
    );

    let fast_config = Config::default().with_tests(15).with_shrinks(5);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Filter identity property passed"),
        result => panic!("Filter identity property failed: {result:?}"),
    }
}

/// Property: Combining map and filter should work correctly
pub fn test_map_filter_combination() {
    let prop = for_all_named(
        Gen::<(Size, Seed)>::tuple_of(arbitrary_size(), arbitrary_seed()),
        "(size, seed)",
        |&(size, seed): &(Size, Seed)| {
            // Map then filter
            let map_then_filter = Gen::int_range(0, 20).map(|x| x * 2).filter(|&x| x < 30);

            let result = map_then_filter.generate(size, seed);
            let value = result.value;

            // Value should be even (mapped from original) and < 30 (filtered)
            value % 2 == 0 && value < 30
        },
    );

    let fast_config = Config::default().with_tests(10).with_shrinks(3);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Map-filter combination property passed"),
        result => panic!("Map-filter combination property failed: {result:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_combinator_property_tests() {
        test_map_determinism();
        test_map_function_application();
        test_map_composition();
        test_bind_determinism();
        test_bind_dependent_bounds();
        test_filter_predicate_correctness();
        test_filter_identity();
        test_map_filter_combination();
    }
}
