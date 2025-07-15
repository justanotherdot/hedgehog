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

/// A shrinking step in the failure progression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShrinkStep {
    /// The counterexample value at this step.
    pub counterexample: String,
    /// The step number (0 = original, 1+ = shrink attempts).
    pub step: usize,
}

/// Outcome of a property test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestResult {
    /// Test passed successfully.
    Pass {
        tests_run: usize,
        property_name: Option<String>,
        module_path: Option<String>,
    },

    /// Test failed with a counterexample.
    Fail {
        counterexample: String,
        tests_run: usize,
        shrinks_performed: usize,
        property_name: Option<String>,
        module_path: Option<String>,
        assertion_type: Option<String>,
        /// The shrinking progression showing how we reached the minimal counterexample.
        shrink_steps: Vec<ShrinkStep>,
    },

    /// Too many test cases were discarded.
    Discard {
        limit: usize,
        property_name: Option<String>,
        module_path: Option<String>,
    },
}

impl fmt::Display for TestResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestResult::Pass {
                tests_run,
                property_name,
                module_path,
            } => {
                // Show module header if available
                if let Some(module) = module_path {
                    writeln!(f, "━━━ {} ━━━", module)?;
                }

                let prop_name = property_name.as_deref().unwrap_or("property");
                write!(f, "  ✓ {} passed {} tests.", prop_name, tests_run)
            }
            TestResult::Fail {
                counterexample,
                tests_run,
                shrinks_performed,
                property_name,
                module_path,
                assertion_type,
                shrink_steps,
            } => {
                // Show module header if available
                if let Some(module) = module_path {
                    writeln!(f, "━━━ {} ━━━", module)?;
                }

                let prop_name = property_name.as_deref().unwrap_or("property");
                writeln!(
                    f,
                    "  ✗ {} failed after {} tests and {} shrinks.",
                    prop_name, tests_run, shrinks_performed
                )?;

                if !shrink_steps.is_empty() {
                    writeln!(f)?;
                    writeln!(f, "    Shrinking progression:")?;
                    for step in shrink_steps {
                        if step.step == 0 {
                            writeln!(f, "      │ Original: {}", step.counterexample)?;
                        } else {
                            writeln!(f, "      │ Step {}: {}", step.step, step.counterexample)?;
                        }
                    }
                    writeln!(f)?;
                }

                // Show assertion type if available
                if let Some(assertion) = assertion_type {
                    writeln!(f, "    === {} ===", assertion)?;
                }

                write!(f, "    Minimal counterexample: {}", counterexample)
            }
            TestResult::Discard {
                limit,
                property_name,
                module_path,
            } => {
                // Show module header if available
                if let Some(module) = module_path {
                    writeln!(f, "━━━ {} ━━━", module)?;
                }

                let prop_name = property_name.as_deref().unwrap_or("property");
                write!(f, "  ⚐ {} gave up after {} discards", prop_name, limit)
            }
        }
    }
}

impl From<HedgehogError> for TestResult {
    fn from(error: HedgehogError) -> Self {
        match error {
            HedgehogError::PropertyFailed {
                counterexample,
                tests_run,
                shrinks_performed,
            } => TestResult::Fail {
                counterexample,
                tests_run,
                shrinks_performed,
                property_name: None,
                module_path: None,
                assertion_type: None,
                shrink_steps: Vec::new(),
            },
            HedgehogError::TooManyDiscards { limit } => TestResult::Discard {
                limit,
                property_name: None,
                module_path: None,
            },
            _ => TestResult::Fail {
                counterexample: error.to_string(),
                tests_run: 0,
                shrinks_performed: 0,
                property_name: None,
                module_path: None,
                assertion_type: None,
                shrink_steps: Vec::new(),
            },
        }
    }
}
