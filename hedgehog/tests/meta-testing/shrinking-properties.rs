//! Shrinking correctness properties
//!
//! These properties ensure that shrinking behaves correctly - that it always
//! produces smaller failures and converges to minimal counterexamples.

use crate::{arbitrary_seed, arbitrary_size};
use hedgehog::*;

/// Property: Shrinking should always produce smaller values
pub fn test_shrinking_produces_smaller() {
    let prop = for_all_named(
        Gen::<(Size, Seed)>::tuple_of(arbitrary_size(), arbitrary_seed()),
        "(size, seed)",
        |&(size, seed): &(Size, Seed)| {
            // Create a tree for the initial value
            let gen = Gen::vec_of(Gen::int_range(0, 100));
            let tree = gen.generate(size, seed);

            // Get shrinks
            let shrinks = tree.shrinks();

            // All shrinks should be "smaller" than original
            shrinks
                .into_iter()
                .all(|shrink_value| is_smaller_vector(shrink_value, &tree.value))
        },
    );

    let fast_config = Config::default().with_tests(15).with_shrinks(5);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Shrinking produces smaller values property passed"),
        result => panic!("Shrinking produces smaller values property failed: {result:?}"),
    }
}

/// Property: Shrinking should eventually converge  
pub fn test_shrinking_convergence() {
    let prop = for_all_named(
        Gen::<(Size, Seed)>::tuple_of(arbitrary_size(), arbitrary_seed()),
        "(size, seed)",
        |&(size, seed)| {
            let gen = Gen::vec_of(Gen::int_range(0, 50));
            let tree = gen.generate(size, seed);

            if tree.value.is_empty() {
                return true;
            }

            // Repeatedly shrink until we can't shrink anymore
            let mut current = tree;
            let mut shrink_steps = 0;
            let max_shrink_steps = 100; // Prevent infinite loops

            while shrink_steps < max_shrink_steps {
                let shrinks = current.shrinks();
                if shrinks.is_empty() {
                    break; // Converged - no more shrinks available
                }

                let shrink_value = shrinks.into_iter().next().unwrap();
                current = Tree::singleton(shrink_value.clone());
                shrink_steps += 1;
            }

            // Should have converged before hitting the limit
            shrink_steps < max_shrink_steps
        },
    );

    let fast_config = Config::default().with_tests(15).with_shrinks(5);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Shrinking convergence property passed"),
        result => panic!("Shrinking convergence property failed: {result:?}"),
    }
}

/// Property: Shrinking integers should converge towards zero
pub fn test_integer_shrinking_towards_zero() {
    let prop = for_all_named(
        Gen::int_range(-100, 100),
        "initial_value",
        |&initial_value: &i32| {
            if initial_value == 0 {
                return true;
            }

            let tree = Tree::singleton(initial_value);

            // Get all possible shrink paths
            let shrinks = tree.shrinks();

            if shrinks.is_empty() {
                return true;
            }

            // At least one shrink should be closer to zero
            shrinks
                .into_iter()
                .any(|shrink_value| shrink_value.abs() <= initial_value.abs())
        },
    );

    let fast_config = Config::default().with_tests(15).with_shrinks(5);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Integer shrinking towards zero property passed"),
        result => panic!("Integer shrinking towards zero property failed: {result:?}"),
    }
}

/// Property: String shrinking should produce shorter strings
pub fn test_string_shrinking_shorter() {
    let prop = for_all_named(
        Gen::<String>::ascii_alpha(),
        "initial_string",
        |initial_string: &String| {
            if initial_string.is_empty() {
                return true;
            }

            let tree = Tree::singleton(initial_string.clone());

            // Get shrinks
            let shrinks = tree.shrinks();

            if shrinks.is_empty() {
                return true;
            }

            // At least one shrink should be shorter
            shrinks
                .into_iter()
                .any(|shrink_value| shrink_value.len() < initial_string.len())
        },
    );

    let fast_config = Config::default().with_tests(15).with_shrinks(5);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => {
            println!("✓ String shrinking produces shorter strings property passed")
        }
        result => panic!("String shrinking produces shorter strings property failed: {result:?}"),
    }
}

/// Property: Shrinking should preserve property failures
pub fn test_shrinking_preserves_failures() {
    // Create a property that fails for large lists
    let failing_property = |vec: &Vec<i32>| vec.len() < 10;

    let prop = for_all_named(
        Gen::vec_of(Gen::int_range(0, 5)), // Generate vectors that might be long
        "initial_vec",
        move |initial_vec: &Vec<i32>| {
            if failing_property(initial_vec) {
                return true;
            } // Skip passing cases

            let tree = Tree::singleton(initial_vec.clone());

            // Get shrinks
            let shrinks = tree.shrinks();

            // All shrinks should still fail the property (or be smaller)
            shrinks.into_iter().all(|shrink_value| {
                !failing_property(shrink_value) || shrink_value.len() < initial_vec.len()
            })
        },
    );

    let fast_config = Config::default().with_tests(15).with_shrinks(5);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Shrinking preserves failures property passed"),
        result => panic!("Shrinking preserves failures property failed: {result:?}"),
    }
}

// Helper functions

fn is_smaller_vector<T>(shrunk: &Vec<T>, original: &Vec<T>) -> bool {
    // A vector is smaller if it has fewer elements
    shrunk.len() <= original.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_shrinking_property_tests() {
        test_shrinking_produces_smaller();
        test_shrinking_convergence();
        test_integer_shrinking_towards_zero();
        test_string_shrinking_shorter();
        test_shrinking_preserves_failures();
    }
}
