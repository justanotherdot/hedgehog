# Getting Started with Hedgehog State Machine Testing

This guide shows you how to use Hedgehog for state machine testing with real-world examples.

## What We're Testing

State machine testing helps you verify that stateful systems behave correctly under various sequences of operations. This is particularly useful for:

- **Resource Management**: Connection pools, file handles, memory allocation
- **Transaction Systems**: Databases, financial operations, shopping carts  
- **Caching Systems**: Cache invalidation, eviction policies, consistency
- **API Clients**: Authentication, rate limiting, retry logic
- **State Machines**: Game states, workflow engines, protocol handlers

## Current Examples

### 1. Banking System ✅ (Fully Implemented)
**Location**: `src/state.rs::test_comprehensive_state_machine_workflow`

```rust
// Tests deposit/withdrawal operations with balance tracking
// Catches: Overdrafts, incorrect balance updates, transaction counting errors
let deposit_cmd = Command::new("deposit", ...)
    .with_require(|state, input| state.is_open && input.amount > 0)
    .with_update(|state, input, _| state.balance += input.amount)
    .with_ensure(|old, new, input, output| { /* verify balance */ });
```

**What it demonstrates:**
- ✅ Preconditions prevent invalid operations
- ✅ State updates track changes correctly  
- ✅ Postconditions catch incorrect behavior
- ✅ Realistic business logic constraints

### 2. Connection Pool Management ✅ (Fully Implemented) 
**Location**: `src/state.rs::test_connection_pool_state_machine`

```rust
// Tests HTTP connection lifecycle with resource limits
// Catches: Connection leaks, pool overflow, request-without-connection errors
let connect_cmd = Command::new("connect", ...)
    .with_require(|state, input| state.can_connect() && !state.is_connected(&input.host))
    .with_update(|state, input, _| state.connection_count += 1)
    .with_ensure(|old, new, input, _| { /* verify connection established */ });
```

**What it demonstrates:**
- ✅ Resource management patterns
- ✅ Capacity constraints (max connections)
- ✅ State-dependent command availability
- ✅ Complex state interactions between commands

## Test Output Examples

### Banking System Test:
```
Generated banking sequence:
  Var0 = deposit($36)
  Var2 = deposit($84) 
  Var4 = deposit($76)
  Var6 = withdraw($21)
  Var8 = deposit($10)
  Var10 = withdraw($4)
  ...
✓ All transactions succeeded with proper state tracking!
```

### Connection Pool Test:
```
Generated connection pool sequence:
  1: Var0 = connect(db.example.com)
  2: Var2 = connect(cache.example.com)  
  3: Var4 = request(GET db.example.com/)
  4: Var6 = disconnect(cache.example.com)
  ...
✓ Connection pool state machine test passed!
```

## What Types of Bugs These Catch

### 1. State Consistency Bugs
```rust
// BUG: Balance not updated correctly
state.balance += input.amount - 1; // Off-by-one error
// ✅ CAUGHT: Postcondition fails because new_state.balance != old_state.balance + input.amount
```

### 2. Resource Leak Bugs  
```rust
// BUG: Connection count not decremented on disconnect
// ✅ CAUGHT: Postcondition fails because connection_count not reduced
```

### 3. Invalid Operation Bugs
```rust
// BUG: Allow withdrawal when balance insufficient
// ✅ CAUGHT: Precondition prevents generation of invalid withdrawals
```

### 4. Race Condition Setup
```rust
// With parallel execution, catches:
// - Concurrent access issues  
// - Non-atomic operations
// - Incorrect synchronization
```

## Adding Your Own Examples

### Step 1: Model Your System State
```rust
#[derive(Debug, Clone, PartialEq)]
struct MySystem {
    // Track all relevant state that affects operations
    resources: HashMap<ResourceId, Resource>,
    limits: ResourceLimits,
    current_state: SystemState,
}
```

### Step 2: Define Command Inputs
```rust
#[derive(Clone, Debug)]
struct MyOperationInput {
    // Parameters for your operation
    resource_id: ResourceId,
    amount: usize,
}

impl Display for MyOperationInput {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}:{}", self.resource_id, self.amount)
    }
}
```

### Step 3: Create Commands
```rust
let my_cmd: Command<MyOperationInput, MyOutput, MySystem, MyOutput> = 
    Command::new("my_operation", input_generator, executor)
        .with_require(|state, input| /* when is this valid? */)
        .with_update(|state, input, output| /* how does state change? */)
        .with_ensure(|old, new, input, output| /* verify correctness */);
```

### Step 4: Test It
```rust
let mut generator = ActionGenerator::new();
generator.add_command(my_cmd);

let initial_state = MySystem::new();
let sequential = generator.generate_sequential(initial_state.clone(), 20);
let result = execute_sequential(initial_state, sequential);

assert!(result.is_ok());
```

## Debugging Failed Tests

When tests fail, you'll see detailed output like:
```
✗ Transaction failed: Balance mismatch: expected 150, got 149
Executing sequence:
  1: Var0 = deposit($100) ✓
  2: Var2 = deposit($50) ✓  
  3: Var4 = withdraw($1) ✗ <- FAILED HERE
```

Common debugging steps:
1. **Check state updates** - Is your model state tracking correctly?
2. **Verify preconditions** - Are invalid operations being prevented?  
3. **Review postconditions** - Are they checking the right invariants?
4. **Examine command interactions** - Do commands affect each other properly?

## Best Practices

### ✅ DO:
- Start with simple examples and add complexity gradually
- Write comprehensive preconditions to prevent invalid sequences
- Make postconditions strict to catch subtle bugs
- Test both successful and error cases
- Use meaningful names for commands and states

### ❌ DON'T:
- Skip precondition validation (leads to invalid test sequences)
- Make postconditions too lenient (misses real bugs)
- Forget to update model state (causes state divergence) 
- Generate inputs that ignore current state
- Test only happy path scenarios

## Next Steps

1. **Try the existing examples** - Run the banking and connection pool tests
2. **Study the patterns** - Look at how commands interact with state
3. **Start small** - Begin with 2-3 simple commands for your system
4. **Add complexity** - Introduce edge cases, resource limits, error conditions
5. **Run frequently** - State machine tests are great for regression testing

## Performance Notes

- **Generation is fast** - Creating sequences is lightweight
- **Execution matches your system** - Test speed depends on actual operations  
- **Start with short sequences** - 10-20 operations, then increase
- **Parallel testing available** - Use `execute_parallel` for concurrency testing

The framework provides automatic shrinking of failing test cases to help you find minimal reproducing examples quickly.

## Real-World Impact

State machine testing has proven effective at finding:
- **Distributed system bugs**: Consensus protocol violations, split-brain scenarios
- **Database issues**: Transaction isolation bugs, deadlocks, corruption
- **Network problems**: Connection management, retry logic, backpressure handling
- **Resource leaks**: File handles, memory, database connections
- **API contract violations**: Invalid state transitions, missing validations

The key insight is that many bugs only appear with specific sequences of operations, which manual testing often misses but property-based state machine testing can find systematically.