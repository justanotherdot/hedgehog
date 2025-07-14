//! Error types for Hedgehog property-based testing.

use std::fmt;
use thiserror::Error;

/// Main error type for Hedgehog property testing.
#[derive(Error, Debug)]
pub enum HedgehogError {
    /// Property test failed with a counterexample.
    #[error("Property test failed: {counterexample}")]
    PropertyFailed {
        counterexample: String,
        tests_run: usize,
        shrinks_performed: usize,
    },
    
    /// Too many test cases were discarded.
    #[error("Too many test cases discarded (limit: {limit})")]
    TooManyDiscards { limit: usize },
    
    /// Generator failed to produce a value.
    #[error("Generator failed: {reason}")]
    GeneratorFailed { reason: String },
    
    /// Invalid configuration.
    #[error("Invalid configuration: {message}")]
    InvalidConfig { message: String },
}

/// Result type for Hedgehog operations.
pub type Result<T> = std::result::Result<T, HedgehogError>;

/// Outcome of a property test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestResult {
    /// Test passed successfully.
    Pass,
    
    /// Test failed with a counterexample.
    Fail {
        counterexample: String,
        tests_run: usize,
        shrinks_performed: usize,
    },
    
    /// Too many test cases were discarded.
    Discard { limit: usize },
}

impl fmt::Display for TestResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestResult::Pass => write!(f, "✓ Property test passed"),
            TestResult::Fail { counterexample, tests_run, shrinks_performed } => {
                write!(f, "✗ Property test failed after {} tests and {} shrinks: {}", 
                       tests_run, shrinks_performed, counterexample)
            }
            TestResult::Discard { limit } => {
                write!(f, "? Property test gave up after {} discards", limit)
            }
        }
    }
}

impl From<HedgehogError> for TestResult {
    fn from(error: HedgehogError) -> Self {
        match error {
            HedgehogError::PropertyFailed { counterexample, tests_run, shrinks_performed } => {
                TestResult::Fail { counterexample, tests_run, shrinks_performed }
            }
            HedgehogError::TooManyDiscards { limit } => {
                TestResult::Discard { limit }
            }
            _ => TestResult::Fail { 
                counterexample: error.to_string(),
                tests_run: 0,
                shrinks_performed: 0,
            }
        }
    }
}