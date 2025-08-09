//! Core functionality for Hedgehog property-based testing.
//!
//! This crate provides the fundamental building blocks for property-based testing
//! with Hedgehog, including generators, properties, and shrinking.

pub mod data;
pub mod error;
pub mod gen;
pub mod parallel;
pub mod property;
pub mod state;
pub mod targeted;
pub mod tree;

// Re-export the main types
pub use data::*;
pub use error::*;
pub use gen::*;
pub use parallel::*;
pub use property::*;
pub use state::*;
pub use targeted::*;
pub use tree::*;
