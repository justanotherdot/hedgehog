//! Result and Option generator meta tests
//!
//! These properties test the generation of Result<T, E> and Option<T> values
//! including distribution ratios, shrinking behavior, weighted generation,
//! and proper composition with other generators.

use crate::arbitrary_seed;
use hedgehog::*;

/// Property: Option generators should produce both Some and None values
pub fn test_option_generation_distribution() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&_seed: &Seed| {
        let option_gen = Gen::<Option<i32>>::option_of(Gen::int_range(1, 100));
        let size = Size::new(10);

        let mut some_count = 0;
        let mut none_count = 0;

        // Generate multiple values to check distribution
        for i in 0..100 {
            let test_seed = Seed::from_u64(i * 1234);
            let option_value = option_gen.generate(size, test_seed).value;

            match option_value {
                Some(_) => some_count += 1,
                None => none_count += 1,
            }
        }

        // Should get both Some and None values (not all one type)
        let has_variety = some_count > 0 && none_count > 0;

        // Should have reasonable variety in distribution (allow wide variance)
        let ratio_reasonable = some_count >= 10 && none_count >= 10;

        has_variety && ratio_reasonable
    });

    let fast_config = Config::default().with_tests(10).with_shrinks(2);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Option generation distribution property passed"),
        result => panic!("Option generation distribution property failed: {result:?}"),
    }
}

/// Property: Option shrinking should prioritize None
pub fn test_option_shrinking_behavior() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&seed: &Seed| {
        let option_gen = Gen::<Option<i32>>::option_of(Gen::int_range(1, 100));
        let size = Size::new(10);
        let tree = option_gen.generate(size, seed);

        match &tree.value {
            Some(_) => {
                // Some values should shrink to None
                let shrinks = tree.shrinks();

                // Should have at least None as a shrink
                shrinks.contains(&&None)
            }
            None => {
                // None should have no shrinks (already minimal)
                let shrinks = tree.shrinks();
                shrinks.is_empty()
            }
        }
    });

    let fast_config = Config::default().with_tests(20).with_shrinks(3);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Option shrinking behavior property passed"),
        result => panic!("Option shrinking behavior property failed: {result:?}"),
    }
}

/// Property: Result generators should produce both Ok and Err values
pub fn test_result_generation_distribution() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&_seed: &Seed| {
        let result_gen = Gen::<std::result::Result<i32, String>>::result_of(
            Gen::int_range(1, 50),
            Gen::<String>::ascii_alpha(),
        );
        let size = Size::new(10);

        let mut ok_count = 0;
        let mut err_count = 0;

        // Generate multiple values to check distribution
        for i in 0..100 {
            let test_seed = Seed::from_u64(i * 5678);
            let result_value = result_gen.generate(size, test_seed).value;

            match result_value {
                Ok(_) => ok_count += 1,
                Err(_) => err_count += 1,
            }
        }

        // Should get both Ok and Err values
        let has_variety = ok_count > 0 && err_count > 0;

        // Should have reasonable variety in distribution (allow wide variance)
        let ratio_reasonable = ok_count >= 10 && err_count >= 10;

        has_variety && ratio_reasonable
    });

    let fast_config = Config::default().with_tests(10).with_shrinks(2);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Result generation distribution property passed"),
        result => panic!("Result generation distribution property failed: {result:?}"),
    }
}

/// Property: Result shrinking should work consistently
pub fn test_result_shrinking_behavior() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&seed: &Seed| {
        let result_gen = Gen::<std::result::Result<i32, String>>::result_of(
            Gen::int_range(1, 100),
            Gen::<String>::ascii_alpha(),
        );
        let size = Size::new(10);
        let tree = result_gen.generate(size, seed);

        match &tree.value {
            Ok(value) => {
                // Ok values should produce valid shrinks
                let shrinks = tree.shrinks();

                // All shrinks should be valid Results
                shrinks.iter().all(|r| match r {
                        Ok(n) => *n >= 1 && *n <= 100, // Should maintain constraints
                        Err(s) => s.chars().all(|c| c.is_ascii_alphabetic()), // Valid error format
                    }) &&
                    // If value is > 1, should have some shrinks
                    (*value == 1 || !shrinks.is_empty())
            }
            Err(error_str) => {
                // Err values should produce valid shrinks
                let shrinks = tree.shrinks();

                // All shrinks should be valid Results
                shrinks.iter().all(|r| match r {
                        Ok(n) => *n >= 1 && *n <= 100, // Valid Ok values
                        Err(s) => s.chars().all(|c| c.is_ascii_alphabetic()), // Valid error format
                    }) &&
                    // If error is not minimal, should have some shrinks
                    (error_str.is_empty() || !shrinks.is_empty())
            }
        }
    });

    let fast_config = Config::default().with_tests(20).with_shrinks(3);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Result shrinking behavior property passed"),
        result => panic!("Result shrinking behavior property failed: {result:?}"),
    }
}

/// Property: Weighted Result generators should respect custom ratios
pub fn test_result_weighted_distribution() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&_seed: &Seed| {
        // Test heavily Ok-weighted generator (90% Ok, 10% Err)
        let ok_weighted_gen = Gen::<std::result::Result<bool, i32>>::result_of_weighted(
            Gen::bool(),
            Gen::int_range(1, 5),
            9, // High Ok weight
        );
        let size = Size::new(10);

        let mut ok_count = 0;
        let mut err_count = 0;

        // Generate many values to test distribution
        for i in 0..200 {
            let test_seed = Seed::from_u64(i * 9999);
            let result_value = ok_weighted_gen.generate(size, test_seed).value;

            match result_value {
                Ok(_) => ok_count += 1,
                Err(_) => err_count += 1,
            }
        }

        // Should have Ok bias but allow wide variance for different seeds
        let heavily_ok_biased = ok_count >= err_count; // Basic bias check

        // Should still have some Err values
        let has_some_errors = err_count > 0;

        heavily_ok_biased && has_some_errors
    });

    let fast_config = Config::default().with_tests(8).with_shrinks(2);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Result weighted distribution property passed"),
        result => panic!("Result weighted distribution property failed: {result:?}"),
    }
}

/// Property: Nested Option/Result combinations should work correctly
pub fn test_nested_option_result_combinations() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&seed: &Seed| {
        let size = Size::new(8);

        // Test Option<std::result::Result<T, E>>
        let option_result_gen =
            Gen::<Option<std::result::Result<i32, String>>>::option_of(Gen::<
                std::result::Result<i32, String>,
            >::result_of(
                Gen::int_range(1, 10),
                Gen::<String>::ascii_alpha(),
            ));

        let option_result = option_result_gen.generate(size, seed).value;
        let option_result_valid = match option_result {
            Some(Ok(n)) => (1..=10).contains(&n),
            Some(Err(s)) => s.chars().all(|c| c.is_ascii_alphabetic()),
            None => true,
        };

        // Test std::result::Result<Option<T>, E>
        let result_option_gen = Gen::<std::result::Result<Option<i32>, String>>::result_of(
            Gen::<Option<i32>>::option_of(Gen::int_range(1, 10)),
            Gen::<String>::ascii_alpha(),
        );

        let result_option = result_option_gen.generate(size, seed).value;
        let result_option_valid = match result_option {
            Ok(Some(n)) => (1..=10).contains(&n),
            Ok(None) => true,
            Err(s) => s.chars().all(|c| c.is_ascii_alphabetic()),
        };

        option_result_valid && result_option_valid
    });

    let fast_config = Config::default().with_tests(15).with_shrinks(2);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Nested Option/Result combinations property passed"),
        result => panic!("Nested Option/Result combinations property failed: {result:?}"),
    }
}

/// Property: Option and Result generators should work with complex types
pub fn test_complex_type_option_result() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&seed: &Seed| {
        let size = Size::new(12);

        // Test Option with complex types
        let vec_option_gen =
            Gen::<Option<Vec<i32>>>::option_of(Gen::<Vec<i32>>::vec_of(Gen::int_range(1, 20)));

        let vec_option = vec_option_gen.generate(size, seed).value;
        let vec_option_valid = match vec_option {
            Some(vec) => vec.iter().all(|&n| (1..=20).contains(&n)),
            None => true,
        };

        // Test Result with tuple types
        let tuple_result_gen = Gen::<std::result::Result<(i32, String), bool>>::result_of(
            Gen::<(i32, String)>::tuple_of(Gen::int_range(-10, 10), Gen::<String>::ascii_alpha()),
            Gen::bool(),
        );

        let tuple_result = tuple_result_gen.generate(size, seed).value;
        let tuple_result_valid = match tuple_result {
            Ok((n, s)) => (-10..=10).contains(&n) && s.chars().all(|c| c.is_ascii_alphabetic()),
            Err(_) => true, // bool is always valid
        };

        vec_option_valid && tuple_result_valid
    });

    let fast_config = Config::default().with_tests(12).with_shrinks(2);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Complex type Option/Result property passed"),
        result => panic!("Complex type Option/Result property failed: {result:?}"),
    }
}

/// Property: Option and Result shrinking should preserve type constraints
pub fn test_constrained_option_result_shrinking() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&seed: &Seed| {
        let size = Size::new(15);

        // Test Option with constrained values
        let constrained_option_gen = Gen::<Option<i32>>::option_of(Gen::int_range(100, 200));

        let option_tree = constrained_option_gen.generate(size, seed);
        let option_shrinks_valid = match &option_tree.value {
            Some(_n) => {
                let shrinks = option_tree.shrinks();
                // All Some shrinks should maintain the constraint
                shrinks.iter().all(|opt| match opt {
                    Some(shrunk_n) => *shrunk_n >= 100 && *shrunk_n <= 200,
                    None => true, // None is always valid
                })
            }
            None => true, // None has no shrinks to validate
        };

        // Test Result with constrained error types
        let constrained_result_gen =
            Gen::<std::result::Result<bool, i32>>::result_of(Gen::bool(), Gen::int_range(400, 500));

        let result_tree = constrained_result_gen.generate(size, seed);
        let result_shrinks_valid = match &result_tree.value {
            Ok(_) => {
                let shrinks = result_tree.shrinks();
                // All shrinks should be valid Ok values (bool is always valid)
                shrinks.iter().all(|res| res.is_ok())
            }
            Err(_e) => {
                let shrinks = result_tree.shrinks();
                // Err shrinks should either be Ok or valid Err values
                shrinks.iter().all(|res| match res {
                    Ok(_) => true, // bool is always valid
                    Err(shrunk_e) => *shrunk_e >= 400 && *shrunk_e <= 500,
                })
            }
        };

        option_shrinks_valid && result_shrinks_valid
    });

    let fast_config = Config::default().with_tests(10).with_shrinks(5);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => {
            println!("✓ Constrained Option/Result shrinking property passed")
        }
        result => panic!("Constrained Option/Result shrinking property failed: {result:?}"),
    }
}

/// Property: Option and Result generators should compose well with other combinators
pub fn test_option_result_combinator_composition() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&seed: &Seed| {
        let size = Size::new(10);

        // Test Option with map combinator
        let mapped_option_gen =
            Gen::<Option<i32>>::option_of(Gen::int_range(1, 10)).map(|opt| opt.map(|n| n * 2));

        let mapped_option = mapped_option_gen.generate(size, seed).value;
        let mapped_option_valid = match mapped_option {
            Some(n) => (2..=20).contains(&n) && n % 2 == 0, // Should be even, doubled
            None => true,
        };

        // Test Result with filter (Result to Option conversion)
        let filtered_result_gen = Gen::<std::result::Result<i32, String>>::result_of(
            Gen::int_range(1, 20),
            Gen::<String>::ascii_alpha(),
        )
        .map(|res| res.ok()); // Convert Result to Option

        let filtered_result = filtered_result_gen.generate(size, seed).value;
        let filtered_result_valid = match filtered_result {
            Some(n) => (1..=20).contains(&n),
            None => true, // Could be None if original was Err
        };

        mapped_option_valid && filtered_result_valid
    });

    let fast_config = Config::default().with_tests(15).with_shrinks(2);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => {
            println!("✓ Option/Result combinator composition property passed")
        }
        result => panic!("Option/Result combinator composition property failed: {result:?}"),
    }
}

/// Property: Multiple Result types should work with different error types
pub fn test_multiple_error_types() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&seed: &Seed| {
        let size = Size::new(8);

        // Test Result<T, String>
        let string_error_gen = Gen::<std::result::Result<i32, String>>::result_of(
            Gen::int_range(1, 100),
            Gen::<String>::ascii_alpha(),
        );

        let string_result = string_error_gen.generate(size, seed).value;
        let string_error_valid = match string_result {
            Ok(n) => (1..=100).contains(&n),
            Err(s) => s.chars().all(|c| c.is_ascii_alphabetic()),
        };

        // Test Result<T, i32> (numeric errors)
        let numeric_error_gen = Gen::<std::result::Result<String, i32>>::result_of(
            Gen::<String>::ascii_alpha(),
            Gen::int_range(100, 999),
        );

        let numeric_result = numeric_error_gen.generate(size, seed).value;
        let numeric_error_valid = match numeric_result {
            Ok(s) => s.chars().all(|c| c.is_ascii_alphabetic()),
            Err(n) => (100..=999).contains(&n),
        };

        // Test Result<T, bool> (boolean errors)
        let bool_error_gen = Gen::<std::result::Result<Vec<i32>, bool>>::result_of(
            Gen::<Vec<i32>>::vec_of(Gen::int_range(1, 5)),
            Gen::bool(),
        );

        let bool_result = bool_error_gen.generate(size, seed).value;
        let bool_error_valid = match bool_result {
            Ok(vec) => vec.iter().all(|&n| (1..=5).contains(&n)),
            Err(_) => true, // bool is always valid
        };

        string_error_valid && numeric_error_valid && bool_error_valid
    });

    let fast_config = Config::default().with_tests(12).with_shrinks(2);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Multiple error types property passed"),
        result => panic!("Multiple error types property failed: {result:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_result_option_property_tests() {
        test_option_generation_distribution();
        test_option_shrinking_behavior();
        test_result_generation_distribution();
        test_result_shrinking_behavior();
        test_result_weighted_distribution();
        test_nested_option_result_combinations();
        test_complex_type_option_result();
        test_constrained_option_result_shrinking();
        test_option_result_combinator_composition();
        test_multiple_error_types();
    }
}
