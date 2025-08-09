# Targeted Property Testing: PROPER vs Hedgehog Comparison

This document compares the targeted property-based testing implementations between PROPER (Erlang) and our Hedgehog (Rust) implementation.

## Architecture Overview

### PROPER Architecture
- **Gen Server-based**: Uses a dedicated gen_server process (`proper_target`) for managing search state
- **Strategy Pattern**: Search strategies are separate modules implementing `proper_target` behavior
- **Centralized State**: All targeting state is managed in the gen server process
- **Macro-driven API**: Heavy reliance on Erlang macros (`?FORALL_TARGETED`, `?MAXIMIZE`, `?USERNF`)
- **Runtime Hook System**: Integrates with PROPER's existing type system through runtime hooks

### Hedgehog Architecture  
- **Direct Function Calls**: Self-contained search structures with direct method calls
- **Struct-based Design**: Search strategies are structs with trait implementations
- **Local State**: Each search instance manages its own state
- **Function-based API**: Uses functions like `for_all_targeted` with closure parameters
- **Compile-time Integration**: Integrates with Hedgehog's generator system at compile time

## Core API Comparison

### PROPER API
```erlang
% Basic targeted property
prop_example() ->
  ?FORALL_TARGETED(X, Type,
    begin
      UV = compute_utility(X),
      ?MAXIMIZE(UV),
      property_condition(X)
    end).

% Custom neighborhood function
prop_custom() ->
  ?FORALL_TARGETED(X, ?USERNF(Type, custom_nf()),
    begin
      ?MINIMIZE(cost(X)),
      condition(X)
    end).

% Configuration through process dictionary
proper:quickcheck(prop_example(), 
  [{numtests, 1000}, {search_steps, 500}]).
```

### Hedgehog API
```rust
// Basic targeted property
fn prop_example() {
    let generator = Gen::<i32>::from_range(Range::new(0, 100));
    
    let utility_function = |input: &i32, _result: &TargetedResult| -> f64 {
        compute_utility(*input)
    };
    
    let test_function = |input: &i32| -> TargetedResult {
        if property_condition(*input) {
            TargetedResult::Pass { /* ... */ }
        } else {
            TargetedResult::Fail { /* ... */ }
        }
    };
    
    let neighborhood = IntegerNeighborhood::new(10);
    let config = TargetedConfig {
        objective: SearchObjective::Maximize,
        search_steps: 500,
        // ...
    };
    
    let search = for_all_targeted_with_config(
        generator, utility_function, test_function, neighborhood, config
    );
    
    let (result, stats) = search.search(&Config::default());
}
```

## Feature Comparison Matrix

| Feature | PROPER | Hedgehog | Notes |
|---------|---------|----------|--------|
| **Search Strategies** |
| Simulated Annealing | ✅ | ✅ | Both implement SA with temperature scheduling |
| Hill Climbing | ✅ | ➖ | PROPER has built-in; Hedgehog can simulate with high temperature |
| Custom Strategies | ✅ | ✅ | PROPER: behavior modules; Hedgehog: trait implementation |
| **Neighborhood Functions** |
| Auto-generated | ✅ | ➖ | PROPER auto-generates from type structure |
| User-defined | ✅ | ✅ | Both support custom neighborhood functions |
| Built-in Types | ✅ | ✅ | PROPER: comprehensive; Hedgehog: i32, f64, Vec, String |
| **Configuration** |
| Search Steps | ✅ | ✅ | Both configurable |
| Temperature Control | ✅ | ✅ | Both support custom temperature functions |
| Time Limits | ➖ | ✅ | Hedgehog has explicit timeout support |
| Objective Direction | ✅ | ✅ | Both support maximize/minimize |
| **Integration** |
| Type System | ✅ | ✅ | PROPER: runtime hooks; Hedgehog: compile-time traits |
| Shrinking | ✅ | ➖ | PROPER integrates with existing shrinking; Hedgehog uses regular shrinking |
| Examples | ✅ | ✅ | Both provide comprehensive examples |
| **Statistics & Reporting** |
| Search Progress | ➖ | ✅ | Hedgehog provides detailed search statistics |
| Convergence Info | ➖ | ✅ | Hedgehog tracks convergence and acceptance rates |
| Utility History | ➖ | ✅ | Hedgehog maintains utility progression |

## Detailed Feature Analysis

### 1. Search Strategy Implementation

**PROPER:**
```erlang
% Simulated annealing with custom acceptance function
acceptance_function_standard(EnergyCurrent, EnergyNew, Temperature) ->
  case EnergyNew > EnergyCurrent of
    true -> true;  % Always accept better
    false ->
      AcceptanceProbability = math:exp(-(EnergyCurrent - EnergyNew) / Temperature),
      ?RANDOM_PROBABILITY < AcceptanceProbability
  end.
```

**Hedgehog:**
```rust
fn should_accept(&self, current_utility: f64, neighbor_utility: f64, 
                 temperature: f64, rng: &mut dyn RngCore) -> bool {
    if self.is_better_utility(neighbor_utility, current_utility) {
        true // Always accept better solutions
    } else {
        let delta = match self.config.objective {
            SearchObjective::Maximize => neighbor_utility - current_utility,
            SearchObjective::Minimize => current_utility - neighbor_utility,
        };
        let probability = (-delta / temperature).exp();
        rng.gen::<f64>() < probability
    }
}
```

**Analysis:**
- Both implement standard SA acceptance probability
- PROPER uses global random state; Hedgehog uses explicit RNG
- Hedgehog supports both maximize/minimize objectives explicitly

### 2. Neighborhood Function Design

**PROPER:**
```erlang
% Auto-generated neighborhood functions based on type structure
integer_gen_sa(Type) ->
  fun(PrevInstance, Temp) ->
    Base = PrevInstance,
    Delta = trunc(?TEMP(Temp) * 100),
    integer(Base - Delta, Base + Delta)
  end.

% User-defined neighborhood function  
custom_list_nf() ->
  fun(Prev, _T) ->
    case Prev of
      [] -> [integer()];
      _ -> 
        Max = lists:max(Prev),
        vector(length(Prev) * 2, integer(Max, inf))
    end
  end.
```

**Hedgehog:**
```rust
impl NeighborhoodFunction<i32> for IntegerNeighborhood {
    fn neighbor(&self, input: &i32, temperature: f64, rng: &mut dyn RngCore) -> Option<i32> {
        let temp_factor = (temperature / 100.0).min(1.0).max(0.01);
        let max_delta = ((self.max_change as f64) * temp_factor) as i32;
        let max_delta = max_delta.max(1);
        
        let delta = rng.gen_range(-max_delta..=max_delta);
        Some(input.saturating_add(delta))
    }
}

// User-defined neighborhood function for vectors
impl<T, F> NeighborhoodFunction<Vec<T>> for VecNeighborhood<T, F>
where T: Clone, F: NeighborhoodFunction<T> {
    fn neighbor(&self, input: &Vec<T>, temperature: f64, rng: &mut dyn RngCore) -> Option<Vec<T>> {
        // Modifies random elements using element neighborhood function
        // ...
    }
}
```

**Analysis:**
- PROPER auto-generates neighborhood functions from type structure (major advantage)
- Hedgehog requires explicit neighborhood function implementations
- PROPER's approach is more general but less controllable
- Hedgehog's approach is more explicit and type-safe

### 3. State Management

**PROPER:**
```erlang
% Gen server state record
-record(sa_data,
        {k_max = 0,
         k_current = 0,
         p = fun (_, _, _) -> false end,
         last_energy = null,
         last_update = 0,
         temperature = 1.0,
         temp_func = ?TEMP_FUN}).

% State updates through gen_server calls
update_fitness(Fitness, Target, Data) ->
  case P(Energy, Fitness, Temperature) of
    true ->
      proper_gen_next:update_caches(accept),
      {NewTarget, NewData};
    false ->
      proper_gen_next:update_caches(reject),
      {Target, UpdatedData}
  end.
```

**Hedgehog:**
```rust
pub struct SearchStats {
    pub evaluations: usize,
    pub accepted_moves: usize,
    pub best_utility: f64,
    pub final_temperature: f64,
    pub search_time: Duration,
    pub utility_history: Vec<f64>,
    pub converged: bool,
}

// Direct state management in search method
let mut temperature = self.config.initial_temperature;
let mut step = 0;

while step < self.config.search_steps && temperature > self.config.min_temperature {
    // Generate and evaluate neighbor
    stats.evaluations += 1;
    
    if self.should_accept(current_utility, neighbor_utility, temperature, &mut rng) {
        stats.accepted_moves += 1;
        // Accept new state
    }
    
    temperature *= self.config.cooling_rate;
    step += 1;
}
```

**Analysis:**
- PROPER uses process-based state management (distributed system friendly)
- Hedgehog uses direct struct-based state (better for single-process scenarios)  
- Hedgehog provides much richer statistics and progress tracking
- PROPER's gen_server approach enables better concurrency

### 4. Integration with Property Testing Framework

**PROPER:**
```erlang
% Macro expands to proper:targeted call
?FORALL_TARGETED(X, Type, Property)
% ↓
proper:targeted(Type, fun(X) -> Property end)

% Uses existing PROPER infrastructure
-record(prop, {params, body, env, imports, whenfail}).
```

**Hedgehog:**
```rust  
// Separate API that returns results compatible with main framework
pub fn search(&self, test_config: &Config) -> (TargetedResult, SearchStats) {
    // ... search implementation
    (best_result, stats)
}

// Can be integrated with regular properties
match search.search(&Config::default()) {
    (TargetedResult::Fail { counterexample, .. }, _) => {
        println!("Found counterexample: {}", counterexample);
    }
    (TargetedResult::Pass { .. }, stats) => {
        println!("Search completed, best utility: {}", stats.best_utility);
    }
    _ => {}
}
```

**Analysis:**
- PROPER has deeper integration with existing property testing infrastructure
- Hedgehog has more explicit separation, which provides clearer interfaces
- PROPER can leverage existing features like shrinking more easily
- Hedgehog provides more detailed result types and statistics

## Strengths and Weaknesses

### PROPER Strengths
1. **Auto-generated Neighborhoods**: Automatically creates neighborhood functions from type structure
2. **Deep Integration**: Seamlessly integrates with existing PROPER features
3. **Mature Implementation**: Production-ready with extensive real-world usage
4. **Flexible Architecture**: Gen server architecture enables distributed scenarios
5. **Comprehensive Type Support**: Works with all PROPER types automatically

### PROPER Weaknesses  
1. **Limited Statistics**: Minimal search progress information
2. **Complex Architecture**: Gen server + macro system increases complexity
3. **Runtime Overhead**: Process communication and macro expansion overhead
4. **Less Explicit Control**: Auto-generation can be harder to fine-tune

### Hedgehog Strengths
1. **Rich Statistics**: Detailed search progress, convergence info, utility history
2. **Explicit Design**: Clear separation of concerns and explicit APIs
3. **Type Safety**: Compile-time guarantees and trait-based design
4. **Performance**: Direct function calls without process overhead
5. **Flexible Configuration**: Comprehensive configuration options with time limits

### Hedgehog Weaknesses
1. **Manual Neighborhoods**: Requires explicit neighborhood function implementations  
2. **Limited Type Coverage**: Only supports basic types (i32, f64, Vec, String)
3. **No Shrinking Integration**: Doesn't leverage Hedgehog's shrinking system for targeted results
4. **Newer Implementation**: Less battle-tested than PROPER

## Recommendations for Improvement

### For Hedgehog
1. **Auto-generated Neighborhoods**: Implement derive macros for automatic neighborhood function generation
2. **Extended Type Support**: Add neighborhood functions for more complex types (structs, enums)
3. **Shrinking Integration**: Integrate targeted results with Hedgehog's shrinking system
4. **Performance Benchmarks**: Add benchmarking suite comparing with random generation

### Implementation Suggestions

```rust
// Auto-generated neighborhood functions using derive macros
#[derive(Generate, NeighborhoodFunction)]
struct User {
    #[neighborhood(range = 10)]
    age: u32,
    #[neighborhood(alphabet = "abcdefghijklmnopqrstuvwxyz")]
    name: String,
    active: bool,
}

// Integrated shrinking for targeted results
impl TargetedResult {
    pub fn shrink(&self, shrinker: impl Shrink<T>) -> Vec<TargetedResult> {
        // Integrate with existing shrinking system
        // while preserving utility information
    }
}
```

## Conclusion

Both implementations are well-designed but serve different purposes:

- **PROPER** excels in automatic neighborhood generation and deep framework integration, making it easier to apply to existing codebases with minimal changes.

- **Hedgehog** excels in explicit control, rich statistics, and type safety, making it better for scenarios requiring detailed search analysis and fine-grained control.

The choice between them depends on:
- **Use PROPER** when you want automatic neighborhood generation and seamless integration with existing property tests
- **Use Hedgehog** when you need detailed search statistics, explicit control, and compile-time safety guarantees

Our Hedgehog implementation successfully captures the core concepts from PROPER while adapting them to Rust's type system and providing enhanced observability into the search process.