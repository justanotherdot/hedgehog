# Coverage-Guided Generation Design

This document explores the design space for implementing coverage-guided generation in Hedgehog, comparing different approaches and their tradeoffs.

## The Goal

Traditional property testing generates inputs randomly or with simple distributions. Coverage-guided generation uses **feedback from code execution** to bias generation toward inputs that explore new code paths.

```rust
// Traditional: Random within bounds
let gen = Gen::string_range(1, 100);  // May never hit edge cases

// Coverage-guided: Feedback-driven exploration
let gen = Gen::string_range(1, 100).with_coverage_guidance(test_fn);
// Evolves toward inputs that discover new code paths
```

## Approach 1: Statistical Fitness (Recommended MVP)

Use user-provided scoring functions to guide generation without compiler instrumentation.

### Implementation
```rust
pub trait FitnessFunction<T> {
    fn fitness(&self, input: &T) -> f64;
}

impl<T> Gen<T> {
    pub fn with_fitness<F>(self, fitness_fn: F) -> FitnessGen<T, F>
    where F: FitnessFunction<T>
    {
        // Evolutionary algorithm:
        // 1. Generate candidates
        // 2. Score each with fitness function  
        // 3. Keep high-scoring inputs for mutation
        // 4. Bias future generation toward successful patterns
    }
}

// Usage:
struct ParserFitness;
impl FitnessFunction<String> for ParserFitness {
    fn fitness(&self, input: &String) -> f64 {
        match parse(input) {
            Ok(ast) => ast.complexity_score(),      // Deeper parsing = higher score
            Err(ParseError::Timeout) => 100.0,     // Timeouts are very interesting!
            Err(ParseError::InvalidSyntax) => 10.0, // Syntax errors moderately interesting
            Err(ParseError::Empty) => 1.0,         // Empty input is boring
        }
    }
}

let gen = Gen::string().with_fitness(ParserFitness);
```

### Advantages
- ✅ **Zero instrumentation overhead**
- ✅ **Works with any existing code**
- ✅ **User controls what "interesting" means**
- ✅ **Can implement in pure Rust**
- ✅ **No compiler integration needed**

### Disadvantages  
- ❌ **Less precise than true coverage**
- ❌ **Requires user to design fitness function**
- ❌ **May miss subtle coverage patterns**

### Instrumentation Required
**None!** Users just provide a scoring function.

## Approach 2: LLVM-Based Coverage (Future Enhancement)

Use LLVM's built-in coverage instrumentation for precise code path tracking.

### How LLVM Coverage Works
```rust
// Original code:
fn parse_function(input: &str) -> Result<Function, Error> {
    if input.starts_with("fn") {
        let name = extract_name(input)?;
        if name.is_empty() {
            return Err(Error::EmptyName);
        }
        Ok(Function { name })
    } else {
        Err(Error::NotAFunction)
    }
}

// After LLVM instrumentation:
fn parse_function(input: &str) -> Result<Function, Error> {
    __coverage_trace_pc_guard(guard_1);  // Function entry
    if input.starts_with("fn") {
        __coverage_trace_pc_guard(guard_2);  // True branch
        let name = extract_name(input)?;
        __coverage_trace_pc_guard(guard_3);  // After call
        if name.is_empty() {
            __coverage_trace_pc_guard(guard_4);  // Empty name branch
            return Err(Error::EmptyName);
        }
        __coverage_trace_pc_guard(guard_5);  // Non-empty name
        Ok(Function { name })
    } else {
        __coverage_trace_pc_guard(guard_6);  // False branch
        Err(Error::NotAFunction)
    }
}
```

### Implementation Strategy
```rust
// Build with coverage instrumentation
// RUSTFLAGS="-C instrument-coverage" cargo build

pub struct CoverageGen<T, F> {
    base_gen: Gen<T>,
    test_fn: F,
    coverage_map: HashMap<u32, u32>,  // guard_id -> hit_count
    interesting_inputs: Vec<T>,
}

impl<T, F> CoverageGen<T, F> 
where 
    T: Clone + 'static,
    F: Fn(&T) -> TestResult,
{
    pub fn generate(&mut self, size: Size, seed: Seed) -> Tree<T> {
        let mut best_input = None;
        let mut best_new_coverage = 0;
        
        // Try multiple candidates
        for _ in 0..10 {
            let candidate = if self.interesting_inputs.is_empty() {
                // Cold start: random generation
                self.base_gen.generate(size, seed).value
            } else {
                // Hot start: mutate interesting inputs 80% of time
                if should_mutate(seed) {
                    self.mutate_interesting_input(seed)
                } else {
                    self.base_gen.generate(size, seed).value  
                }
            };
            
            // Clear coverage counters
            unsafe { __coverage_clear(); }
            
            // Run test with coverage tracking
            let _result = (self.test_fn)(&candidate);
            
            // Collect coverage data
            let new_coverage = self.count_new_coverage();
            
            if new_coverage > best_new_coverage {
                best_input = Some(candidate);
                best_new_coverage = new_coverage;
            }
        }
        
        // Update knowledge base
        if let Some(input) = &best_input {
            if best_new_coverage > 0 {
                self.interesting_inputs.push(input.clone());
                self.update_coverage_map();
            }
        }
        
        Tree::singleton(best_input.unwrap_or_else(|| {
            self.base_gen.generate(size, seed).value
        }))
    }
}
```

### Advantages
- ✅ **Precise coverage tracking**
- ✅ **Automatic instrumentation**  
- ✅ **No user effort for instrumentation**
- ✅ **Industry-proven approach** (AFL, libFuzzer)

### Disadvantages
- ❌ **2-10x performance overhead during instrumented runs**
- ❌ **Requires special build configuration**
- ❌ **More complex implementation**
- ❌ **Only works with coverage-enabled builds**

### Instrumentation Required
**Massive but automatic!** Every basic block gets instrumented by the compiler, but users don't write any instrumentation code.

## Approach 3: Hybrid Strategy

Combine both approaches for maximum flexibility:

```rust
impl<T> Gen<T> {
    // Phase 1: Statistical guidance (always available)
    pub fn with_fitness<F>(self, fitness_fn: F) -> FitnessGen<T, F> { }
    
    // Phase 2: Automatic coverage (when available)  
    #[cfg(feature = "llvm-coverage")]
    pub fn with_coverage<F>(self, test_fn: F) -> CoverageGen<T, F> { }
    
    // Phase 3: User choice
    pub fn guided<F>(self, test_fn: F) -> GuidedGen<T, F> {
        #[cfg(feature = "llvm-coverage")]
        return self.with_coverage(test_fn);
        
        #[cfg(not(feature = "llvm-coverage"))]
        return self.with_fitness(DefaultFitness::new(test_fn));
    }
}
```

## Comparison Summary

| Approach | Instrumentation | Performance | Precision | User Effort | Implementation |
|----------|----------------|-------------|-----------|-------------|----------------|
| **Statistical** | None | No overhead | Medium | Medium | Simple |
| **LLVM Coverage** | Automatic | 2-10x slower | Perfect | Low | Complex |
| **Hybrid** | Optional | Configurable | Variable | Low | Medium |

## Recommended Implementation Plan

### Phase 1: Statistical MVP (1-2 weeks)
- Implement fitness-based generation
- Provide common fitness function patterns
- Zero instrumentation overhead
- Works with all existing code

### Phase 2: LLVM Integration (2-4 weeks)  
- Add LLVM coverage support behind feature flag
- Automatic instrumentation when enabled
- Fallback to statistical when not available

### Phase 3: Polish & Optimization (1-2 weeks)
- Performance tuning
- Better mutation strategies  
- Documentation and examples
- Integration with existing property testing

## Open Questions

1. **API Design**: How does coverage-guided generation integrate with existing `Gen<T>` API?

2. **Build Integration**: How do users enable coverage instrumentation? Cargo feature? Environment variable?

3. **Performance**: Is the overhead acceptable for CI/testing use cases?

4. **Scope**: Should this be core Hedgehog feature or separate `hedgehog-coverage` crate?

5. **Fallback Strategy**: What happens when coverage instrumentation fails or isn't available?

## Conclusion

Coverage-guided generation is technically feasible in Rust and would significantly enhance Hedgehog's bug-finding capabilities. The statistical approach provides immediate value with zero overhead, while LLVM integration offers industry-standard precision for advanced users.

Starting with the statistical approach allows us to validate the concept and API design before investing in the more complex LLVM integration.