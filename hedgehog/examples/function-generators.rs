//! Function generator examples demonstrating how to test higher-order functions
//! and functional composition using Hedgehog's function generators.

use hedgehog::*;

fn main() {
    println!("🦔 Hedgehog Function Generator Examples\n");

    basic_function_generation();
    predicate_testing();
    comparator_testing();
}

fn basic_function_generation() {
    println!("=== Basic Function Generation ===");

    // Generate functions from integers to strings
    let input_gen = Gen::int_range(0, 5);
    let output_gen = Gen::<String>::ascii_alpha();
    let function_gen = Gen::<Box<dyn Fn(i32) -> String>>::function_of(
        input_gen,
        output_gen,
        "default".to_string(),
    );

    // Sample a generated function
    let seed = Seed::from_u64(42);
    let function_tree = function_gen.generate(Size::new(10), seed);
    let func = &function_tree.value;

    println!("Generated function mappings:");
    for i in 0..6 {
        let result = func(i);
        println!("  f({i}) = \"{result}\"");
    }
    println!("  f(999) = \"{}\" (default)\n", func(999));

    // Demonstrate shrinking
    let shrinks = function_tree.shrinks();
    println!("Function has {} possible shrinks", shrinks.len());
    if let Some(first_shrink) = shrinks.first() {
        println!("First shrink mapping f(0) = \"{}\"", first_shrink(0));
    }
    println!();
}

fn predicate_testing() {
    println!("=== Predicate Function Testing ===");

    // Generate predicates and test filter operations
    let accepted_values_gen = Gen::<Vec<i32>>::vec_of(Gen::int_range(1, 5));
    let predicate_gen = Gen::<Box<dyn Fn(i32) -> bool>>::predicate_from_set(accepted_values_gen);

    // Sample a predicate
    let seed = Seed::from_u64(123);
    let predicate_tree = predicate_gen.generate(Size::new(5), seed);
    let predicate = &predicate_tree.value;

    println!("Generated predicate behavior:");
    for i in 1..8 {
        let accepted = predicate(i);
        println!("  predicate({i}) = {accepted}");
    }

    // Test filter property: filtered elements satisfy predicate
    fn test_filter_property() {
        let list_gen = Gen::<Vec<i32>>::vec_of(Gen::int_range(1, 10));

        let property = for_all(list_gen, |list: &Vec<i32>| {
            // Create a simple predicate: accept even numbers
            let filtered: Vec<i32> = list.iter().cloned().filter(|x| x % 2 == 0).collect();
            filtered.iter().all(|&x| x % 2 == 0)
        });

        match property.run(&Config::default()) {
            TestResult::Pass { tests_run, .. } => {
                println!("✓ Filter property passed {tests_run} tests");
            }
            TestResult::Fail { counterexample, .. } => {
                println!("✗ Filter property failed with: {counterexample}");
            }
            _ => {}
        }
    }

    test_filter_property();
    println!();
}

fn comparator_testing() {
    println!("=== Comparator Function Testing ===");

    // Test constant comparator
    let comparator_gen = Gen::<Box<dyn Fn(i32, i32) -> std::cmp::Ordering>>::constant_comparator(
        std::cmp::Ordering::Equal,
    );
    let seed = Seed::from_u64(456);
    let comparator_tree = comparator_gen.generate(Size::new(10), seed);
    let comparator = &comparator_tree.value;

    println!("Generated constant comparator behavior:");
    println!("  cmp(1, 2) = {:?}", comparator(1, 2));
    println!("  cmp(5, 5) = {:?}", comparator(5, 5));
    println!("  cmp(10, 3) = {:?}", comparator(10, 3));

    // Test sorting property: sorted list is sorted
    fn test_sorting_property() {
        let list_gen = Gen::<Vec<i32>>::vec_of(Gen::int_range(1, 10));

        let property = for_all(list_gen, |list: &Vec<i32>| {
            let mut sorted_list = list.clone();
            sorted_list.sort(); // Use standard ordering

            // Check if sorted list is actually sorted
            for i in 0..sorted_list.len().saturating_sub(1) {
                if sorted_list[i] > sorted_list[i + 1] {
                    return false;
                }
            }
            true
        });

        match property.run(&Config::default()) {
            TestResult::Pass { tests_run, .. } => {
                println!("✓ Sorting property passed {tests_run} tests");
            }
            TestResult::Fail { counterexample, .. } => {
                println!("✗ Sorting property failed with: {counterexample}");
            }
            _ => {}
        }
    }

    test_sorting_property();
    println!();

    println!("Function generator examples completed! 🎉");
}
