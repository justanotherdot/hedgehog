//! Core functionality for Hedgehog property-based testing.
//!
//! This crate provides the fundamental building blocks for property-based testing
//! with Hedgehog, including generators, properties, and shrinking.

pub mod data;
pub mod error;
pub mod gen;
pub mod property;
pub mod random;
pub mod range;
pub mod shrink;
pub mod tree;

// Re-export the main types
pub use data::*;
pub use error::*;
pub use gen::*;
pub use property::*;
pub use random::*;
pub use range::*;
pub use shrink::*;
pub use tree::*;