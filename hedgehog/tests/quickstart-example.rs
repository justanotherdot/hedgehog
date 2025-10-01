// Test that the Quick Start example compiles and works
use hedgehog::*;

fn safe_divide(a: i32, b: i32) -> Option<i32> {
    if b == 0 {
        None
    } else if a == i32::MIN && b == -1 {
        None // Avoid overflow
    } else {
        Some(a / b)
    }
}

#[test]
fn prop_division_safety() {
    // Critical edge cases that must always be tested
    let critical_cases = vec![
        (10, 0),        // Division by zero
        (i32::MAX, 1),  // Maximum value
        (i32::MIN, -1), // Potential overflow
    ];

    let prop = for_all_named(
        Gen::<(i32, i32)>::tuple_of(Gen::int_range(-50, 50), Gen::int_range(-5, 5)),
        "input",
        |&(a, b)| match safe_divide(a, b) {
            Some(result) => b != 0 && !(a == i32::MIN && b == -1) && result == a / b,
            None => b == 0 || (a == i32::MIN && b == -1),
        },
    )
    .with_examples(critical_cases); // Examples tested first, then random pairs

    assert!(matches!(
        prop.run(&Config::default()),
        TestResult::Pass { .. }
    ));
}
