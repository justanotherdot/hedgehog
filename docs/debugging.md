# Hedgehog Rust Debugging Design

## The Challenge

Hedgehog's debugging output is **absolutely first-class** and a major part of what makes it so powerful. It provides:

1. **Rich failure reports** with minimal counterexamples
2. **Diff support** showing expected vs actual values
3. **Source location tracking** pinpointing exactly where tests fail
4. **Shrink path visualization** showing the journey to minimal failing case
5. **Annotations and context** for complex test scenarios

This is **not tough to add** but requires careful design to integrate well with Rust's testing ecosystem.

## Current Hedgehog Output Examples

### Haskell/F# Style Output
```
*** Failed! Falsifiable (after 13 tests and 5 shrinks):
'a'
42

This failure can be reproduced by running:
> recheck (Size 10) (Seed 1234567890 9876543210) <property>
```

### Rich Diff Output
```
*** Failed! Falsifiable (after 7 tests and 12 shrinks):
Expected: [1, 2, 3]
Actual:   [1, 2, 4]
          -----^

Source: test.hs:42:5
```

### Annotation Context
```
*** Failed! Falsifiable (after 21 tests and 8 shrinks):
User { name: "Alice", age: 25 }
├─ name: "Alice"
├─ age: 25
└─ invalid: age must be >= 18 && <= 65

Failed at: src/user.rs:156:9
```

## Rust Design Strategy

### Core Types

```rust
#[derive(Debug, Clone)]
pub struct FailureReport {
    pub tests_run: usize,
    pub shrinks_performed: usize,
    pub counterexample: String,
    pub annotations: Vec<Annotation>,
    pub source_location: Option<SourceLocation>,
    pub recheck_info: Option<RecheckInfo>,
}

#[derive(Debug, Clone)]
pub struct Annotation {
    pub label: String,
    pub value: String,
    pub source_location: Option<SourceLocation>,
}

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone)]
pub struct RecheckInfo {
    pub size: Size,
    pub seed: Seed,
    pub shrink_path: Vec<ShrinkStep>,
}
```

### Annotation System

```rust
pub trait Annotate {
    fn annotate(&self, label: &str) -> String;
}

// Automatic implementation for Debug types
impl<T: fmt::Debug> Annotate for T {
    fn annotate(&self, label: &str) -> String {
        format!("{}: {:?}", label, self)
    }
}

// Rich diff support
pub trait Diff {
    fn diff(&self, other: &Self) -> DiffResult;
}

pub struct DiffResult {
    pub expected: String,
    pub actual: String,
    pub diff_lines: Vec<DiffLine>,
}

pub enum DiffLine {
    Same(String),
    Added(String),
    Removed(String),
    Changed { from: String, to: String },
}
```

### Property Context Building

```rust
pub struct PropertyContext {
    annotations: Vec<Annotation>,
    source_location: Option<SourceLocation>,
}

impl PropertyContext {
    pub fn annotate<T: Annotate>(&mut self, label: &str, value: &T) {
        self.annotations.push(Annotation {
            label: label.to_string(),
            value: value.annotate(label),
            source_location: Some(source_location!()),
        });
    }
    
    pub fn assert_eq<T: PartialEq + Diff>(&mut self, left: &T, right: &T) -> bool {
        if left == right {
            true
        } else {
            let diff = left.diff(right);
            self.annotations.push(Annotation {
                label: "assertion_failed".to_string(),
                value: format!("Expected: {}\nActual: {}", diff.expected, diff.actual),
                source_location: Some(source_location!()),
            });
            false
        }
    }
}
```

### Source Location Tracking

```rust
// Macro for capturing source location
macro_rules! source_location {
    () => {
        SourceLocation {
            file: file!().to_string(),
            line: line!(),
            column: column!(),
        }
    };
}

// Property macro with source tracking
macro_rules! property {
    ($gen:expr, $test:expr) => {{
        let mut ctx = PropertyContext::new();
        ctx.source_location = Some(source_location!());
        
        check_with_context($gen, ctx, $test)
    }};
}
```

### Rich Reporting

```rust
impl fmt::Display for FailureReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "*** Failed! Falsifiable (after {} tests and {} shrinks):", 
                 self.tests_run, self.shrinks_performed)?;
        
        // Show counterexample
        writeln!(f, "{}", self.counterexample)?;
        
        // Show annotations
        for annotation in &self.annotations {
            writeln!(f, "├─ {}", annotation.value)?;
        }
        
        // Show source location
        if let Some(ref loc) = self.source_location {
            writeln!(f, "└─ Failed at: {}:{}:{}", loc.file, loc.line, loc.column)?;
        }
        
        // Show recheck info
        if let Some(ref recheck) = self.recheck_info {
            writeln!(f, "\nThis failure can be reproduced by running:")?;
            writeln!(f, "> recheck {:?} {:?} <property>", recheck.size, recheck.seed)?;
        }
        
        Ok(())
    }
}
```

### Integration with Rust Testing

#### Standard Test Integration

```rust
#[test]
fn test_reverse_property() {
    let gen = Gen::range(1..=100).vec(0..=20);
    
    let result = property!(gen, |xs| {
        let mut ctx = PropertyContext::new();
        ctx.annotate("input", &xs);
        
        let reversed: Vec<_> = xs.iter().rev().cloned().collect();
        let double_reversed: Vec<_> = reversed.iter().rev().cloned().collect();
        
        ctx.annotate("reversed", &reversed);
        ctx.annotate("double_reversed", &double_reversed);
        
        ctx.assert_eq(&xs, &double_reversed)
    });
    
    match result {
        TestResult::Pass => {},
        TestResult::Fail(report) => {
            // Rich output integrates with Rust's test framework
            panic!("\n{}", report);
        },
        TestResult::Discard => panic!("Too many discards"),
    }
}
```

#### Output in `cargo test`

```
---- test_reverse_property stdout ----
*** Failed! Falsifiable (after 13 tests and 5 shrinks):
[1, 2]
├─ input: [1, 2]
├─ reversed: [2, 1]
├─ double_reversed: [1, 2]
└─ Failed at: src/lib.rs:42:9

This failure can be reproduced by running:
> recheck Size(10) Seed(1234567890, 9876543210) <property>

thread 'test_reverse_property' panicked at 'Property test failed'
```

#### Custom Test Runner

```rust
pub fn run_property_tests() {
    let mut results = Vec::new();
    
    // Collect all property tests
    results.push(("reverse_property", test_reverse_property()));
    results.push(("sort_property", test_sort_property()));
    
    // Custom reporting
    for (name, result) in results {
        match result {
            TestResult::Pass => println!("✓ {}", name),
            TestResult::Fail(report) => {
                println!("✗ {}", name);
                println!("{}", report);
            },
            TestResult::Discard => println!("? {} (too many discards)", name),
        }
    }
}
```

### Advanced Features

#### Diff Visualization

```rust
impl Diff for Vec<i32> {
    fn diff(&self, other: &Self) -> DiffResult {
        let expected = format!("{:?}", self);
        let actual = format!("{:?}", other);
        
        // Generate character-level diff
        let diff_lines = generate_diff_lines(&expected, &actual);
        
        DiffResult { expected, actual, diff_lines }
    }
}

fn generate_diff_lines(expected: &str, actual: &str) -> Vec<DiffLine> {
    // Use a diffing library like `similar` or implement basic diff
    // to show exactly what changed
    vec![
        DiffLine::Same("[1, 2, ".to_string()),
        DiffLine::Removed("3".to_string()),
        DiffLine::Added("4".to_string()),
        DiffLine::Same("]".to_string()),
    ]
}
```

#### Shrink Path Visualization

```rust
impl fmt::Display for ShrinkPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Shrink path:")?;
        for (i, step) in self.steps.iter().enumerate() {
            writeln!(f, "  {}: {}", i, step)?;
        }
        Ok(())
    }
}
```

#### Property Composition

```rust
pub fn compose_properties<T>(
    gen: Gen<T>,
    properties: Vec<(&str, Box<dyn Fn(&T, &mut PropertyContext) -> bool>)>
) -> TestResult {
    // Run multiple properties and collect all failures
    let mut all_failures = Vec::new();
    
    for (name, property) in properties {
        let mut ctx = PropertyContext::new();
        if !property(&value, &mut ctx) {
            all_failures.push((name, ctx));
        }
    }
    
    // Rich reporting for multiple property failures
    if !all_failures.is_empty() {
        TestResult::Fail(FailureReport::from_multiple_failures(all_failures))
    } else {
        TestResult::Pass
    }
}
```

## Integration Strategy

### Phase 1: Basic Reporting
- Implement basic failure reporting with counterexamples
- Source location tracking via macros
- Simple annotation system

### Phase 2: Rich Diffs
- Implement diff visualization for common types
- Add support for custom diff implementations
- Integrate with existing assertion libraries

### Phase 3: Advanced Features
- Shrink path visualization
- Property composition
- Custom test runners
- IDE integration

### Phase 4: Ecosystem Integration
- Integration with `cargo test`
- Support for test frameworks like `rstest`
- Benchmark integration
- CI/CD friendly output

## Key Design Principles

1. **Zero-cost when passing** - No performance overhead for successful tests
2. **Rich failure information** - Detailed debugging info when tests fail
3. **Source location tracking** - Always know where failures occurred
4. **Rust-idiomatic** - Feels natural in Rust's testing ecosystem
5. **Extensible** - Easy to add custom annotations and diff implementations

This design preserves Hedgehog's excellent debugging experience while feeling natural in Rust's testing ecosystem. The key is building the annotation and context system into the core property testing framework, not as an afterthought.