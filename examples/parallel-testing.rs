//! Parallel testing example demonstrating performance improvements and concurrent testing.

use hedgehog_core::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    println!("Parallel Testing Examples");
    println!("========================");
    
    // Example 1: Basic parallel property testing for performance
    example_parallel_performance();
    
    // Example 2: Simple shared state testing
    example_shared_state_testing();
    
    // Example 3: Performance comparison
    example_performance_comparison();
}

/// Example 1: Basic parallel property testing
fn example_parallel_performance() {
    println!("\n1. Basic Parallel Property Testing");
    println!("-----------------------------------");
    
    let config = Config::default().with_tests(1000);
    
    // Create a parallel property that tests integer addition is commutative
    let parallel_prop = for_all_parallel(
        Gen::tuple_of(Gen::int_range(1, 100), Gen::int_range(1, 100)),
        |(a, b)| a + b == b + a,
        4 // Use 4 threads
    );
    
    let start = Instant::now();
    let result = parallel_prop.run(&config);
    let duration = start.elapsed();
    
    match result.outcome {
        TestResult::Pass { tests_run, .. } => {
            println!("✓ {} tests passed in {:?}", tests_run, duration);
            println!("  Speedup: {:.2}x", result.performance.speedup_factor);
            println!("  Thread efficiency: {:.1}%", result.performance.thread_efficiency * 100.0);
        }
        TestResult::Fail { counterexample, .. } => {
            println!("✗ Test failed with counterexample: {}", counterexample);
        }
        _ => println!("? Unexpected test result"),
    }
}

/// Example 2: Testing a shared counter with potential race conditions
fn example_shared_state_testing() {
    println!("\n2. Shared State Testing");
    println!("-----------------------");
    
    struct SharedCounter {
        value: Arc<Mutex<i32>>,
    }
    
    impl SharedCounter {
        fn new() -> Self {
            SharedCounter {
                value: Arc::new(Mutex::new(0)),
            }
        }
        
        fn increment(&self) -> i32 {
            let mut guard = self.value.lock().unwrap();
            *guard += 1;
            // Simulate some work that could expose race conditions
            thread::sleep(Duration::from_micros(1));
            *guard
        }
        
        fn get(&self) -> i32 {
            *self.value.lock().unwrap()
        }
    }
    
    let config = Config::default().with_tests(100);
    
    // Test that concurrent increments work correctly
    let counter = Arc::new(SharedCounter::new());
    let parallel_prop = parallel_property(
        Gen::unit(), // We don't need any input
        {
            let counter = Arc::clone(&counter);
            move |_| {
                // Increment the counter from this thread
                let result = counter.increment();
                
                // Basic property: the result should always be positive
                if result > 0 {
                    TestResult::Pass {
                        tests_run: 1,
                        property_name: Some("counter_increment".to_string()),
                        module_path: None,
                    }
                } else {
                    TestResult::Fail {
                        counterexample: format!("Counter result was {}", result),
                        tests_run: 1,
                        shrinks_performed: 0,
                        property_name: Some("counter_increment".to_string()),
                        module_path: None,
                        assertion_type: Some("Positive Counter".to_string()),
                        shrink_steps: Vec::new(),
                    }
                }
            }
        },
        ParallelConfig {
            thread_count: 8,
            work_distribution: WorkDistribution::RoundRobin,
            timeout: Some(Duration::from_secs(5)),
            detect_non_determinism: true,
        }
    );
    
    let result = parallel_prop.run(&config);
    
    match result.outcome {
        TestResult::Pass { tests_run, .. } => {
            println!("✓ {} concurrent increments completed successfully", tests_run);
            println!("  Final counter value: {}", counter.get());
            println!("  Thread failures: {}", result.concurrency_issues.thread_failures.len());
            
            // The final value should equal the number of tests if no race conditions occurred
            let expected = tests_run;
            let actual = counter.get();
            if actual == expected as i32 {
                println!("✓ No race conditions detected: {} == {}", actual, expected);
            } else {
                println!("⚠ Potential race condition: expected {}, got {}", expected, actual);
            }
        }
        TestResult::Fail { counterexample, .. } => {
            println!("✗ Test failed: {}", counterexample);
        }
        _ => println!("? Unexpected test result"),
    }
}

/// Example 3: Performance comparison between sequential and parallel testing
fn example_performance_comparison() {
    println!("\n3. Performance Comparison");
    println!("-------------------------");
    
    let test_count = 2000;
    let config = Config::default().with_tests(test_count);
    
    // Simulate expensive computation
    let expensive_test = |&n: &i32| {
        // Simulate some CPU work
        let mut sum = 0;
        let iterations = (n.abs() % 1000).max(1); // Ensure at least 1 iteration
        for i in 0..iterations {
            sum += i * i;
        }
        sum >= 0 // This will always be true
    };
    
    // Sequential testing using regular property
    let sequential_prop = Property::for_all(Gen::int_range(1, 100), expensive_test);
    
    println!("Running {} tests sequentially...", test_count);
    let start = Instant::now();
    let sequential_result = sequential_prop.run(&config);
    let sequential_time = start.elapsed();
    
    // Parallel testing
    let parallel_prop = for_all_parallel(Gen::int_range(1, 100), expensive_test, 4);
    
    println!("Running {} tests in parallel (4 threads)...", test_count);
    let start = Instant::now();
    let parallel_result = parallel_prop.run(&config);
    let parallel_time = start.elapsed();
    
    // Results
    match (&sequential_result, &parallel_result.outcome) {
        (TestResult::Pass { .. }, TestResult::Pass { .. }) => {
            println!("\n📊 Performance Results:");
            println!("  Sequential time: {:?}", sequential_time);
            println!("  Parallel time:   {:?}", parallel_time);
            
            let speedup = sequential_time.as_secs_f64() / parallel_time.as_secs_f64();
            println!("  Actual speedup:  {:.2}x", speedup);
            println!("  Reported speedup: {:.2}x", parallel_result.performance.speedup_factor);
            println!("  Thread efficiency: {:.1}%", parallel_result.performance.thread_efficiency * 100.0);
            
            if speedup > 1.5 {
                println!("✓ Significant performance improvement achieved!");
            } else {
                println!("ℹ Limited speedup (overhead may be affecting small computations)");
            }
        }
        _ => println!("⚠ One or both tests failed"),
    }
}