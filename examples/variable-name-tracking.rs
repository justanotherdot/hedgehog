//! Demonstration of variable name tracking in failure reporting.

use hedgehog::*;

fn main() {
    println!("Variable Name Tracking in Failure Reporting");
    println!("===========================================");
    println!();

    // Example 1: Basic variable name tracking
    println!("1. Basic variable name tracking");
    println!("   Testing property with named variable 'n'");

    let prop = for_all_named(Gen::int_range(5, 20), "n", |&n| n < 10);
    match prop.run(&Config::default().with_tests(10)) {
        TestResult::Pass { .. } => println!("   Unexpectedly passed"),
        TestResult::Fail { .. } => {
            println!("   Property failed as expected - check the output format!");
            println!("   {}", prop.run(&Config::default().with_tests(10)));
        }
        _ => println!("   Unexpected result"),
    }
    println!();

    // Example 2: String variable name tracking
    println!("2. String variable name tracking");
    println!("   Testing property with named variable 'text'");

    let string_prop = for_all_named(
        Gen::<String>::alpha_with_range(Range::new(5, 15)),
        "text",
        |text| text.len() < 8,
    );
    match string_prop.run(&Config::default().with_tests(10)) {
        TestResult::Pass { .. } => println!("   Unexpectedly passed"),
        TestResult::Fail { .. } => {
            println!("   Property failed as expected - check the output format!");
            println!("   {}", string_prop.run(&Config::default().with_tests(10)));
        }
        _ => println!("   Unexpected result"),
    }
    println!();

    // Example 3: Multiple named variables (simulated)
    println!("3. Comparing with unnamed variable");
    println!("   Testing the same property without variable name");

    let unnamed_prop = for_all(Gen::int_range(5, 20), |&n| n < 10);
    match unnamed_prop.run(&Config::default().with_tests(10)) {
        TestResult::Pass { .. } => println!("   Unexpectedly passed"),
        TestResult::Fail { .. } => {
            println!("   Property failed as expected - notice the different format!");
            println!("   {}", unnamed_prop.run(&Config::default().with_tests(10)));
        }
        _ => println!("   Unexpected result"),
    }
    println!();

    // Example 4: Demonstrating the Haskell Hedgehog-style output
    println!("4. Understanding the output format");
    println!();
    println!("   With variable names (for_all_named):");
    println!("   ━━━ module_name ━━━");
    println!("     ✗ property failed after N tests and M shrinks.");
    println!();
    println!("       Shrinking progression:");
    println!("         │ forAll 0 = 15 -- n");
    println!("         │ forAll 1 = 12 -- n");
    println!("         │ forAll 2 = 10 -- n");
    println!();
    println!("   Without variable names (for_all):");
    println!("   ━━━ module_name ━━━");
    println!("     ✗ property failed after N tests and M shrinks.");
    println!();
    println!("       Shrinking progression:");
    println!("         │ Original: 15");
    println!("         │ Step 1: 12");
    println!("         │ Step 2: 10");
    println!();
    println!("   The variable names help identify which input caused the failure,");
    println!("   making debugging much easier in complex properties with multiple inputs.");
}
