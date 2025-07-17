//! Property definitions for property-based testing.

use crate::error::ShrinkStep;
use crate::{data::*, error::*, gen::*, tree::*};
use std::collections::HashMap;

/// Statistics gathered during property testing.
#[derive(Debug, Clone, PartialEq)]
pub struct TestStatistics {
    pub classifications: HashMap<String, usize>,
    pub collections: HashMap<String, Vec<f64>>,
    pub total_tests: usize,
}

impl TestStatistics {
    pub fn new() -> Self {
        TestStatistics {
            classifications: HashMap::new(),
            collections: HashMap::new(),
            total_tests: 0,
        }
    }

    pub fn record_classification(&mut self, name: &str) {
        *self.classifications.entry(name.to_string()).or_insert(0) += 1;
    }

    pub fn record_collection(&mut self, name: &str, value: f64) {
        self.collections
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(value);
    }
}

/// A property that can be tested with generated inputs.
pub struct Property<T> {
    generator: Gen<T>,
    test_function: Box<dyn Fn(&T) -> TestResult>,
    variable_name: Option<String>,
    classifications: Vec<(String, Box<dyn Fn(&T) -> bool>)>,
    collections: Vec<(String, Box<dyn Fn(&T) -> f64>)>,
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
            classifications: Vec::new(),
            collections: Vec::new(),
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

    /// Add a classification to categorize test inputs.
    pub fn classify<F>(mut self, name: &str, predicate: F) -> Self
    where
        F: Fn(&T) -> bool + 'static,
    {
        self.classifications
            .push((name.to_string(), Box::new(predicate)));
        self
    }

    /// Add a collection to gather numerical statistics from test inputs.
    pub fn collect<F>(mut self, name: &str, extractor: F) -> Self
    where
        F: Fn(&T) -> f64 + 'static,
    {
        self.collections
            .push((name.to_string(), Box::new(extractor)));
        self
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
        let mut statistics = TestStatistics::new();

        for test_num in 0..config.test_limit {
            let size = Size::new((test_num * config.size_limit) / config.test_limit);
            let (test_seed, next_seed) = seed.split();
            seed = next_seed;

            let tree = self.generator.generate(size, test_seed);

            // Collect statistics from the generated value
            self.collect_statistics(&tree.value, &mut statistics);

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

        statistics.total_tests = config.test_limit;

        // Return PassWithStatistics only if we have classifications or collections
        if !self.classifications.is_empty() || !self.collections.is_empty() {
            TestResult::PassWithStatistics {
                tests_run: config.test_limit,
                property_name: property_name.map(|s| s.to_string()),
                module_path: module_path.map(|s| s.to_string()),
                statistics,
            }
        } else {
            TestResult::Pass {
                tests_run: config.test_limit,
                property_name: property_name.map(|s| s.to_string()),
                module_path: module_path.map(|s| s.to_string()),
            }
        }
    }

    /// Collect statistics from a test input.
    fn collect_statistics(&self, value: &T, statistics: &mut TestStatistics) {
        // Apply all classifications
        for (name, predicate) in &self.classifications {
            if predicate(value) {
                statistics.record_classification(name);
            }
        }

        // Apply all collections
        for (name, extractor) in &self.collections {
            let extracted_value = extractor(value);
            statistics.record_collection(name, extracted_value);
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
                TestResult::PassWithStatistics { .. } => continue,
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

    #[test]
    fn test_property_classification() {
        // Test that property classification works correctly
        let gen = Gen::int_range(-10, 10);
        let prop = for_all(gen, |&x| x >= -10 && x <= 10) // Always passes
            .classify("negative", |&x| x < 0)
            .classify("zero", |&x| x == 0)
            .classify("positive", |&x| x > 0)
            .collect("value", |&x| x as f64);

        let config = Config::default().with_tests(50);
        let result = prop.run(&config);

        match result {
            TestResult::PassWithStatistics { statistics, .. } => {
                // Should have some classifications (but zero might not always appear with only 50 tests)
                assert!(!statistics.classifications.is_empty());

                // Should at least have negative or positive (much more likely than zero)
                assert!(
                    statistics.classifications.contains_key("negative")
                        || statistics.classifications.contains_key("positive")
                );

                // Should have collected values
                assert!(statistics.collections.contains_key("value"));
                let values = &statistics.collections["value"];
                assert_eq!(values.len(), 50);

                // Values should be in range
                for &value in values {
                    assert!(value >= -10.0 && value <= 10.0);
                }

                assert_eq!(statistics.total_tests, 50);
            }
            other => panic!("Expected PassWithStatistics, got: {:?}", other),
        }
    }

    #[test]
    fn test_classification_with_nan_values() {
        // Test that NaN and infinite values are handled gracefully
        let gen = Gen::int_range(1, 5);
        let prop = for_all(gen, |&x| x > 0)
            .collect("problematic", |&x| match x {
                1 => f64::NAN,
                2 => f64::INFINITY,
                3 => f64::NEG_INFINITY,
                _ => x as f64,
            })
            .collect("normal", |&x| x as f64);

        let config = Config::default().with_tests(20);
        let result = prop.run(&config);

        match &result {
            TestResult::PassWithStatistics { statistics, .. } => {
                // Should have both collections
                assert!(statistics.collections.contains_key("problematic"));
                assert!(statistics.collections.contains_key("normal"));

                // The output should format without panicking
                let output = format!("{}", result);
                assert!(output.contains("normal"));
                // Problematic collection might not appear if all values are NaN/infinite
            }
            other => panic!("Expected PassWithStatistics, got: {:?}", other),
        }
    }

    #[test]
    fn snapshot_classification_output() {
        // Test the output formatting for classifications with deterministic result
        let statistics = TestStatistics {
            classifications: {
                let mut map = std::collections::HashMap::new();
                map.insert("small".to_string(), 14);
                map.insert("large".to_string(), 16);
                map
            },
            collections: {
                let mut map = std::collections::HashMap::new();
                map.insert("value".to_string(), vec![1.0, 5.0, 10.0, 15.0, 20.0]);
                map
            },
            total_tests: 30,
        };

        let result = TestResult::PassWithStatistics {
            tests_run: 30,
            property_name: Some("test_classification".to_string()),
            module_path: Some("hedgehog_core::property::tests".to_string()),
            statistics,
        };

        let output = format!("{}", result);
        archetype::snap("classification_output", output);
    }
}
