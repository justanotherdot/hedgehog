# State Machine Testing Tutorial

This tutorial walks you through state machine testing step-by-step, from simple examples to real-world applications.

## Step 1: Your First Test (2 minutes)

Let's test a simple counter that can only increment:

```rust
use hedgehog_core::{state::*, gen::Gen, tree::Tree};

#[derive(Debug, Clone, PartialEq)]
struct Counter {
    value: i32,
}

#[derive(Clone, Debug)]
struct AddInput { amount: i32 }

impl std::fmt::Display for AddInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "+{}", self.amount)
    }
}

#[test]
fn test_simple_counter() {
    let mut generator = ActionGenerator::new();
    
    let add_cmd: Command<AddInput, i32, Counter, i32> = Command::new(
        "add".to_string(),
        |_state: &Counter| Some(Gen::new(|_size, seed| {
            let (amount, _) = seed.next_bounded(10);
            Tree::singleton(AddInput { amount: amount as i32 + 1 })
        })),
        |input: AddInput| input.amount,
    )
    .with_update(|state: &mut Counter, input: &AddInput, _output: &Var<i32>| {
        state.value += input.amount;
    });
    
    generator.add_command(add_cmd);
    
    let initial = Counter { value: 0 };
    let test = generator.generate_sequential(initial.clone(), 5);
    
    execute_sequential(initial, test).unwrap();
}
```

**What this does:**
- Generates 5 random "add" operations
- Each adds between 1-10 to the counter
- Verifies the operations complete without errors

## Step 2: Add Constraints (5 minutes)

Now let's add business rules - the counter can't exceed 100:

```rust
#[derive(Debug, Clone, PartialEq)]
struct BoundedCounter {
    value: i32,
    max_value: i32,
}

impl BoundedCounter {
    fn new() -> Self { Self { value: 0, max_value: 100 } }
    fn can_add(&self, amount: i32) -> bool { self.value + amount <= self.max_value }
}

#[test]
fn test_bounded_counter() {
    let mut generator = ActionGenerator::new();
    
    let add_cmd: Command<AddInput, i32, BoundedCounter, i32> = Command::new(
        "add".to_string(),
        |state: &BoundedCounter| {
            if state.value < state.max_value {
                Some(Gen::new(|_size, seed| {
                    let remaining = state.max_value - state.value;
                    let max_add = std::cmp::min(remaining, 10);
                    let (amount, _) = seed.next_bounded(max_add as u64);
                    Tree::singleton(AddInput { amount: amount as i32 + 1 })
                }))
            } else {
                None // Can't add when at max
            }
        },
        |input: AddInput| input.amount,
    )
    .with_require(|state: &BoundedCounter, input: &AddInput| {
        state.can_add(input.amount) // Precondition: addition must be valid
    })
    .with_update(|state: &mut BoundedCounter, input: &AddInput, _output: &Var<i32>| {
        state.value += input.amount;
    })
    .with_ensure(|old_state: &BoundedCounter, new_state: &BoundedCounter, input: &AddInput, output: &i32| {
        // Postcondition: verify the operation worked correctly
        if new_state.value != old_state.value + input.amount {
            Err(format!("Expected {}, got {}", old_state.value + input.amount, new_state.value))
        } else if new_state.value > new_state.max_value {
            Err("Counter exceeded maximum".to_string())
        } else if *output != input.amount {
            Err("Incorrect return value".to_string())
        } else {
            Ok(())
        }
    });
    
    generator.add_command(add_cmd);
    
    let initial = BoundedCounter::new();
    let test = generator.generate_sequential(initial.clone(), 20);
    
    execute_sequential(initial, test).unwrap();
}
```

**What's new:**
- **Input generator checks state** - Only generates valid amounts
- **Preconditions** - Prevents invalid operations 
- **Postconditions** - Verifies correctness after execution

## Step 3: Multiple Commands (10 minutes)

Let's add a reset command to make testing more interesting:

```rust
#[derive(Clone, Debug)]
struct ResetInput;

impl std::fmt::Display for ResetInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "reset")
    }
}

#[test]
fn test_counter_with_reset() {
    let mut generator = ActionGenerator::new();
    
    // Add command (same as before)
    let add_cmd: Command<AddInput, i32, BoundedCounter, i32> = Command::new(
        "add".to_string(),
        |state: &BoundedCounter| {
            if state.value < state.max_value {
                Some(Gen::new(|_size, seed| {
                    let remaining = state.max_value - state.value;
                    let max_add = std::cmp::min(remaining, 10);
                    let (amount, _) = seed.next_bounded(max_add as u64);
                    Tree::singleton(AddInput { amount: amount as i32 + 1 })
                }))
            } else {
                None
            }
        },
        |input: AddInput| input.amount,
    )
    .with_require(|state: &BoundedCounter, input: &AddInput| state.can_add(input.amount))
    .with_update(|state: &mut BoundedCounter, input: &AddInput, _| state.value += input.amount)
    .with_ensure(|old, new, input, output| {
        if new.value != old.value + input.amount {
            Err("Add failed".to_string())
        } else {
            Ok(())
        }
    });
    
    // Reset command
    let reset_cmd: Command<ResetInput, i32, BoundedCounter, i32> = Command::new(
        "reset".to_string(),
        |state: &BoundedCounter| {
            if state.value > 0 {
                Some(Gen::constant(ResetInput))
            } else {
                None // No point resetting when already at 0
            }
        },
        |_input: ResetInput| 0, // Always returns 0
    )
    .with_update(|state: &mut BoundedCounter, _: &ResetInput, _| {
        state.value = 0;
    })
    .with_ensure(|_old, new, _input, output| {
        if new.value != 0 {
            Err("Reset failed to set value to 0".to_string())
        } else if *output != 0 {
            Err("Reset should return 0".to_string())
        } else {
            Ok(())
        }
    });
    
    generator.add_command(add_cmd);
    generator.add_command(reset_cmd);
    
    let initial = BoundedCounter::new();
    let test = generator.generate_sequential(initial.clone(), 15);
    
    println!("Generated counter operations:");
    for (i, action) in test.actions.iter().enumerate() {
        println!("  {}: {}", i + 1, action.display_action());
    }
    
    execute_sequential(initial, test).unwrap();
}
```

**What's new:**
- **Multiple commands** - Framework randomly selects between add/reset
- **State-dependent availability** - Reset only available when value > 0
- **Command interactions** - Reset enables more adds by freeing up capacity

## Step 4: Understanding the Type Parameters

The `Command<Input, Output, State, M>` type has four parameters:

- **`Input`** - The parameters for your command (e.g., `AddInput`, `ResetInput`)
- **`Output`** - What the postconditions see (usually same as `M`)
- **`State`** - Your model state (e.g., `BoundedCounter`)
- **`M`** - What your execute function returns (converted to `Output`)

Most of the time `Output` and `M` are the same type.

## Step 5: Common Patterns

### Resource Management
```rust
.with_require(|state, _| state.resources_available())
.with_update(|state, _, _| state.allocate_resource())
.with_ensure(|old, new, _, _| new.resources_used == old.resources_used + 1)
```

### State Validation
```rust
.with_ensure(|_, new_state, _, _| {
    if !new_state.is_valid() {
        Err("System invariant violated".to_string())
    } else {
        Ok(())
    }
})
```

### Conditional Commands
```rust
|state: &MyState| {
    if state.can_perform_operation() {
        Some(generate_input_for_state(state))
    } else {
        None // Command not available in this state
    }
}
```

## Step 6: Debugging Failed Tests

When tests fail, you'll see output like:
```
thread 'test_counter_with_reset' panicked at src/lib.rs:42:5:
assertion failed: result.is_ok()

Executing: Var0 = add(+5)
Executing: Var2 = reset
Executing: Var4 = add(+10)  ← FAILED HERE
Error: Add failed
```

**Common debugging steps:**

1. **Check your state updates** - Is the model state changing correctly?
```rust
// Bad: forgot to update state
.with_update(|state, input, _| {
    // Missing: state.value += input.amount;
})
```

2. **Verify preconditions** - Are invalid operations being prevented?
```rust  
// Add some debug output
.with_require(|state, input| {
    let can_add = state.can_add(input.amount);
    println!("Can add {} to {}? {}", input.amount, state.value, can_add);
    can_add
})
```

3. **Check postconditions** - Are they testing the right things?
```rust
// Bad: too strict
.with_ensure(|old, new, input, _| {
    if new.value == old.value + input.amount {  // Should be != for error
        Err("Values should be different".to_string())
    } else {
        Ok(())
    }
})
```

## Next Steps

You're now ready for more complex examples:

- **Banking System** - Multiple account operations, balance tracking
- **Connection Pools** - Resource limits, lifecycle management  
- **Caches** - Expiration, eviction, memory management
- **Databases** - Transactions, consistency, concurrent access

See the main documentation for these advanced examples.

## Key Takeaways

1. **Start simple** - Begin with 1-2 commands, add complexity gradually
2. **Model your domain** - State should track everything that affects command behavior
3. **Use preconditions** - Prevent invalid command sequences from being generated
4. **Write thorough postconditions** - They're your safety net for catching bugs
5. **Debug step by step** - Add print statements to understand what's happening

State machine testing systematically explores your system's behavior in ways manual testing can't match!