//! Property definitions for property-based testing.

use crate::error::ShrinkStep;
use crate::{data::*, error::*, gen::*, tree::*};

/// A property that can be tested with generated inputs.
pub struct Property<T> {
    generator: Gen<T>,
    test_function: Box<dyn Fn(&T) -> TestResult>,
    variable_name: Option<String>,
}

impl<T> Property<T>
where
    T: 'static + std::fmt::Debug,
{
    /// Create a new property from a generator and test function.
    pub fn new<F>(generator: Gen<T>, test_function: F) -> Self
    where
        F: Fn(&T) -> TestResult + 'static,
    {
        Property {
            generator,
            test_function: Box::new(test_function),
            variable_name: None,
        }
    }

    /// Create a property that checks a boolean condition.
    pub fn for_all<F>(generator: Gen<T>, condition: F) -> Self
    where
        F: Fn(&T) -> bool + 'static,
    {
        Property::new(generator, move |input| {
            if condition(input) {
                TestResult::Pass {
                    tests_run: 1,
                    property_name: None,
                    module_path: None,
                }
            } else {
                TestResult::Fail {
                    counterexample: format!("{:?}", input),
                    tests_run: 0,
                    shrinks_performed: 0,
                    property_name: None,
                    module_path: None,
                    assertion_type: Some("Boolean Condition".to_string()),
                    shrink_steps: Vec::new(),
                }
            }
        })
    }

    /// Create a property that checks a boolean condition with a named variable.
    pub fn for_all_named<F>(generator: Gen<T>, variable_name: &str, condition: F) -> Self
    where
        F: Fn(&T) -> bool + 'static,
    {
        let mut property = Property::new(generator, move |input| {
            if condition(input) {
                TestResult::Pass {
                    tests_run: 1,
                    property_name: None,
                    module_path: None,
                }
            } else {
                TestResult::Fail {
                    counterexample: format!("{:?}", input),
                    tests_run: 0,
                    shrinks_performed: 0,
                    property_name: None,
                    module_path: None,
                    assertion_type: Some("Boolean Condition".to_string()),
                    shrink_steps: Vec::new(),
                }
            }
        });
        property.variable_name = Some(variable_name.to_string());
        property
    }

    /// Run this property with the given configuration.
    pub fn run(&self, config: &Config) -> TestResult {
        self.run_with_context(config, None, None)
    }

    /// Run this property with the given configuration and context information.
    pub fn run_with_context(
        &self,
        config: &Config,
        property_name: Option<&str>,
        module_path: Option<&str>,
    ) -> TestResult {
        let mut seed = Seed::random();

        for test_num in 0..config.test_limit {
            let size = Size::new((test_num * config.size_limit) / config.test_limit);
            let (test_seed, next_seed) = seed.split();
            seed = next_seed;

            let tree = self.generator.generate(size, test_seed);

            match self.check_tree(&tree, config) {
                TestResult::Pass { .. } => continue,
                TestResult::Fail {
                    counterexample,
                    shrinks_performed,
                    shrink_steps,
                    assertion_type,
                    ..
                } => {
                    return TestResult::Fail {
                        counterexample,
                        tests_run: test_num + 1,
                        shrinks_performed,
                        property_name: property_name.map(|s| s.to_string()),
                        module_path: module_path.map(|s| s.to_string()),
                        assertion_type,
                        shrink_steps,
                    }
                }
                other => return other,
            }
        }

        TestResult::Pass {
            tests_run: config.test_limit,
            property_name: property_name.map(|s| s.to_string()),
            module_path: module_path.map(|s| s.to_string()),
        }
    }

    /// Check a single tree, attempting to shrink on failure.
    fn check_tree(&self, tree: &Tree<T>, config: &Config) -> TestResult {
        match (self.test_function)(&tree.value) {
            TestResult::Pass { .. } => TestResult::Pass {
                tests_run: 1,
                property_name: None,
                module_path: None,
            },
            TestResult::Fail {
                counterexample,
                tests_run,
                shrinks_performed,
                assertion_type,
                ..
            } => {
                // Try to shrink the failing case
                let (shrunk_counterexample, shrink_steps) = self.shrink_failure(tree, config);

                TestResult::Fail {
                    counterexample: shrunk_counterexample.unwrap_or(counterexample),
                    tests_run,
                    shrinks_performed: shrinks_performed
                        .saturating_add(shrink_steps.len().saturating_sub(1)),
                    property_name: None,
                    module_path: None,
                    assertion_type,
                    shrink_steps,
                }
            }
            other => other,
        }
    }

    /// Attempt to find a smaller failing case through shrinking.
    fn shrink_failure<'a>(
        &self,
        tree: &'a Tree<T>,
        config: &Config,
    ) -> (Option<String>, Vec<ShrinkStep>) {
        let mut shrink_steps = Vec::new();
        let mut current_failure = &tree.value;
        let mut shrink_count = 0;

        // Add the original failing value as step 0
        shrink_steps.push(ShrinkStep {
            counterexample: format!("{:?}", current_failure),
            step: 0,
            variable_name: self.variable_name.clone(),
        });

        // Simple breadth-first shrinking
        for shrink_value in tree.shrinks() {
            if shrink_count >= config.shrink_limit {
                break;
            }

            match (self.test_function)(shrink_value) {
                TestResult::Fail { .. } => {
                    current_failure = shrink_value;
                    shrink_count += 1;

                    // Record this shrinking step
                    shrink_steps.push(ShrinkStep {
                        counterexample: format!("{:?}", shrink_value),
                        step: shrink_count,
                        variable_name: self.variable_name.clone(),
                    });
                }
                TestResult::Pass { .. } => continue,
                TestResult::Discard { .. } => continue,
            }
        }

        if shrink_count > 0 {
            (Some(format!("{:?}", current_failure)), shrink_steps)
        } else {
            (None, shrink_steps)
        }
    }
}

/// Create a property for a generator and test function.
pub fn property<T, F>(generator: Gen<T>, test_function: F) -> Property<T>
where
    T: 'static + std::fmt::Debug,
    F: Fn(&T) -> TestResult + 'static,
{
    Property::new(generator, test_function)
}

/// Create a property that checks a boolean condition.
pub fn for_all<T, F>(generator: Gen<T>, condition: F) -> Property<T>
where
    T: 'static + std::fmt::Debug,
    F: Fn(&T) -> bool + 'static,
{
    Property::for_all(generator, condition)
}

/// Create a property that checks a boolean condition with a named variable.
pub fn for_all_named<T, F>(generator: Gen<T>, variable_name: &str, condition: F) -> Property<T>
where
    T: 'static + std::fmt::Debug,
    F: Fn(&T) -> bool + 'static,
{
    Property::for_all_named(generator, variable_name, condition)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_success() {
        let gen = Gen::bool();
        let prop = for_all(gen, |&b| b == true || b == false);
        let config = Config::default();

        match prop.run(&config) {
            TestResult::Pass { .. } => (),
            other => panic!("Expected success, got: {:?}", other),
        }
    }

    #[test]
    fn test_property_failure() {
        // Use a deterministic approach: test that NOT all integers are positive
        let gen = Gen::int_range(-5, 5);
        let prop = for_all(gen, |&x| x > 0); // Will fail on negative values
        let config = Config::default().with_tests(20);

        match prop.run(&config) {
            TestResult::Fail { .. } => (),
            other => panic!("Expected failure, got: {:?}", other),
        }
    }

    #[test]
    fn test_boolean_generator_reliability() {
        // Test that boolean generator with SplitMix64 produces both true and false
        let gen = Gen::bool();
        let prop = for_all(gen, |&b| b == true); // Will fail on false
        let config = Config::default().with_tests(50);

        match prop.run(&config) {
            TestResult::Fail { .. } => (), // Expected - should find false values
            other => panic!(
                "Boolean generator should produce both true and false, got: {:?}",
                other
            ),
        }
    }

    #[test]
    fn test_variable_name_tracking() {
        // Test that variable names are tracked in shrinking progression
        let prop = for_all_named(Gen::int_range(5, 20), "n", |&n| n < 10);
        let result = prop.run(&Config::default().with_tests(10));

        if let TestResult::Fail { shrink_steps, .. } = result {
            // Check that variable names are present in shrink steps
            assert!(!shrink_steps.is_empty());
            for step in shrink_steps {
                assert_eq!(step.variable_name, Some("n".to_string()));
            }
        } else {
            panic!("Expected a failing test result for variable name tracking");
        }
    }

    #[test]
    fn snapshot_failure_reporting() {
        // Test enhanced failure reporting with shrinking progression
        // Use a deterministic result for consistent testing

        // Create a deterministic result for consistent testing
        let result = TestResult::Fail {
            counterexample: "7".to_string(),
            tests_run: 1,
            shrinks_performed: 3,
            property_name: Some("snapshot_failure_reporting".to_string()),
            module_path: Some("hedgehog_core::property::tests".to_string()),
            assertion_type: Some("Boolean Condition".to_string()),
            shrink_steps: vec![
                ShrinkStep {
                    counterexample: "20".to_string(),
                    step: 0,
                    variable_name: None,
                },
                ShrinkStep {
                    counterexample: "10".to_string(),
                    step: 1,
                    variable_name: None,
                },
                ShrinkStep {
                    counterexample: "5".to_string(),
                    step: 2,
                    variable_name: None,
                },
                ShrinkStep {
                    counterexample: "7".to_string(),
                    step: 3,
                    variable_name: None,
                },
            ],
        };

        // Capture the failure output for regression testing
        let output = format!("{}", result);
        archetype::snap("enhanced_failure_reporting", output);
    }

    #[test]
    fn snapshot_variable_name_reporting() {
        // Test enhanced failure reporting with variable names
        let expected_result = TestResult::Fail {
            counterexample: "7".to_string(),
            tests_run: 1,
            shrinks_performed: 3,
            property_name: Some("snapshot_variable_name_reporting".to_string()),
            module_path: Some("hedgehog_core::property::tests".to_string()),
            assertion_type: Some("Boolean Condition".to_string()),
            shrink_steps: vec![
                ShrinkStep {
                    counterexample: "20".to_string(),
                    step: 0,
                    variable_name: Some("n".to_string()),
                },
                ShrinkStep {
                    counterexample: "10".to_string(),
                    step: 1,
                    variable_name: Some("n".to_string()),
                },
                ShrinkStep {
                    counterexample: "5".to_string(),
                    step: 2,
                    variable_name: Some("n".to_string()),
                },
                ShrinkStep {
                    counterexample: "7".to_string(),
                    step: 3,
                    variable_name: Some("n".to_string()),
                },
            ],
        };

        let formatted_output = format!("{}", expected_result);
        archetype::snap("variable_name_failure_reporting", formatted_output);
    }

    #[test]
    fn snapshot_success_reporting() {
        // Test enhanced success reporting
        let gen = Gen::int_range(1, 50);
        let prop = for_all(gen, |&x| x > 0);
        let config = Config::default().with_tests(20);

        let result = prop.run_with_context(
            &config,
            Some("snapshot_success_reporting"),
            Some("hedgehog_core::property::tests"),
        );

        // Capture the success output for regression testing
        match result {
            TestResult::Pass { .. } => {
                let output = format!("{}", result);
                archetype::snap("enhanced_success_reporting", output);
            }
            other => panic!("Expected success, got: {:?}", other),
        }
    }
}
