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
    /// Deadlock information if detected
    pub deadlock_info: Option<DeadlockInfo>,
    /// Whether timeout was detected
    pub timeout_detected: bool,
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
    /// Details about detected deadlocks
    pub deadlock_details: Vec<DeadlockInfo>,
}

/// Information about a detected deadlock.
#[derive(Debug, Clone)]
pub struct DeadlockInfo {
    /// Test input that triggered the deadlock
    pub input: String,
    /// Threads that were involved in the deadlock
    pub threads_involved: Vec<usize>,
    /// Duration before timeout
    pub timeout_duration: Duration,
    /// Timestamp when deadlock was detected
    pub detected_at: std::time::SystemTime,
}

/// Error types for thread joining operations.
#[derive(Debug)]
enum JoinError {
    /// Thread timed out (possible deadlock)
    Timeout,
    /// Thread panicked
    Panic,
}

/// A concurrent scenario definition for complex interleaving tests.
#[derive(Clone)]
pub struct ConcurrentScenario<T> {
    /// Name of the scenario
    pub name: String,
    /// Operations to execute concurrently
    pub operations: Vec<Operation<T>>,
    /// Synchronization barriers
    pub barriers: Vec<Barrier>,
    /// Expected interleaving constraints
    pub constraints: Vec<InterleavingConstraint>,
}

/// A single operation in a concurrent scenario.
#[derive(Clone)]
pub struct Operation<T> {
    /// Unique identifier for this operation
    pub id: String,
    /// The function to execute
    pub function: Arc<dyn Fn(&T) -> TestResult + Send + Sync>,
    /// Thread to run on (None = any available thread)
    pub thread_id: Option<usize>,
    /// Dependencies - operations that must complete before this one
    pub depends_on: Vec<String>,
    /// Whether this operation is optional (can be skipped if dependencies fail)
    pub optional: bool,
}

/// Synchronization barrier in concurrent scenarios.
#[derive(Debug, Clone)]
pub struct Barrier {
    /// Name of the barrier
    pub name: String,
    /// Operations that must reach this barrier
    pub operations: Vec<String>,
    /// Timeout for the barrier
    pub timeout: Option<Duration>,
}

/// Constraints on operation interleaving.
#[derive(Debug, Clone)]
pub enum InterleavingConstraint {
    /// Operation A must happen before operation B
    Before { before: String, after: String },
    /// Operations must happen atomically (no interleaving)
    Atomic { operations: Vec<String> },
    /// Operations must never run simultaneously
    Exclusive { operations: Vec<String> },
    /// At least one of these operations must succeed
    OneOf { operations: Vec<String> },
}

/// Builder for creating concurrent scenarios.
pub struct ConcurrentScenarioBuilder<T> {
    scenario: ConcurrentScenario<T>,
}

impl<T> ConcurrentScenarioBuilder<T> {
    /// Create a new scenario builder.
    pub fn new(name: &str) -> Self {
        ConcurrentScenarioBuilder {
            scenario: ConcurrentScenario {
                name: name.to_string(),
                operations: Vec::new(),
                barriers: Vec::new(),
                constraints: Vec::new(),
            },
        }
    }

    /// Add an operation to the scenario.
    pub fn operation<F>(mut self, id: &str, function: F) -> Self 
    where 
        F: Fn(&T) -> TestResult + Send + Sync + 'static,
    {
        self.scenario.operations.push(Operation {
            id: id.to_string(),
            function: Arc::new(function),
            thread_id: None,
            depends_on: Vec::new(),
            optional: false,
        });
        self
    }

    /// Add an operation that depends on other operations.
    pub fn operation_depends_on<F>(mut self, id: &str, depends_on: Vec<&str>, function: F) -> Self 
    where 
        F: Fn(&T) -> TestResult + Send + Sync + 'static,
    {
        self.scenario.operations.push(Operation {
            id: id.to_string(),
            function: Arc::new(function),
            thread_id: None,
            depends_on: depends_on.into_iter().map(|s| s.to_string()).collect(),
            optional: false,
        });
        self
    }

    /// Add an operation on a specific thread.
    pub fn operation_on_thread<F>(mut self, id: &str, thread_id: usize, function: F) -> Self 
    where 
        F: Fn(&T) -> TestResult + Send + Sync + 'static,
    {
        self.scenario.operations.push(Operation {
            id: id.to_string(),
            function: Arc::new(function),
            thread_id: Some(thread_id),
            depends_on: Vec::new(),
            optional: false,
        });
        self
    }

    /// Add a synchronization barrier.
    pub fn barrier(mut self, name: &str, operations: Vec<&str>) -> Self {
        self.scenario.barriers.push(Barrier {
            name: name.to_string(),
            operations: operations.into_iter().map(|s| s.to_string()).collect(),
            timeout: Some(Duration::from_secs(10)),
        });
        self
    }

    /// Add a constraint that one operation must happen before another.
    pub fn before(mut self, before: &str, after: &str) -> Self {
        self.scenario.constraints.push(InterleavingConstraint::Before {
            before: before.to_string(),
            after: after.to_string(),
        });
        self
    }

    /// Add a constraint that operations must be atomic (no interleaving).
    pub fn atomic(mut self, operations: Vec<&str>) -> Self {
        self.scenario.constraints.push(InterleavingConstraint::Atomic {
            operations: operations.into_iter().map(|s| s.to_string()).collect(),
        });
        self
    }

    /// Add a constraint that operations are mutually exclusive.
    pub fn exclusive(mut self, operations: Vec<&str>) -> Self {
        self.scenario.constraints.push(InterleavingConstraint::Exclusive {
            operations: operations.into_iter().map(|s| s.to_string()).collect(),
        });
        self
    }

    /// Build the scenario.
    pub fn build(self) -> ConcurrentScenario<T> {
        self.scenario
    }
}

impl<T> ConcurrentScenario<T> 
where
    T: 'static + std::fmt::Debug + Clone + Send + Sync,
{
    /// Execute the scenario with given input.
    pub fn execute(&self, input: &T) -> ScenarioResult {
        let start_time = Instant::now();
        let mut operation_results = std::collections::HashMap::new();
        let mut constraint_violations = Vec::new();
        let deadlocks_detected = false;

        // For now, implement a simple sequential execution with constraint checking
        // TODO: Implement proper concurrent execution with barriers and dependencies
        
        for operation in &self.operations {
            let result = (operation.function)(input);
            operation_results.insert(operation.id.clone(), result);
        }

        // Check constraints
        let constraints_satisfied = self.check_constraints(&operation_results, &mut constraint_violations);

        ScenarioResult {
            scenario_name: self.name.clone(),
            operation_results,
            constraints_satisfied,
            constraint_violations,
            execution_time: start_time.elapsed(),
            deadlocks_detected,
        }
    }

    /// Check if all constraints are satisfied.
    fn check_constraints(
        &self, 
        _results: &std::collections::HashMap<String, TestResult>,
        violations: &mut Vec<String>
    ) -> bool {
        // For now, just return true - proper constraint checking requires execution order tracking
        // TODO: Implement actual constraint validation based on execution traces
        for constraint in &self.constraints {
            match constraint {
                InterleavingConstraint::Before { before, after } => {
                    // This would need execution timestamps to verify
                    // For now, just log what we're checking
                    if violations.is_empty() { // Placeholder to avoid unused variable warning
                        violations.push(format!("Cannot verify 'before' constraint: {} -> {}", before, after));
                    }
                }
                InterleavingConstraint::Atomic { operations } => {
                    violations.push(format!("Cannot verify 'atomic' constraint for operations: {:?}", operations));
                }
                InterleavingConstraint::Exclusive { operations } => {
                    violations.push(format!("Cannot verify 'exclusive' constraint for operations: {:?}", operations));
                }
                InterleavingConstraint::OneOf { operations } => {
                    violations.push(format!("Cannot verify 'one_of' constraint for operations: {:?}", operations));
                }
            }
        }
        
        // For now, return true if no violations were found
        violations.is_empty()
    }
}

/// Result of executing a concurrent scenario.
#[derive(Debug, Clone)]
pub struct ScenarioResult {
    /// Name of the scenario that was executed
    pub scenario_name: String,
    /// Results from each operation
    pub operation_results: std::collections::HashMap<String, TestResult>,
    /// Whether all constraints were satisfied
    pub constraints_satisfied: bool,
    /// Constraint violations found
    pub constraint_violations: Vec<String>,
    /// Total execution time
    pub execution_time: Duration,
    /// Whether any deadlocks were detected
    pub deadlocks_detected: bool,
}
/// A property that tests the same input from multiple threads simultaneously.
pub struct ConcurrentProperty<T, F>
where
    F: Fn(&T) -> TestResult + Send + Sync,
{
    /// Generator for test inputs
    pub generator: Gen<T>,
    /// Thread-safe test function
    pub test_function: Arc<F>,
    /// Number of threads to run concurrently
    pub thread_count: usize,
    /// Timeout for each concurrent test
    pub timeout: Option<Duration>,
    /// Variable name for debugging
    pub variable_name: Option<String>,
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

impl<T, F> ConcurrentProperty<T, F>
where
    T: 'static + std::fmt::Debug + Clone + Send + Sync,
    F: Fn(&T) -> TestResult + Send + Sync + 'static,
{
    /// Create a new concurrent property.
    pub fn new(generator: Gen<T>, test_function: F, thread_count: usize) -> Self {
        ConcurrentProperty {
            generator,
            test_function: Arc::new(test_function),
            thread_count,
            timeout: Some(Duration::from_secs(10)),
            variable_name: None,
        }
    }

    /// Set a variable name for debugging.
    pub fn with_variable_name(mut self, name: &str) -> Self {
        self.variable_name = Some(name.to_string());
        self
    }

    /// Set a timeout for concurrent tests.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Run concurrent tests on generated inputs to detect non-deterministic behavior.
    pub fn run(&self, test_config: &Config) -> Vec<ConcurrentTestResult> {
        let mut results = Vec::new();
        let mut seed = crate::data::Seed::random();

        for i in 0..test_config.test_limit {
            let size = crate::data::Size::new((i * test_config.size_limit) / test_config.test_limit);
            let (test_seed, next_seed) = seed.split();
            seed = next_seed;

            // Generate a single input to test concurrently
            let tree = self.generator.generate(size, test_seed);
            let input = tree.value;

            // Test this input concurrently from multiple threads
            let concurrent_result = self.test_input_concurrently(&input);
            results.push(concurrent_result);
        }

        results
    }

    /// Test a single input from multiple threads simultaneously to detect race conditions.
    fn test_input_concurrently(&self, input: &T) -> ConcurrentTestResult {
        self.test_input_concurrently_with_deadlock_detection(input)
    }

    /// Enhanced concurrent testing with deadlock detection.
    fn test_input_concurrently_with_deadlock_detection(&self, input: &T) -> ConcurrentTestResult {
        let mut thread_handles = Vec::new();
        let timeout_duration = self.timeout.unwrap_or(Duration::from_secs(10));
        let test_start = Instant::now();

        // Clone input for each thread
        for thread_id in 0..self.thread_count {
            let input_clone = input.clone();
            let test_function = Arc::clone(&self.test_function);

            let handle = thread::spawn(move || {
                let thread_start = Instant::now();
                let result = test_function(&input_clone);
                let thread_duration = thread_start.elapsed();
                (thread_id, result, thread_duration)
            });

            thread_handles.push(handle);
        }

        // Collect results from all threads with timeout detection
        let mut thread_results = Vec::new();
        let mut execution_times = Vec::new();
        let mut race_conditions_detected = 0;
        let mut timeout_detected = false;
        let mut hanging_threads = Vec::new();

        for (idx, handle) in thread_handles.into_iter().enumerate() {
            // Check if we've already exceeded our timeout
            let elapsed = test_start.elapsed();
            if elapsed > timeout_duration {
                timeout_detected = true;
                hanging_threads.push(idx);
                
                // Add a timeout failure result
                thread_results.push(TestResult::Fail {
                    counterexample: format!("Thread {} timed out after {:?} with input: {:?}", idx, elapsed, input),
                    tests_run: 1,
                    shrinks_performed: 0,
                    property_name: self.variable_name.clone(),
                    module_path: None,
                    assertion_type: Some("Deadlock/Timeout".to_string()),
                    shrink_steps: Vec::new(),
                });
                execution_times.push(timeout_duration);
                race_conditions_detected += 1;
                continue;
            }

            // Try to join with remaining timeout
            let remaining_timeout = timeout_duration - elapsed;
            let join_result = self.join_with_timeout(handle, remaining_timeout);

            match join_result {
                Ok((_thread_id, result, duration)) => {
                    thread_results.push(result);
                    execution_times.push(duration);
                }
                Err(JoinError::Timeout) => {
                    timeout_detected = true;
                    hanging_threads.push(idx);
                    
                    thread_results.push(TestResult::Fail {
                        counterexample: format!("Thread {} timed out after {:?} with input: {:?}", idx, timeout_duration, input),
                        tests_run: 1,
                        shrinks_performed: 0,
                        property_name: self.variable_name.clone(),
                        module_path: None,
                        assertion_type: Some("Deadlock/Timeout".to_string()),
                        shrink_steps: Vec::new(),
                    });
                    execution_times.push(timeout_duration);
                    race_conditions_detected += 1;
                }
                Err(JoinError::Panic) => {
                    // Thread panicked - this is a concurrency issue
                    thread_results.push(TestResult::Fail {
                        counterexample: format!("Thread {} panicked with input: {:?}", idx, input),
                        tests_run: 1,
                        shrinks_performed: 0,
                        property_name: self.variable_name.clone(),
                        module_path: None,
                        assertion_type: Some("Thread Panic".to_string()),
                        shrink_steps: Vec::new(),
                    });
                    execution_times.push(Duration::from_secs(0));
                    race_conditions_detected += 1;
                }
            }
        }

        // Analyze results for determinism
        let deterministic = if timeout_detected { false } else { self.analyze_determinism(&thread_results) };
        if !deterministic && !timeout_detected {
            race_conditions_detected += 1;
        }

        // If we detected timeouts, it could indicate a deadlock
        if timeout_detected {
            race_conditions_detected += hanging_threads.len();
        }

        // Generate deadlock info if timeout was detected
        let deadlock_info = if timeout_detected {
            Some(DeadlockInfo {
                input: format!("{:?}", input),
                threads_involved: hanging_threads,
                timeout_duration,
                detected_at: std::time::SystemTime::now(),
            })
        } else {
            None
        };
        ConcurrentTestResult {
            deterministic,
            results: thread_results,
            race_conditions_detected,
            execution_times,
            deadlock_info,
            timeout_detected,
        }
    }

    /// Join a thread handle with timeout support.
    fn join_with_timeout(&self, handle: thread::JoinHandle<(usize, TestResult, Duration)>, timeout: Duration) -> std::result::Result<(usize, TestResult, Duration), JoinError> {
        // Rust's JoinHandle doesn't have built-in timeout, so we simulate it
        // In a production implementation, you'd want to use a more sophisticated approach
        // For now, we'll use a simple busy-wait approach
        let start = Instant::now();
        let mut handle = Some(handle);
        
        while start.elapsed() < timeout {
            if let Some(h) = &handle {
                if h.is_finished() {
                    match handle.take().unwrap().join() {
                        Ok(result) => return Ok(result),
                        Err(_) => return Err(JoinError::Panic),
                    }
                }
            }
            thread::sleep(Duration::from_millis(10)); // Small delay to avoid busy-waiting
        }
        
        // If we get here, we timed out
        Err(JoinError::Timeout)
    }
    /// Analyze thread results to determine if they are deterministic.
    fn analyze_determinism(&self, results: &[TestResult]) -> bool {
        if results.is_empty() {
            return true;
        }

        let first_result_type = Self::result_type(&results[0]);
        
        // Check if all results have the same type and outcome
        for result in results.iter().skip(1) {
            if Self::result_type(result) != first_result_type {
                return false;
            }
            
            // For failures, also check if counterexamples are the same
            match (&results[0], result) {
                (TestResult::Fail { counterexample: ce1, .. }, TestResult::Fail { counterexample: ce2, .. }) => {
                    if ce1 != ce2 {
                        return false;
                    }
                }
                _ => {}
            }
        }
        
        true
    }

    /// Get a simplified result type for comparison.
    fn result_type(result: &TestResult) -> &'static str {
        match result {
            TestResult::Pass { .. } => "pass",
            TestResult::PassWithStatistics { .. } => "pass_with_stats",
            TestResult::Fail { .. } => "fail",
            TestResult::Discard { .. } => "discard",
        }
    }
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
            deadlock_details: Vec::new(),
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

/// Test the same input simultaneously from multiple threads to detect race conditions.
/// 
/// This function takes a single generated input and tests it concurrently from multiple threads.
/// It detects non-deterministic behavior by comparing results across threads.
pub fn for_all_concurrent<T, F>(
    generator: Gen<T>, 
    condition: F, 
    thread_count: usize
) -> ConcurrentProperty<T, impl Fn(&T) -> TestResult + Send + Sync>
where
    T: 'static + std::fmt::Debug + Clone + Send + Sync,
    F: Fn(&T) -> bool + Send + Sync + 'static,
{
    ConcurrentProperty::new(generator, move |input| {
        if condition(input) {
            TestResult::Pass {
                tests_run: 1,
                property_name: None,
                module_path: None,
            }
        } else {
            TestResult::Fail {
                counterexample: format!("{:?}", input),
                tests_run: 1,
                shrinks_performed: 0,
                property_name: None,
                module_path: None,
                assertion_type: Some("Boolean Condition".to_string()),
                shrink_steps: Vec::new(),
            }
        }
    }, thread_count)
}

/// Create a concurrent scenario builder.
pub fn concurrent_scenario<T>(name: &str) -> ConcurrentScenarioBuilder<T> {
    ConcurrentScenarioBuilder::new(name)
}

/// Systematic interleaving exploration for finding race conditions.
pub struct InterleavingExplorer<T, F> 
where
    F: Fn(&T) -> TestResult + Send + Sync,
{
    /// Generator for test inputs
    pub generator: Gen<T>,
    /// Test function to run with different interleavings
    pub test_function: Arc<F>,
    /// Number of concurrent operations to explore
    pub operation_count: usize,
    /// Maximum number of interleavings to explore (to prevent combinatorial explosion)
    pub max_interleavings: usize,
    /// Timeout for each interleaving test
    pub timeout: Option<Duration>,
}

/// Result of systematic interleaving exploration.
#[derive(Debug, Clone)]
pub struct InterleavingResult {
    /// Total number of interleavings explored
    pub interleavings_explored: usize,
    /// Number of interleavings that failed
    pub failed_interleavings: usize,
    /// Specific interleaving patterns that caused failures
    pub failing_patterns: Vec<InterleavingPattern>,
    /// Whether all interleavings were deterministic
    pub deterministic: bool,
    /// Race conditions detected
    pub race_conditions_detected: usize,
}

/// A specific interleaving pattern that caused a failure.
#[derive(Debug, Clone)]
pub struct InterleavingPattern {
    /// The sequence of thread operations
    pub sequence: Vec<ThreadOperation>,
    /// The failure that occurred
    pub failure_result: TestResult,
    /// Threads involved in this pattern
    pub threads_involved: Vec<usize>,
}

/// A single thread operation in an interleaving.
#[derive(Debug, Clone)]
pub struct ThreadOperation {
    /// Thread ID that performed the operation
    pub thread_id: usize,
    /// Step number in the overall execution
    pub step: usize,
    /// Operation identifier
    pub operation: String,
}

impl<T, F> InterleavingExplorer<T, F>
where
    T: 'static + std::fmt::Debug + Clone + Send + Sync,
    F: Fn(&T) -> TestResult + Send + Sync + 'static,
{
    /// Create a new interleaving explorer.
    pub fn new(generator: Gen<T>, test_function: F) -> Self {
        InterleavingExplorer {
            generator,
            test_function: Arc::new(test_function),
            operation_count: 3, // Default to 3 concurrent operations
            max_interleavings: 50, // Limit to prevent combinatorial explosion
            timeout: Some(Duration::from_secs(1)),
        }
    }

    /// Set the number of concurrent operations to explore.
    pub fn with_operations(mut self, count: usize) -> Self {
        self.operation_count = count;
        self
    }

    /// Set the maximum number of interleavings to explore.
    pub fn with_max_interleavings(mut self, max: usize) -> Self {
        self.max_interleavings = max;
        self
    }

    /// Set timeout for each interleaving test.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Explore different interleavings systematically.
    pub fn explore(&self, test_config: &Config) -> Vec<InterleavingResult> {
        let mut results = Vec::new();
        let mut seed = crate::data::Seed::random();

        for i in 0..test_config.test_limit {
            let size = crate::data::Size::new((i * test_config.size_limit) / test_config.test_limit);
            let (test_seed, next_seed) = seed.split();
            seed = next_seed;

            // Generate a single input to test with different interleavings
            let tree = self.generator.generate(size, test_seed);
            let input = tree.value;

            // Explore all possible interleavings for this input
            let interleaving_result = self.explore_input_interleavings(&input);
            results.push(interleaving_result);
        }

        results
    }

    /// Explore all interleavings for a specific input.
    fn explore_input_interleavings(&self, input: &T) -> InterleavingResult {
        let mut interleavings_explored = 0;
        let mut failed_interleavings = 0;
        let mut failing_patterns = Vec::new();
        let mut race_conditions_detected = 0;

        // For now, implement a simplified version that just tests with different random seeds
        // TODO: Implement true systematic interleaving exploration
        
        for interleaving_attempt in 0..self.max_interleavings {
            interleavings_explored += 1;
            
            // For now, just run the test function multiple times
            // A real implementation would control thread scheduling
            let results = self.run_concurrent_test_with_controlled_scheduling(input, interleaving_attempt);
            
            // Check if this interleaving produced different results (indicating race conditions)
            if !self.is_deterministic(&results) {
                failed_interleavings += 1;
                race_conditions_detected += 1;
                
                // Create a failing pattern
                let pattern = InterleavingPattern {
                    sequence: self.generate_thread_sequence(interleaving_attempt),
                    failure_result: results.get(0).cloned().unwrap_or_else(|| TestResult::Fail {
                        counterexample: format!("Non-deterministic result for input: {:?}", input),
                        tests_run: 1,
                        shrinks_performed: 0,
                        property_name: None,
                        module_path: None,
                        assertion_type: Some("Race Condition".to_string()),
                        shrink_steps: Vec::new(),
                    }),
                    threads_involved: (0..self.operation_count).collect(),
                };
                
                failing_patterns.push(pattern);
            }
        }

        InterleavingResult {
            interleavings_explored,
            failed_interleavings,
            failing_patterns,
            deterministic: failed_interleavings == 0,
            race_conditions_detected,
        }
    }

    /// Run concurrent test with controlled scheduling (simplified implementation).
    fn run_concurrent_test_with_controlled_scheduling(&self, input: &T, _schedule_seed: usize) -> Vec<TestResult> {
        // For now, just run the same test from multiple threads
        // A real implementation would use thread scheduling control
        let mut handles = Vec::new();
        
        for _thread_id in 0..self.operation_count {
            let input_clone = input.clone();
            let test_function = Arc::clone(&self.test_function);
            
            let handle = thread::spawn(move || {
                test_function(&input_clone)
            });
            
            handles.push(handle);
        }
        
        let mut results = Vec::new();
        for handle in handles {
            match handle.join() {
                Ok(result) => results.push(result),
                Err(_) => results.push(TestResult::Fail {
                    counterexample: format!("Thread panic with input: {:?}", input),
                    tests_run: 1,
                    shrinks_performed: 0,
                    property_name: None,
                    module_path: None,
                    assertion_type: Some("Thread Panic".to_string()),
                    shrink_steps: Vec::new(),
                }),
            }
        }
        
        results
    }

    /// Check if results from different threads are deterministic.
    fn is_deterministic(&self, results: &[TestResult]) -> bool {
        if results.is_empty() {
            return true;
        }
        
        let first_result_type = Self::result_type(&results[0]);
        results.iter().all(|r| Self::result_type(r) == first_result_type)
    }

    /// Get a simplified result type for comparison.
    fn result_type(result: &TestResult) -> &'static str {
        match result {
            TestResult::Pass { .. } => "pass",
            TestResult::PassWithStatistics { .. } => "pass_with_stats", 
            TestResult::Fail { .. } => "fail",
            TestResult::Discard { .. } => "discard",
        }
    }

    /// Generate a thread sequence for a given interleaving attempt (placeholder).
    fn generate_thread_sequence(&self, attempt: usize) -> Vec<ThreadOperation> {
        // This is a simplified implementation
        // A real implementation would generate actual interleaving sequences
        (0..self.operation_count).map(|thread_id| {
            ThreadOperation {
                thread_id,
                step: attempt * self.operation_count + thread_id,
                operation: format!("op_{}", thread_id),
            }
        }).collect()
    }
}

/// Create an interleaving explorer for systematic race condition detection.
pub fn interleaving_explorer<T, F>(generator: Gen<T>, test_function: F) -> InterleavingExplorer<T, F>
where
    T: 'static + std::fmt::Debug + Clone + Send + Sync,
    F: Fn(&T) -> TestResult + Send + Sync + 'static,
{
    InterleavingExplorer::new(generator, test_function)
}

/// Load testing configuration for stress testing concurrent systems.
#[derive(Debug, Clone)]
pub struct LoadTestConfig {
    /// Number of concurrent threads to spawn
    pub thread_count: usize,
    /// Duration to sustain the load
    pub duration: Duration,
    /// Operations per second target (None = unlimited)
    pub ops_per_second: Option<usize>,
    /// Ramp-up time to reach target load
    pub ramp_up_duration: Duration,
    /// Cool-down time after reaching target
    pub cool_down_duration: Duration,
    /// Whether to collect detailed timing statistics
    pub collect_stats: bool,
}

impl Default for LoadTestConfig {
    fn default() -> Self {
        LoadTestConfig {
            thread_count: num_cpus::get(),
            duration: Duration::from_secs(10),
            ops_per_second: None,
            ramp_up_duration: Duration::from_secs(2),
            cool_down_duration: Duration::from_secs(1),
            collect_stats: true,
        }
    }
}

/// Statistics collected during load testing.
#[derive(Debug, Clone)]
pub struct LoadTestStats {
    /// Total operations completed
    pub operations_completed: usize,
    /// Total operations that failed
    pub operations_failed: usize,
    /// Average operations per second
    pub avg_ops_per_second: f64,
    /// Peak operations per second
    pub peak_ops_per_second: f64,
    /// Average response time
    pub avg_response_time: Duration,
    /// 95th percentile response time
    pub p95_response_time: Duration,
    /// 99th percentile response time  
    pub p99_response_time: Duration,
    /// Maximum response time
    pub max_response_time: Duration,
    /// Response time distribution
    pub response_times: Vec<Duration>,
    /// Thread utilization (0.0 to 1.0)
    pub thread_utilization: f64,
    /// Whether any deadlocks were detected
    pub deadlocks_detected: usize,
    /// Memory usage statistics (if available)
    pub memory_usage_mb: Option<f64>,
}

/// Result of a load test execution.
#[derive(Debug, Clone)]
pub struct LoadTestResult {
    /// Test configuration used
    pub config: LoadTestConfig,
    /// Performance statistics
    pub stats: LoadTestStats,
    /// Individual thread results
    pub thread_results: Vec<TestResult>,
    /// Test phases timing
    pub phase_timings: LoadTestPhases,
    /// Overall success rate
    pub success_rate: f64,
}

/// Timing information for different phases of load testing.
#[derive(Debug, Clone)]
pub struct LoadTestPhases {
    /// Time spent ramping up
    pub ramp_up_time: Duration,
    /// Time spent at steady state
    pub steady_state_time: Duration,
    /// Time spent cooling down
    pub cool_down_time: Duration,
    /// Total test execution time
    pub total_time: Duration,
}

/// Load generator for stress testing concurrent systems.
pub struct LoadGenerator<T, F> 
where
    F: Fn(&T) -> TestResult + Send + Sync,
{
    /// Generator for test inputs
    pub generator: Gen<T>,
    /// Test function to execute under load
    pub test_function: Arc<F>,
    /// Load test configuration
    pub config: LoadTestConfig,
}

impl<T, F> LoadGenerator<T, F>
where
    T: 'static + std::fmt::Debug + Clone + Send + Sync,
    F: Fn(&T) -> TestResult + Send + Sync + 'static,
{
    /// Create a new load generator.
    pub fn new(generator: Gen<T>, test_function: F, config: LoadTestConfig) -> Self {
        LoadGenerator {
            generator,
            test_function: Arc::new(test_function),
            config,
        }
    }

    /// Execute the load test.
    pub fn run_load_test(&self) -> LoadTestResult {
        let start_time = Instant::now();
        
        // Pre-generate test inputs to avoid generator contention during load test
        let input_count = (self.config.duration.as_secs() as usize + 10) * self.config.thread_count;
        let test_inputs = self.generate_test_inputs(input_count);
        
        let mut thread_handles = Vec::new();
        let mut stats = LoadTestStats::default();
        
        // Phase 1: Ramp up
        let ramp_up_start = Instant::now();
        println!("🚀 Load test ramp-up starting with {} threads...", self.config.thread_count);
        
        // Spawn worker threads
        for thread_id in 0..self.config.thread_count {
            let inputs = test_inputs.clone();
            let test_function = Arc::clone(&self.test_function);
            let config = self.config.clone();
            let thread_start_delay = Duration::from_millis(
                (thread_id as u64 * self.config.ramp_up_duration.as_millis() as u64) / self.config.thread_count as u64
            );
            
            let handle = thread::spawn(move || {
                // Stagger thread starts during ramp-up
                thread::sleep(thread_start_delay);
                
                Self::worker_thread(thread_id, inputs, test_function, config)
            });
            
            thread_handles.push(handle);
        }
        
        let ramp_up_time = ramp_up_start.elapsed();
        
        // Phase 2: Steady state (wait for load test to complete)
        let steady_state_start = Instant::now();
        thread::sleep(self.config.duration);
        let steady_state_time = steady_state_start.elapsed();
        
        // Phase 3: Cool down and collect results
        let cool_down_start = Instant::now();
        println!("🔽 Load test cooling down...");
        
        let mut thread_results = Vec::new();
        let mut all_response_times = Vec::new();
        let mut total_ops = 0;
        let mut failed_ops = 0;
        
        for handle in thread_handles {
            match handle.join() {
                Ok((thread_stats, response_times)) => {
                    total_ops += thread_stats.operations_completed;
                    failed_ops += thread_stats.operations_failed;
                    all_response_times.extend(response_times);
                    
                    thread_results.push(TestResult::Pass {
                        tests_run: thread_stats.operations_completed,
                        property_name: Some("load_test".to_string()),
                        module_path: None,
                    });
                }
                Err(_) => {
                    thread_results.push(TestResult::Fail {
                        counterexample: "Thread panicked during load test".to_string(),
                        tests_run: 0,
                        shrinks_performed: 0,
                        property_name: Some("load_test".to_string()),
                        module_path: None,
                        assertion_type: Some("Thread Panic".to_string()),
                        shrink_steps: Vec::new(),
                    });
                }
            }
        }
        
        let cool_down_time = cool_down_start.elapsed();
        let total_time = start_time.elapsed();
        
        // Calculate statistics
        all_response_times.sort();
        let avg_response_time = if !all_response_times.is_empty() {
            all_response_times.iter().sum::<Duration>() / all_response_times.len() as u32
        } else {
            Duration::from_secs(0)
        };
        
        let p95_response_time = if !all_response_times.is_empty() {
            let index = (all_response_times.len() as f64 * 0.95) as usize;
            all_response_times.get(index).copied().unwrap_or(Duration::from_secs(0))
        } else {
            Duration::from_secs(0)
        };
        
        let p99_response_time = if !all_response_times.is_empty() {
            let index = (all_response_times.len() as f64 * 0.99) as usize;
            all_response_times.get(index).copied().unwrap_or(Duration::from_secs(0))
        } else {
            Duration::from_secs(0)
        };
        
        let max_response_time = all_response_times.last().copied().unwrap_or(Duration::from_secs(0));
        let avg_ops_per_second = total_ops as f64 / steady_state_time.as_secs_f64();
        
        stats.operations_completed = total_ops;
        stats.operations_failed = failed_ops;
        stats.avg_ops_per_second = avg_ops_per_second;
        stats.peak_ops_per_second = avg_ops_per_second; // Simplified for now
        stats.avg_response_time = avg_response_time;
        stats.p95_response_time = p95_response_time;
        stats.p99_response_time = p99_response_time;
        stats.max_response_time = max_response_time;
        stats.response_times = all_response_times;
        stats.thread_utilization = if thread_results.is_empty() { 0.0 } else { 
            thread_results.len() as f64 / self.config.thread_count as f64 
        };
        stats.deadlocks_detected = 0; // Would need more sophisticated detection
        stats.memory_usage_mb = None;
        
        LoadTestResult {
            config: self.config.clone(),
            stats,
            thread_results,
            phase_timings: LoadTestPhases {
                ramp_up_time,
                steady_state_time,
                cool_down_time,
                total_time,
            },
            success_rate: if total_ops > 0 { 
                (total_ops - failed_ops) as f64 / total_ops as f64 
            } else { 
                0.0 
            },
        }
    }

    /// Generate test inputs for load testing.
    fn generate_test_inputs(&self, count: usize) -> Vec<T> {
        let mut inputs = Vec::with_capacity(count);
        let mut seed = crate::data::Seed::random();
        
        for i in 0..count {
            let size = crate::data::Size::new((i % 100) + 1); // Vary size
            let (test_seed, next_seed) = seed.split();
            seed = next_seed;
            
            let tree = self.generator.generate(size, test_seed);
            inputs.push(tree.value);
        }
        
        inputs
    }

    /// Worker thread for load testing.
    fn worker_thread(
        _thread_id: usize,
        inputs: Vec<T>,
        test_function: Arc<F>,
        config: LoadTestConfig,
    ) -> (LoadTestStats, Vec<Duration>) {
        let start_time = Instant::now();
        let mut operations_completed = 0;
        let mut operations_failed = 0;
        let mut response_times = Vec::new();
        let mut input_iter = inputs.iter().cycle();
        
        // Run until duration expires
        while start_time.elapsed() < config.duration {
            if let Some(input) = input_iter.next() {
                let op_start = Instant::now();
                let result = test_function(input);
                let response_time = op_start.elapsed();
                
                if config.collect_stats {
                    response_times.push(response_time);
                }
                
                match result {
                    TestResult::Pass { .. } => operations_completed += 1,
                    TestResult::Fail { .. } => {
                        operations_completed += 1;
                        operations_failed += 1;
                    }
                    _ => operations_completed += 1,
                }
                
                // Rate limiting if specified
                if let Some(target_ops_per_sec) = config.ops_per_second {
                    let target_interval = Duration::from_secs_f64(1.0 / target_ops_per_sec as f64);
                    if response_time < target_interval {
                        thread::sleep(target_interval - response_time);
                    }
                }
            }
        }
        
        let thread_stats = LoadTestStats {
            operations_completed,
            operations_failed,
            avg_ops_per_second: operations_completed as f64 / config.duration.as_secs_f64(),
            peak_ops_per_second: 0.0, // Would need more sophisticated tracking
            avg_response_time: Duration::from_secs(0), // Calculated later
            p95_response_time: Duration::from_secs(0),
            p99_response_time: Duration::from_secs(0),
            max_response_time: Duration::from_secs(0),
            response_times: Vec::new(),
            thread_utilization: 1.0,
            deadlocks_detected: 0,
            memory_usage_mb: None,
        };
        
        (thread_stats, response_times)
    }
}

impl Default for LoadTestStats {
    fn default() -> Self {
        LoadTestStats {
            operations_completed: 0,
            operations_failed: 0,
            avg_ops_per_second: 0.0,
            peak_ops_per_second: 0.0,
            avg_response_time: Duration::from_secs(0),
            p95_response_time: Duration::from_secs(0),
            p99_response_time: Duration::from_secs(0),
            max_response_time: Duration::from_secs(0),
            response_times: Vec::new(),
            thread_utilization: 0.0,
            deadlocks_detected: 0,
            memory_usage_mb: None,
        }
    }
}

/// Create a load generator for stress testing.
pub fn load_generator<T, F>(generator: Gen<T>, test_function: F) -> LoadGenerator<T, F>
where
    T: 'static + std::fmt::Debug + Clone + Send + Sync,
    F: Fn(&T) -> TestResult + Send + Sync + 'static,
{
    LoadGenerator::new(generator, test_function, LoadTestConfig::default())
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

    #[test]
    fn test_concurrent_property_basic() {
        let prop = for_all_concurrent(
            Gen::int_range(1, 10),
            |&n| n > 0 && n <= 10,
            4 // 4 threads
        );
        
        let results = prop.run(&Config::default().with_tests(5));
        
        // Should have 5 concurrent test results (one for each input)
        assert_eq!(results.len(), 5);
        
        // Each result should have been tested by 4 threads
        for result in &results {
            assert_eq!(result.results.len(), 4);
            // For a simple boolean test, all threads should be deterministic
            assert!(result.deterministic);
            assert_eq!(result.race_conditions_detected, 0);
        }
    }

    #[test]
    fn test_concurrent_determinism_detection() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;
        
        // Create a test that has non-deterministic behavior
        let flip_flop = Arc::new(AtomicBool::new(false));
        
        let prop = ConcurrentProperty::new(
            Gen::unit(), // We don't need varied input for this test
            {
                let flip_flop = Arc::clone(&flip_flop);
                move |_| {
                    // This creates non-deterministic behavior
                    let current = flip_flop.load(Ordering::SeqCst);
                    flip_flop.store(!current, Ordering::SeqCst);
                    
                    if current {
                        TestResult::Pass {
                            tests_run: 1,
                            property_name: None,
                            module_path: None,
                        }
                    } else {
                        TestResult::Fail {
                            counterexample: "non-deterministic".to_string(),
                            tests_run: 1,
                            shrinks_performed: 0,
                            property_name: None,
                            module_path: None,
                            assertion_type: Some("Flip Flop".to_string()),
                            shrink_steps: Vec::new(),
                        }
                    }
                }
            },
            4 // 4 threads
        );
        
        let results = prop.run(&Config::default().with_tests(3));
        
        // Should detect non-deterministic behavior
        let non_deterministic_count = results.iter()
            .filter(|r| !r.deterministic)
            .count();
            
        // Should find at least some non-deterministic results
        assert!(non_deterministic_count > 0, "Should detect non-deterministic behavior");
    }

    #[test] 
    fn test_concurrent_property_with_variable_name() {
        let prop = for_all_concurrent(
            Gen::int_range(1, 5),
            |&n| n > 0,
            2 // 2 threads
        ).with_variable_name("test_value");
        
        let results = prop.run(&Config::default().with_tests(2));
        
        assert_eq!(results.len(), 2);
        // Variable name should be preserved in thread results
        for result in &results {
            if !result.results.is_empty() {
                // Check if any results have the variable name (depends on test outcome)
                for thread_result in &result.results {
                    if let TestResult::Fail { property_name: Some(name), .. } = thread_result {
                        assert_eq!(name, "test_value");
                    }
                }
            }
        }
    }

    #[test]
    fn test_concurrent_result_type_analysis() {
        // Test the result type analysis
        let pass_result = TestResult::Pass { 
            tests_run: 1, 
            property_name: None, 
            module_path: None 
        };
        let fail_result = TestResult::Fail {
            counterexample: "test".to_string(),
            tests_run: 1,
            shrinks_performed: 0,
            property_name: None,
            module_path: None,
            assertion_type: None,
            shrink_steps: Vec::new(),
        };
        
        assert_eq!(ConcurrentProperty::<(), fn(&()) -> TestResult>::result_type(&pass_result), "pass");
        assert_eq!(ConcurrentProperty::<(), fn(&()) -> TestResult>::result_type(&fail_result), "fail");
    }

    #[test]
    fn test_deadlock_detection_with_timeout() {
        use std::sync::{Arc, Mutex};
        
        // Create a test that will cause deadlock by having threads wait on each other
        let mutex1 = Arc::new(Mutex::new(0));
        let mutex2 = Arc::new(Mutex::new(0));
        
        let prop = ConcurrentProperty::new(
            Gen::unit(),
            {
                let m1 = Arc::clone(&mutex1);
                let m2 = Arc::clone(&mutex2);
                move |_| {
                    // This creates a potential deadlock scenario
                    let _guard1 = m1.lock().unwrap();
                    thread::sleep(Duration::from_millis(50)); // Hold lock and wait
                    let _guard2 = m2.try_lock(); // Try to get second lock
                    
                    TestResult::Pass {
                        tests_run: 1,
                        property_name: None,
                        module_path: None,
                    }
                }
            },
            3 // 3 threads competing for locks
        ).with_timeout(Duration::from_millis(100)); // Short timeout
        
        let results = prop.run(&Config::default().with_tests(1));
        
        // Should have 1 result
        assert_eq!(results.len(), 1);
        
        let result = &results[0];
        
        // Should detect either timeout or non-deterministic behavior
        assert!(result.timeout_detected || !result.deterministic || result.race_conditions_detected > 0,
                "Should detect concurrency issues (timeout, non-deterministic, or race conditions)");
    }

    #[test]
    fn test_deadlock_info_generation() {
        use std::sync::{Arc, Mutex};
        
        // Create a mutex that will cause long delays
        let slow_mutex = Arc::new(Mutex::new(0));
        
        let prop = ConcurrentProperty::new(
            Gen::constant(42),
            {
                let mutex = Arc::clone(&slow_mutex);
                move |input| {
                    let _guard = mutex.lock().unwrap();
                    // Simulate very slow operation that will timeout
                    thread::sleep(Duration::from_millis(200));
                    
                    TestResult::Pass {
                        tests_run: 1,
                        property_name: Some(format!("slow_test_{}", input)),
                        module_path: None,
                    }
                }
            },
            2 // 2 threads competing for the mutex
        ).with_timeout(Duration::from_millis(50)); // Very short timeout
        
        let results = prop.run(&Config::default().with_tests(1));
        
        assert_eq!(results.len(), 1);
        let result = &results[0];
        
        // Should detect timeout
        assert!(result.timeout_detected, "Should detect timeout");
        
        // Should have deadlock info
        if let Some(deadlock_info) = &result.deadlock_info {
            assert!(deadlock_info.input.contains("42"), "Deadlock info should contain input value");
            assert!(!deadlock_info.threads_involved.is_empty(), "Should have threads involved");
            assert!(deadlock_info.timeout_duration >= Duration::from_millis(50), "Should record timeout duration");
        } else {
            panic!("Should have deadlock info when timeout is detected");
        }
    }

    #[test]
    fn test_concurrent_scenario_builder() {
        // Test building a simple scenario
        let scenario = concurrent_scenario("test_scenario")
            .operation("op1", |n: &i32| {
                if *n > 0 {
                    TestResult::Pass { tests_run: 1, property_name: None, module_path: None }
                } else {
                    TestResult::Fail {
                        counterexample: format!("{}", n),
                        tests_run: 1,
                        shrinks_performed: 0,
                        property_name: None,
                        module_path: None,
                        assertion_type: None,
                        shrink_steps: Vec::new(),
                    }
                }
            })
            .operation("op2", |n: &i32| {
                TestResult::Pass { 
                    tests_run: 1, 
                    property_name: Some(format!("op2_{}", n)),
                    module_path: None 
                }
            })
            .before("op1", "op2")
            .build();

        assert_eq!(scenario.name, "test_scenario");
        assert_eq!(scenario.operations.len(), 2);
        assert_eq!(scenario.constraints.len(), 1);
        
        // Check operation IDs
        assert_eq!(scenario.operations[0].id, "op1");
        assert_eq!(scenario.operations[1].id, "op2");
        
        // Check constraint
        match &scenario.constraints[0] {
            InterleavingConstraint::Before { before, after } => {
                assert_eq!(before, "op1");
                assert_eq!(after, "op2");
            }
            _ => panic!("Expected Before constraint"),
        }
    }

    #[test]
    fn test_scenario_execution() {
        let scenario = concurrent_scenario("simple_test")
            .operation("increment", |_n: &i32| {
                TestResult::Pass { 
                    tests_run: 1, 
                    property_name: Some("increment".to_string()),
                    module_path: None 
                }
            })
            .operation("double", |n: &i32| {
                let doubled = n * 2;
                if doubled > *n {
                    TestResult::Pass { 
                        tests_run: 1, 
                        property_name: Some("double".to_string()),
                        module_path: None 
                    }
                } else {
                    TestResult::Fail {
                        counterexample: format!("Doubling {} failed", n),
                        tests_run: 1,
                        shrinks_performed: 0,
                        property_name: Some("double".to_string()),
                        module_path: None,
                        assertion_type: None,
                        shrink_steps: Vec::new(),
                    }
                }
            })
            .build();

        let result = scenario.execute(&5);
        
        assert_eq!(result.scenario_name, "simple_test");
        assert_eq!(result.operation_results.len(), 2);
        assert!(result.operation_results.contains_key("increment"));
        assert!(result.operation_results.contains_key("double"));
        
        // Both operations should pass
        match result.operation_results.get("increment").unwrap() {
            TestResult::Pass { .. } => {},
            _ => panic!("increment should pass"),
        }
        
        match result.operation_results.get("double").unwrap() {
            TestResult::Pass { .. } => {},
            _ => panic!("double should pass"),
        }
    }

    #[test]
    fn test_scenario_with_dependencies() {
        let scenario = concurrent_scenario("dependency_test")
            .operation("setup", |_: &i32| {
                TestResult::Pass { 
                    tests_run: 1, 
                    property_name: Some("setup".to_string()),
                    module_path: None 
                }
            })
            .operation_depends_on("main", vec!["setup"], |n: &i32| {
                TestResult::Pass { 
                    tests_run: 1, 
                    property_name: Some(format!("main_{}", n)),
                    module_path: None 
                }
            })
            .operation_depends_on("cleanup", vec!["main"], |_: &i32| {
                TestResult::Pass { 
                    tests_run: 1, 
                    property_name: Some("cleanup".to_string()),
                    module_path: None 
                }
            })
            .build();

        assert_eq!(scenario.operations.len(), 3);
        
        // Check dependencies
        let main_op = scenario.operations.iter().find(|op| op.id == "main").unwrap();
        assert_eq!(main_op.depends_on, vec!["setup"]);
        
        let cleanup_op = scenario.operations.iter().find(|op| op.id == "cleanup").unwrap();
        assert_eq!(cleanup_op.depends_on, vec!["main"]);
        
        let result = scenario.execute(&10);
        assert_eq!(result.operation_results.len(), 3);
    }

    #[test]
    fn test_interleaving_explorer_creation() {
        let explorer = interleaving_explorer(
            Gen::int_range(1, 100),
            |&n| {
                if n > 50 {
                    TestResult::Pass { tests_run: 1, property_name: None, module_path: None }
                } else {
                    TestResult::Fail {
                        counterexample: format!("{}", n),
                        tests_run: 1,
                        shrinks_performed: 0,
                        property_name: None,
                        module_path: None,
                        assertion_type: None,
                        shrink_steps: Vec::new(),
                    }
                }
            }
        );

        assert_eq!(explorer.operation_count, 3);
        assert_eq!(explorer.max_interleavings, 50);
        assert!(explorer.timeout.is_some());
    }

    #[test]
    fn test_interleaving_explorer_configuration() {
        let explorer = interleaving_explorer(
            Gen::bool(),
            |&b| if b { 
                TestResult::Pass { tests_run: 1, property_name: None, module_path: None }
            } else { 
                TestResult::Fail {
                    counterexample: "false".to_string(),
                    tests_run: 1,
                    shrinks_performed: 0,
                    property_name: None,
                    module_path: None,
                    assertion_type: None,
                    shrink_steps: Vec::new(),
                }
            }
        )
        .with_operations(5)
        .with_max_interleavings(20)
        .with_timeout(Duration::from_millis(500));

        assert_eq!(explorer.operation_count, 5);
        assert_eq!(explorer.max_interleavings, 20);
        assert_eq!(explorer.timeout, Some(Duration::from_millis(500)));
    }

    #[test]  
    fn test_interleaving_explorer_basic_run() {
        let explorer = interleaving_explorer(
            Gen::constant(42),
            |&n| {
                // Simple deterministic test
                if n == 42 {
                    TestResult::Pass { tests_run: 1, property_name: None, module_path: None }
                } else {
                    TestResult::Fail {
                        counterexample: format!("Expected 42, got {}", n),
                        tests_run: 1,
                        shrinks_performed: 0,
                        property_name: None,
                        module_path: None,
                        assertion_type: None,
                        shrink_steps: Vec::new(),
                    }
                }
            }
        ).with_max_interleavings(10);

        let results = explorer.explore(&Config::default().with_tests(2));
        
        assert_eq!(results.len(), 2);
        
        for result in &results {
            assert_eq!(result.interleavings_explored, 10);
            // For a deterministic test, should not detect race conditions
            assert!(result.deterministic, "Deterministic test should not have race conditions");
            assert_eq!(result.race_conditions_detected, 0);
        }
    }

    #[test]
    fn test_interleaving_pattern_structure() {
        use std::sync::atomic::{AtomicBool, Ordering};
        
        // Test with intentionally non-deterministic behavior
        let flip = Arc::new(AtomicBool::new(false));
        
        let explorer = interleaving_explorer(
            Gen::unit(),
            {
                let flip = Arc::clone(&flip);
                move |_| {
                    let current = flip.load(Ordering::SeqCst);
                    flip.store(!current, Ordering::SeqCst);
                    
                    if current {
                        TestResult::Pass { tests_run: 1, property_name: None, module_path: None }
                    } else {
                        TestResult::Fail {
                            counterexample: "flipped to false".to_string(),
                            tests_run: 1,
                            shrinks_performed: 0,
                            property_name: None,
                            module_path: None,
                            assertion_type: Some("Non-deterministic".to_string()),
                            shrink_steps: Vec::new(),
                        }
                    }
                }
            }
        ).with_max_interleavings(20);

        let results = explorer.explore(&Config::default().with_tests(1));
        
        assert_eq!(results.len(), 1);
        let result = &results[0];
        
        // Should potentially detect non-determinism
        if !result.deterministic {
            assert!(result.race_conditions_detected > 0);
            assert!(!result.failing_patterns.is_empty());
            
            // Check pattern structure
            let pattern = &result.failing_patterns[0];
            assert!(!pattern.sequence.is_empty());
            assert!(!pattern.threads_involved.is_empty());
            
            // Should have thread operations
            for op in &pattern.sequence {
                assert!(op.thread_id < explorer.operation_count);
                assert!(!op.operation.is_empty());
            }
        }
    }

    #[test]
    fn test_load_test_config_defaults() {
        let config = LoadTestConfig::default();
        
        assert!(config.thread_count > 0);
        assert!(config.duration > Duration::from_secs(0));
        assert!(config.ramp_up_duration >= Duration::from_secs(0));
        assert!(config.cool_down_duration >= Duration::from_secs(0));
        assert!(config.collect_stats);
    }

    #[test]
    fn test_load_generator_creation() {
        let config = LoadTestConfig {
            thread_count: 2,
            duration: Duration::from_millis(100),
            ops_per_second: Some(10),
            ramp_up_duration: Duration::from_millis(10),
            cool_down_duration: Duration::from_millis(10),
            collect_stats: true,
        };
        
        let generator = LoadGenerator::new(
            Gen::int_range(1, 100),
            |&n| {
                if n > 0 {
                    TestResult::Pass { tests_run: 1, property_name: None, module_path: None }
                } else {
                    TestResult::Fail {
                        counterexample: format!("{}", n),
                        tests_run: 1,
                        shrinks_performed: 0,
                        property_name: None,
                        module_path: None,
                        assertion_type: None,
                        shrink_steps: Vec::new(),
                    }
                }
            },
            config.clone(),
        );
        
        assert_eq!(generator.config.thread_count, 2);
        assert_eq!(generator.config.duration, Duration::from_millis(100));
        assert_eq!(generator.config.ops_per_second, Some(10));
    }

    #[test]
    fn test_load_generator_basic_run() {
        let config = LoadTestConfig {
            thread_count: 2,
            duration: Duration::from_millis(100), // Very short test
            ops_per_second: None,
            ramp_up_duration: Duration::from_millis(5),
            cool_down_duration: Duration::from_millis(5),
            collect_stats: true,
        };
        
        let generator = LoadGenerator::new(
            Gen::constant(42),
            |&n| {
                // Simple test that should always pass
                TestResult::Pass { 
                    tests_run: 1, 
                    property_name: Some(format!("test_{}", n)),
                    module_path: None 
                }
            },
            config,
        );
        
        let result = generator.run_load_test();
        
        // Check basic result structure
        assert_eq!(result.config.thread_count, 2);
        assert!(result.stats.operations_completed > 0, "Should complete some operations");
        assert_eq!(result.stats.operations_failed, 0, "All operations should pass");
        assert!(result.success_rate >= 0.95, "Success rate should be high");
        assert!(result.stats.avg_ops_per_second > 0.0, "Should have positive ops/sec");
        
        // Check phase timings
        assert!(result.phase_timings.total_time >= Duration::from_millis(100));
        assert!(result.phase_timings.ramp_up_time >= Duration::from_millis(0));
        assert!(result.phase_timings.steady_state_time >= Duration::from_millis(90));
        
        // Check thread results
        assert_eq!(result.thread_results.len(), 2, "Should have results from 2 threads");
        
        for thread_result in &result.thread_results {
            match thread_result {
                TestResult::Pass { tests_run, .. } => {
                    assert!(*tests_run > 0, "Each thread should complete some tests");
                }
                _ => panic!("All thread results should be Pass"),
            }
        }
    }

    #[test]
    fn test_load_generator_with_failures() {
        let config = LoadTestConfig {
            thread_count: 1,
            duration: Duration::from_millis(50),
            ops_per_second: None,
            ramp_up_duration: Duration::from_millis(2),
            cool_down_duration: Duration::from_millis(2),
            collect_stats: true,
        };
        
        let generator = LoadGenerator::new(
            Gen::int_range(1, 10),
            |&n| {
                // Test that fails for even numbers
                if n % 2 == 0 {
                    TestResult::Fail {
                        counterexample: format!("Even number: {}", n),
                        tests_run: 1,
                        shrinks_performed: 0,
                        property_name: Some("even_test".to_string()),
                        module_path: None,
                        assertion_type: Some("Even Number".to_string()),
                        shrink_steps: Vec::new(),
                    }
                } else {
                    TestResult::Pass { 
                        tests_run: 1, 
                        property_name: Some("odd_test".to_string()),
                        module_path: None 
                    }
                }
            },
            config,
        );
        
        let result = generator.run_load_test();
        
        // Should have both passing and failing operations
        assert!(result.stats.operations_completed > 0);
        // Success rate should be less than 100% due to even number failures
        assert!(result.success_rate < 1.0, "Should have some failures");
        assert!(result.success_rate > 0.0, "Should have some successes");
    }

    #[test]
    fn test_load_test_response_time_stats() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        
        let counter = Arc::new(AtomicUsize::new(0));
        
        let config = LoadTestConfig {
            thread_count: 1,
            duration: Duration::from_millis(50),
            ops_per_second: None,
            ramp_up_duration: Duration::from_millis(2),
            cool_down_duration: Duration::from_millis(2),
            collect_stats: true,
        };
        
        let generator = LoadGenerator::new(
            Gen::unit(),
            {
                let counter = Arc::clone(&counter);
                move |_| {
                    counter.fetch_add(1, Ordering::SeqCst);
                    // Add a small delay to create measurable response times
                    thread::sleep(Duration::from_micros(10));
                    TestResult::Pass { tests_run: 1, property_name: None, module_path: None }
                }
            },
            config,
        );
        
        let result = generator.run_load_test();
        
        // Check response time statistics
        assert!(result.stats.avg_response_time > Duration::from_micros(5), 
                "Average response time should reflect the sleep");
        assert!(result.stats.max_response_time >= result.stats.avg_response_time,
                "Max response time should be >= average");
        assert!(result.stats.p95_response_time >= Duration::from_micros(5),
                "P95 should reflect the sleep time");
        
        // Check that we collected response times
        assert!(!result.stats.response_times.is_empty(), "Should collect response times");
        
        // Verify counter was incremented
        let final_count = counter.load(Ordering::SeqCst);
        assert_eq!(final_count, result.stats.operations_completed, 
                  "Counter should match operations completed");
    }
}