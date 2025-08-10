//! Parallel and concurrent testing meta tests
//!
//! These properties test the parallel testing infrastructure including
//! parallel property execution, concurrent system testing, deadlock detection,
//! race condition detection, and load testing capabilities.

use hedgehog::*;
use crate::arbitrary_seed;
use std::time::Duration;
use std::sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering}};

/// Property: Parallel property execution should distribute work correctly
pub fn test_parallel_work_distribution() {
    let prop = for_all_named(
        arbitrary_seed(),
        "seed",
        |&_seed: &Seed| {
            let config = Config::default().with_tests(100);
            let parallel_config = ParallelConfig {
                thread_count: 4,
                work_distribution: WorkDistribution::RoundRobin,
                ..ParallelConfig::default()
            };
            
            let parallel_prop = parallel_property(
                Gen::int_range(1, 50),
                |&n| {
                    if n > 0 && n <= 50 {
                        TestResult::Pass {
                            tests_run: 1,
                            property_name: None,
                            module_path: None,
                        }
                    } else {
                        TestResult::Fail {
                            counterexample: format!("{}", n),
                            tests_run: 1,
                            shrinks_performed: 0,
                            property_name: None,
                            module_path: None,
                            assertion_type: Some("Range Check".to_string()),
                            shrink_steps: Vec::new(),
                        }
                    }
                },
                parallel_config,
            );
            
            let result = parallel_prop.run(&config);
            
            // Should complete all tests successfully
            match result.outcome {
                TestResult::Pass { tests_run, .. } => {
                    tests_run == 100 &&
                    result.thread_results.len() == 4 &&
                    result.performance.speedup_factor > 0.0 &&
                    result.performance.thread_efficiency > 0.0 &&
                    result.concurrency_issues.thread_failures.is_empty()
                }
                _ => false,
            }
        }
    );
    
    let fast_config = Config::default().with_tests(5).with_shrinks(2);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Parallel work distribution property passed"),
        result => panic!("Parallel work distribution property failed: {:?}", result),
    }
}

/// Property: Different work distribution strategies should work correctly
pub fn test_work_distribution_strategies() {
    let prop = for_all_named(
        arbitrary_seed(),
        "seed", 
        |&_seed: &Seed| {
            let config = Config::default().with_tests(50);
            let strategies = vec![
                WorkDistribution::RoundRobin,
                WorkDistribution::ChunkBased,
                WorkDistribution::WorkStealing,
            ];
            
            for strategy in strategies {
                let parallel_config = ParallelConfig {
                    thread_count: 3,
                    work_distribution: strategy,
                    ..ParallelConfig::default()
                };
                
                let parallel_prop = parallel_property(
                    Gen::int_range(1, 20),
                    |&n| TestResult::Pass {
                        tests_run: 1,
                        property_name: Some(format!("strategy_test_{}", n)),
                        module_path: None,
                    },
                    parallel_config,
                );
                
                let result = parallel_prop.run(&config);
                
                // All strategies should complete successfully
                match result.outcome {
                    TestResult::Pass { tests_run, .. } => {
                        if tests_run != 50 || result.thread_results.len() != 3 {
                            return false;
                        }
                    }
                    _ => return false,
                }
            }
            
            true
        }
    );
    
    let fast_config = Config::default().with_tests(3).with_shrinks(2);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Work distribution strategies property passed"),
        result => panic!("Work distribution strategies property failed: {:?}", result),
    }
}

/// Property: Concurrent property testing should detect non-deterministic behavior  
pub fn test_concurrent_non_determinism_detection() {
    let prop = for_all_named(
        arbitrary_seed(),
        "seed",
        |&_seed: &Seed| {
            // Create a non-deterministic test using atomic counter
            let counter = Arc::new(AtomicUsize::new(0));
            
            let concurrent_prop = ConcurrentProperty::new(
                Gen::unit(),
                {
                    let counter = Arc::clone(&counter);
                    move |_| {
                        let count = counter.fetch_add(1, Ordering::SeqCst);
                        
                        // This creates non-deterministic behavior based on execution order
                        if count % 2 == 0 {
                            TestResult::Pass {
                                tests_run: 1,
                                property_name: Some("non_deterministic".to_string()),
                                module_path: None,
                            }
                        } else {
                            TestResult::Fail {
                                counterexample: format!("count: {}", count),
                                tests_run: 1,
                                shrinks_performed: 0,
                                property_name: Some("non_deterministic".to_string()),
                                module_path: None,
                                assertion_type: Some("Counter Parity".to_string()),
                                shrink_steps: Vec::new(),
                            }
                        }
                    }
                },
                4, // 4 threads
            ).with_timeout(Duration::from_millis(500));
            
            let results = concurrent_prop.run(&Config::default().with_tests(3));
            
            // Should detect non-deterministic behavior in at least some tests
            let non_deterministic_count = results.iter()
                .filter(|r| !r.deterministic)
                .count();
                
            // Should find some non-deterministic results due to race conditions
            non_deterministic_count > 0 && results.len() == 3
        }
    );
    
    let fast_config = Config::default().with_tests(3).with_shrinks(1);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Concurrent non-determinism detection property passed"),
        result => panic!("Concurrent non-determinism detection property failed: {:?}", result),
    }
}

/// Property: Deadlock detection should identify hanging threads
pub fn test_deadlock_detection() {
    let prop = for_all_named(
        arbitrary_seed(),
        "seed",
        |&_seed: &Seed| {
            // Create a scenario that might cause deadlock with competing mutexes
            let mutex1 = Arc::new(Mutex::new(0));
            let mutex2 = Arc::new(Mutex::new(0));
            
            let concurrent_prop = ConcurrentProperty::new(
                Gen::unit(),
                {
                    let m1 = Arc::clone(&mutex1);
                    let m2 = Arc::clone(&mutex2);
                    move |_| {
                        let _guard1 = m1.lock().unwrap();
                        std::thread::sleep(Duration::from_millis(20)); // Hold lock
                        
                        // Try to get second lock (might cause contention)
                        match m2.try_lock() {
                            Ok(_guard2) => TestResult::Pass {
                                tests_run: 1,
                                property_name: Some("deadlock_test".to_string()),
                                module_path: None,
                            },
                            Err(_) => TestResult::Pass { // Contention is expected
                                tests_run: 1,
                                property_name: Some("deadlock_test_contention".to_string()),
                                module_path: None,
                            },
                        }
                    }
                },
                3, // 3 threads competing for locks
            ).with_timeout(Duration::from_millis(100)); // Short timeout
            
            let results = concurrent_prop.run(&Config::default().with_tests(2));
            
            // Should complete without crashing and detect any timeout/contention issues
            results.len() == 2 && 
            results.iter().all(|r| r.execution_times.len() == 3) // All threads should complete or timeout
        }
    );
    
    let fast_config = Config::default().with_tests(2).with_shrinks(1);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Deadlock detection property passed"),
        result => panic!("Deadlock detection property failed: {:?}", result),
    }
}

/// Property: Parallel execution should provide performance benefits
pub fn test_parallel_performance_benefits() {
    let prop = for_all_named(
        arbitrary_seed(),
        "seed",
        |&_seed: &Seed| {
            let config = Config::default().with_tests(100);
            
            // Compare single-threaded vs multi-threaded execution
            let single_threaded = parallel_property(
                Gen::int_range(1, 100),
                |&n| {
                    // Add small delay to simulate work
                    std::thread::sleep(Duration::from_micros(10));
                    TestResult::Pass {
                        tests_run: 1,
                        property_name: Some(format!("perf_test_{}", n)),
                        module_path: None,
                    }
                },
                ParallelConfig {
                    thread_count: 1,
                    ..ParallelConfig::default()
                },
            );
            
            let multi_threaded = parallel_property(
                Gen::int_range(1, 100),
                |&n| {
                    std::thread::sleep(Duration::from_micros(10));
                    TestResult::Pass {
                        tests_run: 1,
                        property_name: Some(format!("perf_test_{}", n)),
                        module_path: None,
                    }
                },
                ParallelConfig {
                    thread_count: 4,
                    ..ParallelConfig::default()
                },
            );
            
            let single_result = single_threaded.run(&config);
            let multi_result = multi_threaded.run(&config);
            
            // Multi-threaded should show better efficiency metrics
            match (&single_result.outcome, &multi_result.outcome) {
                (TestResult::Pass { .. }, TestResult::Pass { .. }) => {
                    multi_result.performance.speedup_factor >= 1.0 &&
                    multi_result.performance.thread_efficiency > 0.0 &&
                    single_result.performance.thread_efficiency > 0.0
                }
                _ => false,
            }
        }
    );
    
    let fast_config = Config::default().with_tests(2).with_shrinks(1);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Parallel performance benefits property passed"),
        result => panic!("Parallel performance benefits property failed: {:?}", result),
    }
}

/// Property: Load testing should handle sustained load correctly
pub fn test_load_testing_sustained_load() {
    let prop = for_all_named(
        arbitrary_seed(),
        "seed",
        |&_seed: &Seed| {
            let config = LoadTestConfig {
                thread_count: 2,
                duration: Duration::from_millis(100), // Short test
                ops_per_second: Some(50), // Rate limiting
                ramp_up_duration: Duration::from_millis(10),
                cool_down_duration: Duration::from_millis(10),
                collect_stats: true,
            };
            
            let load_generator = LoadGenerator::new(
                Gen::int_range(1, 50),
                |&n| {
                    // Simulate work with small delay
                    std::thread::sleep(Duration::from_micros(100));
                    
                    if n > 0 && n <= 50 {
                        TestResult::Pass {
                            tests_run: 1,
                            property_name: Some("load_test".to_string()),
                            module_path: None,
                        }
                    } else {
                        TestResult::Fail {
                            counterexample: format!("Invalid value: {}", n),
                            tests_run: 1,
                            shrinks_performed: 0,
                            property_name: Some("load_test".to_string()),
                            module_path: None,
                            assertion_type: Some("Range Check".to_string()),
                            shrink_steps: Vec::new(),
                        }
                    }
                },
                config,
            );
            
            let result = load_generator.run_load_test();
            
            // Should complete load test with reasonable metrics
            result.stats.operations_completed > 0 &&
            result.success_rate >= 0.8 && // Most operations should succeed
            result.stats.avg_ops_per_second > 0.0 &&
            result.thread_results.len() == 2 &&
            result.phase_timings.total_time >= Duration::from_millis(100)
        }
    );
    
    let fast_config = Config::default().with_tests(2).with_shrinks(1);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Load testing sustained load property passed"),
        result => panic!("Load testing sustained load property failed: {:?}", result),
    }
}

/// Property: Concurrent scenarios should execute operations correctly  
pub fn test_concurrent_scenario_execution() {
    let prop = for_all_named(
        arbitrary_seed(),
        "seed",
        |&_seed: &Seed| {
            let scenario = concurrent_scenario("test_scenario")
                .operation("setup", |n: &i32| {
                    TestResult::Pass {
                        tests_run: 1,
                        property_name: Some(format!("setup_{}", n)),
                        module_path: None,
                    }
                })
                .operation("main_work", |n: &i32| {
                    if *n > 0 {
                        TestResult::Pass {
                            tests_run: 1,
                            property_name: Some(format!("main_{}", n)),
                            module_path: None,
                        }
                    } else {
                        TestResult::Fail {
                            counterexample: format!("Invalid input: {}", n),
                            tests_run: 1,
                            shrinks_performed: 0,
                            property_name: Some(format!("main_{}", n)),
                            module_path: None,
                            assertion_type: Some("Positive Check".to_string()),
                            shrink_steps: Vec::new(),
                        }
                    }
                })
                .operation("cleanup", |_n: &i32| {
                    TestResult::Pass {
                        tests_run: 1,
                        property_name: Some("cleanup".to_string()),
                        module_path: None,
                    }
                })
                .before("setup", "main_work")
                .before("main_work", "cleanup")
                .build();
            
            // Test with positive input
            let result = scenario.execute(&42);
            
            result.scenario_name == "test_scenario" &&
            result.operation_results.len() == 3 &&
            result.operation_results.contains_key("setup") &&
            result.operation_results.contains_key("main_work") &&
            result.operation_results.contains_key("cleanup") &&
            result.execution_time > Duration::from_nanos(0)
        }
    );
    
    let fast_config = Config::default().with_tests(5).with_shrinks(2);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Concurrent scenario execution property passed"),
        result => panic!("Concurrent scenario execution property failed: {:?}", result),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_parallel_property_tests() {
        test_parallel_work_distribution();
        test_work_distribution_strategies();
        test_concurrent_non_determinism_detection();
        test_deadlock_detection();
        test_parallel_performance_benefits();
        test_load_testing_sustained_load();
        test_concurrent_scenario_execution();
    }
}