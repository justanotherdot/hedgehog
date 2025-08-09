//! Example integration demonstration
//!
//! This example shows how to mix explicit test examples with property-based testing
//! to ensure critical edge cases are always tested while getting broad coverage.

use hedgehog::*;

fn main() {
    println!("=== Example Integration Demonstration ===\n");

    // Example 1: Testing a division function with critical edge cases
    example_division_safety();
    
    // Example 2: Testing string parsing with known problematic inputs
    example_string_parsing();
    
    // Example 3: Different integration strategies
    example_integration_strategies();
    
    // Example 4: Using examples to test regression cases
    example_regression_testing();
}

/// Example 1: Ensure division function handles edge cases properly
fn example_division_safety() {
    println!("1. Testing division with critical edge cases");
    
    fn safe_divide(a: i32, b: i32) -> Option<i32> {
        if b == 0 {
            None
        } else if a == i32::MIN && b == -1 {
            None // Avoid overflow
        } else {
            Some(a / b)
        }
    }
    
    // Test with explicit examples that are critical edge cases
    let critical_examples = vec![
        (10, 0),        // Division by zero
        (i32::MAX, 1),  // Maximum value
        (i32::MIN, -1), // Potential overflow  
        (0, 5),         // Zero dividend
        (7, 1),         // Identity division
    ];
    
    let prop = for_all(
        Gen::<(i32, i32)>::tuple_of(
            Gen::int_range(-100, 100),
            Gen::int_range(-10, 10)
        ), 
        |&(a, b)| {
            match safe_divide(a, b) {
                Some(result) => {
                    // If division succeeded, b must be non-zero and not overflow case
                    b != 0 && !(a == i32::MIN && b == -1) && result == a / b
                }
                None => {
                    // If division failed, either b is zero or it's the overflow case
                    b == 0 || (a == i32::MIN && b == -1)
                }
            }
        }
    ).with_examples(critical_examples);
    
    match prop.run(&Config::default().with_tests(50)) {
        TestResult::Pass { tests_run, .. } => {
            println!("   ✓ Division safety property passed {} tests", tests_run);
            println!("     (Including critical edge cases: div by 0, overflow, etc.)");
        }
        TestResult::Fail { counterexample, .. } => {
            println!("   ✗ Division safety failed with: {}", counterexample);
        }
        _ => {}
    }
    println!();
}

/// Example 2: String parsing with known problematic inputs
fn example_string_parsing() {
    println!("2. Testing string parsing with known edge cases");
    
    fn parse_positive_number(s: &str) -> std::result::Result<u32, String> {
        if s.is_empty() {
            return Err("Empty string".to_string());
        }
        
        s.parse::<u32>()
            .map_err(|_| format!("Invalid number: {}", s))
    }
    
    // Examples of strings that historically caused issues
    let problematic_examples = vec![
        "".to_string(),           // Empty string
        "0".to_string(),          // Zero (valid)
        "-1".to_string(),         // Negative (invalid for u32)
        "abc".to_string(),        // Non-numeric
        "123abc".to_string(),     // Mixed
        "4294967295".to_string(), // u32::MAX
        "4294967296".to_string(), // u32::MAX + 1 (overflow)
        " 123 ".to_string(),      // Whitespace
    ];
    
    let prop = for_all(
        Gen::<String>::ascii_printable(),
        |s| {
            let result = parse_positive_number(s);
            
            // Property: if parsing succeeds, it should be a valid positive number
            match result {
                Ok(n) => n > 0 || s == "0", // Zero is allowed  
                Err(_) => true, // Failures are okay, we're testing the function doesn't panic
            }
        }
    ).with_examples(problematic_examples);
    
    match prop.run(&Config::default().with_tests(100)) {
        TestResult::Pass { tests_run, .. } => {
            println!("   ✓ String parsing property passed {} tests", tests_run);
            println!("     (Including problematic inputs: empty, overflow, mixed content)");
        }
        TestResult::Fail { counterexample, .. } => {
            println!("   ✗ String parsing failed with: {}", counterexample);
        }
        _ => {}
    }
    println!();
}

/// Example 3: Different integration strategies
fn example_integration_strategies() {
    println!("3. Demonstrating different example integration strategies");
    
    let examples = vec![1, 5, 10];
    
    // Strategy 1: Examples first (default)
    println!("   Testing with ExamplesFirst strategy:");
    let prop1 = for_all(Gen::int_range(20, 30), |&x| x > 0)
        .with_examples(examples.clone());
    
    match prop1.run(&Config::default().with_tests(8)) {
        TestResult::Pass { tests_run, .. } => {
            println!("     ✓ Passed {} tests (examples tested first)", tests_run);
        }
        _ => {}
    }
    
    // Strategy 2: Mixed throughout
    println!("   Testing with Mixed strategy:");
    let prop2 = for_all(Gen::int_range(20, 30), |&x| x > 0)
        .with_examples_strategy(examples.clone(), ExampleStrategy::Mixed);
        
    match prop2.run(&Config::default().with_tests(12)) {
        TestResult::Pass { tests_run, .. } => {
            println!("     ✓ Passed {} tests (examples mixed throughout)", tests_run);
        }
        _ => {}
    }
    
    // Strategy 3: Generated values first, then examples
    println!("   Testing with GeneratedFirst strategy:");
    let prop3 = for_all(Gen::int_range(20, 30), |&x| x > 0)
        .with_examples_strategy(examples.clone(), ExampleStrategy::GeneratedFirst);
        
    match prop3.run(&Config::default().with_tests(10)) {
        TestResult::Pass { tests_run, .. } => {
            println!("     ✓ Passed {} tests (generated first, then examples)", tests_run);
        }
        _ => {}
    }
    
    // Strategy 4: Examples only for first 3 tests
    println!("   Testing with ExamplesUpTo(3) strategy:");
    let prop4 = for_all(Gen::int_range(20, 30), |&x| x > 0)
        .with_examples_strategy(examples, ExampleStrategy::ExamplesUpTo(3));
        
    match prop4.run(&Config::default().with_tests(10)) {
        TestResult::Pass { tests_run, .. } => {
            println!("     ✓ Passed {} tests (examples only in first 3)", tests_run);
        }
        _ => {}
    }
    println!();
}

/// Example 4: Using examples for regression testing
fn example_regression_testing() {
    println!("4. Using examples to prevent regressions");
    
    // Simulate a function that had bugs in the past
    fn process_data(data: Vec<i32>) -> Vec<i32> {
        let mut result = Vec::new();
        for &item in &data {
            // This function had bugs with these specific cases in the past
            if item == 0 {
                continue; // Skip zeros (this was a bug that got fixed)
            }
            if item < 0 {
                if item == i32::MIN {
                    result.push(i32::MIN); // Can't negate MIN, keep as-is
                } else {
                    result.push(item.abs()); // Convert negatives to positive
                }
            } else {
                result.push(item);
            }
        }
        result
    }
    
    // Examples from past bug reports that should always be tested
    let regression_examples = vec![
        vec![], // Empty list (caused panic in v1.0)
        vec![0], // Zero handling (was incorrectly processed in v1.1) 
        vec![-1, -5], // Negative handling (abs wasn't applied in v1.2)
        vec![0, 1, -2, 3], // Mixed case (combination issues in v1.3)
        vec![i32::MIN], // Extreme value (overflow in abs() in v1.4)
    ];
    
    let prop = for_all(
        Gen::<Vec<i32>>::vec_of(Gen::int_range(-50, 50)),
        |data| {
            let result = process_data(data.clone());
            
            // Properties that should hold after the fixes:
            // 1. No panics (implicit - if we get here, no panic occurred)
            // 2. All results should be non-negative (except i32::MIN -> still negative due to overflow)
            // 3. Length should be <= original (due to zero filtering)
            result.len() <= data.len() &&
            result.iter().all(|&x| x >= 0 || x == i32::MIN)
        }
    )
    .with_examples(regression_examples)
    .classify("empty_input", |data| data.is_empty())
    .classify("has_zeros", |data| data.contains(&0))
    .classify("has_negatives", |data| data.iter().any(|&x| x < 0))
    .collect("input_length", |data| data.len() as f64);
    
    match prop.run(&Config::default().with_tests(100)) {
        TestResult::PassWithStatistics { tests_run, statistics, .. } => {
            println!("   ✓ Regression testing passed {} tests", tests_run);
            println!("     Ensured all past bug cases are covered:");
            
            for (classification, count) in &statistics.classifications {
                let percentage = (*count as f64 / tests_run as f64) * 100.0;
                println!("     - {}: {:.1}% of tests", classification, percentage);
            }
            
            if let Some(lengths) = statistics.collections.get("input_length") {
                let avg_length = lengths.iter().sum::<f64>() / lengths.len() as f64;
                println!("     - Average input length: {:.1}", avg_length);
            }
        }
        TestResult::Fail { counterexample, .. } => {
            println!("   ✗ Regression testing failed with: {}", counterexample);
            println!("     This indicates a new bug or regression!");
        }
        _ => {}
    }
    
    println!("\n=== Example Integration Complete ===");
    println!("Examples show how explicit test cases ensure critical scenarios");
    println!("are always covered while property-based testing provides broad coverage.");
}