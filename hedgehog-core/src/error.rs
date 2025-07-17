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

    /// Invalid generator construction.
    #[error("Invalid generator: {message}")]
    InvalidGenerator { message: String },
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
    /// Optional variable name for this input (e.g., "xs", "n", "input").
    pub variable_name: Option<String>,
}

/// Outcome of a property test.
#[derive(Debug, Clone, PartialEq)]
pub enum TestResult {
    /// Test passed successfully.
    Pass {
        tests_run: usize,
        property_name: Option<String>,
        module_path: Option<String>,
    },

    /// Test passed successfully with statistics.
    PassWithStatistics {
        tests_run: usize,
        property_name: Option<String>,
        module_path: Option<String>,
        statistics: crate::property::TestStatistics,
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
            TestResult::PassWithStatistics {
                tests_run,
                property_name,
                module_path,
                statistics,
            } => {
                // Show module header if available
                if let Some(module) = module_path {
                    writeln!(f, "━━━ {} ━━━", module)?;
                }

                let prop_name = property_name.as_deref().unwrap_or("property");
                writeln!(f, "  ✓ {} passed {} tests.", prop_name, tests_run)?;

                // Show classification distribution
                if !statistics.classifications.is_empty() {
                    writeln!(f)?;
                    writeln!(f, "  Test data distribution:")?;
                    let mut classification_names: Vec<_> =
                        statistics.classifications.keys().collect();
                    classification_names.sort();
                    for name in classification_names {
                        let count = statistics.classifications[name];
                        let percentage = (count as f64 / statistics.total_tests as f64) * 100.0;
                        writeln!(f, "    {:>3.0}% {}", percentage, name)?;
                    }
                }

                // Show collection statistics
                if !statistics.collections.is_empty() {
                    writeln!(f)?;
                    writeln!(f, "  Test data statistics:")?;
                    let mut collection_names: Vec<_> = statistics.collections.keys().collect();
                    collection_names.sort();
                    for name in collection_names {
                        let values = &statistics.collections[name];
                        if !values.is_empty() {
                            // Filter out NaN and infinite values for robust statistics
                            let finite_values: Vec<f64> =
                                values.iter().copied().filter(|v| v.is_finite()).collect();

                            if !finite_values.is_empty() {
                                let min =
                                    finite_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                                let max = finite_values
                                    .iter()
                                    .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                                let avg =
                                    finite_values.iter().sum::<f64>() / finite_values.len() as f64;

                                let mut sorted = finite_values.clone();
                                sorted.sort_by(|a, b| {
                                    a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
                                });
                                let median = if sorted.len() % 2 == 0 {
                                    (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
                                } else {
                                    sorted[sorted.len() / 2]
                                };

                                writeln!(
                                    f,
                                    "    {}: min={:.1}, max={:.1}, avg={:.1}, median={:.1}",
                                    name, min, max, avg, median
                                )?;
                            }
                        }
                    }
                }

                Ok(())
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
                            if let Some(ref var_name) = step.variable_name {
                                writeln!(
                                    f,
                                    "      │ forAll 0 = {} -- {}",
                                    step.counterexample, var_name
                                )?;
                            } else {
                                writeln!(f, "      │ Original: {}", step.counterexample)?;
                            }
                        } else {
                            if let Some(ref var_name) = step.variable_name {
                                writeln!(
                                    f,
                                    "      │ forAll {} = {} -- {}",
                                    step.step, step.counterexample, var_name
                                )?;
                            } else {
                                writeln!(f, "      │ Step {}: {}", step.step, step.counterexample)?;
                            }
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
