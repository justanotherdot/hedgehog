# Hedgehog Implementation Plan

## Current Status
✅ **Foundation Complete**
- Core property testing API
- Generator system with shrinking
- Variable name tracking
- Derive macros
- Comprehensive test suite

## Phase 1: Quick Wins (3-4 days)

### 1.1 Example Integration (1 day)
**Goal**: Mix explicit test cases with generated ones
**API Design**:
```rust
#[test]
fn test_parsing() {
    let prop = for_all(Gen::string(), |s| parse(s).is_ok() || parse(s).is_err())
        .with_examples(&["", "null", "true", "false", "{}", "[]"]);
    // Always tests examples first, then generates random inputs
}
```

**Implementation**:
- Extend `Property` struct to store examples
- Modify `run()` to execute examples before generation
- Add `.with_examples()` method
- Update failure reporting to distinguish example vs generated failures

### 1.2 Property Classification (1-2 days)
**Goal**: See distribution of generated test data
**API Design**:
```rust
let prop = for_all(Gen::vec_int(), |vec| {
    vec.reverse(); vec.reverse(); vec
}).classify("empty", |vec| vec.is_empty())
  .classify("small", |vec| vec.len() < 10)
  .collect("length", |vec| vec.len());
```

**Implementation**:
- Add classification system to property execution
- Track statistics during test runs
- Display distribution in test output
- Support both categorical (`classify`) and numerical (`collect`) data

### 1.3 Dictionary Support (1 day)
**Goal**: Domain-specific token injection
**API Design**:
```rust
let json_dict = Dictionary::new()
    .add_tokens(&["null", "true", "false"])
    .add_tokens(&["{", "}", "[", "]"]);

let prop = for_all(Gen::string().with_dictionary(json_dict), |s| {
    json_parser(s) // More realistic inputs
});
```

**Implementation**:
- Create `Dictionary` type for storing tokens
- Extend string generators to use dictionaries
- Implement weighted selection between dictionary and random generation
- Support multiple dictionaries per generator

## Phase 2: Major Features (1 week)

### 2.1 State Machine Testing (3-5 days)
**Goal**: Test stateful systems systematically
**Priority**: **HIGH** - This is the most impactful missing feature

**API Design**:
```rust
#[derive(Debug, Clone)]
enum Command {
    Insert(String, i32),
    Remove(String),
    Get(String),
}

#[derive(Debug, Clone, Default)]
struct Model {
    data: HashMap<String, i32>,
}

impl StateMachine for MySystem {
    type State = Model;
    type Command = Command;
    
    fn commands(state: &Self::State) -> Gen<Self::Command> {
        // Generate valid commands based on current state
        if state.data.is_empty() {
            Gen::frequency(&[
                (10, Command::Insert(Gen::string(), Gen::int())),
                (1, Command::Get(Gen::string())), // Mostly inserts when empty
            ])
        } else {
            Gen::frequency(&[
                (3, Command::Insert(Gen::string(), Gen::int())),
                (2, Command::Remove(Gen::from_keys(&state.data))),
                (5, Command::Get(Gen::from_keys(&state.data))),
            ])
        }
    }
    
    fn precondition(state: &Self::State, cmd: &Self::Command) -> bool {
        match cmd {
            Command::Remove(key) => state.data.contains_key(key),
            _ => true,
        }
    }
    
    fn postcondition(old_state: &Self::State, cmd: &Self::Command, result: &Result<Response>) -> bool {
        // Verify real system result matches expected model behavior
        match (cmd, result) {
            (Command::Get(key), Ok(value)) => {
                old_state.data.get(key) == Some(value)
            }
            _ => true,
        }
    }
    
    fn next_state(state: &mut Self::State, cmd: &Self::Command) {
        // Update model state
        match cmd {
            Command::Insert(key, value) => { state.data.insert(key.clone(), *value); }
            Command::Remove(key) => { state.data.remove(key); }
            Command::Get(_) => {} // No state change
        }
    }
}

#[test]
fn test_database_properties() {
    let prop = state_machine_property::<MySystem>()
        .max_commands(50)
        .initial_state(Model::default());
    
    assert!(prop.run(&Config::default()).is_pass());
}
```

**Implementation**:
- Create `StateMachine` trait
- Command generation with state-dependent preconditions
- Model state tracking and updates
- Real system execution and postcondition checking
- Command sequence shrinking
- Parallel execution support (find race conditions)

### 2.2 Function Generators (2-3 days)
**Goal**: Generate functions as test inputs
**API Design**:
```rust
let prop = for_all(
    (Gen::function(Gen::int(), Gen::bool()), Gen::vec_int()),
    |(predicate, vec)| {
        let filtered: Vec<_> = vec.iter().filter(|&x| predicate(*x)).collect();
        filtered.len() <= vec.len()
    }
);
```

**Implementation**:
- Function representation (lookup table vs decision tree)
- Deterministic function generation
- Function shrinking strategies
- Support for multiple argument functions

### 2.3 Coverage-Guided Generation (2-3 days)
**Goal**: Use coverage feedback to improve generation
**Implementation**:
- Integration with `cargo-llvm-cov` or similar
- Coverage feedback loop
- Bias generation towards unexplored paths
- Performance optimization to avoid overhead

## Phase 3: Advanced Features (1-2 weeks)

### 3.1 Regression Corpus (2-3 days)
**Goal**: Automatic persistence of failing cases
- Already documented in `regression-corpus.md`
- Implementation similar to PropTest's approach

### 3.2 Parallel Property Testing (3-4 days)
**Goal**: Find race conditions and concurrency bugs
- Multi-threaded state machine execution
- Race condition detection
- Timing-sensitive property testing

### 3.3 Fault Injection (2-3 days)
**Goal**: Systematic failure testing
- Network failure injection
- Disk I/O failure simulation
- Timeout and resource exhaustion testing

### 3.4 Compositional Strategies (1-2 days)
**Goal**: Better generator composition
- PropTest-style strategy composition
- Macro support for complex generators

## Implementation Strategy

### Revised Timeline

**Week 1: Phase 1 - Quick Wins (3-4 days)**
- Day 1: Example integration
- Day 2: Property classification  
- Day 3: Dictionary support
- Day 4: Polish and testing

**Week 2: Phase 2 - Major Features**
- Days 1-3: State machine testing (core implementation)
- Day 4: Function generators (basic version)
- Day 5: Coverage-guided generation (basic version)

**Week 3: Phase 3 - Advanced Features**
- Days 1-2: Regression corpus
- Days 3-4: Parallel property testing
- Day 5: Fault injection basics

**Total: 2-3 weeks** for comprehensive feature set

### Success Criteria

**Phase 1 Complete When**:
- Can mix explicit examples with generated tests
- Property classification shows test data distribution
- Dictionary support works for domain-specific generation
- All existing tests still pass

**Phase 2 Complete When**:
- State machine testing can test a real stateful system (e.g., HashMap, database)
- Function generators can test higher-order functions
- Coverage-guided generation demonstrably improves test effectiveness

**Phase 3 Complete When**:
- Regression corpus automatically saves/replays failures
- Parallel testing can find real race conditions
- Fault injection can systematically test error handling

## Risk Assessment

**High Risk**:
- **State machine testing complexity** - This is a substantial feature
- **Coverage integration** - May require deep toolchain integration

**Medium Risk**:
- **Function generator shrinking** - Complex algorithms
- **Performance impact** - Coverage guidance could slow tests

**Low Risk**:
- **Example integration** - Straightforward implementation
- **Dictionary support** - Well-understood feature

## Success Metrics

1. **Adoption**: Other Rust projects start using Hedgehog
2. **Uniqueness**: Features not available in other Rust property testing libraries
3. **Stability**: No regressions in existing functionality
4. **Performance**: New features don't significantly slow down testing
5. **Documentation**: Each feature has clear examples and documentation

This plan balances immediate value (Phase 1) with long-term strategic features (Phase 2) while maintaining a sustainable development pace.