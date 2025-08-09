# Advanced Hedgehog Features Guide

This guide covers the sophisticated features of Hedgehog that go beyond basic property testing, based on comprehensive testing and validation of these features.

## Table of Contents
- [Targeted Property Testing](#targeted-property-testing)
- [Parallel and Concurrent Testing](#parallel-and-concurrent-testing)
- [State Machine Testing](#state-machine-testing)
- [Advanced String Generation](#advanced-string-generation)
- [Result and Option Generators](#result-and-option-generators)
- [Custom Generator Composition](#custom-generator-composition)

## Targeted Property Testing

Targeted property testing uses search strategies like simulated annealing to guide input generation toward inputs more likely to find bugs.

### Basic Usage

```rust
use hedgehog::*;
use hedgehog::targeted::*;

// Define a utility function that guides the search
let utility_function = |input: &i32, _result: &TargetedResult| -> f64 {
    // Higher utility for values closer to 100 (our "interesting" region)
    let distance = (input - 100).abs() as f64;
    100.0 - distance
};

// Create a test function that returns TargetedResult
let test_function = |input: &i32| -> TargetedResult {
    if *input > 90 && *input < 110 {
        // This range is more likely to reveal bugs
        TargetedResult::Fail {
            counterexample: format!("Found edge case: {}", input),
            tests_run: 1,
            shrinks_performed: 0,
            property_name: Some("edge_case_test".to_string()),
            module_path: None,
            assertion_type: Some("Range Check".to_string()),
            shrink_steps: Vec::new(),
            utility: 0.0,
        }
    } else {
        TargetedResult::Pass {
            tests_run: 1,
            property_name: Some("edge_case_test".to_string()),
            module_path: None,
            utility: 0.0,
        }
    }
};

// Configure the search
let config = TargetedConfig {
    search_steps: 100,
    initial_temperature: 50.0,
    cooling_rate: 0.95,
    objective: SearchObjective::Maximize,
    ..Default::default()
};

// Run targeted testing
let search = for_all_targeted_with_config(
    Gen::int_range(1, 200),
    utility_function,
    test_function,
    IntegerNeighborhood::new(10),
    config,
);

let (result, stats) = search.search(&Config::default().with_tests(1));
println!("Search found result with {} evaluations", stats.evaluations);
```

### Custom Neighborhoods

Create custom neighborhood functions for complex types:

```rust
use hedgehog::targeted::NeighborhoodFunction;

struct CustomNeighborhood;

impl NeighborhoodFunction<MyComplexType> for CustomNeighborhood {
    fn neighbor(&self, input: &MyComplexType, temperature: f64, rng: &mut dyn RngCore) -> Option<MyComplexType> {
        // Generate a similar instance based on temperature
        // Higher temperature = more dramatic changes
        let change_factor = temperature / 100.0;
        // ... your custom logic here
        Some(modified_input)
    }
    
    fn max_distance(&self) -> f64 {
        10.0 // Maximum change this function can make
    }
}
```

## Parallel and Concurrent Testing

Hedgehog supports parallel execution and concurrent testing scenarios.

### Basic Parallel Properties

```rust
use hedgehog::parallel::*;

let parallel_config = ParallelConfig {
    thread_count: 4,
    work_distribution: WorkDistribution::WorkStealing,
    ..ParallelConfig::default()
};

let parallel_prop = parallel_property(
    Gen::int_range(1, 1000),
    |&n| {
        // Your property logic here
        TestResult::Pass {
            tests_run: 1,
            property_name: Some(format!("parallel_test_{}", n)),
            module_path: None,
        }
    },
    parallel_config,
);

let result = parallel_prop.run(&Config::default().with_tests(100));
println!("Parallel execution achieved {:.2}x speedup", result.performance.speedup_factor);
```

### Concurrent System Testing

Test non-deterministic behavior and race conditions:

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

let shared_counter = Arc::new(AtomicUsize::new(0));

let concurrent_prop = ConcurrentProperty::new(
    Gen::unit(),
    {
        let counter = Arc::clone(&shared_counter);
        move |_| {
            let count = counter.fetch_add(1, Ordering::SeqCst);
            
            // Test for race conditions
            if count % 2 == 0 {
                TestResult::Pass { tests_run: 1, property_name: None, module_path: None }
            } else {
                TestResult::Fail {
                    counterexample: format!("Race condition at count: {}", count),
                    tests_run: 1,
                    shrinks_performed: 0,
                    property_name: None,
                    module_path: None,
                    assertion_type: Some("Race Condition".to_string()),
                    shrink_steps: Vec::new(),
                }
            }
        }
    },
    4, // Thread count
).with_timeout(Duration::from_millis(1000));

let results = concurrent_prop.run(&Config::default().with_tests(5));

// Analyze results for non-deterministic behavior
for result in &results {
    if !result.deterministic {
        println!("Found non-deterministic behavior!");
    }
}
```

### Load Testing

```rust
let load_config = LoadTestConfig {
    thread_count: 8,
    duration: Duration::from_secs(30),
    ops_per_second: Some(1000),
    ramp_up_duration: Duration::from_secs(5),
    cool_down_duration: Duration::from_secs(5),
    collect_stats: true,
};

let load_generator = LoadGenerator::new(
    Gen::int_range(1, 100),
    |&n| {
        // Simulate work
        std::thread::sleep(Duration::from_micros(100));
        TestResult::Pass { tests_run: 1, property_name: None, module_path: None }
    },
    load_config,
);

let load_result = load_generator.run_load_test();
println!("Sustained {:.2} ops/second with {:.1}% success rate", 
         load_result.stats.avg_ops_per_second, 
         load_result.success_rate * 100.0);
```

## State Machine Testing

Test stateful systems by generating sequences of commands.

### Basic State Machine

```rust
use hedgehog::state::*;

// 1. Define your system state
#[derive(Debug, Clone, PartialEq)]
struct BankAccount {
    balance: i32,
    is_open: bool,
}

impl BankAccount {
    fn new() -> Self {
        Self { balance: 0, is_open: true }
    }
}

// 2. Define command inputs
#[derive(Clone, Debug)]
struct DepositInput { amount: i32 }

impl std::fmt::Display for DepositInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "deposit ${}", self.amount)
    }
}

// 3. Create an action generator
let mut generator = ActionGenerator::new();

// 4. Add commands with preconditions, state updates, and postconditions
let deposit_cmd: Command<DepositInput, i32, BankAccount, i32> = Command::new(
    "deposit".to_string(),
    |state: &BankAccount| {
        if state.is_open {
            Some(Gen::int_range(1, 100).map(|amount| DepositInput { amount }))
        } else {
            None
        }
    },
    |input: DepositInput| input.amount, // The actual operation
)
.with_require(|state: &BankAccount, input: &DepositInput| {
    state.is_open && input.amount > 0
})
.with_update(|state: &mut BankAccount, input: &DepositInput, _output: &Var<i32>| {
    state.balance += input.amount;
})
.with_ensure(|old_state: &BankAccount, new_state: &BankAccount, input: &DepositInput, output: &i32| {
    if new_state.balance != old_state.balance + input.amount {
        Err("Balance not updated correctly".to_string())
    } else if *output != input.amount {
        Err("Wrong return value".to_string())
    } else {
        Ok(())
    }
});

generator.add_command(deposit_cmd);

// 5. Generate and run test sequences
let initial_state = BankAccount::new();
let sequence = generator.generate_sequential(initial_state.clone(), 10);

println!("Generated sequence:");
for action in &sequence.actions {
    println!("  {}", action.display_action());
}

let result = execute_sequential(initial_state, sequence);
match result {
    Ok(()) => println!("✓ All operations succeeded"),
    Err(e) => println!("✗ Operation failed: {}", e),
}
```

### Complex State Machines

For more complex systems, add multiple commands with dependencies:

```rust
// Withdrawal command that depends on balance
let withdraw_cmd: Command<WithdrawInput, std::result::Result<i32, String>, BankAccount, std::result::Result<i32, String>> = Command::new(
    "withdraw".to_string(),
    |state: &BankAccount| {
        if state.is_open && state.balance > 0 {
            let max_withdraw = std::cmp::min(state.balance, 50);
            Some(Gen::int_range(1, max_withdraw).map(|amount| WithdrawInput { amount }))
        } else {
            None
        }
    },
    |input: WithdrawInput| {
        if input.amount > 0 {
            Ok(input.amount)
        } else {
            Err("Invalid amount".to_string())
        }
    },
)
.with_require(|state: &BankAccount, input: &WithdrawInput| {
    state.is_open && state.balance >= input.amount
})
.with_update(|state: &mut BankAccount, input: &WithdrawInput, _output| {
    state.balance -= input.amount;
})
.with_ensure(|old_state, new_state, input, output| {
    match output {
        Ok(amount) if *amount == input.amount && new_state.balance == old_state.balance - input.amount => Ok(()),
        _ => Err("Withdrawal failed".to_string()),
    }
});
```

## Advanced String Generation

Hedgehog provides sophisticated string generation capabilities.

### Specialized String Generators

```rust
// Web domains
let domain_gen = Gen::<String>::web_domain();
let domain = domain_gen.generate(Size::new(10), Seed::random()).value;
// Generates: "api.example.com", "db.mysite.org", etc.

// Email addresses  
let email_gen = Gen::<String>::email_address();
let email = email_gen.generate(Size::new(10), Seed::random()).value;
// Generates: "user@domain.com", "admin@site.org", etc.

// SQL identifiers
let safe_sql_gen = Gen::<String>::sql_identifier(false); // No keywords
let risky_sql_gen = Gen::<String>::sql_identifier(true);  // May include keywords

// Programming tokens
let rust_tokens = Gen::<String>::programming_tokens(&[
    "fn", "let", "mut", "pub", "struct", "enum", "impl", "trait"
]);
```

### Custom Character Sets

```rust
// Create a hex digit generator
let hex_chars = Gen::one_of(vec![
    Gen::constant('0'), Gen::constant('1'), Gen::constant('2'), Gen::constant('3'),
    Gen::constant('4'), Gen::constant('5'), Gen::constant('6'), Gen::constant('7'),
    Gen::constant('8'), Gen::constant('9'), Gen::constant('A'), Gen::constant('B'),
    Gen::constant('C'), Gen::constant('D'), Gen::constant('E'), Gen::constant('F'),
]).unwrap();

let hex_string_gen = Gen::<String>::string_of(hex_chars);
```

### Length-Controlled Strings

```rust
// Fixed length range
let fixed_range_gen = Gen::<String>::alpha_with_range(Range::new(5, 15));

// Linear distribution (favors shorter strings)
let linear_gen = Gen::<String>::alphanumeric_with_range(Range::linear(1, 20));
```

## Result and Option Generators

Generate and test error handling patterns effectively.

### Basic Option/Result Generation

```rust
// Option generators
let maybe_int = Gen::<Option<i32>>::option_of(Gen::int_range(1, 100));
// Generates Some(value) ~75% of the time, None ~25%

// Result generators
let result_gen = Gen::<std::result::Result<i32, String>>::result_of(
    Gen::int_range(1, 100),    // Success values
    Gen::<String>::ascii_alpha(), // Error values
);
// Generates Ok(value) ~75% of the time, Err(error) ~25%

// Weighted results
let heavily_ok_gen = Gen::<std::result::Result<bool, i32>>::result_of_weighted(
    Gen::bool(),
    Gen::int_range(1, 10),
    9, // 90% Ok, 10% Err
);
```

### Complex Nested Types

```rust
// Option<Result<T, E>>
let option_result_gen = Gen::<Option<std::result::Result<Vec<i32>, String>>>::option_of(
    Gen::<std::result::Result<Vec<i32>, String>>::result_of(
        Gen::<Vec<i32>>::vec_of(Gen::int_range(1, 100)),
        Gen::<String>::ascii_alpha(),
    )
);

// Result<Option<T>, E>
let result_option_gen = Gen::<std::result::Result<Option<String>, i32>>::result_of(
    Gen::<Option<String>>::option_of(Gen::<String>::ascii_alpha()),
    Gen::int_range(1, 10),
);
```

### Testing Error Paths

```rust
let prop = for_all(result_gen, |result: &std::result::Result<i32, String>| {
    match result {
        Ok(value) => {
            // Test success path
            *value >= 1 && *value <= 100
        }
        Err(error) => {
            // Test error path
            error.chars().all(|c| c.is_ascii_alphabetic()) && !error.is_empty()
        }
    }
});
```

## Custom Generator Composition

Create sophisticated generator patterns for complex data structures.

### Recursive Generators

```rust
// Generate tree-like structures
fn tree_gen(depth: usize) -> Gen<Vec<i32>> {
    if depth == 0 {
        Gen::constant(vec![]) // Leaf
    } else {
        let leaf = Gen::<Vec<i32>>::vec_of(Gen::int_range(1, 10));
        let branch = tree_gen(depth - 1).map(|subtree| {
            let mut result = vec![0]; // Branch marker
            result.extend(subtree);
            result
        });
        
        Gen::frequency(vec![
            WeightedChoice::new(3, leaf),   // 60% leaves
            WeightedChoice::new(2, branch), // 40% branches
        ]).unwrap_or(Gen::constant(vec![]))
    }
}

let tree = tree_gen(4);
```

### Conditional Generation

```rust
let conditional_gen = Gen::new(|size, seed| {
    let (choice_seed, value_seed) = seed.split();
    let (choice, _) = choice_seed.next_bounded(3);

    match choice {
        0 => Gen::int_range(1, 10).generate(size, value_seed),        // Small
        1 => Gen::int_range(100, 1000).generate(size, value_seed),   // Large  
        _ => Gen::int_range(-50, -1).generate(size, value_seed),     // Negative
    }
});
```

### Dependent Generation

```rust
// Generate pairs where the second value depends on the first
let dependent_gen = Gen::int_range(1, 10).bind(|first| {
    Gen::int_range(first, first + 10).map(move |second| (first, second))
});
```

## Best Practices

1. **Start Simple**: Begin with basic generators and compose them into complex ones
2. **Use Appropriate Shrinking**: Ensure your generators shrink toward simpler, more minimal cases
3. **Validate Invariants**: Always test that your generators maintain type invariants
4. **Test Distribution**: Verify that probabilistic generators produce expected distributions
5. **Compose Thoughtfully**: Build complex generators from well-tested simple ones
6. **Monitor Performance**: Use appropriate size limits for recursive and complex generators

## Performance Considerations

- Use `Size` parameters to control generation complexity
- Limit recursive depth in recursive generators
- Consider using `frequency` and `one_of` for controlled randomness
- Monitor memory usage with large collection generators
- Use parallel testing for CPU-intensive properties

This guide covers the advanced features that have been thoroughly tested and validated. Each pattern shown here has corresponding meta tests that ensure correctness and reliability.