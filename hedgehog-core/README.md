# Hedgehog State Machine Testing

Fast, reliable state machine testing for Rust. Generate sequences of operations and verify your stateful systems work correctly under all conditions.

## Quick Start (5 minutes)

Add to your `Cargo.toml`:
```toml
[dependencies]
hedgehog-core = "0.1.0"
```

Create your first state machine test:

```rust
use hedgehog_core::state::*;
use hedgehog_core::gen::Gen;
use hedgehog_core::tree::Tree;

// 1. Define your system state
#[derive(Debug, Clone, PartialEq)]
struct Counter {
    value: i32,
}

impl Counter {
    fn new() -> Self { Self { value: 0 } }
}

// 2. Define command inputs
#[derive(Clone, Debug)]
struct AddInput { amount: i32 }

impl std::fmt::Display for AddInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "+{}", self.amount)
    }
}

#[test]
fn test_counter() {
    let mut generator = ActionGenerator::new();
    
    // 3. Create a command
    let add_cmd: Command<AddInput, i32, Counter, i32> = Command::new(
        "add".to_string(),
        |_state: &Counter| Some(Gen::new(|_size, seed| {
            let (amount, _) = seed.next_bounded(10);
            Tree::singleton(AddInput { amount: amount as i32 + 1 })
        })),
        |input| input.amount, // The actual operation
    )
    .with_update(|state, input, _output| {
        state.value += input.amount; // Update model state
    });
    
    generator.add_command(add_cmd);
    
    // 4. Generate and run test
    let initial = Counter::new();
    let test = generator.generate_sequential(initial.clone(), 5);
    
    execute_sequential(initial, test).unwrap();
}
```

That's it! This will generate random sequences of add operations and verify they work correctly.

## Real-World Examples

### Banking System
```rust
// Tests deposits, withdrawals, balance tracking
// Catches: overdrafts, incorrect balances, state inconsistencies

let deposit = Command::new("deposit", input_gen, execute)
    .with_require(|state, input| state.is_open && input.amount > 0)
    .with_update(|state, input, _| state.balance += input.amount)
    .with_ensure(|old, new, input, _| {
        assert_eq!(new.balance, old.balance + input.amount)
    });
```

### Connection Pool
```rust  
// Tests connection lifecycle, resource limits
// Catches: leaks, pool overflow, invalid requests

let connect = Command::new("connect", input_gen, execute)
    .with_require(|state, input| state.can_connect() && !state.is_connected(&input.host))
    .with_update(|state, input, _| state.connection_count += 1)
    .with_ensure(|old, new, _, _| new.connection_count == old.connection_count + 1);
```

## Core Concepts

- **Commands** define operations with preconditions, execution, and postconditions
- **State** is your model of the system being tested  
- **Generation** creates random sequences of valid operations
- **Execution** runs the operations and verifies postconditions
- **Shrinking** finds minimal failing examples automatically

## Common Patterns

**Resource Management**:
```rust
.with_require(|state, _| state.resources_available())
.with_update(|state, _, _| state.allocate_resource())
.with_ensure(|_, new_state, _, _| !new_state.has_leaks())
```

**State Transitions**:
```rust
.with_require(|state, input| state.can_transition_to(&input.new_state))
.with_update(|state, input, _| state.current = input.new_state)
.with_ensure(|_, new_state, input, _| new_state.current == input.new_state)
```

**Invariant Checking**:
```rust
.with_ensure(|_, new_state, _, _| {
    if !new_state.invariants_hold() {
        Err("System invariant violated".to_string())
    } else {
        Ok(())
    }
})
```

## Key Types

- `Command<Input, Output, State, M>` - Defines an operation
- `ActionGenerator<State>` - Generates test sequences  
- `Sequential<State, M>` - A sequence of operations to execute
- `Gen<T>` - Random value generator
- `Tree<T>` - Generated value with shrinking support

## Documentation

**Learning Path:**
1. **Start here**: [Tutorial](docs/tutorial.md) - Step-by-step from simple to complex (20 minutes)
2. **Reference**: [API Reference](docs/api-reference.md) - Complete type and function documentation  
3. **Advanced**: [State Machine Testing Guide](docs/state-machine-testing.md) - Comprehensive patterns and best practices
4. **Examples**: [Getting Started](docs/getting-started.md) - Real-world examples and debugging tips

**Quick Path**: Copy the example above → Follow the [Tutorial](docs/tutorial.md) → Build your tests

## Common Issues

**Import errors**: Use `use hedgehog_core::{state::*, gen::Gen, tree::Tree};`

**Compilation errors**: Make sure your Input type implements `Clone`, `Debug`, and `Display`

**No operations generated**: Check your command's input generator returns `Some(Gen<Input>)`

**Tests failing**: Verify your `with_update` actually changes the model state correctly

## Performance

- Generation is fast (creating test sequences)
- Execution speed matches your actual operations
- Start with short sequences (5-10 operations) and scale up
- Automatic shrinking finds minimal failing examples

State machine testing systematically finds bugs that manual testing typically misses!