//! Hedgehog property-based testing library.
//!
//! This is the main entry point for the Hedgehog library, providing
//! a convenient API for property-based testing in Rust.
//!
//! # Features
//!
//! - **Explicit generators** - Generators are first-class values you compose
//! - **Integrated shrinking** - Shrinks obey invariants by construction
//! - **Property classification** - Inspect test data distribution and statistics
//! - **Distribution shaping** - Control probability distributions for realistic test data
//! - **Variable name tracking** - Enhanced failure reporting with named inputs
//!
//! # Quick Start
//!
//! ```rust
//! use hedgehog::*;
//!
//! let gen = Gen::<Vec<i32>>::vec_of(Gen::int_range(1, 100));
//! let prop = for_all(gen, |xs: &Vec<i32>| {
//!     let reversed: Vec<_> = xs.iter().rev().cloned().collect();
//!     let double_reversed: Vec<_> = reversed.iter().rev().cloned().collect();
//!     *xs == double_reversed
//! });
//!
//! match prop.run(&Config::default()) {
//!     TestResult::Pass { .. } => (), // Test passed
//!     result => panic!("Property failed: {:?}", result),
//! }
//! ```
//!
//! # Property Classification
//!
//! Inspect the distribution of your test data:
//!
//! ```rust
//! use hedgehog::*;
//!
//! let prop = for_all(Gen::int_range(-10, 10), |&x| x >= -10 && x <= 10)
//!     .classify("negative", |&x| x < 0)
//!     .classify("zero", |&x| x == 0)  
//!     .classify("positive", |&x| x > 0)
//!     .collect("absolute_value", |&x| x.abs() as f64);
//!
//! match prop.run(&Config::default()) {
//!     TestResult::PassWithStatistics { statistics, .. } => {
//!         // Shows distribution percentages and collected statistics
//!     }
//!     _ => {}
//! }
//! ```

pub use hedgehog_core::*;

// Re-export derive macros when available
#[cfg(feature = "derive")]
pub use hedgehog_derive::*;

// Curated test data collections
pub mod corpus;
