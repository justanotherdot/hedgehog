//! Basic example demonstrating Hedgehog property-based testing.

use hedgehog::*;

fn main() {
    println!("Hedgehog Property-Based Testing Examples");
    println!();

    // Example 1: Simple boolean property
    println!("Testing boolean property: all booleans are either true or false");
    let bool_gen = Gen::bool();
    let bool_prop = for_all(bool_gen, |&b| b == true || b == false);
    match bool_prop.run(&Config::default()) {
        TestResult::Pass { .. } => println!("Boolean property passed"),
        result => println!("Boolean property failed: {:?}", result),
    }
    println!();

    // Example 2: Integer property with shrinking
    println!("Testing integer property: x + 0 = x");
    let int_gen = Gen::int_range(-100, 100);
    let addition_prop = for_all(int_gen, |&x| x + 0 == x);
    match addition_prop.run(&Config::default()) {
        TestResult::Pass { .. } => println!("Addition identity property passed"),
        result => println!("Addition identity property failed: {:?}", result),
    }
    println!();

    // Example 3: Property that should fail (to demonstrate shrinking)
    println!("Testing property that should fail: all integers are positive");
    let pos_gen = Gen::int_range(-10, 10);
    let positive_prop = for_all(pos_gen, |&x| x > 0);
    match positive_prop.run(&Config::default().with_tests(20)) {
        TestResult::Pass { .. } => println!("Positive property passed (unexpected)"),
        TestResult::Fail {
            counterexample,
            tests_run,
            shrinks_performed,
            ..
        } => {
            println!("Positive property failed as expected:");
            println!("  Counterexample: {}", counterexample);
            println!("  Tests run: {}", tests_run);
            println!("  Shrinks performed: {}", shrinks_performed);
        }
        result => println!("Unexpected result: {:?}", result),
    }
    println!();

    // Example 4: Combining generators with map
    println!("Testing mapped generator: absolute value is always non-negative");
    let abs_gen = Gen::int_range(-50, 50).map(|x| x.abs());
    let abs_prop = for_all(abs_gen, |&x| x >= 0);
    match abs_prop.run(&Config::default()) {
        TestResult::Pass { .. } => println!("Absolute value property passed"),
        result => println!("Absolute value property failed: {:?}", result),
    }
    println!();

    // Example 5: String generators
    println!("Testing string property: all strings contain themselves");
    let string_gen = Gen::<String>::ascii_alpha();
    let string_prop = for_all(string_gen, |s: &String| s.contains(s));
    match string_prop.run(&Config::default()) {
        TestResult::Pass { .. } => println!("String contains property passed"),
        result => println!("String contains property failed: {:?}", result),
    }
    println!();

    // Example 6: Vector generators
    println!("Testing vector property: length matches element count");
    let vec_gen = Gen::<Vec<i32>>::vec_int();
    let vec_prop = for_all(vec_gen, |v: &Vec<i32>| v.len() == v.iter().count());
    match vec_prop.run(&Config::default()) {
        TestResult::Pass { .. } => println!("Vector length property passed"),
        result => println!("Vector length property failed: {:?}", result),
    }
    println!();

    // Example 7: Option generators
    println!("Testing option property: map preserves structure");
    let option_gen = Gen::<Option<i32>>::option_of(Gen::int_range(0, 100));
    let option_prop = for_all(option_gen, |opt: &Option<i32>| {
        let doubled = opt.map(|x| x * 2);
        opt.is_none() == doubled.is_none()
    });
    match option_prop.run(&Config::default()) {
        TestResult::Pass { .. } => println!("Option map property passed"),
        result => println!("Option map property failed: {:?}", result),
    }
    println!();

    // Example 8: Tuple generators
    println!("Testing tuple property: components are accessible");
    let tuple_gen =
        Gen::<(String, i32)>::tuple_of(Gen::<String>::ascii_alpha(), Gen::int_range(1, 10));
    let tuple_prop = for_all(tuple_gen, |(_s, n): &(String, i32)| *n >= 1 && *n <= 10);
    match tuple_prop.run(&Config::default()) {
        TestResult::Pass { .. } => println!("Tuple property passed"),
        result => println!("Tuple property failed: {:?}", result),
    }
}
