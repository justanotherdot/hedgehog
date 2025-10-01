//! Custom generator composition meta tests
//!
//! These properties test advanced patterns of generator composition including
//! recursive generators, conditional generation, and complex data structures.

use crate::arbitrary_seed;
use hedgehog::*;

/// Property: Recursive generator patterns should work correctly
pub fn test_recursive_composition() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&seed: &Seed| {
        let size = Size::new(5); // Keep small for recursive structures

        // Create a tree-like recursive structure generator
        fn tree_gen(depth: usize) -> Gen<Vec<i32>> {
            if depth == 0 {
                Gen::constant(vec![])
            } else {
                let leaf = Gen::<Vec<i32>>::vec_of(Gen::int_range(1, 10));
                let branch = tree_gen(depth - 1).map(|subtree| {
                    let mut result = vec![0]; // Branch marker
                    result.extend(subtree);
                    result
                });

                Gen::frequency(vec![
                    WeightedChoice::new(3, leaf),   // 60% leaves
                    WeightedChoice::new(2, branch), // 40% branches
                ])
                .unwrap_or(Gen::constant(vec![]))
            }
        }

        let recursive_gen = tree_gen(3);
        let tree_result = recursive_gen.generate(size, seed).value;

        // Should generate valid tree structures
        let is_valid_tree = tree_result.iter().all(|&x| (0..=10).contains(&x));
        let reasonable_size = tree_result.len() <= 20; // Prevent explosion

        is_valid_tree && reasonable_size
    });

    let fast_config = Config::default().with_tests(10).with_shrinks(2);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Recursive composition property passed"),
        result => panic!("Recursive composition property failed: {result:?}"),
    }
}

/// Property: Conditional generator selection should work
pub fn test_conditional_composition() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&seed: &Seed| {
        let size = Size::new(8);

        // Generator that chooses different strategies based on conditions
        let conditional_gen = Gen::new(|size, seed| {
            let (choice_seed, value_seed) = seed.split();
            let (choice, _) = choice_seed.next_bounded(3);

            match choice {
                0 => {
                    // Small numbers strategy
                    let small_gen = Gen::int_range(1, 10);
                    small_gen.generate(size, value_seed)
                }
                1 => {
                    // Large numbers strategy
                    let large_gen = Gen::int_range(100, 1000);
                    large_gen.generate(size, value_seed)
                }
                _ => {
                    // Negative numbers strategy
                    let negative_gen = Gen::int_range(-50, -1);
                    negative_gen.generate(size, value_seed)
                }
            }
        });

        let result = conditional_gen.generate(size, seed).value;

        // Should generate numbers in one of the expected ranges
        let in_small_range = (1..=10).contains(&result);
        let in_large_range = (100..=1000).contains(&result);
        let in_negative_range = (-50..=-1).contains(&result);

        in_small_range || in_large_range || in_negative_range
    });

    let fast_config = Config::default().with_tests(15).with_shrinks(2);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Conditional composition property passed"),
        result => panic!("Conditional composition property failed: {result:?}"),
    }
}

/// Property: Complex nested composition should work
pub fn test_deeply_nested_composition() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&seed: &Seed| {
        let size = Size::new(6);

        // Create a deeply nested composition:
        // Option<Result<Vec<(i32, String)>, bool>>
        let complex_gen =
            Gen::<Option<std::result::Result<Vec<(i32, String)>, bool>>>::option_of(Gen::<
                std::result::Result<Vec<(i32, String)>, bool>,
            >::result_of(
                Gen::<Vec<(i32, String)>>::vec_of(Gen::<(i32, String)>::tuple_of(
                    Gen::int_range(-5, 5),
                    Gen::<String>::ascii_alpha(),
                )),
                Gen::bool(),
            ));

        let result = complex_gen.generate(size, seed).value;

        // Validate the deeply nested structure
        match result {
            Some(Ok(vec)) => vec
                .iter()
                .all(|(n, s)| *n >= -5 && *n <= 5 && s.chars().all(|c| c.is_ascii_alphabetic())),
            Some(Err(_)) => true, // bool is always valid
            None => true,
        }
    });

    let fast_config = Config::default().with_tests(12).with_shrinks(2);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Deeply nested composition property passed"),
        result => panic!("Deeply nested composition property failed: {result:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_composition_property_tests() {
        test_recursive_composition();
        test_conditional_composition();
        test_deeply_nested_composition();
    }
}
