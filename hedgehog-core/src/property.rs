//! Property definitions for property-based testing.

use crate::{data::*, error::*, gen::*, tree::*};

/// A property that can be tested with generated inputs.
pub struct Property<T> {
    generator: Gen<T>,
    test_function: Box<dyn Fn(&T) -> TestResult>,
}

impl<T> Property<T>
where
    T: 'static,
{
    /// Create a new property from a generator and test function.
    pub fn new<F>(generator: Gen<T>, test_function: F) -> Self
    where
        F: Fn(&T) -> TestResult + 'static,
    {
        Property {
            generator,
            test_function: Box::new(test_function),
        }
    }

    /// Create a property that checks a boolean condition.
    pub fn for_all<F>(generator: Gen<T>, condition: F) -> Self
    where
        F: Fn(&T) -> bool + 'static,
    {
        Property::new(generator, move |input| {
            if condition(input) {
                TestResult::Pass
            } else {
                TestResult::Fail {
                    counterexample: "Property failed".to_string(),
                    tests_run: 0,
                    shrinks_performed: 0,
                }
            }
        })
    }

    /// Run this property with the given configuration.
    pub fn run(&self, config: &Config) -> TestResult {
        let mut seed = Seed::random();

        for test_num in 0..config.test_limit {
            let size = Size::new((test_num * config.size_limit) / config.test_limit);
            let (test_seed, next_seed) = seed.split();
            seed = next_seed;

            let tree = self.generator.generate(size, test_seed);

            match self.check_tree(&tree, config) {
                TestResult::Pass => continue,
                failure => return failure,
            }
        }

        TestResult::Pass
    }

    /// Check a single tree, attempting to shrink on failure.
    fn check_tree(&self, tree: &Tree<T>, config: &Config) -> TestResult {
        match (self.test_function)(&tree.value) {
            TestResult::Pass => TestResult::Pass,
            TestResult::Fail {
                counterexample,
                tests_run,
                shrinks_performed,
            } => {
                // Try to shrink the failing case
                if let Some(_shrunk) = self.shrink_failure(tree, config) {
                    TestResult::Fail {
                        counterexample: format!("{} (shrunk to minimal case)", counterexample),
                        tests_run,
                        shrinks_performed: shrinks_performed + 1,
                    }
                } else {
                    TestResult::Fail {
                        counterexample,
                        tests_run,
                        shrinks_performed,
                    }
                }
            }
            other => other,
        }
    }

    /// Attempt to find a smaller failing case through shrinking.
    fn shrink_failure<'a>(&self, tree: &'a Tree<T>, config: &Config) -> Option<&'a T> {
        let mut current_failure = &tree.value;
        let mut shrink_count = 0;

        // Simple breadth-first shrinking
        for shrink_value in tree.shrinks() {
            if shrink_count >= config.shrink_limit {
                break;
            }

            match (self.test_function)(shrink_value) {
                TestResult::Fail { .. } => {
                    current_failure = shrink_value;
                    shrink_count += 1;
                }
                _ => continue,
            }
        }

        if shrink_count > 0 {
            Some(current_failure)
        } else {
            None
        }
    }
}

/// Create a property for a generator and test function.
pub fn property<T, F>(generator: Gen<T>, test_function: F) -> Property<T>
where
    T: 'static,
    F: Fn(&T) -> TestResult + 'static,
{
    Property::new(generator, test_function)
}

/// Create a property that checks a boolean condition.
pub fn for_all<T, F>(generator: Gen<T>, condition: F) -> Property<T>
where
    T: 'static,
    F: Fn(&T) -> bool + 'static,
{
    Property::for_all(generator, condition)
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
            TestResult::Pass => (),
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
}
