//! Example demonstrating enhanced shrinking strategies.

use hedgehog_core::*;

fn main() {
    println!("Testing enhanced shrinking strategies");
    println!();

    // Integer shrinking: should shrink towards origin
    println!("Testing integer shrinking (should fail and show shrinking)");
    let int_gen = Gen::int_range(-20, 20);
    let int_prop = for_all(int_gen, |&x| x == 0); // Will fail, showing shrinking to 0
    match int_prop.run(&Config::default().with_tests(50)) {
        TestResult::Fail {
            counterexample,
            shrinks_performed,
            ..
        } => {
            println!(
                "Integer shrinking worked: {}, shrinks: {}",
                counterexample, shrinks_performed
            );
        }
        result => println!("Unexpected result: {:?}", result),
    }
    println!();

    // String shrinking: should simplify characters and remove them
    println!("Testing string shrinking (should fail and show character simplification)");
    let string_gen = Gen::<String>::ascii_printable();
    let string_prop = for_all(string_gen, |s: &String| s.is_empty()); // Will fail, showing shrinking
    match string_prop.run(&Config::default().with_tests(30)) {
        TestResult::Fail {
            counterexample,
            shrinks_performed,
            ..
        } => {
            println!(
                "String shrinking worked: '{}', shrinks: {}",
                counterexample, shrinks_performed
            );
        }
        result => println!("Unexpected result: {:?}", result),
    }
    println!();

    // Vector shrinking: should remove elements and shrink individual elements
    println!("Testing vector shrinking (should fail and show element removal)");
    let vec_gen = Gen::<Vec<i32>>::vec_of(Gen::int_range(10, 100));
    let vec_prop = for_all(vec_gen, |v: &Vec<i32>| v.is_empty()); // Will fail, showing shrinking
    match vec_prop.run(&Config::default().with_tests(30)) {
        TestResult::Fail {
            counterexample,
            shrinks_performed,
            ..
        } => {
            println!(
                "Vector shrinking worked: {:?}, shrinks: {}",
                counterexample, shrinks_performed
            );
        }
        result => println!("Unexpected result: {:?}", result),
    }
    println!();

    // Option shrinking: should shrink to None
    println!("Testing Option shrinking (should fail and show shrinking to None)");
    let option_gen = Gen::<Option<String>>::option_of(Gen::<String>::ascii_alpha());
    let option_prop = for_all(option_gen, |opt: &Option<String>| opt.is_none()); // Will fail on Some
    match option_prop.run(&Config::default().with_tests(50)) {
        TestResult::Fail {
            counterexample,
            shrinks_performed,
            ..
        } => {
            println!(
                "Option shrinking worked: {:?}, shrinks: {}",
                counterexample, shrinks_performed
            );
        }
        result => println!("Unexpected result: {:?}", result),
    }
    println!();

    // Result shrinking: should try to shrink to Ok values
    println!("Testing Result shrinking (should fail and show shrinking to Ok)");
    let result_gen = Gen::<std::result::Result<i32, String>>::result_of(
        Gen::int_range(1, 5),
        Gen::<String>::ascii_alpha(),
    );
    let result_prop = for_all(result_gen, |r: &std::result::Result<i32, String>| {
        matches!(r, Err(_)) // Will fail on Ok values, showing shrinking
    });
    match result_prop.run(&Config::default().with_tests(50)) {
        TestResult::Fail {
            counterexample,
            shrinks_performed,
            ..
        } => {
            println!(
                "Result shrinking worked: {:?}, shrinks: {}",
                counterexample, shrinks_performed
            );
        }
        result => println!("Unexpected result: {:?}", result),
    }
    println!();

    // Complex nested shrinking
    println!("Testing complex nested shrinking");
    let nested_gen = Gen::<Vec<Option<String>>>::vec_of(Gen::<Option<String>>::option_of(
        Gen::<String>::ascii_alphanumeric(),
    ));
    let nested_prop = for_all(nested_gen, |v: &Vec<Option<String>>| {
        v.len() < 2 && v.iter().all(|opt| opt.is_none()) // Very restrictive, will show deep shrinking
    });
    match nested_prop.run(&Config::default().with_tests(20)) {
        TestResult::Fail {
            counterexample,
            shrinks_performed,
            ..
        } => {
            println!(
                "Complex nested shrinking worked: {:?}, shrinks: {}",
                counterexample, shrinks_performed
            );
        }
        result => println!("Unexpected result: {:?}", result),
    }

    println!();
    println!("Shrinking demonstration complete");
}
