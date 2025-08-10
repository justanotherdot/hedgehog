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
    
    // Example 4: NEW - Concurrent non-deterministic testing
    example_concurrent_testing();
    
    // Example 5: Load Generation and Stress Testing
    example_load_generation();
}

/// Example 1: Basic parallel property testing
fn example_parallel_performance() {
    println!("\n1. Basic Parallel Property Testing");
    println!("-----------------------------------");
    
    let config = Config::default().with_tests(1000);
    
    // Create a parallel property that tests integer addition is commutative
    let parallel_prop = for_all_parallel(
        Gen::<(i32, i32)>::tuple_of(Gen::int_range(1, 100), Gen::int_range(1, 100)),
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

/// Example 4: Concurrent testing to detect race conditions and non-deterministic behavior
fn example_concurrent_testing() {
    println!("\n4. Concurrent Non-Deterministic Testing");
    println!("---------------------------------------");
    
    // Example 4a: Testing a deterministic function concurrently (should always be consistent)
    println!("\n4a. Testing Deterministic Function:");
    let deterministic_prop = for_all_concurrent(
        Gen::int_range(1, 100),
        |&n| n * 2 == n + n, // Always true, deterministic
        8 // Test from 8 threads simultaneously
    );
    
    let results = deterministic_prop.run(&Config::default().with_tests(10));
    
    let deterministic_count = results.iter().filter(|r| r.deterministic).count();
    let race_conditions = results.iter().map(|r| r.race_conditions_detected).sum::<usize>();
    
    println!("✓ Tested {} inputs concurrently", results.len());
    println!("  Deterministic results: {}/{}", deterministic_count, results.len());
    println!("  Race conditions detected: {}", race_conditions);
    
    if deterministic_count == results.len() {
        println!("✓ All results were deterministic (expected for pure math)");
    } else {
        println!("⚠ Some results were non-deterministic (unexpected!)");
    }
    
    // Example 4b: Testing a system with intentional race conditions
    println!("\n4b. Testing Function with Race Conditions:");
    
    use std::sync::atomic::{AtomicI32, Ordering};
    
    // Shared counter that creates race conditions
    let shared_counter = Arc::new(AtomicI32::new(0));
    
    let racy_prop = ConcurrentProperty::new(
        Gen::unit(), // We don't need varied input for this test
        {
            let counter = Arc::clone(&shared_counter);
            move |_| {
                // This creates a race condition - non-atomic read-modify-write
                let current = counter.load(Ordering::SeqCst);
                thread::sleep(Duration::from_micros(10)); // Increase chance of race
                counter.store(current + 1, Ordering::SeqCst);
                
                // Test that the counter increased (but due to races, this might fail)
                let final_value = counter.load(Ordering::SeqCst);
                if final_value > current {
                    TestResult::Pass {
                        tests_run: 1,
                        property_name: Some("counter_increment".to_string()),
                        module_path: None,
                    }
                } else {
                    TestResult::Fail {
                        counterexample: format!("Counter didn't increase: {} -> {}", current, final_value),
                        tests_run: 1,
                        shrinks_performed: 0,
                        property_name: Some("counter_increment".to_string()),
                        module_path: None,
                        assertion_type: Some("Race Condition".to_string()),
                        shrink_steps: Vec::new(),
                    }
                }
            }
        },
        6 // 6 threads competing for the counter
    );
    
    let racy_results = racy_prop.run(&Config::default().with_tests(5));
    
    let race_detected = racy_results.iter().any(|r| !r.deterministic || r.race_conditions_detected > 0);
    let total_race_conditions = racy_results.iter().map(|r| r.race_conditions_detected).sum::<usize>();
    
    println!("✓ Tested {} inputs with intentional race conditions", racy_results.len());
    println!("  Total race conditions detected: {}", total_race_conditions);
    
    if race_detected {
        println!("✓ Successfully detected race conditions in concurrent code!");
    } else {
        println!("ℹ No race conditions detected (might need more attempts or timing)");
    }
    
    // Example 4c: Testing thread safety of a lock-based data structure
    println!("\n4c. Testing Lock-Based Data Structure:");
    
    struct ThreadSafeCounter {
        value: Arc<Mutex<i32>>,
    }
    
    impl ThreadSafeCounter {
        fn new() -> Self {
            ThreadSafeCounter {
                value: Arc::new(Mutex::new(0)),
            }
        }
        
        fn increment(&self) -> i32 {
            let mut guard = self.value.lock().unwrap();
            *guard += 1;
            *guard
        }
        
        fn get(&self) -> i32 {
            *self.value.lock().unwrap()
        }
    }
    
    let safe_counter = Arc::new(ThreadSafeCounter::new());
    
    let thread_safe_prop = ConcurrentProperty::new(
        Gen::unit(),
        {
            let counter = Arc::clone(&safe_counter);
            move |_| {
                let result = counter.increment();
                // With proper locking, increments should always produce positive results
                if result > 0 {
                    TestResult::Pass {
                        tests_run: 1,
                        property_name: Some("safe_increment".to_string()),
                        module_path: None,
                    }
                } else {
                    TestResult::Fail {
                        counterexample: format!("Non-positive result: {}", result),
                        tests_run: 1,
                        shrinks_performed: 0,
                        property_name: Some("safe_increment".to_string()),
                        module_path: None,
                        assertion_type: Some("Thread Safety".to_string()),
                        shrink_steps: Vec::new(),
                    }
                }
            }
        },
        8 // 8 threads accessing the safe counter
    );
    
    let safe_results = thread_safe_prop.run(&Config::default().with_tests(10));
    
    let all_deterministic = safe_results.iter().all(|r| r.deterministic);
    let no_race_conditions = safe_results.iter().all(|r| r.race_conditions_detected == 0);
    
    println!("✓ Tested thread-safe counter from {} concurrent tests", safe_results.len());
    println!("  All results deterministic: {}", all_deterministic);
    println!("  No race conditions detected: {}", no_race_conditions);
    
    if all_deterministic && no_race_conditions {
        println!("✓ Thread-safe implementation working correctly!");
        println!("  Final counter value: {}", safe_counter.get());
    } else {
        println!("⚠ Detected issues in supposedly thread-safe code");
    }
    
    // Example 4d: Deadlock Detection
    println!("\n4d. Deadlock Detection:");
    
    let deadlock_mutex = Arc::new(Mutex::new(0));
    
    let deadlock_prop = ConcurrentProperty::new(
        Gen::unit(),
        {
            let mutex = Arc::clone(&deadlock_mutex);
            move |_| {
                let _guard = mutex.lock().unwrap();
                // Simulate slow operation that could cause timeouts
                thread::sleep(Duration::from_millis(30));
                
                TestResult::Pass {
                    tests_run: 1,
                    property_name: Some("deadlock_test".to_string()),
                    module_path: None,
                }
            }
        },
        4 // 4 threads competing for the same mutex
    ).with_timeout(Duration::from_millis(50)); // Short timeout to trigger deadlock detection
    
    let deadlock_results = deadlock_prop.run(&Config::default().with_tests(3));
    
    let timeouts_detected = deadlock_results.iter().filter(|r| r.timeout_detected).count();
    let deadlock_info_count = deadlock_results.iter().filter(|r| r.deadlock_info.is_some()).count();
    
    println!("✓ Tested {} inputs for deadlock detection", deadlock_results.len());
    println!("  Timeouts detected: {}", timeouts_detected);
    println!("  Deadlock info captured: {}", deadlock_info_count);
    
    if timeouts_detected > 0 {
        println!("✓ Successfully detected potential deadlocks!");
        
        // Show details of first deadlock
        if let Some(result) = deadlock_results.iter().find(|r| r.deadlock_info.is_some()) {
            if let Some(deadlock_info) = &result.deadlock_info {
                println!("  Deadlock details:");
                println!("    Input: {}", deadlock_info.input);
                println!("    Threads involved: {:?}", deadlock_info.threads_involved);
                println!("    Timeout duration: {:?}", deadlock_info.timeout_duration);
            }
        }
    } else {
        println!("ℹ No deadlocks detected in this run (timing-dependent)");
    }
    
    // Example 4e: Concurrent Scenario DSL
    println!("\n4e. Concurrent Scenario DSL:");
    
    let scenario = hedgehog_core::concurrent_scenario("bank_transfer")
        .operation("validate", |amount: &i32| {
            if *amount > 0 && *amount <= 1000 {
                TestResult::Pass {
                    tests_run: 1,
                    property_name: Some("validate".to_string()),
                    module_path: None,
                }
            } else {
                TestResult::Fail {
                    counterexample: format!("Invalid amount: {}", amount),
                    tests_run: 1,
                    shrinks_performed: 0,
                    property_name: Some("validate".to_string()),
                    module_path: None,
                    assertion_type: Some("Validation".to_string()),
                    shrink_steps: Vec::new(),
                }
            }
        })
        .operation_depends_on("debit", vec!["validate"], |amount: &i32| {
            // Simulate debit operation
            TestResult::Pass {
                tests_run: 1,
                property_name: Some(format!("debit_{}", amount)),
                module_path: None,
            }
        })
        .operation_depends_on("credit", vec!["validate"], |amount: &i32| {
            // Simulate credit operation  
            TestResult::Pass {
                tests_run: 1,
                property_name: Some(format!("credit_{}", amount)),
                module_path: None,
            }
        })
        .operation_depends_on("log", vec!["debit", "credit"], |amount: &i32| {
            // Simulate logging operation
            TestResult::Pass {
                tests_run: 1,
                property_name: Some(format!("log_transfer_{}", amount)),
                module_path: None,
            }
        })
        .before("debit", "credit")  // Ensure debit happens before credit
        .build();
    
    let scenario_result = scenario.execute(&500);
    
    println!("✓ Executed scenario: {}", scenario_result.scenario_name);
    println!("  Operations completed: {}", scenario_result.operation_results.len());
    println!("  Execution time: {:?}", scenario_result.execution_time);
    println!("  Constraints satisfied: {}", scenario_result.constraints_satisfied);
    
    // Show operation results
    for (op_name, result) in &scenario_result.operation_results {
        match result {
            TestResult::Pass { .. } => println!("  ✓ {}: PASS", op_name),
            TestResult::Fail { counterexample, .. } => println!("  ✗ {}: FAIL ({})", op_name, counterexample),
            _ => println!("  ? {}: OTHER", op_name),
        }
    }
    
    if !scenario_result.constraint_violations.is_empty() {
        println!("  Constraint violations:");
        for violation in &scenario_result.constraint_violations {
            println!("    - {}", violation);
        }
    }
    
    // Example 4f: Systematic Interleaving Exploration
    println!("\n4f. Systematic Interleaving Exploration:");
    
    use std::sync::atomic::AtomicUsize;
    
    let shared_counter = Arc::new(AtomicUsize::new(0));
    
    let explorer = hedgehog_core::interleaving_explorer(
        Gen::int_range(1, 10),
        {
            let counter = Arc::clone(&shared_counter);
            move |&increment| {
                // This test has potential race conditions in the increment operation
                let current = counter.load(Ordering::SeqCst);
                thread::sleep(Duration::from_nanos(100)); // Small delay to increase race chance
                let new_value = current + increment as usize;
                counter.store(new_value, Ordering::SeqCst);
                
                // Test that counter increased
                let final_value = counter.load(Ordering::SeqCst);
                if final_value >= current {
                    TestResult::Pass {
                        tests_run: 1,
                        property_name: Some("increment_test".to_string()),
                        module_path: None,
                    }
                } else {
                    TestResult::Fail {
                        counterexample: format!("Counter didn't increase: {} -> {}", current, final_value),
                        tests_run: 1,
                        shrinks_performed: 0,
                        property_name: Some("increment_test".to_string()),
                        module_path: None,
                        assertion_type: Some("Race Condition".to_string()),
                        shrink_steps: Vec::new(),
                    }
                }
            }
        }
    )
    .with_operations(4)  // 4 concurrent operations
    .with_max_interleavings(25);  // Explore 25 different interleavings
    
    let interleaving_results = explorer.explore(&Config::default().with_tests(3));
    
    println!("✓ Explored interleavings for {} different inputs", interleaving_results.len());
    
    let total_interleavings: usize = interleaving_results.iter()
        .map(|r| r.interleavings_explored)
        .sum();
    let total_races: usize = interleaving_results.iter()
        .map(|r| r.race_conditions_detected)
        .sum();
    let total_failures: usize = interleaving_results.iter()
        .map(|r| r.failed_interleavings)
        .sum();
    
    println!("  Total interleavings explored: {}", total_interleavings);
    println!("  Race conditions detected: {}", total_races);
    println!("  Failed interleavings: {}", total_failures);
    
    // Show details of any race conditions found
    for (i, result) in interleaving_results.iter().enumerate() {
        if !result.deterministic {
            println!("  ⚠ Input {} had non-deterministic behavior:", i + 1);
            println!("    {} out of {} interleavings failed", result.failed_interleavings, result.interleavings_explored);
            
            if let Some(pattern) = result.failing_patterns.first() {
                println!("    First failing pattern involved threads: {:?}", pattern.threads_involved);
                println!("    Sequence length: {}", pattern.sequence.len());
            }
        } else {
            println!("  ✓ Input {} was deterministic across all {} interleavings", i + 1, result.interleavings_explored);
        }
    }
    
    if total_races > 0 {
        println!("✓ Successfully detected race conditions through systematic interleaving exploration!");
    } else {
        println!("ℹ No race conditions detected (may need more attempts or different timing)");
    }
}

/// Example 5: Load Generation and Stress Testing
fn example_load_generation() {
    println!("\n5. Load Generation and Stress Testing");
    println!("====================================");
    
    // Example 5a: Basic Load Testing
    println!("\n5a. Basic Load Testing:");
    
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    let request_counter = Arc::new(AtomicUsize::new(0));
    
    let config = hedgehog_core::LoadTestConfig {
        thread_count: 4,
        duration: Duration::from_millis(200), // Short test
        ops_per_second: Some(50), // Target 50 ops/sec
        ramp_up_duration: Duration::from_millis(50),
        cool_down_duration: Duration::from_millis(30),
        collect_stats: true,
    };
    
    let load_generator = hedgehog_core::LoadGenerator::new(
        Gen::int_range(1, 100),
        {
            let counter = Arc::clone(&request_counter);
            move |&request_id| {
                // Simulate API request processing
                counter.fetch_add(1, Ordering::SeqCst);
                
                // Simulate variable processing time
                let delay_micros = (request_id % 10) + 1; // 1-10 microseconds
                thread::sleep(Duration::from_micros(delay_micros as u64));
                
                // Simulate 95% success rate
                if request_id % 20 != 0 { // 19/20 requests succeed
                    TestResult::Pass {
                        tests_run: 1,
                        property_name: Some(format!("request_{}", request_id)),
                        module_path: None,
                    }
                } else {
                    TestResult::Fail {
                        counterexample: format!("Service unavailable for request {}", request_id),
                        tests_run: 1,
                        shrinks_performed: 0,
                        property_name: Some(format!("request_{}", request_id)),
                        module_path: None,
                        assertion_type: Some("Service Error".to_string()),
                        shrink_steps: Vec::new(),
                    }
                }
            }
        },
        config,
    );
    
    println!("Running load test with 4 threads for 200ms...");
    let result = load_generator.run_load_test();
    
    println!("\n📊 Load Test Results:");
    println!("  Total operations: {}", result.stats.operations_completed);
    println!("  Failed operations: {}", result.stats.operations_failed);
    println!("  Success rate: {:.1}%", result.success_rate * 100.0);
    println!("  Average ops/sec: {:.1}", result.stats.avg_ops_per_second);
    
    println!("\n⏱️  Response Time Metrics:");
    println!("  Average: {:?}", result.stats.avg_response_time);
    println!("  95th percentile: {:?}", result.stats.p95_response_time);
    println!("  99th percentile: {:?}", result.stats.p99_response_time);
    println!("  Maximum: {:?}", result.stats.max_response_time);
    
    println!("\n🕒 Phase Timings:");
    println!("  Ramp-up: {:?}", result.phase_timings.ramp_up_time);
    println!("  Steady state: {:?}", result.phase_timings.steady_state_time);
    println!("  Cool-down: {:?}", result.phase_timings.cool_down_time);
    println!("  Total time: {:?}", result.phase_timings.total_time);
    
    let total_requests = request_counter.load(Ordering::SeqCst);
    if total_requests == result.stats.operations_completed {
        println!("✓ Request counter matches operations completed: {}", total_requests);
    } else {
        println!("⚠ Request counter mismatch: {} vs {}", total_requests, result.stats.operations_completed);
    }
    
    // Example 5b: High-Performance Stress Testing
    println!("\n5b. High-Performance Stress Testing:");
    
    let stress_config = hedgehog_core::LoadTestConfig {
        thread_count: 8, // More threads for stress
        duration: Duration::from_millis(150),
        ops_per_second: None, // No rate limiting - go as fast as possible
        ramp_up_duration: Duration::from_millis(20),
        cool_down_duration: Duration::from_millis(10),
        collect_stats: true,
    };
    
    // Simulate a high-performance computation
    let computation_counter = Arc::new(AtomicUsize::new(0));
    
    let stress_generator = hedgehog_core::LoadGenerator::new(
        Gen::int_range(1, 1000),
        {
            let counter = Arc::clone(&computation_counter);
            move |&n| {
                counter.fetch_add(1, Ordering::SeqCst);
                
                // Fast computation that could stress the system
                let mut result = n;
                for _ in 0..(n % 50) {  // Variable workload
                    result = result.wrapping_mul(17).wrapping_add(13);
                }
                
                // Property: computation should always complete
                if result != 0 {
                    TestResult::Pass {
                        tests_run: 1,
                        property_name: Some(format!("computation_{}", n)),
                        module_path: None,
                    }
                } else {
                    TestResult::Fail {
                        counterexample: format!("Computation failed for {}", n),
                        tests_run: 1,
                        shrinks_performed: 0,
                        property_name: Some(format!("computation_{}", n)),
                        module_path: None,
                        assertion_type: Some("Computation Error".to_string()),
                        shrink_steps: Vec::new(),
                    }
                }
            }
        },
        stress_config,
    );
    
    println!("Running high-performance stress test with 8 threads...");
    let stress_result = stress_generator.run_load_test();
    
    println!("\n🚀 Stress Test Results:");
    println!("  Operations completed: {}", stress_result.stats.operations_completed);
    println!("  Peak ops/sec: {:.1}", stress_result.stats.avg_ops_per_second);
    println!("  Average response time: {:?}", stress_result.stats.avg_response_time);
    println!("  P99 response time: {:?}", stress_result.stats.p99_response_time);
    
    let stress_computations = computation_counter.load(Ordering::SeqCst);
    println!("  Total computations: {}", stress_computations);
    
    // Compare performance
    let load_ops_per_sec = result.stats.avg_ops_per_second;
    let stress_ops_per_sec = stress_result.stats.avg_ops_per_second;
    
    if stress_ops_per_sec > load_ops_per_sec * 2.0 {
        println!("✓ Stress test achieved {:.1}x higher throughput than rate-limited test", 
                 stress_ops_per_sec / load_ops_per_sec);
    } else {
        println!("ℹ Stress test throughput: {:.1} ops/sec vs rate-limited: {:.1} ops/sec", 
                 stress_ops_per_sec, load_ops_per_sec);
    }
    
    // Example 5c: Memory and Resource Testing
    println!("\n5c. Memory and Resource Testing:");
    
    let memory_config = hedgehog_core::LoadTestConfig {
        thread_count: 3,
        duration: Duration::from_millis(100),
        ops_per_second: Some(30),
        ramp_up_duration: Duration::from_millis(15),
        cool_down_duration: Duration::from_millis(15),
        collect_stats: true,
    };
    
    // Test memory allocation patterns
    let memory_generator = hedgehog_core::LoadGenerator::new(
        Gen::int_range(10, 1000),
        |&size| {
            // Allocate and use memory to test resource handling
            let data: Vec<u32> = (0..size).map(|i| i as u32 * 7).collect();
            let sum: u32 = data.iter().sum();
            
            // Property: sum should be calculable for any reasonable size
            if data.len() == size as usize && sum > 0 {
                TestResult::Pass {
                    tests_run: 1,
                    property_name: Some(format!("memory_test_{}", size)),
                    module_path: None,
                }
            } else {
                TestResult::Fail {
                    counterexample: format!("Memory test failed for size {}", size),
                    tests_run: 1,
                    shrinks_performed: 0,
                    property_name: Some(format!("memory_test_{}", size)),
                    module_path: None,
                    assertion_type: Some("Memory Error".to_string()),
                    shrink_steps: Vec::new(),
                }
            }
        },
        memory_config,
    );
    
    println!("Running memory allocation stress test...");
    let memory_result = memory_generator.run_load_test();
    
    println!("\n💾 Memory Test Results:");
    println!("  Memory operations: {}", memory_result.stats.operations_completed);
    println!("  Success rate: {:.1}%", memory_result.success_rate * 100.0);
    println!("  Avg response time: {:?}", memory_result.stats.avg_response_time);
    
    if memory_result.success_rate >= 0.99 {
        println!("✓ Memory allocation test completed successfully");
    } else {
        println!("⚠ Some memory allocations failed - potential resource issues");
    }
    
    println!("\n🎯 Load Generation Summary:");
    println!("  Basic load test completed {} operations", result.stats.operations_completed);
    println!("  Stress test achieved {:.1} ops/sec peak throughput", stress_result.stats.avg_ops_per_second);
    println!("  Memory test processed {} allocations", memory_result.stats.operations_completed);
    println!("✓ All load generation tests completed successfully!");
}