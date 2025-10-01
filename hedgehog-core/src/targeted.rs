//! Targeted property-based testing with search-guided generation.
//!
//! This module provides targeted property-based testing capabilities that use
//! search strategies like simulated annealing to guide input generation toward
//! inputs that are more likely to find bugs or explore interesting behaviors.
//!
//! The approach is inspired by the research presented in:
//! "Targeted property-based testing" by Andreas Löscher and Konstantinos Sagonas (ISSTA 2017)
//! Available at: <http://proper.softlab.ntua.gr/papers/issta2017.pdf>

use crate::{data::*, gen::*};
use rand::{Rng, RngCore};
use std::time::{Duration, Instant};

/// Result of a targeted property test that includes utility information.
#[derive(Debug, Clone)]
pub enum TargetedResult {
    /// Property passed with associated utility value
    Pass {
        tests_run: usize,
        property_name: Option<String>,
        module_path: Option<String>,
        utility: f64,
    },
    /// Property failed with counterexample and utility
    Fail {
        counterexample: String,
        tests_run: usize,
        shrinks_performed: usize,
        property_name: Option<String>,
        module_path: Option<String>,
        assertion_type: Option<String>,
        shrink_steps: Vec<String>,
        utility: f64,
    },
    /// Test was discarded
    Discard { tests_run: usize },
}

impl TargetedResult {
    /// Extract the utility value from the result.
    pub fn utility(&self) -> Option<f64> {
        match self {
            TargetedResult::Pass { utility, .. } => Some(*utility),
            TargetedResult::Fail { utility, .. } => Some(*utility),
            TargetedResult::Discard { .. } => None,
        }
    }

    /// Check if the test passed.
    pub fn is_pass(&self) -> bool {
        matches!(self, TargetedResult::Pass { .. })
    }

    /// Check if the test failed.
    pub fn is_fail(&self) -> bool {
        matches!(self, TargetedResult::Fail { .. })
    }
}

/// Search objective for targeted testing.
#[derive(Debug, Clone)]
pub enum SearchObjective {
    /// Maximize the utility function
    Maximize,
    /// Minimize the utility function  
    Minimize,
}

/// Configuration for targeted property testing.
#[derive(Debug, Clone)]
pub struct TargetedConfig {
    /// Search objective (maximize or minimize utility)
    pub objective: SearchObjective,
    /// Number of search steps to perform
    pub search_steps: usize,
    /// Initial temperature for simulated annealing
    pub initial_temperature: f64,
    /// Cooling rate for temperature scheduling
    pub cooling_rate: f64,
    /// Minimum temperature (search stops when reached)
    pub min_temperature: f64,
    /// Number of random samples to try before starting search
    pub initial_samples: usize,
    /// Maximum time to spend on targeted search
    pub max_search_time: Option<Duration>,
}

impl Default for TargetedConfig {
    fn default() -> Self {
        TargetedConfig {
            objective: SearchObjective::Maximize,
            search_steps: 1000,
            initial_temperature: 100.0,
            cooling_rate: 0.95,
            min_temperature: 0.01,
            initial_samples: 100,
            max_search_time: Some(Duration::from_secs(60)),
        }
    }
}

/// Statistics about the search process.
#[derive(Debug, Clone)]
pub struct SearchStats {
    /// Total number of evaluations performed
    pub evaluations: usize,
    /// Number of accepted moves during search
    pub accepted_moves: usize,
    /// Best utility value found
    pub best_utility: f64,
    /// Final temperature when search ended
    pub final_temperature: f64,
    /// Time spent in search phase
    pub search_time: Duration,
    /// Utility values over time (for analysis)
    pub utility_history: Vec<f64>,
    /// Whether search converged before hitting limits
    pub converged: bool,
}

/// A neighborhood function that generates similar inputs for search.
pub trait NeighborhoodFunction<T> {
    /// Generate a neighbor of the given input.
    /// Returns None if no valid neighbor can be generated.
    fn neighbor(&self, input: &T, temperature: f64, rng: &mut dyn RngCore) -> Option<T>;

    /// Get the maximum distance this function can create between inputs.
    /// Used for scaling temperature effects.
    fn max_distance(&self) -> f64 {
        1.0
    }
}

type UtilityFn<T> = Box<dyn Fn(&T, &TargetedResult) -> f64>;
type TestFn<T> = Box<dyn Fn(&T) -> TargetedResult>;

/// Simulated annealing search strategy for targeted testing.
pub struct SimulatedAnnealing<T> {
    /// The generator to use for initial random sampling
    generator: Gen<T>,
    /// Function that computes utility values
    utility_function: UtilityFn<T>,
    /// Function that tests the property
    test_function: TestFn<T>,
    /// Neighborhood function for generating similar inputs
    neighborhood: Box<dyn NeighborhoodFunction<T>>,
    /// Search configuration
    config: TargetedConfig,
}

impl<T> SimulatedAnnealing<T>
where
    T: 'static + std::fmt::Debug + Clone,
{
    /// Create a new simulated annealing search.
    pub fn new<U, F, N>(
        generator: Gen<T>,
        utility_function: U,
        test_function: F,
        neighborhood: N,
        config: TargetedConfig,
    ) -> Self
    where
        U: Fn(&T, &TargetedResult) -> f64 + 'static,
        F: Fn(&T) -> TargetedResult + 'static,
        N: NeighborhoodFunction<T> + 'static,
    {
        SimulatedAnnealing {
            generator,
            utility_function: Box::new(utility_function),
            test_function: Box::new(test_function),
            neighborhood: Box::new(neighborhood),
            config,
        }
    }

    /// Run the targeted search.
    pub fn search(&self, test_config: &Config) -> (TargetedResult, SearchStats) {
        let start_time = Instant::now();
        let mut rng = rand::thread_rng();
        let mut stats = SearchStats {
            evaluations: 0,
            accepted_moves: 0,
            best_utility: f64::NEG_INFINITY,
            final_temperature: self.config.initial_temperature,
            search_time: Duration::from_secs(0),
            utility_history: Vec::new(),
            converged: false,
        };

        // Phase 1: Initial random sampling to find starting point
        let best_input = self.initial_sampling(&mut stats, &mut rng, test_config);
        let mut current_input = best_input.clone();
        let current_result = (self.test_function)(&current_input);
        let mut current_utility = (self.utility_function)(&current_input, &current_result);

        let mut best_result = current_result.clone();
        let mut best_utility = current_utility;

        stats.best_utility = best_utility;
        stats.utility_history.push(current_utility);

        // Phase 2: Simulated annealing search
        let mut temperature = self.config.initial_temperature;
        let mut step = 0;

        while step < self.config.search_steps && temperature > self.config.min_temperature {
            // Check time limit
            if let Some(max_time) = self.config.max_search_time {
                if start_time.elapsed() > max_time {
                    break;
                }
            }

            // Generate neighbor
            if let Some(neighbor) =
                self.neighborhood
                    .neighbor(&current_input, temperature, &mut rng)
            {
                let neighbor_result = (self.test_function)(&neighbor);
                let neighbor_utility = (self.utility_function)(&neighbor, &neighbor_result);

                stats.evaluations += 1;
                stats.utility_history.push(neighbor_utility);

                // Decide whether to accept the neighbor
                if self.should_accept(current_utility, neighbor_utility, temperature, &mut rng) {
                    current_input = neighbor;
                    current_utility = neighbor_utility;
                    stats.accepted_moves += 1;

                    // Update best if this is better
                    if self.is_better_utility(neighbor_utility, best_utility) {
                        best_result = neighbor_result;
                        best_utility = neighbor_utility;
                        stats.best_utility = best_utility;
                    }
                }
            }

            // Cool down temperature
            temperature *= self.config.cooling_rate;
            step += 1;
        }

        stats.final_temperature = temperature;
        stats.search_time = start_time.elapsed();
        stats.converged = temperature <= self.config.min_temperature;

        (best_result, stats)
    }

    /// Perform initial random sampling to find a good starting point.
    fn initial_sampling(
        &self,
        stats: &mut SearchStats,
        _rng: &mut dyn RngCore,
        test_config: &Config,
    ) -> T {
        let mut seed = Seed::random();
        let mut best_input = None;
        let mut best_utility = match self.config.objective {
            SearchObjective::Maximize => f64::NEG_INFINITY,
            SearchObjective::Minimize => f64::INFINITY,
        };

        for i in 0..self.config.initial_samples {
            let size = Size::new((i * test_config.size_limit) / self.config.initial_samples);
            let (sample_seed, next_seed) = seed.split();
            seed = next_seed;

            let tree = self.generator.generate(size, sample_seed);
            let input = tree.value;
            let result = (self.test_function)(&input);
            let utility = (self.utility_function)(&input, &result);

            stats.evaluations += 1;

            if self.is_better_utility(utility, best_utility) {
                best_input = Some(input);
                best_utility = utility;
            }
        }

        best_input.unwrap_or_else(|| {
            // Fallback to generating one more sample
            let tree = self.generator.generate(Size::new(50), Seed::random());
            tree.value
        })
    }

    /// Determine if we should accept a neighbor based on utility and temperature.
    fn should_accept(
        &self,
        current_utility: f64,
        neighbor_utility: f64,
        temperature: f64,
        rng: &mut dyn RngCore,
    ) -> bool {
        if self.is_better_utility(neighbor_utility, current_utility) {
            true // Always accept better solutions
        } else {
            // Accept worse solutions with probability based on temperature
            let delta = match self.config.objective {
                SearchObjective::Maximize => neighbor_utility - current_utility,
                SearchObjective::Minimize => current_utility - neighbor_utility,
            };

            let probability = (-delta / temperature).exp();
            rng.gen::<f64>() < probability
        }
    }

    /// Check if the first utility is better than the second according to objective.
    fn is_better_utility(&self, utility1: f64, utility2: f64) -> bool {
        match self.config.objective {
            SearchObjective::Maximize => utility1 > utility2,
            SearchObjective::Minimize => utility1 < utility2,
        }
    }
}

/// Convenience function to create a targeted property test with simulated annealing.
pub fn for_all_targeted<T, U, F, N>(
    generator: Gen<T>,
    utility_function: U,
    test_function: F,
    neighborhood: N,
) -> SimulatedAnnealing<T>
where
    T: 'static + std::fmt::Debug + Clone,
    U: Fn(&T, &TargetedResult) -> f64 + 'static,
    F: Fn(&T) -> TargetedResult + 'static,
    N: NeighborhoodFunction<T> + 'static,
{
    SimulatedAnnealing::new(
        generator,
        utility_function,
        test_function,
        neighborhood,
        TargetedConfig::default(),
    )
}

/// Convenience function with custom configuration.
pub fn for_all_targeted_with_config<T, U, F, N>(
    generator: Gen<T>,
    utility_function: U,
    test_function: F,
    neighborhood: N,
    config: TargetedConfig,
) -> SimulatedAnnealing<T>
where
    T: 'static + std::fmt::Debug + Clone,
    U: Fn(&T, &TargetedResult) -> f64 + 'static,
    F: Fn(&T) -> TargetedResult + 'static,
    N: NeighborhoodFunction<T> + 'static,
{
    SimulatedAnnealing::new(
        generator,
        utility_function,
        test_function,
        neighborhood,
        config,
    )
}

// ===== Built-in Neighborhood Functions =====

/// Neighborhood function for integers that adds/subtracts small random values.
#[derive(Debug, Clone)]
pub struct IntegerNeighborhood {
    /// Maximum change to apply (scaled by temperature)
    pub max_change: i32,
}

impl IntegerNeighborhood {
    /// Create a new integer neighborhood function.
    pub fn new(max_change: i32) -> Self {
        IntegerNeighborhood { max_change }
    }
}

impl Default for IntegerNeighborhood {
    fn default() -> Self {
        IntegerNeighborhood::new(10)
    }
}

impl NeighborhoodFunction<i32> for IntegerNeighborhood {
    fn neighbor(&self, input: &i32, temperature: f64, rng: &mut dyn RngCore) -> Option<i32> {
        // Scale change amount by temperature (higher temp = bigger changes)
        let temp_factor = (temperature / 100.0).clamp(0.01, 1.0);
        let max_delta = ((self.max_change as f64) * temp_factor) as i32;
        let max_delta = max_delta.max(1);

        let delta = rng.gen_range(-max_delta..=max_delta);
        Some(input.saturating_add(delta))
    }

    fn max_distance(&self) -> f64 {
        self.max_change as f64
    }
}

/// Neighborhood function for floating point numbers.
#[derive(Debug, Clone)]
pub struct FloatNeighborhood {
    /// Maximum change to apply (scaled by temperature)
    pub max_change: f64,
}

impl FloatNeighborhood {
    /// Create a new float neighborhood function.
    pub fn new(max_change: f64) -> Self {
        FloatNeighborhood { max_change }
    }
}

impl Default for FloatNeighborhood {
    fn default() -> Self {
        FloatNeighborhood::new(1.0)
    }
}

impl NeighborhoodFunction<f64> for FloatNeighborhood {
    fn neighbor(&self, input: &f64, temperature: f64, rng: &mut dyn RngCore) -> Option<f64> {
        // Scale change amount by temperature
        let temp_factor = (temperature / 100.0).clamp(0.001, 1.0);
        let max_delta = self.max_change * temp_factor;

        let delta = rng.gen_range(-max_delta..=max_delta);
        Some(input + delta)
    }

    fn max_distance(&self) -> f64 {
        self.max_change
    }
}

/// Neighborhood function for vectors that modifies random elements.
#[derive(Debug, Clone)]
pub struct VecNeighborhood<T, F> {
    /// Function to generate neighbors for individual elements
    element_neighborhood: F,
    /// Probability of modifying each element (0.0 to 1.0)
    modification_probability: f64,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, F> VecNeighborhood<T, F>
where
    T: Clone,
    F: NeighborhoodFunction<T>,
{
    /// Create a new vector neighborhood function.
    pub fn new(element_neighborhood: F, modification_probability: f64) -> Self {
        VecNeighborhood {
            element_neighborhood,
            modification_probability: modification_probability.clamp(0.0, 1.0),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, F> NeighborhoodFunction<Vec<T>> for VecNeighborhood<T, F>
where
    T: Clone,
    F: NeighborhoodFunction<T>,
{
    fn neighbor(&self, input: &Vec<T>, temperature: f64, rng: &mut dyn RngCore) -> Option<Vec<T>> {
        if input.is_empty() {
            return None;
        }

        let mut result = input.clone();
        let mut modified = false;

        for element in result.iter_mut() {
            if rng.gen::<f64>() < self.modification_probability {
                if let Some(new_element) =
                    self.element_neighborhood
                        .neighbor(element, temperature, rng)
                {
                    *element = new_element;
                    modified = true;
                }
            }
        }

        if modified {
            Some(result)
        } else {
            // Fallback: modify at least one random element
            let index = rng.gen_range(0..input.len());
            if let Some(new_element) =
                self.element_neighborhood
                    .neighbor(&input[index], temperature, rng)
            {
                let mut result = input.clone();
                result[index] = new_element;
                Some(result)
            } else {
                None
            }
        }
    }

    fn max_distance(&self) -> f64 {
        // Maximum distance is if we modify all elements
        self.element_neighborhood.max_distance() * self.modification_probability
    }
}

/// Neighborhood function for strings that performs small edits.
#[derive(Debug, Clone)]
pub struct StringNeighborhood {
    /// Characters to use for insertions and replacements
    pub alphabet: Vec<char>,
}

impl StringNeighborhood {
    /// Create a new string neighborhood function.
    pub fn new(alphabet: Vec<char>) -> Self {
        StringNeighborhood { alphabet }
    }

    /// Create a default string neighborhood with ASCII alphanumeric characters.
    pub fn ascii_alphanumeric() -> Self {
        let mut alphabet = Vec::new();
        alphabet.extend('a'..='z');
        alphabet.extend('A'..='Z');
        alphabet.extend('0'..='9');
        StringNeighborhood::new(alphabet)
    }
}

impl Default for StringNeighborhood {
    fn default() -> Self {
        StringNeighborhood::ascii_alphanumeric()
    }
}

impl NeighborhoodFunction<String> for StringNeighborhood {
    fn neighbor(&self, input: &String, temperature: f64, rng: &mut dyn RngCore) -> Option<String> {
        if self.alphabet.is_empty() {
            return None;
        }

        let mut chars: Vec<char> = input.chars().collect();

        // Choose operation based on temperature (higher temp = more dramatic changes)
        let temp_factor = (temperature / 100.0).min(1.0);
        let operation = rng.gen::<f64>();

        if operation < 0.4 + temp_factor * 0.2 && !chars.is_empty() {
            // Replace a random character
            let index = rng.gen_range(0..chars.len());
            let new_char = self.alphabet[rng.gen_range(0..self.alphabet.len())];
            chars[index] = new_char;
        } else if operation < 0.7 + temp_factor * 0.1 {
            // Insert a random character
            let index = rng.gen_range(0..=chars.len());
            let new_char = self.alphabet[rng.gen_range(0..self.alphabet.len())];
            chars.insert(index, new_char);
        } else if !chars.is_empty() && chars.len() > 1 {
            // Delete a random character
            let index = rng.gen_range(0..chars.len());
            chars.remove(index);
        } else {
            // Fallback: replace or insert
            if chars.is_empty() {
                let new_char = self.alphabet[rng.gen_range(0..self.alphabet.len())];
                chars.push(new_char);
            } else {
                let index = rng.gen_range(0..chars.len());
                let new_char = self.alphabet[rng.gen_range(0..self.alphabet.len())];
                chars[index] = new_char;
            }
        }

        Some(chars.into_iter().collect())
    }

    fn max_distance(&self) -> f64 {
        // Rough estimate: one edit operation per character
        1.0
    }
}
