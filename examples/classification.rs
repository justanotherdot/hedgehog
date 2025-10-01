//! Demonstrates property classification and data collection.

use hedgehog::*;

fn main() {
    println!("=== Property Classification Demo ===\n");

    // Example 1: Basic classification with integer ranges
    println!("1. Integer range classification:");
    let gen = Gen::int_range(-20, 20);
    let prop = for_all(gen, |&x| (-20..=20).contains(&x))
        .classify("negative", |&x| x < 0)
        .classify("zero", |&x| x == 0)
        .classify("positive", |&x| x > 0)
        .classify("small", |&x| x.abs() < 5)
        .classify("large", |&x| x.abs() >= 15)
        .collect("value", |&x| x as f64)
        .collect("absolute_value", |&x| x.abs() as f64);

    let config = Config::default().with_tests(100);
    let result = prop.run_with_context(&config, Some("integer_classification"), None);
    println!("{result}\n");

    // Example 2: String classification
    println!("2. String classification:");
    let string_gen = Gen::<String>::alpha_with_range(Range::exponential(0, 50));
    let string_prop = for_all(string_gen, |s| !s.contains("invalid"))
        .classify("empty", |s| s.is_empty())
        .classify("short", |s| s.len() < 10)
        .classify("medium", |s| s.len() >= 10 && s.len() < 30)
        .classify("long", |s| s.len() >= 30)
        .classify("has_vowels", |s| {
            s.chars().any(|c| "aeiouAEIOU".contains(c))
        })
        .collect("length", |s| s.len() as f64)
        .collect("vowel_count", |s| {
            s.chars().filter(|c| "aeiouAEIOU".contains(*c)).count() as f64
        });

    let result = string_prop.run_with_context(&config, Some("string_classification"), None);
    println!("{result}\n");

    // Example 3: Vector classification
    println!("3. Vector classification:");
    let vec_gen = Gen::<Vec<i32>>::vec_of(Gen::int_range(1, 100));
    let vec_prop = for_all(vec_gen, |vec| vec.iter().all(|&x| x > 0))
        .classify("empty", |vec| vec.is_empty())
        .classify("singleton", |vec| vec.len() == 1)
        .classify("small", |vec| vec.len() > 1 && vec.len() <= 5)
        .classify("medium", |vec| vec.len() > 5 && vec.len() <= 20)
        .classify("large", |vec| vec.len() > 20)
        .classify("sorted", |vec| vec.windows(2).all(|w| w[0] <= w[1]))
        .collect("length", |vec| vec.len() as f64)
        .collect("sum", |vec| vec.iter().sum::<i32>() as f64)
        .collect("average", |vec| {
            if vec.is_empty() {
                0.0
            } else {
                vec.iter().sum::<i32>() as f64 / vec.len() as f64
            }
        });

    let result = vec_prop.run_with_context(&config, Some("vector_classification"), None);
    println!("{result}\n");
}
