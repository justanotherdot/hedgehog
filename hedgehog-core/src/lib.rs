//! Core functionality for Hedgehog property-based testing.
//!
//! This crate provides the fundamental building blocks for property-based testing
//! with Hedgehog, including generators, properties, and shrinking.

pub mod data;
pub mod error;
pub mod gen;
pub mod property;
pub mod state;
pub mod tree;

// Re-export the main types
pub use data::*;
pub use error::*;
pub use gen::*;
pub use property::*;
pub use state::*;
pub use tree::*;
