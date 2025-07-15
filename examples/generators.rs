//! Example demonstrating all available generators.

use hedgehog_core::*;

fn main() {
    println!("Testing all available generators");
    println!();

    // Test vector generators
    println!("Testing vector generators");
    let vec_gen = Gen::<Vec<i32>>::vec_of(Gen::int_range(-10, 10));
    let vec_prop = for_all(vec_gen, |v: &Vec<i32>| {
        v.iter().all(|&x| x >= -10 && x <= 10)
    });
    match vec_prop.run(&Config::default().with_tests(50)) {
        TestResult::Pass => println!("Vector bounds property passed"),
        result => println!("Vector bounds property failed: {:?}", result),
    }

    // Test vector integer convenience method
    println!("Testing vec_int convenience method");
    let vec_int_gen = Gen::<Vec<i32>>::vec_int();
    let vec_int_prop = for_all(vec_int_gen, |v: &Vec<i32>| {
        v.iter().all(|&x| x >= -100 && x <= 100)
    });
    match vec_int_prop.run(&Config::default().with_tests(50)) {
        TestResult::Pass => println!("Vec<i32> convenience property passed"),
        result => println!("Vec<i32> convenience property failed: {:?}", result),
    }

    // Test vector boolean convenience method
    println!("Testing vec_bool convenience method");
    let vec_bool_gen = Gen::<Vec<bool>>::vec_bool();
    let vec_bool_prop = for_all(vec_bool_gen, |v: &Vec<bool>| {
        v.iter().all(|&b| b == true || b == false)
    });
    match vec_bool_prop.run(&Config::default().with_tests(50)) {
        TestResult::Pass => println!("Vec<bool> convenience property passed"),
        result => println!("Vec<bool> convenience property failed: {:?}", result),
    }
    println!();

    // Test option generators
    println!("Testing option generators");
    let option_gen = Gen::<Option<i32>>::option_of(Gen::int_range(1, 100));
    let option_prop = for_all(option_gen, |opt: &Option<i32>| match opt {
        None => true,
        Some(x) => *x >= 1 && *x <= 100,
    });
    match option_prop.run(&Config::default().with_tests(50)) {
        TestResult::Pass => println!("Option<i32> bounds property passed"),
        result => println!("Option<i32> bounds property failed: {:?}", result),
    }

    // Test that option generators produce both Some and None
    println!("Testing option distribution");
    let option_test_gen = Gen::<Option<bool>>::option_of(Gen::bool());
    let option_some_prop = for_all(option_test_gen, |opt: &Option<bool>| {
        opt.is_some() // This should fail, proving we get None values
    });
    match option_some_prop.run(&Config::default().with_tests(100)) {
        TestResult::Fail { .. } => println!("Option produces None values (expected failure)"),
        TestResult::Pass => println!("WARNING: Option generator only produced Some values"),
        result => println!("Unexpected result: {:?}", result),
    }
    println!();

    // Test tuple generators
    println!("Testing tuple generators");
    let tuple_gen = Gen::<(i32, bool)>::tuple_of(Gen::int_range(-50, 50), Gen::bool());
    let tuple_prop = for_all(tuple_gen, |(x, b): &(i32, bool)| {
        *x >= -50 && *x <= 50 && (*b == true || *b == false)
    });
    match tuple_prop.run(&Config::default().with_tests(50)) {
        TestResult::Pass => println!("Tuple (i32, bool) property passed"),
        result => println!("Tuple (i32, bool) property failed: {:?}", result),
    }

    // Test nested structures
    println!("Testing nested structures");
    let nested_gen = Gen::<Vec<Option<String>>>::vec_of(Gen::<Option<String>>::option_of(
        Gen::<String>::ascii_alpha(),
    ));
    let nested_prop = for_all(nested_gen, |v: &Vec<Option<String>>| {
        v.iter().all(|opt| match opt {
            None => true,
            Some(s) => s.chars().all(|c| c.is_ascii_alphabetic()),
        })
    });
    match nested_prop.run(&Config::default().with_tests(30)) {
        TestResult::Pass => println!("Nested Vec<Option<String>> property passed"),
        result => println!("Nested Vec<Option<String>> property failed: {:?}", result),
    }

    // Test tuple with vectors
    println!("Testing complex tuple with vectors");
    let complex_tuple_gen = Gen::<(Vec<i32>, String)>::tuple_of(
        Gen::<Vec<i32>>::vec_int(),
        Gen::<String>::ascii_alphanumeric(),
    );
    let complex_tuple_prop = for_all(complex_tuple_gen, |(vec, string): &(Vec<i32>, String)| {
        vec.iter().all(|&x| x >= -100 && x <= 100)
            && string.chars().all(|c| c.is_ascii_alphanumeric())
    });
    match complex_tuple_prop.run(&Config::default().with_tests(30)) {
        TestResult::Pass => println!("Complex tuple (Vec<i32>, String) property passed"),
        result => println!(
            "Complex tuple (Vec<i32>, String) property failed: {:?}",
            result
        ),
    }

    // Test Result generators
    println!("Testing Result generators");
    let result_gen = Gen::<std::result::Result<i32, String>>::result_of(
        Gen::int_range(1, 100),
        Gen::<String>::ascii_alpha(),
    );
    let result_prop = for_all(result_gen, |r: &std::result::Result<i32, String>| match r {
        Ok(n) => *n >= 1 && *n <= 100,
        Err(s) => s.chars().all(|c| c.is_ascii_alphabetic()),
    });
    match result_prop.run(&Config::default().with_tests(50)) {
        TestResult::Pass => println!("Result<i32, String> property passed"),
        result => println!("Result<i32, String> property failed: {:?}", result),
    }

    // Test that Result generators produce both Ok and Err
    println!("Testing Result distribution");
    let result_test_gen =
        Gen::<std::result::Result<bool, i32>>::result_of(Gen::bool(), Gen::int_range(-10, 10));
    let result_ok_prop = for_all(result_test_gen, |r: &std::result::Result<bool, i32>| {
        r.is_ok() // This should fail, proving we get Err values
    });
    match result_ok_prop.run(&Config::default().with_tests(100)) {
        TestResult::Fail { .. } => println!("Result produces Err values (expected failure)"),
        TestResult::Pass => println!("WARNING: Result generator only produced Ok values"),
        result => println!("Unexpected result: {:?}", result),
    }

    // Test weighted Result generator
    println!("Testing weighted Result generator");
    let weighted_result_gen = Gen::<std::result::Result<i32, String>>::result_of_weighted(
        Gen::int_range(1, 10),
        Gen::<String>::ascii_alpha(),
        9, // 90% Ok, 10% Err
    );
    let weighted_result_prop = for_all(
        weighted_result_gen,
        |r: &std::result::Result<i32, String>| {
            match r {
                Ok(n) => *n >= 1 && *n <= 10,
                Err(_s) => true, // Any string is valid
            }
        },
    );
    match weighted_result_prop.run(&Config::default().with_tests(30)) {
        TestResult::Pass => println!("Weighted Result generator property passed"),
        result => println!("Weighted Result generator property failed: {:?}", result),
    }

    println!();
    println!("Generator testing complete");
}
