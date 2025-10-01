# Regression Corpus

## Overview

A regression corpus is a collection of previously failing test cases that are saved and replayed on subsequent test runs. This provides several benefits:

1. **Faster regression detection** - Known failing cases are tried first
2. **Deterministic replay** - Exact failing inputs can be reproduced
3. **Accumulative testing** - Build up a collection of edge cases over time
4. **CI/CD integration** - Corpus files can be committed to version control

## Inspiration

This approach is used successfully by several property testing libraries:
- **Proptest** (Rust) - Saves failing cases to `proptest-regressions/` directory
- **Hypothesis** (Python) - Maintains a database of interesting examples
- **AFL** (Fuzzing) - Builds corpus of inputs that trigger new code paths

## Proposed Implementation

### File Format

Store failing cases in a human-readable format (JSON or TOML):

```json
{
  "version": "1.0",
  "property": "test_string_parsing",
  "cases": [
    {
      "input": "\"\\u{0000}\"",
      "seed": 12345,
      "timestamp": "2024-01-15T10:30:00Z",
      "shrinks": 15
    },
    {
      "input": "\"\\n\\r\\t\"",
      "seed": 67890,
      "timestamp": "2024-01-16T14:20:00Z",
      "shrinks": 8
    }
  ]
}
```

### Directory Structure

```
project/
├── proptest-regressions/  # Following proptest convention
│   ├── test_string_parsing.json
│   ├── test_number_validation.json
│   └── test_data_structure.json
└── src/
    └── lib.rs
```

### Configuration

```rust
#[test]
fn test_string_parsing() {
    let config = Config::default()
        .with_corpus_file("proptest-regressions/test_string_parsing.json")
        .with_corpus_max_size(100);
    
    let prop = for_all(Gen::<String>::ascii(), |s| {
        parse_string(s).is_ok()
    });
    
    assert!(prop.run(&config).is_pass());
}
```

### Behavior

1. **On test start**: Load corpus file and try all saved cases first
2. **On failure**: Save the minimal failing case to corpus
3. **On success**: Continue with normal random generation
4. **Corpus management**: Automatically prune old/duplicate cases

### Integration Points

- **Config API**: Add corpus-related configuration options
- **Property API**: Modify `run()` to check corpus first
- **Serialization**: Add traits for serializing/deserializing test inputs
- **File management**: Handle corpus file creation, updates, and cleanup

## Implementation Phases

### Phase 1: Basic Corpus Support
- Add corpus file reading/writing
- Implement corpus-first execution
- Basic JSON format support

### Phase 2: Advanced Features
- Corpus pruning and deduplication
- Multiple format support (JSON, TOML, binary)
- Configurable corpus size limits

### Phase 3: Integration
- CI/CD best practices documentation
- IDE integration for corpus management
- Performance optimizations

## Benefits

- **Faster CI builds** - Known regressions caught immediately
- **Better debugging** - Exact failing cases preserved
- **Improved test coverage** - Accumulate edge cases over time
- **Team collaboration** - Share interesting test cases via version control

## Considerations

- **File management** - Corpus files need to be maintained
- **Version control** - Decision on whether to commit corpus files
- **Performance** - Large corpus files could slow down test startup
- **Serialization** - Need robust serialization for complex types

This feature would significantly improve the developer experience by making property test failures more deterministic and easier to reproduce.