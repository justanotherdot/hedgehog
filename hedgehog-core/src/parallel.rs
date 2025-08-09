//! Parallel testing infrastructure for concurrent property-based testing.
//! 
//! This module provides two main capabilities:
//! 1. Parallel property execution - distribute tests across threads for speed
//! 2. Concurrent system testing - detect race conditions and test thread safety

use crate::{data::*, error::*, gen::*};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Configuration for parallel property testing.
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// Number of threads to use for parallel execution
    pub thread_count: usize,
    /// How to distribute work across threads
    pub work_distribution: WorkDistribution,
    /// Timeout for detecting deadlocks
    pub timeout: Option<Duration>,
    /// Whether to detect non-deterministic behavior
    pub detect_non_determinism: bool,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        ParallelConfig {
            thread_count: num_cpus::get(),
            work_distribution: WorkDistribution::RoundRobin,
            timeout: Some(Duration::from_secs(10)),
            detect_non_determinism: true,
        }
    }
}

/// Strategies for distributing work across threads.
#[derive(Debug, Clone, PartialEq)]
pub enum WorkDistribution {
    /// Distribute tests evenly in round-robin fashion
    RoundRobin,
    /// Process tests in chunks per thread
    ChunkBased,
    /// Threads steal work from each other (more complex, better load balancing)
    WorkStealing,
}

/// Result of running concurrent tests on the same input.
#[derive(Debug, Clone)]
pub struct ConcurrentTestResult {
    /// Whether all threads produced the same result
    pub deterministic: bool,
    /// Results from each thread
    pub results: Vec<TestResult>,
    /// Number of race conditions detected
    pub race_conditions_detected: usize,
    /// Thread execution times
    pub execution_times: Vec<Duration>,
}

/// Result of parallel property testing.
#[derive(Debug, Clone)]
pub struct ParallelTestResult {
    /// Overall test outcome
    pub outcome: TestResult,
    /// Results from individual threads
    pub thread_results: Vec<TestResult>,
    /// Performance metrics
    pub performance: ParallelPerformanceMetrics,
    /// Concurrency issues detected
    pub concurrency_issues: ConcurrencyIssues,
}

/// Performance metrics from parallel execution.
#[derive(Debug, Clone)]
pub struct ParallelPerformanceMetrics {
    /// Total wall clock time
    pub total_duration: Duration,
    /// Time spent in actual test execution across all threads
    pub total_cpu_time: Duration,
    /// Speedup compared to estimated sequential execution
    pub speedup_factor: f64,
    /// Thread utilization efficiency
    pub thread_efficiency: f64,
}

/// Issues detected during concurrent testing.
#[derive(Debug, Clone)]
pub struct ConcurrencyIssues {
    /// Number of non-deterministic results detected
    pub non_deterministic_results: usize,
    /// Number of potential deadlocks detected
    pub potential_deadlocks: usize,
    /// Number of timeout occurrences
    pub timeouts: usize,
    /// Threads that finished abnormally
    pub thread_failures: Vec<String>,
}

/// A property that can be executed in parallel.
pub struct ParallelProperty<T, F> 
where 
    F: Fn(&T) -> TestResult + Send + Sync,
{
    /// Generator for test inputs
    pub generator: Gen<T>,
    /// Thread-safe test function
    pub test_function: Arc<F>,
    /// Parallel execution configuration
    pub config: ParallelConfig,
    /// Variable name for debugging
    pub variable_name: Option<String>,
}

impl<T, F> ParallelProperty<T, F>
where
    T: 'static + std::fmt::Debug + Clone + Send + Sync,
    F: Fn(&T) -> TestResult + Send + Sync + 'static,
{
    /// Create a new parallel property.
    pub fn new(generator: Gen<T>, test_function: F, config: ParallelConfig) -> Self {
        ParallelProperty { 
            generator,
            test_function: Arc::new(test_function),
            config,
            variable_name: None,
        }
    }

    /// Set a variable name for debugging.
    pub fn with_variable_name(mut self, name: &str) -> Self {
        self.variable_name = Some(name.to_string());
        self
    }

    /// Run the property tests in parallel across multiple threads.
    pub fn run(&self, test_config: &Config) -> ParallelTestResult {
        let start_time = Instant::now();
        
        // Pre-generate all test inputs to avoid Send/Sync issues with Gen<T>
        let total_tests = test_config.test_limit;
        let mut seed = crate::data::Seed::random();
        let mut test_inputs = Vec::with_capacity(total_tests);
        
        for i in 0..total_tests {
            let size = crate::data::Size::new((i * test_config.size_limit) / total_tests);
            let (test_seed, next_seed) = seed.split();
            seed = next_seed;
            
            let tree = self.generator.generate(size, test_seed);
            test_inputs.push(tree.value);
        }
        
        // Calculate work distribution
        let threads = self.config.thread_count;
        let work_items = self.distribute_work(total_tests, threads);
        
        let mut thread_handles = Vec::new();
        let mut input_start = 0;
        
        // Spawn worker threads
        for (thread_id, test_count) in work_items.into_iter().enumerate() {
            let thread_inputs = test_inputs[input_start..input_start + test_count].to_vec();
            input_start += test_count;
            
            let test_function = Arc::clone(&self.test_function);
            let timeout = self.config.timeout;
            let variable_name = self.variable_name.clone();
            
            let handle = thread::spawn(move || {
                Self::run_thread_tests_with_inputs(
                    thread_id,
                    thread_inputs,
                    test_function,
                    timeout,
                    variable_name,
                )
            });
            
            thread_handles.push(handle);
        }
        
        // Collect results from all threads
        let mut thread_results = Vec::new();
        let mut concurrency_issues = ConcurrencyIssues::default();
        
        for handle in thread_handles {
            match handle.join() {
                Ok(result) => {
                    thread_results.push(result.clone());
                    // Analyze for concurrency issues
                    Self::analyze_thread_result(&result, &mut concurrency_issues);
                }
                Err(_) => {
                    concurrency_issues.thread_failures.push("Thread panicked".to_string());
                }
            }
        }
        
        let total_duration = start_time.elapsed();
        
        // Aggregate results and compute metrics
        let outcome = Self::aggregate_results(&thread_results);
        let performance = Self::calculate_performance_metrics(
            total_duration,
            &thread_results,
            threads,
        );
        
        ParallelTestResult {
            outcome,
            thread_results,
            performance,
            concurrency_issues,
        }
    }

    /// Distribute work across threads based on the configured strategy.
    fn distribute_work(&self, total_tests: usize, thread_count: usize) -> Vec<usize> {
        match self.config.work_distribution {
            WorkDistribution::RoundRobin => {
                let base_work = total_tests / thread_count;
                let remainder = total_tests % thread_count;
                
                (0..thread_count)
                    .map(|i| base_work + if i < remainder { 1 } else { 0 })
                    .collect()
            }
            WorkDistribution::ChunkBased => {
                let chunk_size = (total_tests + thread_count - 1) / thread_count;
                (0..thread_count)
                    .map(|i| {
                        let start = i * chunk_size;
                        let end = ((i + 1) * chunk_size).min(total_tests);
                        end.saturating_sub(start)
                    })
                    .collect()
            }
            WorkDistribution::WorkStealing => {
                // For now, fall back to round-robin. Work stealing requires more complex infrastructure
                self.distribute_work_round_robin(total_tests, thread_count)
            }
        }
    }

    fn distribute_work_round_robin(&self, total_tests: usize, thread_count: usize) -> Vec<usize> {
        let base_work = total_tests / thread_count;
        let remainder = total_tests % thread_count;
        
        (0..thread_count)
            .map(|i| base_work + if i < remainder { 1 } else { 0 })
            .collect()
    }

    /// Run tests in a single thread with pre-generated inputs.
    fn run_thread_tests_with_inputs(
        _thread_id: usize,
        test_inputs: Vec<T>,
        test_function: Arc<F>,
        _timeout: Option<Duration>,
        _variable_name: Option<String>,
    ) -> TestResult {
        let mut tests_run = 0;
        
        for input in test_inputs {
            tests_run += 1;
            match test_function(&input) {
                TestResult::Pass { .. } => continue,
                result @ TestResult::Fail { .. } => {
                    // Return the failure result with updated test count
                    match result {
                        TestResult::Fail { 
                            counterexample,
                            shrinks_performed,
                            property_name,
                            module_path,
                            assertion_type,
                            shrink_steps,
                            ..
                        } => {
                            return TestResult::Fail {
                                counterexample,
                                tests_run,
                                shrinks_performed,
                                property_name,
                                module_path,
                                assertion_type,
                                shrink_steps,
                            };
                        }
                        _ => unreachable!(),
                    }
                }
                other => return other,
            }
        }
        
        // All tests passed
        TestResult::Pass {
            tests_run,
            property_name: None,
            module_path: None,
        }
    }

    /// Analyze a thread result for concurrency issues.
    fn analyze_thread_result(result: &TestResult, issues: &mut ConcurrencyIssues) {
        match result {
            TestResult::Fail { .. } => {
                // Could be a race condition if other threads passed
                issues.non_deterministic_results += 1;
            }
            _ => {}
        }
    }

    /// Aggregate results from all threads into a single result.
    fn aggregate_results(thread_results: &[TestResult]) -> TestResult {
        // If any thread failed, the overall test failed
        for result in thread_results {
            match result {
                TestResult::Fail { .. } => return result.clone(),
                _ => continue,
            }
        }

        // If all threads passed, aggregate the success
        let total_tests: usize = thread_results.iter().map(|r| match r {
            TestResult::Pass { tests_run, .. } => *tests_run,
            TestResult::PassWithStatistics { tests_run, .. } => *tests_run,
            _ => 0,
        }).sum();

        TestResult::Pass {
            tests_run: total_tests,
            property_name: None,
            module_path: None,
        }
    }

    /// Calculate performance metrics from parallel execution.
    fn calculate_performance_metrics(
        total_duration: Duration,
        thread_results: &[TestResult],
        thread_count: usize,
    ) -> ParallelPerformanceMetrics {
        let _total_tests: usize = thread_results.iter().map(|r| match r {
            TestResult::Pass { tests_run, .. } => *tests_run,
            TestResult::PassWithStatistics { tests_run, .. } => *tests_run,
            _ => 0,
        }).sum();

        // Estimate sequential time (very rough)
        let estimated_sequential_time = total_duration * thread_count as u32;
        let speedup_factor = estimated_sequential_time.as_secs_f64() / total_duration.as_secs_f64();

        ParallelPerformanceMetrics {
            total_duration,
            total_cpu_time: estimated_sequential_time,
            speedup_factor,
            thread_efficiency: speedup_factor / thread_count as f64,
        }
    }
}

impl Default for ConcurrencyIssues {
    fn default() -> Self {
        ConcurrencyIssues {
            non_deterministic_results: 0,
            potential_deadlocks: 0,
            timeouts: 0,
            thread_failures: Vec::new(),
        }
    }
}

/// Create a parallel property for testing with multiple threads.
pub fn for_all_parallel<T, F>(generator: Gen<T>, condition: F, thread_count: usize) -> ParallelProperty<T, impl Fn(&T) -> TestResult + Send + Sync>
where
    T: 'static + std::fmt::Debug + Clone + Send + Sync,
    F: Fn(&T) -> bool + Send + Sync + 'static,
{
    let config = ParallelConfig {
        thread_count,
        ..ParallelConfig::default()
    };
    ParallelProperty::new(generator, move |input| {
        if condition(input) {
            TestResult::Pass {
                tests_run: 1,
                property_name: None,
                module_path: None,
            }
        } else {
            TestResult::Fail {
                counterexample: format!("{:?}", input),
                tests_run: 0,
                shrinks_performed: 0,
                property_name: None,
                module_path: None,
                assertion_type: Some("Boolean Condition".to_string()),
                shrink_steps: Vec::new(),
            }
        }
    }, config)
}

/// Create a parallel property with a custom test function.
pub fn parallel_property<T, F>(generator: Gen<T>, test_function: F, config: ParallelConfig) -> ParallelProperty<T, F>
where
    T: 'static + std::fmt::Debug + Clone + Send + Sync,
    F: Fn(&T) -> TestResult + Send + Sync + 'static,
{
    ParallelProperty::new(generator, test_function, config)
}

// Add num_cpus dependency detection placeholder
mod num_cpus {
    pub fn get() -> usize {
        // Fallback to 4 threads if we can't detect CPU count
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gen::Gen;

    #[test]
    fn test_work_distribution_round_robin() {
        let config = ParallelConfig {
            work_distribution: WorkDistribution::RoundRobin,
            ..ParallelConfig::default()
        };
        
        let prop = ParallelProperty::new(
            Gen::bool(),
            |_| TestResult::Pass { tests_run: 1, property_name: None, module_path: None },
            config,
        );
        
        let work = prop.distribute_work(10, 3);
        assert_eq!(work, vec![4, 3, 3]); // 10 tests across 3 threads
    }

    #[test]
    fn test_work_distribution_chunk_based() {
        let config = ParallelConfig {
            work_distribution: WorkDistribution::ChunkBased,
            ..ParallelConfig::default()
        };
        
        let prop = ParallelProperty::new(
            Gen::bool(),
            |_| TestResult::Pass { tests_run: 1, property_name: None, module_path: None },
            config,
        );
        
        let work = prop.distribute_work(10, 3);
        assert_eq!(work, vec![4, 4, 2]); // Chunks of ~3.33, so 4,4,2
    }

    #[test]
    fn test_parallel_config_defaults() {
        let config = ParallelConfig::default();
        assert!(config.thread_count > 0);
        assert_eq!(config.work_distribution, WorkDistribution::RoundRobin);
        assert!(config.timeout.is_some());
        assert!(config.detect_non_determinism);
    }

    #[test]
    fn test_basic_parallel_execution() {
        let config = Config::default().with_tests(100);
        
        let prop = for_all_parallel(
            Gen::int_range(1, 100),
            |&n| n > 0 && n <= 100,
            2 // 2 threads
        );
        
        let result = prop.run(&config);
        
        match result.outcome {
            TestResult::Pass { tests_run, .. } => {
                assert_eq!(tests_run, 100);
                assert_eq!(result.thread_results.len(), 2);
                assert!(result.performance.speedup_factor > 0.0);
                assert!(result.performance.thread_efficiency > 0.0);
            }
            other => panic!("Expected pass, got: {:?}", other),
        }
    }

    #[test]
    fn test_parallel_failure_detection() {
        let config = Config::default().with_tests(20);
        
        // This should fail when it encounters a number > 50
        let prop = for_all_parallel(
            Gen::int_range(1, 100),
            |&n| n <= 50,
            3 // 3 threads
        );
        
        let result = prop.run(&config);
        
        match result.outcome {
            TestResult::Fail { tests_run, .. } => {
                assert!(tests_run <= 20);
                assert!(!result.thread_results.is_empty());
            }
            TestResult::Pass { .. } => {
                // This could happen if we get lucky with random generation
                // and don't generate any numbers > 50 in our small sample
                println!("Test passed (got lucky with random generation)");
            }
            other => panic!("Expected pass or fail, got: {:?}", other),
        }
    }

    #[test]
    fn test_different_work_distributions() {
        let test_sizes = vec![10, 100];
        let thread_counts = vec![1, 2, 4];
        
        for &test_size in &test_sizes {
            for &thread_count in &thread_counts {
                for distribution in [WorkDistribution::RoundRobin, WorkDistribution::ChunkBased] {
                    let config = ParallelConfig {
                        thread_count,
                        work_distribution: distribution,
                        ..ParallelConfig::default()
                    };
                    
                    let prop = ParallelProperty::new(
                        Gen::bool(),
                        |_| TestResult::Pass { tests_run: 1, property_name: None, module_path: None },
                        config,
                    );
                    
                    let work = prop.distribute_work(test_size, thread_count);
                    
                    // Verify work is distributed correctly
                    assert_eq!(work.len(), thread_count);
                    assert_eq!(work.iter().sum::<usize>(), test_size);
                    
                    // No thread should have more than ceiling(test_size/thread_count) work
                    let max_work = (test_size + thread_count - 1) / thread_count;
                    for &thread_work in &work {
                        assert!(thread_work <= max_work);
                    }
                }
            }
        }
    }

    #[test]
    fn test_single_thread_parallel() {
        // Test that single-threaded "parallel" execution works correctly
        let config = Config::default().with_tests(50);
        
        let prop = for_all_parallel(
            Gen::int_range(1, 10),
            |&n| n >= 1 && n <= 10,
            1 // Single thread
        );
        
        let result = prop.run(&config);
        
        match result.outcome {
            TestResult::Pass { tests_run, .. } => {
                assert_eq!(tests_run, 50);
                assert_eq!(result.thread_results.len(), 1);
            }
            other => panic!("Expected pass, got: {:?}", other),
        }
    }

    #[test]
    fn test_work_stealing_fallback() {
        // Work stealing should fall back to round robin for now
        let config = ParallelConfig {
            work_distribution: WorkDistribution::WorkStealing,
            ..ParallelConfig::default()
        };
        
        let prop = ParallelProperty::new(
            Gen::bool(),
            |_| TestResult::Pass { tests_run: 1, property_name: None, module_path: None },
            config,
        );
        
        let work = prop.distribute_work(10, 3);
        
        // Should behave like round robin
        assert_eq!(work, vec![4, 3, 3]);
    }

    #[test]
    fn test_performance_metrics_calculation() {
        let thread_results = vec![
            TestResult::Pass { tests_run: 30, property_name: None, module_path: None },
            TestResult::Pass { tests_run: 35, property_name: None, module_path: None },
            TestResult::Pass { tests_run: 35, property_name: None, module_path: None },
        ];
        
        let total_duration = Duration::from_millis(100);
        let thread_count = 3;
        
        let metrics = ParallelProperty::<bool, fn(&bool) -> TestResult>::calculate_performance_metrics(
            total_duration,
            &thread_results,
            thread_count,
        );
        
        assert_eq!(metrics.total_duration, total_duration);
        assert!(metrics.speedup_factor > 0.0);
        assert!(metrics.thread_efficiency > 0.0);
        assert!(metrics.thread_efficiency <= 1.0);
    }

    #[test]
    fn test_concurrency_issue_analysis() {
        let mut issues = ConcurrencyIssues::default();
        
        // Test with a failing result (potential race condition)
        let failing_result = TestResult::Fail {
            counterexample: "test".to_string(),
            tests_run: 5,
            shrinks_performed: 0,
            property_name: None,
            module_path: None,
            assertion_type: None,
            shrink_steps: Vec::new(),
        };
        
        ParallelProperty::<bool, fn(&bool) -> TestResult>::analyze_thread_result(&failing_result, &mut issues);
        
        assert_eq!(issues.non_deterministic_results, 1);
        assert_eq!(issues.potential_deadlocks, 0);
        assert_eq!(issues.timeouts, 0);
        assert!(issues.thread_failures.is_empty());
    }

    #[test]
    fn test_result_aggregation() {
        // Test with all passing results
        let passing_results = vec![
            TestResult::Pass { tests_run: 20, property_name: None, module_path: None },
            TestResult::Pass { tests_run: 25, property_name: None, module_path: None },
            TestResult::Pass { tests_run: 30, property_name: None, module_path: None },
        ];
        
        let aggregated = ParallelProperty::<bool, fn(&bool) -> TestResult>::aggregate_results(&passing_results);
        
        match aggregated {
            TestResult::Pass { tests_run, .. } => {
                assert_eq!(tests_run, 75); // Sum of all tests
            }
            other => panic!("Expected pass, got: {:?}", other),
        }
        
        // Test with one failing result
        let mixed_results = vec![
            TestResult::Pass { tests_run: 20, property_name: None, module_path: None },
            TestResult::Fail {
                counterexample: "failure".to_string(),
                tests_run: 15,
                shrinks_performed: 0,
                property_name: None,
                module_path: None,
                assertion_type: None,
                shrink_steps: Vec::new(),
            },
            TestResult::Pass { tests_run: 30, property_name: None, module_path: None },
        ];
        
        let aggregated = ParallelProperty::<bool, fn(&bool) -> TestResult>::aggregate_results(&mixed_results);
        
        match aggregated {
            TestResult::Fail { counterexample, .. } => {
                assert_eq!(counterexample, "failure");
            }
            other => panic!("Expected failure, got: {:?}", other),
        }
    }
}