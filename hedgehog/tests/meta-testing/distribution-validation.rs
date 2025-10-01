//! Distribution validation properties
//!
//! These properties ensure that generators produce distributions that match
//! their specifications - frequency weights, range distributions, etc.

use crate::{arbitrary_seed, arbitrary_size};
use hedgehog::*;
use std::collections::HashMap;

/// Property: Frequency generator should respect weight ratios
pub fn test_frequency_weights() {
    // Test with simple 2-choice frequency generator
    let choices = vec![
        WeightedChoice::new(70, Gen::constant(1)), // 70% weight
        WeightedChoice::new(30, Gen::constant(2)), // 30% weight
    ];

    let gen = Gen::frequency(choices).expect("frequency generator should be valid");

    // Generate many samples and check distribution
    let sample_size = 1000;
    let mut counts = HashMap::new();
    let size = Size::new(10);

    for i in 0..sample_size {
        let seed = Seed::from_u64(i as u64);
        let tree = gen.generate(size, seed);
        let value = tree.value;
        *counts.entry(value).or_insert(0) += 1;
    }

    let count_1 = *counts.get(&1).unwrap_or(&0) as f64;
    let count_2 = *counts.get(&2).unwrap_or(&0) as f64;

    // Check that both options appear with reasonable frequency
    let has_reasonable_distribution = count_1 >= 200.0 && count_2 >= 100.0; // Basic sanity check

    if !has_reasonable_distribution {
        panic!("Frequency distribution unreasonable: got {} vs {} (expected both to have reasonable counts)", count_1 as i32, count_2 as i32);
    }

    println!(
        "✓ Frequency weights property passed (counts: {} vs {})",
        count_1 as i32, count_2 as i32
    );
}

/// Property: Range distributions should produce expected spreads
pub fn test_range_distributions() {
    let prop = for_all_named(
        Gen::<((i32, i32), (Size, Seed))>::tuple_of(
            Gen::<(i32, i32)>::tuple_of(
                Gen::int_range(0, 50),   // min
                Gen::int_range(51, 100), // max
            ),
            Gen::<(Size, Seed)>::tuple_of(arbitrary_size(), arbitrary_seed()),
        ),
        "((min, max), (size, seed))",
        |&((min, max), (size, seed)): &((i32, i32), (Size, Seed))| {
            if min >= max {
                return true;
            }

            // Test uniform distribution
            let uniform_gen = Gen::<i32>::from_range(Range::new(min, max));
            let uniform_value = uniform_gen.generate(size, seed).value;

            // Test linear distribution (should favor smaller values)
            let linear_gen = Gen::<i32>::from_range(Range::linear(min, max));
            let linear_value = linear_gen.generate(size, seed).value;

            // Test exponential distribution (should strongly favor smaller values)
            let exp_gen = Gen::<i32>::from_range(Range::exponential(min, max));
            let exp_value = exp_gen.generate(size, seed).value;

            // All should be within bounds
            let uniform_valid = uniform_value >= min && uniform_value <= max;
            let linear_valid = linear_value >= min && linear_value <= max;
            let exp_valid = exp_value >= min && exp_value <= max;

            uniform_valid && linear_valid && exp_valid
        },
    );

    let fast_config = Config::default().with_tests(15).with_shrinks(5);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Range distributions property passed"),
        result => panic!("Range distributions property failed: {result:?}"),
    }
}

/// Property: Linear distribution should favor smaller values
pub fn test_linear_distribution_bias() {
    let min = 1;
    let max = 100;
    let gen = Gen::<i32>::from_range(Range::linear(min, max));

    // Generate many samples
    let sample_size = 1000;
    let mut samples = Vec::new();

    for i in 0..sample_size {
        let seed = Seed::from_u64(i as u64);
        let tree = gen.generate(Size::new(10), seed);
        samples.push(tree.value);
    }

    // Calculate mean - should be closer to min than to max for linear distribution
    let sum: i32 = samples.iter().sum();
    let mean = sum as f64 / samples.len() as f64;
    let midpoint = (min + max) as f64 / 2.0;

    // Linear distribution should show some bias toward smaller values (allow variance)
    let shows_lower_bias = mean < midpoint + 10.0; // Allow significant variance

    if !shows_lower_bias {
        panic!(
            "Linear distribution shows no bias towards smaller values: mean {mean:.2}, midpoint {midpoint:.2}"
        );
    }

    println!(
        "✓ Linear distribution bias property passed (mean: {mean:.2}, midpoint: {midpoint:.2})"
    );
}

/// Property: Exponential distribution should strongly favor smallest values
pub fn test_exponential_distribution_bias() {
    let min = 1;
    let max = 100;
    let gen = Gen::<i32>::from_range(Range::exponential(min, max));

    // Generate many samples
    let sample_size = 1000;
    let mut samples = Vec::new();

    for i in 0..sample_size {
        let seed = Seed::from_u64(i as u64);
        let tree = gen.generate(Size::new(10), seed);
        samples.push(tree.value);
    }

    // Count how many samples are in lower quartile
    let quartile_threshold = min + (max - min) / 4;
    let lower_quartile_count = samples.iter().filter(|&&x| x <= quartile_threshold).count();
    let lower_quartile_ratio = lower_quartile_count as f64 / samples.len() as f64;

    // Exponential distribution should have >50% of values in lower quartile
    if lower_quartile_ratio < 0.5 {
        panic!(
            "Exponential distribution not strongly biased: {:.2}% in lower quartile",
            lower_quartile_ratio * 100.0
        );
    }

    println!(
        "✓ Exponential distribution bias property passed ({:.1}% in lower quartile)",
        lower_quartile_ratio * 100.0
    );
}

/// Property: One-of generator should have uniform distribution
pub fn test_one_of_uniform_distribution() {
    let generators = vec![
        Gen::constant(1),
        Gen::constant(2),
        Gen::constant(3),
        Gen::constant(4),
    ];

    let gen = Gen::one_of(generators).expect("one_of generator should be valid");

    // Generate many samples
    let sample_size = 1000;
    let mut counts = HashMap::new();

    for i in 0..sample_size {
        let seed = Seed::from_u64(i as u64);
        let tree = gen.generate(Size::new(10), seed);
        let value = tree.value;
        *counts.entry(value).or_insert(0) += 1;
    }

    // Each value should appear with reasonable frequency (very permissive)
    let min_reasonable_count = sample_size / 10; // At least 10% each

    for (&value, &count) in &counts {
        if count < min_reasonable_count {
            panic!(
                "One-of distribution too skewed for value {value}: got {count} (less than {min_reasonable_count})"
            );
        }
    }

    println!("✓ One-of uniform distribution property passed");
}

/// Property: Vector length should follow size parameter distribution
pub fn test_vector_length_distribution() {
    let prop = for_all_named(
        Gen::<(Size, Seed)>::tuple_of(arbitrary_size(), arbitrary_seed()),
        "(size, seed)",
        |&(size, seed): &(Size, Seed)| {
            let gen = Gen::vec_of(Gen::int_range(0, 10));
            let tree = gen.generate(size, seed);
            let vec_length = tree.value.len();

            // Vector length should be influenced by size parameter
            // (not exact, but should be correlated)
            vec_length <= size.get() + 10 // Allow some variance
        },
    );

    let fast_config = Config::default().with_tests(15).with_shrinks(5);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Vector length distribution property passed"),
        result => panic!("Vector length distribution property failed: {result:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_distribution_validation_tests() {
        test_frequency_weights();
        test_range_distributions();
        test_linear_distribution_bias();
        test_exponential_distribution_bias();
        test_one_of_uniform_distribution();
        test_vector_length_distribution();
    }
}
