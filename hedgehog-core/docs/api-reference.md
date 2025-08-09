# API Reference

## Core Types

### `Command<Input, Output, State, M>`

Defines a command that can be executed in state machine tests.

**Type Parameters:**
- `Input` - Parameters for the command (e.g., `AddInput`, `ConnectInput`)  
- `Output` - Type seen by postconditions (usually same as `M`)
- `State` - Your model state type (e.g., `Counter`, `ConnectionPool`)
- `M` - Return type of the execute function

**Constructor:**
```rust
Command::new(
    name: String,                                    // Command name for display
    input_gen: impl Fn(&State) -> Option<Gen<Input>>, // Input generator
    execute: impl Fn(Input) -> M,                    // Execution function
) -> Command<Input, Output, State, M>
```

**Callback Methods:**
```rust
.with_require(impl Fn(&State, &Input) -> bool) -> Self
.with_update(impl Fn(&mut State, &Input, &Var<Output>)) -> Self  
.with_ensure(impl Fn(&State, &State, &Input, &Output) -> Result<(), String>) -> Self
```

### `ActionGenerator<State>`

Generates sequences of commands for testing.

**Methods:**
```rust
ActionGenerator::new() -> Self

add_command<Input, Output, M>(&mut self, command: Command<Input, Output, State, M>)
where
    Input: 'static + Clone + Debug + Display,
    Output: 'static + Clone + Debug + Display,  
    State: 'static + Clone,
    M: 'static + Clone + Into<Output>,

generate_sequential(&self, initial_state: State, num_actions: usize) -> Sequential<State, ()>
where State: Clone
```

### `Sequential<State, M>`

A sequence of actions to execute.

**Fields:**
```rust
pub actions: Vec<Box<dyn ActionTrait<State, M>>>
```

### `Gen<T>`

Random value generator.

**Methods:**
```rust
Gen::new(impl Fn(Size, Seed) -> Tree<T>) -> Gen<T>
Gen::constant(value: T) -> Gen<T>  
Gen::sample(&self) -> T  // Convenience method with default parameters
```

### `Tree<T>` 

Generated value with automatic shrinking support.

**Methods:**
```rust
Tree::singleton(value: T) -> Tree<T>
```

## Execution Functions

### `execute_sequential`

Executes a sequential test, verifying all postconditions.

```rust
execute_sequential<State>(
    initial_state: State,
    sequential: Sequential<State, ()>
) -> Result<(), String>
where State: Clone
```

## Callback Types

### Require Callback
```rust
|state: &State, input: &Input| -> bool
```
- **When**: Before command execution during both generation and execution
- **Purpose**: Prevent invalid commands from being generated/executed  
- **Return**: `true` if command can proceed, `false` to skip

### Update Callback  
```rust
|state: &mut State, input: &Input, output: &Var<Output>|
```
- **When**: After command execution (in both generation and execution phases)
- **Purpose**: Update model state to reflect command effects
- **Note**: Called with symbolic output during generation, concrete during execution

### Ensure Callback
```rust
|old_state: &State, new_state: &State, input: &Input, output: &Output| -> Result<(), String>
```
- **When**: After command execution (execution phase only)
- **Purpose**: Verify command executed correctly and system invariants hold
- **Return**: `Ok(())` for success, `Err(message)` for failure

## Variable Types

### `Symbolic<T>`
Represents future command results during generation.

### `Concrete<T>`  
Holds actual command results during execution.

### `Var<T>`
Enum that can be either `Symbolic<T>` or `Concrete<T>`.

```rust
enum Var<T> {
    Symbolic(Symbolic<T>),
    Concrete(Concrete<T>),
}
```

## Important Traits

### Required for Input Types
```rust
Input: Clone + Debug + Display + 'static
```

### Required for Output Types  
```rust
Output: Clone + Debug + Display + 'static
```

### Required for State Types
```rust
State: Clone + 'static
```

## Usage Patterns

### Basic Command Pattern
```rust
let cmd: Command<MyInput, MyOutput, MyState, MyOutput> = Command::new(
    "my_command".to_string(),
    |state: &MyState| {
        if state.can_execute() {
            Some(generate_input_for_state(state))  
        } else {
            None
        }
    },
    |input: MyInput| execute_operation(input),
)
.with_require(|state, input| state.is_valid_for(input))
.with_update(|state, input, output| state.apply_changes(input, output))  
.with_ensure(|old, new, input, output| verify_operation(old, new, input, output));
```

### Input Generation Patterns
```rust
// Simple constant input
|_state| Some(Gen::constant(MyInput::default()))

// State-dependent input
|state: &MyState| {
    if state.allows_operation() {
        Some(Gen::new(|_size, seed| {
            let (value, _) = seed.next_bounded(state.max_value());
            Tree::singleton(MyInput { value: value as i32 })
        }))
    } else {
        None
    }
}

// Complex conditional input
|state: &MyState| match state.current_mode() {
    Mode::Active => Some(generate_active_input()),
    Mode::Inactive => Some(generate_inactive_input()), 
    Mode::Disabled => None,
}
```

### Error Handling in Postconditions
```rust
.with_ensure(|old_state, new_state, input, output| {
    // Check multiple conditions
    if new_state.value != old_state.value + input.amount {
        return Err(format!("Value mismatch: expected {}, got {}", 
            old_state.value + input.amount, new_state.value));
    }
    
    if !new_state.invariants_hold() {
        return Err("System invariants violated".to_string());
    }
    
    if *output != expected_output(input) {
        return Err(format!("Wrong output: expected {:?}, got {:?}", 
            expected_output(input), output));
    }
    
    Ok(())
})
```

## Common Mistakes

### ❌ Forgetting State Updates
```rust
.with_update(|state, input, _| {
    // BUG: Forgot to actually update state
    // state.value += input.amount;  // Missing!
})
```

### ❌ Wrong Postcondition Logic
```rust  
.with_ensure(|old, new, input, _| {
    // BUG: This checks if values are equal (should be different)
    if new.value == old.value + input.amount {
        Err("Values should be different".to_string())  // Wrong logic
    } else {
        Ok(())
    }
})
```

### ❌ Missing Type Annotations
```rust
// BUG: Compiler can't infer state type
|state| Some(Gen::constant(MyInput::default()))

// FIX: Add explicit type
|state: &MyState| Some(Gen::constant(MyInput::default()))
```

### ❌ Incorrect Imports
```rust
// BUG: Wrong module paths
use hedgehog_core::state::*;  // Correct
use hedgehog_core::gen::Gen;  // Correct  
use hedgehog_core::Tree;      // Wrong - should be tree::Tree
```

## Performance Notes

- **Generation is fast** - Creating command sequences is lightweight
- **Execution speed matches your operations** - Test performance depends on actual command execution
- **Start with short sequences** - Begin with 5-10 operations, scale up as needed  
- **Memory usage is linear** - Proportional to sequence length and state size

## Debugging Tips

1. **Add debug prints to callbacks**:
   ```rust
   .with_require(|state, input| {
       let result = state.can_execute(input);
       println!("Require check: {} -> {}", input, result);
       result
   })
   ```

2. **Verify state updates**:
   ```rust
   .with_update(|state, input, _| {
       println!("Before: {:?}", state);
       state.apply(input);
       println!("After: {:?}", state);
   })
   ```

3. **Check postcondition failures**:
   ```rust
   .with_ensure(|old, new, input, output| {
       if let Err(msg) = verify_operation(old, new, input, output) {
           println!("POSTCONDITION FAILED: {}", msg);
           println!("  Old state: {:?}", old);
           println!("  New state: {:?}", new);  
           println!("  Input: {:?}", input);
           println!("  Output: {:?}", output);
           Err(msg)
       } else {
           Ok(())
       }
   })
   ```

## Parallel Testing API

### Functions

- `for_all_parallel<T, F>(gen: Gen<T>, condition: F, thread_count: usize) -> ParallelProperty<T, ...>`
  - Simple parallel property testing with automatic thread management
- `parallel_property<T, F>(gen: Gen<T>, test_fn: F, config: ParallelConfig) -> ParallelProperty<T, F>`
  - Advanced parallel property testing with custom configuration

### Types

**`ParallelConfig`** - Configuration for parallel execution
```rust
pub struct ParallelConfig {
    pub thread_count: usize,                 // Number of threads to use
    pub work_distribution: WorkDistribution, // How to distribute work
    pub timeout: Option<Duration>,           // Timeout for deadlock detection
    pub detect_non_determinism: bool,        // Enable race condition detection
}
```

**`WorkDistribution`** - Strategy for distributing work across threads
```rust
pub enum WorkDistribution {
    RoundRobin,    // Distribute tests evenly in round-robin fashion
    ChunkBased,    // Process tests in chunks per thread
    WorkStealing,  // Advanced load balancing (falls back to RoundRobin)
}
```

**`ParallelTestResult`** - Results from parallel test execution
```rust
pub struct ParallelTestResult {
    pub outcome: TestResult,                          // Overall test outcome
    pub thread_results: Vec<TestResult>,              // Results from each thread
    pub performance: ParallelPerformanceMetrics,     // Performance data
    pub concurrency_issues: ConcurrencyIssues,       // Race conditions detected
}
```

### Example Usage

```rust
use hedgehog_core::*;

// Basic parallel testing
let prop = for_all_parallel(
    Gen::int_range(1, 100), 
    |&n| n > 0, 
    4  // 4 threads
);

// Advanced configuration
let config = ParallelConfig {
    thread_count: 8,
    work_distribution: WorkDistribution::RoundRobin,
    timeout: Some(Duration::from_secs(30)),
    detect_non_determinism: true,
};

let prop = parallel_property(gen, test_fn, config);
let result = prop.run(&Config::default().with_tests(1000));

println!("Speedup: {:.2}x", result.performance.speedup_factor);
println!("Thread efficiency: {:.1}%", result.performance.thread_efficiency * 100.0);
```

This completes the core API for property-based and state machine testing with Hedgehog.