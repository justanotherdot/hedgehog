//! Hedgehog property-based testing library.
//!
//! This is the main entry point for the Hedgehog library, providing
//! a convenient API for property-based testing in Rust.

pub use hedgehog_core::*;

// Re-export derive macros when available
#[cfg(feature = "derive")]
pub use hedgehog_derive::*;
