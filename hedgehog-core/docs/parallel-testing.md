# Parallel Testing Guide

Hedgehog provides comprehensive parallel testing capabilities that serve two main purposes:

1. **Performance Testing** - Speed up property tests by distributing work across multiple threads
2. **Concurrency Testing** - Detect race conditions and verify thread safety

## Quick Start

### Basic Parallel Property Testing

The simplest way to parallelize a property test is using `for_all_parallel`:

```rust
use hedgehog_core::*;

// Test integer addition commutativity across 4 threads
let prop = for_all_parallel(
    Gen::tuple_of(Gen::int_range(1, 100), Gen::int_range(1, 100)),
    |(a, b)| a + b == b + a,
    4 // Number of threads
);

let config = Config::default().with_tests(1000);
let result = prop.run(&config);

match result.outcome {
    TestResult::Pass { tests_run, .. } => {
        println!("✓ {} tests passed", tests_run);
        println!("Speedup: {:.2}x", result.performance.speedup_factor);
        println!("Thread efficiency: {:.1}%", result.performance.thread_efficiency * 100.0);
    }
    TestResult::Fail { counterexample, .. } => {
        println!("✗ Failed: {}", counterexample);
    }
    _ => {}
}
```

### Advanced Configuration

For more control over parallel execution, use `parallel_property` with custom configuration:

```rust
use hedgehog_core::*;
use std::time::Duration;

let config = ParallelConfig {
    thread_count: 8,
    work_distribution: WorkDistribution::RoundRobin,
    timeout: Some(Duration::from_secs(30)),
    detect_non_determinism: true,
};

let prop = parallel_property(
    Gen::int_range(1, 1000),
    |&n| expensive_computation(n) > 0,
    config
);

let result = prop.run(&Config::default().with_tests(10000));
```

## Work Distribution Strategies

### Round Robin
Distributes tests evenly across threads in round-robin fashion:
- Thread 1: tests 0, 3, 6, 9...
- Thread 2: tests 1, 4, 7, 10...
- Thread 3: tests 2, 5, 8, 11...

```rust
let config = ParallelConfig {
    work_distribution: WorkDistribution::RoundRobin,
    ..ParallelConfig::default()
};
```

### Chunk-Based
Divides tests into contiguous chunks per thread:
- Thread 1: tests 0-33
- Thread 2: tests 34-66  
- Thread 3: tests 67-99

```rust
let config = ParallelConfig {
    work_distribution: WorkDistribution::ChunkBased,
    ..ParallelConfig::default()
};
```

### Work Stealing
Advanced load balancing where threads can steal work from each other when idle. Currently falls back to Round Robin (full implementation coming soon).

```rust
let config = ParallelConfig {
    work_distribution: WorkDistribution::WorkStealing,
    ..ParallelConfig::default()
};
```

## Performance Analysis

Parallel test results include detailed performance metrics:

```rust
let result = prop.run(&config);

println!("Performance Metrics:");
println!("  Total duration: {:?}", result.performance.total_duration);
println!("  CPU time: {:?}", result.performance.total_cpu_time);  
println!("  Speedup: {:.2}x", result.performance.speedup_factor);
println!("  Thread efficiency: {:.1}%", result.performance.thread_efficiency * 100.0);
```

### Understanding Metrics

- **Speedup Factor**: How much faster parallel execution is compared to estimated sequential time
- **Thread Efficiency**: How well threads are utilized (100% = perfect scaling)
- **Total CPU Time**: Estimated total computation time across all threads
- **Total Duration**: Actual wall-clock time for parallel execution

## Concurrency Testing

### Testing Shared State

Parallel testing can help detect race conditions in shared state:

```rust
use std::sync::{Arc, Mutex};

struct Counter {
    value: Arc<Mutex<i32>>,
}

impl Counter {
    fn new() -> Self {
        Counter { value: Arc::new(Mutex::new(0)) }
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

// Test concurrent increments
let counter = Arc::new(Counter::new());

let prop = parallel_property(
    Gen::unit(),
    {
        let counter = Arc::clone(&counter);
        move |_| {
            let result = counter.increment();
            if result > 0 {
                TestResult::Pass {
                    tests_run: 1,
                    property_name: Some("increment".to_string()),
                    module_path: None,
                }
            } else {
                TestResult::Fail {
                    counterexample: format!("Got {}", result),
                    tests_run: 1,
                    shrinks_performed: 0,
                    property_name: Some("increment".to_string()),
                    module_path: None,
                    assertion_type: Some("Positive Result".to_string()),
                    shrink_steps: Vec::new(),
                }
            }
        }
    },
    ParallelConfig {
        thread_count: 8,
        detect_non_determinism: true,
        ..ParallelConfig::default()
    }
);

let config = Config::default().with_tests(100);
let result = prop.run(&config);

// Check for race conditions
if counter.get() != config.test_limit as i32 {
    println!("⚠ Race condition detected!");
}
```

### Concurrency Issue Detection

The parallel testing framework automatically detects various concurrency issues:

```rust
let result = prop.run(&config);

if result.concurrency_issues.non_deterministic_results > 0 {
    println!("⚠ {} non-deterministic results detected", 
             result.concurrency_issues.non_deterministic_results);
}

if result.concurrency_issues.timeouts > 0 {
    println!("⚠ {} timeouts detected", result.concurrency_issues.timeouts);
}

if !result.concurrency_issues.thread_failures.is_empty() {
    println!("⚠ Thread failures: {:?}", result.concurrency_issues.thread_failures);
}
```

## Best Practices

### When to Use Parallel Testing

**Good candidates for parallel testing:**
- CPU-intensive property tests
- Tests with expensive setup/computation
- Large test suites (>1000 tests)
- Concurrency verification

**Avoid parallel testing for:**
- Simple, fast tests (overhead may hurt performance)
- Tests with global state dependencies
- I/O-bound tests (use fewer threads)

### Choosing Thread Count

```rust
// Automatic detection (recommended)
let config = ParallelConfig::default(); // Uses available CPU cores

// Manual configuration
let config = ParallelConfig {
    thread_count: 4, // For CPU-bound tasks
    ..ParallelConfig::default()
};

let config = ParallelConfig {
    thread_count: 2, // For I/O-bound or memory-intensive tasks
    ..ParallelConfig::default()
};
```

### Performance Optimization Tips

1. **Start with default settings** - The framework auto-detects optimal thread count
2. **Use Round Robin for mixed workloads** - More balanced than chunked distribution  
3. **Set appropriate timeouts** - Prevent deadlocks in concurrent tests
4. **Monitor thread efficiency** - Values below 50% suggest too many threads

### Testing for Race Conditions

```rust
// Test pattern for race condition detection
fn test_concurrent_operation() {
    let shared_resource = Arc::new(SharedResource::new());
    
    let prop = parallel_property(
        Gen::operation_sequence(), // Generate operations
        {
            let resource = Arc::clone(&shared_resource);
            move |ops| {
                // Apply operations concurrently
                for op in ops {
                    resource.apply(op);
                }
                
                // Verify invariants
                resource.verify_consistency()
            }
        },
        ParallelConfig {
            thread_count: 8,
            detect_non_determinism: true,
            ..ParallelConfig::default()
        }
    );
    
    let result = prop.run(&Config::default().with_tests(1000));
    
    // Check for race conditions
    assert_eq!(result.concurrency_issues.non_deterministic_results, 0);
}
```

## Debugging Parallel Tests

### Reproducing Failures

When parallel tests fail, they may be harder to reproduce due to non-deterministic thread scheduling. Use deterministic testing:

```rust
// For reproducible tests, use fixed seeds in sequential mode first
let sequential_prop = for_all(gen, condition);
let result = sequential_prop.run(&Config::default().with_seed(Seed::from_u64(12345)));

// Then test the same property in parallel
let parallel_prop = for_all_parallel(gen, condition, 4);
let parallel_result = parallel_prop.run(&Config::default().with_seed(Seed::from_u64(12345)));
```

### Common Issues

**Thread panics:**
```rust
if !result.concurrency_issues.thread_failures.is_empty() {
    println!("Thread failures: {:?}", result.concurrency_issues.thread_failures);
    // Usually indicates: 
    // - Panic in test function
    // - Resource contention
    // - Deadlock
}
```

**Poor performance:**
```rust
if result.performance.thread_efficiency < 0.5 {
    println!("Low thread efficiency: {:.1}%", result.performance.thread_efficiency * 100.0);
    // Consider:
    // - Reducing thread count
    // - Checking for lock contention
    // - Using different work distribution
}
```

## Integration with Property Features

Parallel testing works with all standard Hedgehog property features:

### With Classifications

```rust
let prop = for_all_parallel(Gen::int_range(-100, 100), |&n| n.abs() >= 0, 4)
    .classify("negative", |&n| n < 0)
    .classify("positive", |&n| n > 0)
    .classify("zero", |&n| n == 0);

// Classifications are aggregated across all threads
```

### With Examples

```rust
let critical_values = vec![-1, 0, 1, i32::MAX, i32::MIN];

let prop = for_all_parallel(Gen::int_range(-1000, 1000), |&n| check_invariant(n), 4)
    .with_examples(critical_values); // Examples tested first across threads
```

### With Variable Names

```rust
let prop = for_all_parallel_named(
    Gen::int_range(1, 100),
    "input_value",
    |&n| n > 0,
    4
);

// Variable names preserved in failure reporting
```

## Advanced Topics

### Custom Test Functions

```rust
let prop = parallel_property(
    Gen::complex_data(),
    |data| {
        // Custom TestResult logic
        match validate_data(data) {
            Ok(()) => TestResult::Pass {
                tests_run: 1,
                property_name: Some("data_validation".to_string()),
                module_path: Some(module_path!().to_string()),
            },
            Err(e) => TestResult::Fail {
                counterexample: format!("{:?}: {}", data, e),
                tests_run: 1,
                shrinks_performed: 0,
                property_name: Some("data_validation".to_string()),
                module_path: Some(module_path!().to_string()),
                assertion_type: Some("Validation Error".to_string()),
                shrink_steps: Vec::new(),
            },
        }
    },
    ParallelConfig::default()
);
```

### Performance Benchmarking

```rust
fn benchmark_parallel_vs_sequential() {
    let gen = Gen::expensive_data();
    let test_fn = |data| expensive_validation(data);
    
    // Sequential baseline
    let sequential = for_all(gen.clone(), test_fn);
    let start = Instant::now();
    let sequential_result = sequential.run(&Config::default().with_tests(1000));
    let sequential_time = start.elapsed();
    
    // Parallel comparison  
    let parallel = for_all_parallel(gen, test_fn, 4);
    let start = Instant::now();
    let parallel_result = parallel.run(&Config::default().with_tests(1000));
    let parallel_time = start.elapsed();
    
    println!("Sequential: {:?}", sequential_time);
    println!("Parallel:   {:?}", parallel_time);
    println!("Speedup:    {:.2}x", sequential_time.as_secs_f64() / parallel_time.as_secs_f64());
}
```

## Future Features

The following features are planned for future releases:

- **Systematic Interleaving Exploration** - Test all possible thread interleavings
- **Concurrent Scenario DSL** - Define complex multi-thread test scenarios  
- **Advanced Deadlock Detection** - More sophisticated timeout and monitoring
- **Stress Testing Patterns** - Built-in patterns for load testing
- **Lock-Free Data Structure Testing** - Specialized tools for lock-free algorithms

## API Reference

### Functions

- `for_all_parallel<T, F>(gen: Gen<T>, condition: F, thread_count: usize) -> ParallelProperty<T, ...>`
- `parallel_property<T, F>(gen: Gen<T>, test_fn: F, config: ParallelConfig) -> ParallelProperty<T, F>`

### Types

- `ParallelProperty<T, F>` - A property that can be executed in parallel
- `ParallelConfig` - Configuration for parallel execution
- `ParallelTestResult` - Results from parallel test execution
- `WorkDistribution` - Strategy for distributing work across threads
- `ParallelPerformanceMetrics` - Performance metrics from parallel execution
- `ConcurrencyIssues` - Issues detected during concurrent testing

### Configuration

```rust
pub struct ParallelConfig {
    pub thread_count: usize,
    pub work_distribution: WorkDistribution,
    pub timeout: Option<Duration>,
    pub detect_non_determinism: bool,
}

pub enum WorkDistribution {
    RoundRobin,
    ChunkBased,
    WorkStealing,
}
```