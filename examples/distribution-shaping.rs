//! Demonstration of distribution shaping and range system.

use hedgehog::*;

fn main() {
    println!("Distribution Shaping and Range System Examples");
    println!("==============================================");
    println!();

    // Example 1: Frequency-based choice generators
    println!("1. Frequency-based choice generators");
    println!(
        "   Creating a generator that produces mostly small positive numbers with occasional zeros"
    );

    let weighted_gen = Gen::frequency(vec![
        WeightedChoice::new(1, Gen::constant(0)),      // 10% zeros
        WeightedChoice::new(9, Gen::int_range(1, 10)), // 90% small positive
    ])
    .expect("valid frequency generator");

    println!("   Generated values:");
    for i in 0..10 {
        let seed = Seed::from_u64(i);
        let tree = weighted_gen.generate(Size::new(10), seed);
        println!("     {}", tree.outcome());
    }
    println!();

    // Example 2: Distribution shapes for numeric ranges
    println!("2. Distribution shapes for numeric ranges");

    // Uniform distribution (equal probability across range)
    let uniform_gen = Gen::<i32>::from_range(Range::new(1, 100));
    println!("   Uniform distribution [1, 100]:");
    for i in 0..5 {
        let seed = Seed::from_u64(i);
        let tree = uniform_gen.generate(Size::new(10), seed);
        println!("     {}", tree.outcome());
    }
    println!();

    // Linear distribution (favors smaller values)
    let linear_gen = Gen::<i32>::from_range(Range::linear(1, 100));
    println!("   Linear distribution [1, 100] (favors smaller values):");
    for i in 0..5 {
        let seed = Seed::from_u64(i);
        let tree = linear_gen.generate(Size::new(10), seed);
        println!("     {}", tree.outcome());
    }
    println!();

    // Exponential distribution (strongly favors smaller values)
    let exponential_gen = Gen::<i32>::from_range(Range::exponential(1, 100));
    println!("   Exponential distribution [1, 100] (strongly favors smaller values):");
    for i in 0..5 {
        let seed = Seed::from_u64(i);
        let tree = exponential_gen.generate(Size::new(10), seed);
        println!("     {}", tree.outcome());
    }
    println!();

    // Constant distribution (always same value)
    let constant_gen = Gen::<i32>::from_range(Range::constant(42));
    println!("   Constant distribution (always 42):");
    for i in 0..3 {
        let seed = Seed::from_u64(i);
        let tree = constant_gen.generate(Size::new(10), seed);
        println!("     {}", tree.outcome());
    }
    println!();

    // Example 3: Controlled string length generation
    println!("3. Controlled string length generation");

    // Using the general with_range method
    let short_strings = Gen::<String>::with_range(Range::new(1, 5), Gen::<char>::ascii_alpha());
    println!("   Short strings (1-5 characters):");
    for i in 0..5 {
        let seed = Seed::from_u64(i);
        let tree = short_strings.generate(Size::new(10), seed);
        println!(
            "     '{}' (length: {})",
            tree.outcome(),
            tree.outcome().len()
        );
    }
    println!();

    // Using the cleaner convenience method
    let medium_strings = Gen::<String>::alpha_with_range(Range::linear(5, 15));
    println!("   Medium strings (5-15 characters, linear distribution):");
    for i in 0..5 {
        let seed = Seed::from_u64(i);
        let tree = medium_strings.generate(Size::new(10), seed);
        println!(
            "     '{}' (length: {})",
            tree.outcome(),
            tree.outcome().len()
        );
    }
    println!();

    // Example 4: Floating point ranges
    println!("4. Floating point ranges");

    let unit_floats = Gen::<f64>::from_range(Range::unit()); // [0.0, 1.0]
    println!("   Unit range [0.0, 1.0]:");
    for i in 0..5 {
        let seed = Seed::from_u64(i);
        let tree = unit_floats.generate(Size::new(10), seed);
        println!("     {:.4}", tree.outcome());
    }
    println!();

    let positive_floats = Gen::<f64>::from_range(Range::<f64>::positive()); // Exponential bias
    println!("   Positive range (exponential distribution):");
    for i in 0..5 {
        let seed = Seed::from_u64(i);
        let tree = positive_floats.generate(Size::new(10), seed);
        println!("     {:.4}", tree.outcome());
    }
    println!();

    // Example 5: Property testing with realistic distributions
    println!("5. Property testing with realistic distributions");

    // Test with mostly non-empty strings (avoids the empty string problem)
    let non_empty_strings = Gen::frequency(vec![
        WeightedChoice::new(1, Gen::constant(String::new())), // 5% empty
        WeightedChoice::new(19, Gen::<String>::alpha_with_range(Range::linear(1, 20))), // 95% non-empty
    ])
    .expect("valid frequency generator");

    let string_length_prop = for_all(non_empty_strings, |s: &String| {
        // This property would fail more often with naive string generation
        // but should mostly pass with our weighted approach
        s.len() <= 20
    });

    match string_length_prop.run(&Config::default().with_tests(50)) {
        TestResult::Pass { tests_run, .. } => {
            println!("   String length property passed {tests_run} tests");
        }
        TestResult::Fail { counterexample, .. } => {
            println!("   String length property failed with: '{counterexample}'");
        }
        _ => println!("   Unexpected result"),
    }
    println!();

    // Example 6: One-of generator for simple choices
    println!("6. One-of generator for simple choices");

    let simple_choices = Gen::one_of(vec![
        Gen::constant("red"),
        Gen::constant("green"),
        Gen::constant("blue"),
    ])
    .expect("valid one_of generator");

    println!("   Color choices:");
    for i in 0..8 {
        let seed = Seed::from_u64(i);
        let tree = simple_choices.generate(Size::new(10), seed);
        println!("     {}", tree.outcome());
    }
    println!();

    println!("Distribution shaping provides:");
    println!("- Realistic test data that matches real-world distributions");
    println!("- Better control over edge cases and boundary conditions");
    println!("- Reduced likelihood of pathological cases (like empty strings)");
    println!("- More meaningful counterexamples from property testing");
    println!();
    println!("API Design Notes:");
    println!("- Clean, non-stuttering API: Gen::<String>::with_range() vs Gen::<String>::string_with_range()");
    println!("- Convenience methods: Gen::<String>::alpha_with_range() for common cases");
    println!("- Consistent with numeric generators: Gen::<i32>::from_range()");
    println!("- Type-driven design: Gen::<String> knows it's for strings, no need to repeat");
}
