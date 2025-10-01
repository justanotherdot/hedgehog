# documentation overview

Welcome to the Hedgehog documentation. This page helps you navigate all available documentation.

## getting started

**New to Hedgehog?** Start here:

1. [Getting Started Guide](guides/getting-started.md) - Installation, first test, core concepts
2. [API Reference](guides/api.md) - Comprehensive API documentation with examples

## user guides

Guides for using Hedgehog in your projects:

- [Getting Started](guides/getting-started.md) - Quick start and basic usage
- [API Reference](guides/api.md) - Complete API documentation
- [Distribution Shaping](guides/distribution-shaping.md) - Control probability distributions
- [Variable Name Tracking](guides/variable-name-tracking.md) - Enhanced failure reporting
- [Property Classification](guides/property-classification.md) - Inspect test data distribution
- [Derive Macros](guides/derive-macros.md) - Automatic generator creation
- [Advanced Features](guides/advanced-features.md) - Parallel testing, state machines, targeted testing
- [Debugging](guides/debugging.md) - Troubleshooting and debugging strategies

## design documents

Understanding Hedgehog's implementation and design decisions:

- [Design Overview](design/design.md) - Architecture and design philosophy
- [API Sketch](design/api-sketch.md) - Original API design concepts
- [Splitting Strategies](design/splitting.md) - How shrinking works
- [Coverage-Guided Generation](design/coverage-guided-generation.md) - Future coverage-guided testing
- **Targeted Testing:**
  - [Comparison](design/targeted-testing-comparison.md) - Comparison with PROPER's approach
  - [Effectiveness Analysis](design/targeted-testing-effectiveness-analysis.md) - Performance analysis
  - [Future Improvements](design/targeted-testing-future-improvements.md) - Planned enhancements

## contributing

Information for contributors:

- [Roadmap](contributing/roadmap.md) - Project status and planned features
- [Implementation Plan](contributing/implementation-plan.md) - Detailed development roadmap
- [Release Process](contributing/release.md) - How to cut a release
- [Ideas](contributing/ideas.md) - Feature ideas from other libraries
- [Trophy Case](contributing/trophy.md) - Bugs found using Hedgehog
- **Testing:**
  - [Meta-Testing Lessons](contributing/meta-testing-lessons.md) - Lessons from testing the library
  - [Parallel Testing Plan](contributing/parallel-testing-plan.md) - Parallel testing implementation
  - [Regression Corpus](contributing/regression-corpus.md) - Regression test suite

## quick links

- **Repository:** https://github.com/hedgehogqa/rust-hedgehog
- **Crate:** https://crates.io/crates/hedgehog
- **API Docs:** https://docs.rs/hedgehog

## document organization

```
docs/
├── overview.md              # This file - navigation hub
├── guides/                  # User-facing documentation
│   ├── getting-started.md
│   ├── api.md
│   └── ...
├── design/                  # Implementation details
│   ├── design.md
│   └── ...
└── contributing/            # Contributor information
    ├── roadmap.md
    ├── release.md
    └── ...
```
