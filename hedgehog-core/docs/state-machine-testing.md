# State Machine Testing with Hedgehog

State machine testing is a powerful technique for testing stateful systems by generating sequences of operations and verifying that the system behaves correctly under various scenarios.

## Quick Start

Here's a minimal example testing a simple counter:

```rust
use hedgehog_core::state::*;
use hedgehog_core::gen::Gen;

// 1. Define your state
#[derive(Debug, Clone, PartialEq)]
struct Counter {
    value: i32,
    max_value: i32,
}

impl Counter {
    fn new() -> Self {
        Self { value: 0, max_value: 100 }
    }
}

// 2. Define command inputs  
#[derive(Clone, Debug)]
struct IncrementInput {
    amount: i32,
}

impl std::fmt::Display for IncrementInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.amount)
    }
}

// 3. Create commands with preconditions, actions, and postconditions
fn test_counter_state_machine() {
    let mut generator = ActionGenerator::new();
    
    let increment_cmd: Command<IncrementInput, i32, Counter, i32> = Command::new(
        "increment".to_string(),
        // Generator: How to create random inputs
        |state: &Counter| {
            if state.value < state.max_value {
                Some(Gen::new(|_size, seed| {
                    let (amount, _) = seed.next_bounded(10);
                    crate::tree::Tree::singleton(IncrementInput { 
                        amount: amount as i32 + 1 
                    })
                }))
            } else {
                None // Can't increment beyond max
            }
        },
        // Executor: The actual operation to perform
        |input: IncrementInput| input.amount,
    )
    // Precondition: When is this command allowed?
    .with_require(|state: &Counter, input: &IncrementInput| {
        state.value + input.amount <= state.max_value
    })
    // State update: How does this change the model state?
    .with_update(|state: &mut Counter, input: &IncrementInput, _output: &Var<i32>| {
        state.value += input.amount;
    })
    // Postcondition: Verify the operation worked correctly
    .with_ensure(|old_state: &Counter, new_state: &Counter, input: &IncrementInput, output: &i32| {
        if new_state.value != old_state.value + input.amount {
            Err("Counter value not updated correctly".to_string())
        } else if *output != input.amount {
            Err("Incorrect return value".to_string())
        } else {
            Ok(())
        }
    });
    
    generator.add_command(increment_cmd);
    
    // 4. Generate and run test sequences
    let initial_state = Counter::new();
    let sequential = generator.generate_sequential(initial_state.clone(), 10);
    
    // 5. Execute and verify
    let result = execute_sequential(initial_state, sequential);
    assert!(result.is_ok());
}
```

## Core Concepts

### Commands

A `Command` defines:
- **Input Generation**: How to create random inputs based on current state
- **Execution**: The actual operation to perform  
- **Callbacks**: Preconditions, state updates, and postconditions

### Symbolic vs Concrete Variables

- **Symbolic** variables represent future results during test generation
- **Concrete** variables hold actual values during test execution
- The framework automatically maps between them

### The Three Callback Types

1. **Require** - Preconditions that must be true before execution
2. **Update** - State changes that occur after execution  
3. **Ensure** - Postconditions that verify correct behavior

## Best Practices

### Designing Good Commands

**DO:**
- Make preconditions explicit and comprehensive
- Ensure state updates mirror what the real system does
- Write thorough postconditions that catch edge cases
- Use meaningful command names and input displays

**DON'T:**
- Skip precondition checks - they prevent invalid test sequences
- Forget to update model state - leads to divergence from reality
- Make postconditions too lenient - they should catch real bugs

### Input Generation Strategy

```rust
// Good: Generate inputs appropriate to current state
|state: &BankAccount| {
    if state.balance > 0 {
        let max_withdraw = std::cmp::min(state.balance, 1000);
        Some(Gen::new(move |_size, seed| {
            let (amount, _) = seed.next_bounded(max_withdraw as u64 + 1);
            Tree::singleton(WithdrawInput { amount: amount as i32 })
        }))
    } else {
        None // Can't withdraw from empty account
    }
}

// Bad: Generate inputs without considering state
|_state: &BankAccount| {
    Some(Gen::constant(WithdrawInput { amount: 100 })) // Might overdraw!
}
```

### State Modeling

Your model state should:
- Track all relevant system state that affects command behavior
- Be cheaper to operate on than the real system
- Clearly represent system invariants

```rust
#[derive(Debug, Clone)]
struct FileSystem {
    files: HashMap<PathBuf, String>,      // Track file contents
    directories: HashSet<PathBuf>,        // Track directories
    permissions: HashMap<PathBuf, u32>,   // Track permissions
    current_dir: PathBuf,                 // Track working directory
}
```

## Common Patterns

### Resource Management Testing

Test systems that acquire and release resources:

```rust
// Commands: connect, disconnect, send_request, cleanup
// State: connection_pool, active_requests, resource_limits
// Invariants: connections <= max_connections, no leaked resources
```

### Transaction Testing  

Test transactional systems:

```rust
// Commands: begin_transaction, commit, rollback, insert, update, delete  
// State: transaction_state, table_data, isolation_level
// Invariants: ACID properties, no dirty reads, consistent state
```

### Cache Testing

Test caching behavior:

```rust  
// Commands: put, get, evict, expire, clear
// State: cache_entries, eviction_policy, memory_usage, timestamps
// Invariants: size limits, expiration correctness, cache coherence  
```

## Debugging Failed Tests

When a state machine test fails:

1. **Check the generated sequence** - Does it make sense?
2. **Verify preconditions** - Are invalid commands being generated?  
3. **Examine state updates** - Is the model state tracking correctly?
4. **Review postconditions** - Are they catching the right invariants?

The framework provides detailed failure information showing:
- The sequence of actions that led to failure
- Which postcondition failed and why
- The state at each step of execution

## Integration with Property Testing

State machine tests work great with traditional property tests:

```rust
#[test] 
fn property_counter_never_exceeds_max() {
    // Generate many different counter sequences
    for _ in 0..100 {
        let initial_state = Counter::new();
        let sequential = generator.generate_sequential(initial_state.clone(), 20);
        
        execute_sequential(initial_state, sequential).unwrap();
        // If we get here, all postconditions passed
    }
}
```

## Performance Considerations

- **Generation is fast** - Creating test sequences is lightweight
- **Execution mirrors your system** - Performance depends on actual operations
- **Start small** - Begin with short sequences, increase length as needed
- **Parallel testing** - Use `execute_parallel` for concurrency testing

## Advanced Features

### Custom Shrinking

The framework automatically shrinks failing test cases to minimal examples. You can customize this by implementing shrinking in your input generators.

### Parallel Execution

Test concurrent scenarios:

```rust
let parallel = generator.generate_parallel(
    initial_state,
    5,  // prefix length  
    3,  // branch1 length
    3,  // branch2 length  
);

execute_parallel(initial_state, parallel).unwrap();
```

### Variable Dependencies

Commands can reference outputs of previous commands:

```rust
// Later commands can use symbolic variables from earlier ones
// The framework handles the symbolic→concrete mapping automatically
```

This enables testing complex workflows where operations depend on results of previous operations.