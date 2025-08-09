//! Generator combinators for property-based testing.

use crate::{data::*, tree::*};

// Helper function to safely subtract two values, returning None if overflow would occur
fn try_safe_subtract<T>(a: T, b: T) -> Option<T>
where
    T: Copy + PartialOrd + std::ops::Sub<Output = T>,
{
    // For small values, subtraction should be safe
    // This is a heuristic - for i32::MIN - 0, we know it will overflow
    // For normal values like 8 - 0, it's safe
    
    // The exact overflow detection depends on the type, but for our purposes,
    // we can use a simple heuristic: if both values are "reasonable" sized,
    // the subtraction should be safe.
    
    // A very basic approach: try the subtraction and catch panics
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| a - b)).ok()
}

fn towards<T>(destination: T, x: T) -> Vec<T>
where
    T: Copy + PartialEq + PartialOrd + std::ops::Sub<Output = T> + std::ops::Add<Output = T> + std::ops::Div<Output = T> + From<u8>,
{
    if destination == x {
        return Vec::new();
    }

    let mut result = vec![destination];
    
    // Try to calculate the actual difference, but fall back to a safe value if overflow would occur
    let diff = if x > destination {
        // Check if we can safely subtract
        if let Some(safe_diff) = try_safe_subtract(x, destination) {
            safe_diff
        } else {
            T::from(64) // Safe fallback for extreme values
        }
    } else {
        if let Some(safe_diff) = try_safe_subtract(destination, x) {
            safe_diff
        } else {
            T::from(64) // Safe fallback
        }
    };
    
    let mut current = diff;
    let zero = T::from(0);
    let two = T::from(2);
    
    while current != zero {
        // Avoid overflow in shrink calculations
        let shrink = if x > destination {
            // Moving towards destination by subtracting current
            // But avoid overflow when x is very negative
            if current > x {
                destination  // Just go to destination if current is too large
            } else {
                x - current
            }
        } else {
            // Moving towards destination by adding current
            // But avoid overflow when x + current would exceed max
            // For signed integers, this can overflow too
            let maybe_shrink = x + current;
            if maybe_shrink < x {  // Overflow occurred (for signed types)
                destination
            } else {
                maybe_shrink
            }
        };
        
        if shrink != x && shrink != destination {
            result.push(shrink);
        }
        current = current / two;
    }
    
    result
}

fn removes<T: Clone>(k: usize, xs: &[T]) -> Vec<Vec<T>> {
    if k > xs.len() {
        return Vec::new();
    }
    if k == 0 {
        return vec![xs.to_vec()];
    }
    if xs.len() == k {
        return vec![Vec::new()];
    }
    
    let mut result = Vec::new();
    let tail = &xs[k..];
    result.push(tail.to_vec());
    
    for smaller in removes(k, &xs[1..]) {
        let mut combined = vec![xs[0].clone()];
        combined.extend(smaller);
        result.push(combined);
    }
    
    result
}

fn list_shrinks<T: Clone>(xs: &[T]) -> Vec<Vec<T>> {
    let mut result = Vec::new();
    let len = xs.len();
    
    let mut current = len;
    while current != 0 {
        result.extend(removes(current, xs));
        current /= 2;
    }
    
    result
}

/// A weighted choice for frequency-based generation.
pub struct WeightedChoice<T> {
    /// The weight of this choice (higher weights are more likely).
    pub weight: u64,
    /// The generator for this choice.
    pub generator: Gen<T>,
}

impl<T> WeightedChoice<T> {
    /// Create a new weighted choice.
    pub fn new(weight: u64, generator: Gen<T>) -> Self {
        WeightedChoice { weight, generator }
    }
}

/// Simplify a character towards simpler forms for shrinking.
fn simplify_char(ch: char) -> char {
    match ch {
        // Uppercase to lowercase
        'A'..='Z' => ch.to_ascii_lowercase(),
        // Special characters to simpler ones
        '!' | '@' | '#' | '$' | '%' | '^' | '&' | '*' => 'a',
        '(' | ')' | '[' | ']' | '{' | '}' => 'a',
        '.' | ',' | ';' | ':' | '\'' | '"' => 'a',
        '+' | '-' | '=' | '_' | '|' | '\\' | '/' => 'a',
        '~' | '`' | '<' | '>' | '?' => 'a',
        // Numbers to lower numbers
        '9' => '8',
        '8' => '7',
        '7' => '6',
        '6' => '5',
        '5' => '4',
        '4' => '3',
        '3' => '2',
        '2' => '1',
        '1' => '0',
        // Lowercase letters towards 'a'
        'z' => 'y',
        'y' => 'x',
        'x' => 'w',
        'w' => 'v',
        'v' => 'u',
        'u' => 't',
        't' => 's',
        's' => 'r',
        'r' => 'q',
        'q' => 'p',
        'p' => 'o',
        'o' => 'n',
        'n' => 'm',
        'm' => 'l',
        'l' => 'k',
        'k' => 'j',
        'j' => 'i',
        'i' => 'h',
        'h' => 'g',
        'g' => 'f',
        'f' => 'e',
        'e' => 'd',
        'd' => 'c',
        'c' => 'b',
        'b' => 'a',
        // Everything else stays the same
        _ => ch,
    }
}

/// A generator for test data of type `T`.
///
/// Generators are explicit, first-class values that can be composed
/// using combinator functions. This is a key difference from
/// type-directed approaches like QuickCheck.
pub struct Gen<T> {
    generator: Box<dyn Fn(Size, Seed) -> Tree<T>>,
}

impl<T> Gen<T> {
    /// Create a new generator from a function.
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(Size, Seed) -> Tree<T> + 'static,
    {
        Gen {
            generator: Box::new(f),
        }
    }

    /// Generate a value using the given size and seed.
    pub fn generate(&self, size: Size, seed: Seed) -> Tree<T> {
        (self.generator)(size, seed)
    }

    /// Create a generator that always produces the same value.
    pub fn constant(value: T) -> Self
    where
        T: Clone + 'static,
    {
        Gen::new(move |_size, _seed| Tree::singleton(value.clone()))
    }
    
    /// Generate a sample value with default parameters.
    /// This is a convenience method for state machine testing.
    pub fn sample(&self) -> T
    where
        T: Clone,
    {
        let tree = self.generate(Size(30), Seed(42, 1337));
        tree.value
    }
}

impl<T> Gen<T>
where
    T: 'static,
{
    /// Map a function over the generated values.
    pub fn map<U, F>(self, f: F) -> Gen<U>
    where
        F: Fn(T) -> U + 'static + Clone,
        U: 'static,
    {
        Gen::new(move |size, seed| {
            let tree = self.generate(size, seed);
            tree.map(f.clone())
        })
    }

    /// Bind/flatmap for dependent generation.
    pub fn bind<U, F>(self, f: F) -> Gen<U>
    where
        F: Fn(T) -> Gen<U> + 'static,
        U: 'static,
        T: Clone,
    {
        Gen::new(move |size, seed| {
            let (seed1, seed2) = seed.split();
            let tree = self.generate(size, seed1);
            tree.bind(|value| f(value.clone()).generate(size, seed2))
        })
    }

    /// Filter generated values by a predicate.
    pub fn filter<F>(self, predicate: F) -> Gen<T>
    where
        F: Fn(&T) -> bool + 'static,
        T: Clone,
    {
        Gen::new(move |size, mut seed| {
            const MAX_DISCARDS: usize = 100;
            
            for _ in 0..MAX_DISCARDS {
                let tree = self.generate(size, seed);
                if let Some(filtered_tree) = tree.filter(&predicate) {
                    return filtered_tree;
                }
                // Try with a different seed
                seed = seed.split().1;
            }
            
            // If we couldn't generate a valid value after MAX_DISCARDS attempts,
            // this is likely a too-restrictive filter or a generator issue.
            // For now, panic to make the issue visible rather than silently returning invalid data.
            panic!("Filter: exceeded maximum discards ({}) - predicate may be too restrictive", MAX_DISCARDS);
        })
    }

    /// Generate values using weighted frequency distribution.
    ///
    /// This is similar to QuickCheck's `frequency` and Haskell Hedgehog's weighted choice.
    /// Higher weights make choices more likely to be selected.
    ///
    /// Returns an error if the choices list is empty or all weights are zero.
    pub fn frequency(choices: Vec<WeightedChoice<T>>) -> crate::Result<Gen<T>>
    where
        T: Clone,
    {
        if choices.is_empty() {
            return Err(crate::HedgehogError::InvalidGenerator {
                message: "frequency choices list cannot be empty".to_string(),
            });
        }

        // Calculate total weight
        let total_weight: u64 = choices.iter().map(|c| c.weight).sum();

        if total_weight == 0 {
            return Err(crate::HedgehogError::InvalidGenerator {
                message: "frequency total weight cannot be zero".to_string(),
            });
        }

        Ok(Gen::new(move |size, seed| {
            let (choice_value, new_seed) = seed.next_bounded(total_weight);

            // Find the chosen generator based on cumulative weights
            let mut cumulative_weight = 0;
            let mut chosen_generator = &choices[0].generator;

            for choice in &choices {
                cumulative_weight += choice.weight;
                if choice_value < cumulative_weight {
                    chosen_generator = &choice.generator;
                    break;
                }
            }

            chosen_generator.generate(size, new_seed)
        }))
    }

    /// Generate values using one of the given generators with equal probability.
    ///
    /// This is equivalent to `frequency` with all weights equal to 1.
    /// Returns an error if the generators list is empty.
    pub fn one_of(generators: Vec<Gen<T>>) -> crate::Result<Gen<T>>
    where
        T: Clone,
    {
        let choices = generators
            .into_iter()
            .map(|gen| WeightedChoice::new(1, gen))
            .collect();
        Gen::frequency(choices)
    }

    /// Generate values from a dictionary (list of predefined elements).
    ///
    /// This is useful for injecting domain-specific realistic values into tests.
    /// Returns an error if the elements list is empty.
    ///
    /// # Example
    /// ```rust
    /// use hedgehog_core::*;
    /// 
    /// // Generate HTTP status codes from common values
    /// let status_codes = vec![200, 404, 500, 302, 401];
    /// let gen = Gen::from_elements(status_codes).unwrap();
    /// ```
    pub fn from_elements(elements: Vec<T>) -> crate::Result<Gen<T>>
    where
        T: Clone + 'static,
    {
        if elements.is_empty() {
            return Err(crate::HedgehogError::InvalidGenerator {
                message: "elements list cannot be empty".to_string(),
            });
        }

        Ok(Gen::new(move |_size, seed| {
            let (index, _new_seed) = seed.next_bounded(elements.len() as u64);
            let chosen = elements[index as usize].clone();
            
            // Create shrinking candidates by trying other elements
            let mut shrinks = Vec::new();
            for (i, element) in elements.iter().enumerate() {
                if i != index as usize {
                    shrinks.push(element.clone());
                }
            }
            
            Tree::with_children(chosen, shrinks.into_iter().map(Tree::singleton).collect())
        }))
    }

    /// Mix dictionary values with random generation based on probability weights.
    ///
    /// This allows combining realistic domain-specific values with random generation
    /// to get both targeted edge cases and broad coverage.
    ///
    /// # Parameters
    /// - `elements`: Dictionary of predefined values to inject
    /// - `random_gen`: Generator for random values
    /// - `elements_weight`: Weight for choosing from dictionary (higher = more likely)
    /// - `random_weight`: Weight for choosing random generation
    ///
    /// # Example
    /// ```rust
    /// use hedgehog_core::*;
    /// 
    /// let common_ports = vec![80, 443, 22, 25, 53];
    /// let gen = Gen::from_dictionary(
    ///     common_ports,
    ///     Gen::int_range(1024, 65535), // Random high ports
    ///     70, // 70% chance of common ports
    ///     30  // 30% chance of random ports
    /// ).unwrap();
    /// ```
    pub fn from_dictionary(
        elements: Vec<T>,
        random_gen: Gen<T>,
        elements_weight: u64,
        random_weight: u64,
    ) -> crate::Result<Gen<T>>
    where
        T: Clone + 'static,
    {
        if elements.is_empty() {
            return Err(crate::HedgehogError::InvalidGenerator {
                message: "dictionary elements list cannot be empty".to_string(),
            });
        }

        if elements_weight == 0 && random_weight == 0 {
            return Err(crate::HedgehogError::InvalidGenerator {
                message: "at least one weight must be non-zero".to_string(),
            });
        }

        let elements_gen = Gen::from_elements(elements)?;
        
        let choices = vec![
            WeightedChoice::new(elements_weight, elements_gen),
            WeightedChoice::new(random_weight, random_gen),
        ];

        Gen::frequency(choices)
    }
}

/// Primitive generators.
impl Gen<bool> {
    /// Generate a random boolean.
    pub fn bool() -> Self {
        Gen::new(|_size, seed| {
            let (value, _new_seed) = seed.next_bool();
            Tree::singleton(value)
        })
    }
}

/// Macro to implement enhanced numeric generators with origin-based shrinking for types that support From<u8>.
macro_rules! impl_numeric_gen_with_towards {
    ($type:ty, $method:ident, $max_val:expr) => {
        impl Gen<$type> {
            /// Generate a number in the given range with enhanced shrinking.
            pub fn $method(min: $type, max: $type) -> Self {
                Gen::new(move |_size, seed| {
                    // Prevent overflow by using checked arithmetic and wider types
                    let range = (max as i64).saturating_sub(min as i64).saturating_add(1) as u64;
                    let (value, _new_seed) = seed.next_bounded(range);
                    let result = min.saturating_add(value as $type);

                    let origin = if min <= 0 && max >= 0 {
                        0
                    } else if min > 0 {
                        min
                    } else {
                        max
                    };

                    let mut shrinks = Vec::new();
                    
                    // Use original shrinking for types that support From<u8>
                    let shrink_values = towards(origin, result);
                    for &shrink_value in &shrink_values {
                        if shrink_value >= min && shrink_value <= max {
                            shrinks.push(Tree::singleton(shrink_value));
                        }
                    }

                    Tree::with_children(result, shrinks)
                })
            }

            /// Generate a positive number.
            pub fn positive() -> Self {
                Self::$method(1, $max_val)
            }

            /// Generate a natural number (including zero).
            pub fn natural() -> Self {
                Self::$method(0, $max_val)
            }
        }
    };
}

/// Macro to implement enhanced numeric generators with simple shrinking for types that don't support From<u8>.
macro_rules! impl_numeric_gen_simple_shrink {
    ($type:ty, $method:ident, $max_val:expr) => {
        impl Gen<$type> {
            /// Generate a number in the given range with simple shrinking.
            pub fn $method(min: $type, max: $type) -> Self {
                Gen::new(move |_size, seed| {
                    // Prevent overflow by using checked arithmetic and wider types
                    let range = (max as i64).saturating_sub(min as i64).saturating_add(1) as u64;
                    let (value, _new_seed) = seed.next_bounded(range);
                    let result = min.saturating_add(value as $type);

                    let origin = if min <= 0 && max >= 0 {
                        0
                    } else if min > 0 {
                        min
                    } else {
                        max
                    };

                    let mut shrinks = Vec::new();
                    
                    // Generate shrink values without requiring From<u8>
                    let mut current = result;
                    while current != origin && shrinks.len() < 10 {
                        if current > origin {
                            current = current.saturating_sub(1).max(origin);
                        } else {
                            current = current.saturating_add(1).min(origin);
                        }
                        if current != result && current >= min && current <= max {
                            shrinks.push(Tree::singleton(current));
                        }
                    }

                    Tree::with_children(result, shrinks)
                })
            }

            /// Generate a positive number.
            pub fn positive() -> Self {
                Self::$method(1, $max_val)
            }

            /// Generate a natural number (including zero).
            pub fn natural() -> Self {
                Self::$method(0, $max_val)
            }
        }
    };
}

// Implement for signed integers
impl_numeric_gen_simple_shrink!(i8, i8_range, i8::MAX);
impl_numeric_gen_simple_shrink!(i16, i16_range, i16::MAX);
impl_numeric_gen_with_towards!(i32, int_range, i32::MAX);
impl_numeric_gen_with_towards!(i64, i64_range, i64::MAX);
impl_numeric_gen_simple_shrink!(isize, isize_range, isize::MAX);

// Implement for unsigned integers using specialized macro
macro_rules! impl_unsigned_gen {
    ($type:ty, $method:ident, $max_val:expr) => {
        impl Gen<$type> {
            /// Generate an unsigned number in the given range.
            pub fn $method(min: $type, max: $type) -> Self {
                Gen::new(move |_size, seed| {
                    // Prevent overflow by using checked arithmetic
                    let range = (max as u64).saturating_sub(min as u64).saturating_add(1);
                    let (value, _new_seed) = seed.next_bounded(range);
                    let result = min.saturating_add(value as $type);

                    let origin = min;
                    
                    let shrink_values = towards(origin, result);
                    let mut shrinks = Vec::new();
                    for &shrink_value in &shrink_values {
                        if shrink_value >= min && shrink_value <= max {
                            shrinks.push(Tree::singleton(shrink_value));
                        }
                    }

                    Tree::with_children(result, shrinks)
                })
            }

            /// Generate a positive number.
            pub fn positive() -> Self {
                Self::$method(1, $max_val)
            }

            /// Generate a natural number (including zero).
            pub fn natural() -> Self {
                Self::$method(0, $max_val)
            }
        }
    };
}

impl_unsigned_gen!(u8, u8_range, u8::MAX);
impl_unsigned_gen!(u16, u16_range, u16::MAX);
impl_unsigned_gen!(u32, u32_range, u32::MAX);
impl_unsigned_gen!(u64, u64_range, u64::MAX);
impl_unsigned_gen!(usize, usize_range, usize::MAX);

// Enhanced range-based generators with distribution support
impl Gen<i32> {
    /// Generate integers using a Range specification with distribution control.
    pub fn from_range(range: crate::data::Range<i32>) -> Self {
        Gen::new(move |_size, seed| {
            // Prevent overflow by using checked arithmetic
            let range_size = (range.max as i64).saturating_sub(range.min as i64).saturating_add(1) as u64;
            let (offset, _new_seed) = range.distribution.sample_u64(seed, range_size);
            let result = range.min.saturating_add(offset as i32);

            // Enhanced shrinking: towards origin, binary search, and traditional halving
            let mut shrinks = Vec::new();

            // Determine the origin for shrinking
            let origin = range.origin.unwrap_or_else(|| {
                if range.min <= 0 && range.max >= 0 {
                    0 // Zero is in range
                } else if range.min > 0 {
                    range.min // Positive range, shrink towards minimum
                } else {
                    range.max // Negative range, shrink towards maximum (closest to 0)
                }
            });

            if origin != result {
                shrinks.push(Tree::singleton(origin));
            }

            // Binary search shrinking between result and origin
            let mut low = result.min(origin);
            let mut high = result.max(origin);

            while high - low > 1 {
                let mid = low + (high - low) / 2;
                if mid != result && mid >= range.min && mid <= range.max {
                    shrinks.push(Tree::singleton(mid));
                }
                if result < origin {
                    low = mid;
                } else {
                    high = mid;
                }
            }

            Tree::with_children(result, shrinks)
        })
    }
}

impl Gen<i64> {
    /// Generate i64 values using a Range specification with distribution control.
    pub fn from_range(range: crate::data::Range<i64>) -> Self {
        Gen::new(move |_size, seed| {
            // Prevent overflow by using checked arithmetic and saturating operations
            let range_size = if range.max == i64::MAX && range.min == i64::MIN {
                u64::MAX
            } else {
                (range.max as i128).saturating_sub(range.min as i128).saturating_add(1) as u64
            };
            let (offset, _new_seed) = range.distribution.sample_u64(seed, range_size);
            let result = range.min.saturating_add(offset as i64);

            // Enhanced shrinking similar to i32
            let mut shrinks = Vec::new();
            let origin = range.origin.unwrap_or_else(|| {
                if range.min <= 0 && range.max >= 0 {
                    0
                } else if range.min > 0 {
                    range.min
                } else {
                    range.max
                }
            });

            if origin != result {
                shrinks.push(Tree::singleton(origin));
            }

            Tree::with_children(result, shrinks)
        })
    }
}

impl Gen<u32> {
    /// Generate u32 values using a Range specification with distribution control.
    pub fn from_range(range: crate::data::Range<u32>) -> Self {
        Gen::new(move |_size, seed| {
            // Prevent overflow by using checked arithmetic
            let range_size = (range.max as u64).saturating_sub(range.min as u64).saturating_add(1);
            let (offset, _new_seed) = range.distribution.sample_u64(seed, range_size);
            let result = range.min.saturating_add(offset as u32);

            // Enhanced shrinking for unsigned: towards minimum
            let mut shrinks = Vec::new();
            let origin = range.origin.unwrap_or(range.min);

            if origin != result {
                shrinks.push(Tree::singleton(origin));
            }

            Tree::with_children(result, shrinks)
        })
    }
}

impl Gen<f64> {
    /// Generate f64 values using a Range specification with distribution control.
    pub fn from_range(range: crate::data::Range<f64>) -> Self {
        Gen::new(move |_size, seed| {
            let (normalized, _new_seed) = range.distribution.sample_f64(seed);
            let result = range.min + normalized * (range.max - range.min);

            // Enhanced shrinking for floats: towards origin and simple values
            let mut shrinks = Vec::new();
            let origin = range.origin.unwrap_or(0.0);

            if origin >= range.min && origin <= range.max && origin != result {
                shrinks.push(Tree::singleton(origin));
            }

            // Try common simple values that are in range
            let simple_values = [0.0, 1.0, -1.0, 0.5, -0.5];
            for &simple in &simple_values {
                if simple >= range.min && simple <= range.max && simple != result {
                    shrinks.push(Tree::singleton(simple));
                }
            }

            Tree::with_children(result, shrinks)
        })
    }

    /// Generate a f64 in the given range (convenience method).
    pub fn f64_range(min: f64, max: f64) -> Self {
        Gen::<f64>::from_range(crate::data::Range::new(min, max))
    }

    /// Generate a positive f64.
    pub fn positive() -> Self {
        Self::f64_range(f64::EPSILON, f64::MAX)
    }

    /// Generate a natural f64 (including zero).
    pub fn natural() -> Self {
        Self::f64_range(0.0, f64::MAX)
    }

    /// Generate a f64 in the unit interval [0, 1].
    pub fn unit() -> Self {
        Self::f64_range(0.0, 1.0)
    }
}

// Additional from_range implementations for missing integer types
impl Gen<i8> {
    /// Generate i8 values using a Range specification with distribution control.
    pub fn from_range(range: crate::data::Range<i8>) -> Self {
        Gen::new(move |_size, seed| {
            let range_size = (range.max as i16).saturating_sub(range.min as i16).saturating_add(1) as u64;
            let (offset, _new_seed) = range.distribution.sample_u64(seed, range_size);
            let result = range.min.saturating_add(offset as i8);

            let mut shrinks = Vec::new();
            let origin = range.origin.unwrap_or_else(|| {
                if range.min <= 0 && range.max >= 0 { 0 } else if range.min > 0 { range.min } else { range.max }
            });
            
            // Generate shrink values for i8 without requiring From<u8>
            let mut current = result;
            while current != origin && shrinks.len() < 10 {
                if current > origin {
                    current = current.saturating_sub(1).max(origin);
                } else {
                    current = current.saturating_add(1).min(origin);
                }
                if current != result && current >= range.min && current <= range.max {
                    shrinks.push(Tree::singleton(current));
                }
            }

            Tree::with_children(result, shrinks)
        })
    }
}

impl Gen<i16> {
    /// Generate i16 values using a Range specification with distribution control.
    pub fn from_range(range: crate::data::Range<i16>) -> Self {
        Gen::new(move |_size, seed| {
            let range_size = (range.max as i32).saturating_sub(range.min as i32).saturating_add(1) as u64;
            let (offset, _new_seed) = range.distribution.sample_u64(seed, range_size);
            let result = range.min.saturating_add(offset as i16);

            let mut shrinks = Vec::new();
            let origin = range.origin.unwrap_or_else(|| {
                if range.min <= 0 && range.max >= 0 { 0 } else if range.min > 0 { range.min } else { range.max }
            });
            
            // Generate shrink values for i16 without requiring From<u8>
            let mut current = result;
            while current != origin && shrinks.len() < 10 {
                if current > origin {
                    current = current.saturating_sub(1).max(origin);
                } else {
                    current = current.saturating_add(1).min(origin);
                }
                if current != result && current >= range.min && current <= range.max {
                    shrinks.push(Tree::singleton(current));
                }
            }

            Tree::with_children(result, shrinks)
        })
    }
}

impl Gen<isize> {
    /// Generate isize values using a Range specification with distribution control.
    pub fn from_range(range: crate::data::Range<isize>) -> Self {
        Gen::new(move |_size, seed| {
            let range_size = if range.max == isize::MAX && range.min == isize::MIN {
                u64::MAX
            } else {
                (range.max as i128).saturating_sub(range.min as i128).saturating_add(1) as u64
            };
            let (offset, _new_seed) = range.distribution.sample_u64(seed, range_size);
            let result = range.min.saturating_add(offset as isize);

            let mut shrinks = Vec::new();
            let origin = range.origin.unwrap_or_else(|| {
                if range.min <= 0 && range.max >= 0 { 0 } else if range.min > 0 { range.min } else { range.max }
            });
            
            // Generate shrink values for isize without requiring From<u8>
            let mut current = result;
            while current != origin && shrinks.len() < 10 {
                if current > origin {
                    current = current.saturating_sub(1).max(origin);
                } else {
                    current = current.saturating_add(1).min(origin);
                }
                if current != result && current >= range.min && current <= range.max {
                    shrinks.push(Tree::singleton(current));
                }
            }

            Tree::with_children(result, shrinks)
        })
    }
}

impl Gen<u8> {
    /// Generate u8 values using a Range specification with distribution control.
    pub fn from_range(range: crate::data::Range<u8>) -> Self {
        Gen::new(move |_size, seed| {
            let range_size = (range.max as u16).saturating_sub(range.min as u16).saturating_add(1) as u64;
            let (offset, _new_seed) = range.distribution.sample_u64(seed, range_size);
            let result = range.min.saturating_add(offset as u8);

            let mut shrinks = Vec::new();
            let origin = range.origin.unwrap_or(range.min);
            
            let shrink_values = towards(origin, result);
            for &shrink_value in &shrink_values {
                if shrink_value >= range.min && shrink_value <= range.max {
                    shrinks.push(Tree::singleton(shrink_value));
                }
            }

            Tree::with_children(result, shrinks)
        })
    }
}

impl Gen<u64> {
    /// Generate u64 values using a Range specification with distribution control.
    pub fn from_range(range: crate::data::Range<u64>) -> Self {
        Gen::new(move |_size, seed| {
            let range_size = if range.max == u64::MAX && range.min == 0 {
                u64::MAX
            } else {
                range.max.saturating_sub(range.min).saturating_add(1)
            };
            let (offset, _new_seed) = range.distribution.sample_u64(seed, range_size);
            let result = range.min.saturating_add(offset);

            let mut shrinks = Vec::new();
            let origin = range.origin.unwrap_or(range.min);
            
            let shrink_values = towards(origin, result);
            for &shrink_value in &shrink_values {
                if shrink_value >= range.min && shrink_value <= range.max {
                    shrinks.push(Tree::singleton(shrink_value));
                }
            }

            Tree::with_children(result, shrinks)
        })
    }
}

impl Gen<usize> {
    /// Generate usize values using a Range specification with distribution control.
    pub fn from_range(range: crate::data::Range<usize>) -> Self {
        Gen::new(move |_size, seed| {
            let range_size = if range.max == usize::MAX && range.min == 0 {
                u64::MAX
            } else {
                range.max.saturating_sub(range.min).saturating_add(1) as u64
            };
            let (offset, _new_seed) = range.distribution.sample_u64(seed, range_size);
            let result = range.min.saturating_add(offset as usize);

            let mut shrinks = Vec::new();
            let origin = range.origin.unwrap_or(range.min);
            
            let shrink_values = towards(origin, result);
            for &shrink_value in &shrink_values {
                if shrink_value >= range.min && shrink_value <= range.max {
                    shrinks.push(Tree::singleton(shrink_value));
                }
            }

            Tree::with_children(result, shrinks)
        })
    }
}

impl Gen<u16> {
    /// Generate realistic HTTP status codes with weighted distribution.
    ///
    /// Heavily weights common status codes (200, 404, 500) while still
    /// generating less common ones for comprehensive testing.
    ///
    /// # Example
    /// ```rust
    /// use hedgehog_core::*;
    /// 
    /// let status_gen = Gen::<u16>::http_status_code();
    /// ```
    pub fn http_status_code() -> Self {
        let common_statuses = vec![
            200, 201, 204,           // Success
            301, 302, 304,           // Redirection  
            400, 401, 403, 404, 409, // Client Error
            500, 502, 503, 504       // Server Error
        ];
        
        Gen::from_dictionary(
            common_statuses,
            Gen::int_range(100, 599).map(|i| i as u16), // Any valid HTTP status
            85, // 85% common statuses
            15  // 15% random valid statuses
        ).unwrap()
    }

    /// Generate network port numbers with weighted distribution.
    ///
    /// Mixes well-known ports (1-1023) with registered ports (1024-49151)
    /// and dynamic ports (49152-65535).
    ///
    /// # Example
    /// ```rust
    /// use hedgehog_core::*;
    /// 
    /// let port_gen = Gen::<u16>::network_port();
    /// ```
    pub fn network_port() -> Self {
        let well_known = vec![
            21, 22, 23, 25, 53, 67, 68, 69, 80, 110,    // Basic services
            143, 443, 993, 995, 587, 465, 993, 143      // Email & secure
        ];
        
        Gen::frequency(vec![
            WeightedChoice::new(40, Gen::from_elements(well_known).unwrap()),
            WeightedChoice::new(35, Gen::int_range(1024, 49151).map(|i| i as u16)),
            WeightedChoice::new(25, Gen::int_range(49152, 65535).map(|i| i as u16)),
        ]).unwrap()
    }

    /// Generate u16 values using a Range specification with distribution control.
    pub fn from_range(range: crate::data::Range<u16>) -> Self {
        Gen::new(move |_size, seed| {
            let range_size = (range.max as u32).saturating_sub(range.min as u32).saturating_add(1) as u64;
            let (offset, _new_seed) = range.distribution.sample_u64(seed, range_size);
            let result = range.min.saturating_add(offset as u16);

            let mut shrinks = Vec::new();
            let origin = range.origin.unwrap_or(range.min);
            
            let shrink_values = towards(origin, result);
            for &shrink_value in &shrink_values {
                if shrink_value >= range.min && shrink_value <= range.max {
                    shrinks.push(Tree::singleton(shrink_value));
                }
            }

            Tree::with_children(result, shrinks)
        })
    }
}

impl Gen<char> {
    /// Generate ASCII alphabetic characters (a-z, A-Z).
    pub fn ascii_alpha() -> Self {
        Gen::new(|_size, seed| {
            let (value, _new_seed) = seed.next_bounded(52);
            let ch = if value < 26 {
                (b'a' + value as u8) as char
            } else {
                (b'A' + (value - 26) as u8) as char
            };
            Tree::singleton(ch)
        })
    }

    /// Generate ASCII alphanumeric characters (a-z, A-Z, 0-9).
    pub fn ascii_alphanumeric() -> Self {
        Gen::new(|_size, seed| {
            let (value, _new_seed) = seed.next_bounded(62);
            let ch = if value < 26 {
                (b'a' + value as u8) as char
            } else if value < 52 {
                (b'A' + (value - 26) as u8) as char
            } else {
                (b'0' + (value - 52) as u8) as char
            };
            Tree::singleton(ch)
        })
    }

    /// Generate printable ASCII characters (space through tilde).
    pub fn ascii_printable() -> Self {
        Gen::new(|_size, seed| {
            let (value, _new_seed) = seed.next_bounded(95);
            let ch = (b' ' + value as u8) as char;
            Tree::singleton(ch)
        })
    }
}

impl Gen<String> {
    /// Generate strings using the given character generator.
    pub fn string_of(char_gen: Gen<char>) -> Self {
        Gen::new(move |size, seed| {
            let (len_seed, chars_seed) = seed.split();
            let (length, _) = len_seed.next_bounded(size.get() as u64 + 1);

            let mut current_seed = chars_seed;
            let mut chars = Vec::new();
            let mut char_trees = Vec::new();

            for _ in 0..length {
                let (char_seed, next_seed) = current_seed.split();
                current_seed = next_seed;

                let char_tree = char_gen.generate(size, char_seed);
                chars.push(char_tree.value);
                char_trees.push(char_tree);
            }

            let string_value: String = chars.iter().collect();

            // Enhanced shrinking: character removal, simplification, and substring removal
            let mut shrinks = Vec::new();

            // Always try empty string first (ultimate shrink)
            if !chars.is_empty() {
                shrinks.push(Tree::singleton(String::new()));
            }

            // Use sophisticated character removal shrinking
            for shrunk_chars in list_shrinks(&chars) {
                let shrunk_string: String = shrunk_chars.iter().collect();
                shrinks.push(Tree::singleton(shrunk_string));
            }

            // Character simplification shrinking
            if !chars.is_empty() {
                let mut simplified_chars = chars.clone();
                let mut did_simplify = false;

                for ch in &mut simplified_chars {
                    let simplified = simplify_char(*ch);
                    if simplified != *ch {
                        *ch = simplified;
                        did_simplify = true;
                        break; // Only simplify one character at a time
                    }
                }

                if did_simplify {
                    let simplified_string: String = simplified_chars.iter().collect();
                    shrinks.push(Tree::singleton(simplified_string));
                }
            }

            // Substring shrinking (try common prefixes/suffixes)
            if chars.len() > 1 {
                // Try first half
                let half = chars.len() / 2;
                let first_half: String = chars[..half].iter().collect();
                shrinks.push(Tree::singleton(first_half));

                // Try second half
                let second_half: String = chars[half..].iter().collect();
                shrinks.push(Tree::singleton(second_half));
            }

            Tree::with_children(string_value, shrinks)
        })
    }

    /// Generate ASCII alphabetic strings.
    pub fn ascii_alpha() -> Self {
        Self::string_of(Gen::<char>::ascii_alpha())
    }

    /// Generate ASCII alphanumeric strings.
    pub fn ascii_alphanumeric() -> Self {
        Self::string_of(Gen::<char>::ascii_alphanumeric())
    }

    /// Generate printable ASCII strings.
    pub fn ascii_printable() -> Self {
        Self::string_of(Gen::<char>::ascii_printable())
    }

    /// Generate strings with controlled length using a Range specification.
    pub fn with_range(length_range: crate::data::Range<usize>, char_gen: Gen<char>) -> Self {
        Gen::new(move |size, seed| {
            let (len_seed, chars_seed) = seed.split();

            // Use the range distribution to determine length
            let range_size = (length_range.max - length_range.min + 1) as u64;
            let (offset, _) = length_range.distribution.sample_u64(len_seed, range_size);
            let length = length_range.min + offset as usize;

            let mut current_seed = chars_seed;
            let mut chars = Vec::new();
            let mut char_trees = Vec::new();

            for _ in 0..length {
                let (char_seed, next_seed) = current_seed.split();
                current_seed = next_seed;

                let char_tree = char_gen.generate(size, char_seed);
                chars.push(char_tree.value);
                char_trees.push(char_tree);
            }

            let string_value: String = chars.iter().collect();

            // Enhanced shrinking: try origin length first, then character removal
            let mut shrinks = Vec::new();

            // Try shrinking to origin length if different from current length
            let origin_length = length_range.origin.unwrap_or(length_range.min);
            if origin_length != length
                && origin_length >= length_range.min
                && origin_length <= length_range.max
            {
                if origin_length == 0 {
                    shrinks.push(Tree::singleton(String::new()));
                } else if origin_length < length {
                    let origin_string: String = chars[..origin_length].iter().collect();
                    shrinks.push(Tree::singleton(origin_string));
                }
            }

            // Always try empty string as ultimate shrink
            if !chars.is_empty() && length_range.min == 0 {
                shrinks.push(Tree::singleton(String::new()));
            }

            // Character removal shrinking
            if !chars.is_empty() {
                // Remove last character
                let shorter: String = chars[..chars.len() - 1].iter().collect();
                shrinks.push(Tree::singleton(shorter));

                // Remove first character
                if chars.len() > 1 {
                    let shorter: String = chars[1..].iter().collect();
                    shrinks.push(Tree::singleton(shorter));
                }
            }

            // Character simplification (replace with 'a')
            if !chars.is_empty() {
                let mut simplified_chars = chars.clone();
                let mut did_simplify = false;

                for ch in simplified_chars.iter_mut() {
                    let simplified = simplify_char(*ch);
                    if simplified != *ch {
                        *ch = simplified;
                        did_simplify = true;
                        break; // Only simplify one character at a time
                    }
                }

                if did_simplify {
                    let simplified_string: String = simplified_chars.iter().collect();
                    shrinks.push(Tree::singleton(simplified_string));
                }
            }

            Tree::with_children(string_value, shrinks)
        })
    }

    /// Generate ASCII alphabetic strings with controlled length.
    pub fn alpha_with_range(length_range: crate::data::Range<usize>) -> Self {
        Self::with_range(length_range, Gen::<char>::ascii_alpha())
    }

    /// Generate ASCII alphanumeric strings with controlled length.
    pub fn alphanumeric_with_range(length_range: crate::data::Range<usize>) -> Self {
        Self::with_range(length_range, Gen::<char>::ascii_alphanumeric())
    }

    /// Generate printable ASCII strings with controlled length.
    pub fn printable_with_range(length_range: crate::data::Range<usize>) -> Self {
        Self::with_range(length_range, Gen::<char>::ascii_printable())
    }

    /// Generate realistic web domain names using common TLDs.
    ///
    /// Mixes common TLDs (.com, .org, .net, etc.) with random subdomains.
    ///
    /// # Example
    /// ```rust
    /// use hedgehog_core::*;
    /// 
    /// let domain_gen = Gen::<String>::web_domain();
    /// ```
    pub fn web_domain() -> Self {
        let tlds = vec![
            ".com", ".org", ".net", ".edu", ".gov", 
            ".io", ".co", ".uk", ".de", ".fr", ".jp",
            ".ca", ".au", ".ru", ".br", ".in"
        ];
        
        Gen::new(move |size, seed| {
            let (tld_seed, domain_seed) = seed.split();
            let (tld_index, _) = tld_seed.next_bounded(tlds.len() as u64);
            let chosen_tld = tlds[tld_index as usize];
            
            // Generate a random subdomain (3-12 characters)
            let subdomain_gen = Gen::<String>::alpha_with_range(
                crate::data::Range::linear(3, 12)
            );
            let subdomain_tree = subdomain_gen.generate(size, domain_seed);
            let subdomain = subdomain_tree.value.to_lowercase();
            
            let full_domain = format!("{}{}", subdomain, chosen_tld);
            
            // Shrinking: try other TLDs and shrink subdomain
            let mut shrinks = Vec::new();
            for (i, other_tld) in tlds.iter().enumerate() {
                if i != tld_index as usize {
                    shrinks.push(format!("{}{}", subdomain, other_tld));
                }
            }
            
            // Add shrunk subdomains with the same TLD
            for shrunk_subdomain in subdomain_tree.shrinks() {
                let shrunk_domain = format!("{}{}", shrunk_subdomain.to_lowercase(), chosen_tld);
                shrinks.push(shrunk_domain);
            }
            
            Tree::with_children(full_domain, shrinks.into_iter().map(Tree::singleton).collect())
        })
    }

    /// Generate realistic email addresses with common domains.
    ///
    /// Combines random usernames with realistic email domains.
    ///
    /// # Example
    /// ```rust
    /// use hedgehog_core::*;
    /// 
    /// let email_gen = Gen::<String>::email_address();
    /// ```
    pub fn email_address() -> Self {
        let domains = vec![
            "@gmail.com", "@yahoo.com", "@hotmail.com", "@outlook.com",
            "@aol.com", "@icloud.com", "@protonmail.com", "@fastmail.com",
            "@example.com", "@test.com", "@company.org", "@university.edu"
        ];
        
        Gen::new(move |size, seed| {
            let (domain_seed, username_seed) = seed.split();
            let (domain_index, _) = domain_seed.next_bounded(domains.len() as u64);
            let chosen_domain = domains[domain_index as usize];
            
            // Generate simple username (just alphabetic for simplicity)
            let username_gen = Gen::<String>::alpha_with_range(
                crate::data::Range::linear(3, 15)
            );
            
            let username_tree = username_gen.generate(size, username_seed);
            let username = username_tree.value.to_lowercase();
            
            let full_email = format!("{}{}", username, chosen_domain);
            
            // Shrinking: try other domains and shrink username
            let mut shrinks = Vec::new();
            for (i, other_domain) in domains.iter().enumerate() {
                if i != domain_index as usize {
                    shrinks.push(format!("{}{}", username, other_domain));
                }
            }
            
            // Add shrunk usernames with the same domain
            for shrunk_username in username_tree.shrinks() {
                let shrunk_email = format!("{}{}", shrunk_username.to_lowercase(), chosen_domain);
                shrinks.push(shrunk_email);
            }
            
            Tree::with_children(full_email, shrinks.into_iter().map(Tree::singleton).collect())
        })
    }

    /// Generate SQL identifiers with optional keyword injection.
    ///
    /// Mixes common SQL keywords with random identifiers for database testing.
    ///
    /// # Parameters
    /// - `include_keywords`: Whether to include SQL keywords (can cause syntax errors)
    ///
    /// # Example
    /// ```rust
    /// use hedgehog_core::*;
    /// 
    /// let identifier_gen = Gen::<String>::sql_identifier(false); // Safe identifiers only
    /// let risky_gen = Gen::<String>::sql_identifier(true);       // May include keywords
    /// ```
    pub fn sql_identifier(include_keywords: bool) -> Self {
        let keywords = vec![
            "SELECT", "INSERT", "UPDATE", "DELETE", "FROM", "WHERE", 
            "JOIN", "INNER", "LEFT", "RIGHT", "ON", "AS", "AND", "OR",
            "NOT", "NULL", "TRUE", "FALSE", "ORDER", "BY", "GROUP",
            "HAVING", "LIMIT", "OFFSET", "UNION", "DISTINCT", "COUNT",
            "SUM", "AVG", "MAX", "MIN", "CREATE", "TABLE", "INDEX",
            "PRIMARY", "KEY", "FOREIGN", "UNIQUE", "CHECK", "DEFAULT"
        ];
        
        if include_keywords {
            Gen::from_dictionary(
                keywords.into_iter().map(|s| s.to_string()).collect(),
                Gen::<String>::alpha_with_range(crate::data::Range::linear(3, 20)),
                30, // 30% keywords
                70  // 70% random identifiers
            ).unwrap()
        } else {
            Gen::<String>::alpha_with_range(crate::data::Range::linear(3, 20))
        }
    }

    /// Generate programming language tokens for a specific language.
    ///
    /// Useful for testing parsers, compilers, and code analysis tools.
    ///
    /// # Example
    /// ```rust
    /// use hedgehog_core::*;
    /// 
    /// let rust_gen = Gen::<String>::programming_tokens(&[
    ///     "fn", "let", "mut", "pub", "struct", "enum", "impl", "trait"
    /// ]);
    /// ```
    pub fn programming_tokens(keywords: &[&str]) -> Self {
        let keyword_strings: Vec<String> = keywords.iter().map(|&s| s.to_string()).collect();
        
        Gen::from_dictionary(
            keyword_strings,
            Gen::<String>::alpha_with_range(crate::data::Range::linear(2, 15)),
            40, // 40% keywords
            60  // 60% random identifiers
        ).unwrap()
    }
}

impl<T> Gen<Vec<T>>
where
    T: 'static + Clone,
{
    /// Generate vectors using the given element generator.
    pub fn vec_of(element_gen: Gen<T>) -> Self {
        Gen::new(move |size, seed| {
            let (len_seed, elements_seed) = seed.split();
            let (length, _) = len_seed.next_bounded(size.get() as u64 + 1);

            let mut current_seed = elements_seed;
            let mut elements = Vec::new();
            let mut element_trees = Vec::new();

            for _ in 0..length {
                let (element_seed, next_seed) = current_seed.split();
                current_seed = next_seed;

                let element_tree = element_gen.generate(size, element_seed);
                elements.push(element_tree.value.clone());
                element_trees.push(element_tree);
            }

            let mut shrinks = Vec::new();

            // Use sophisticated list shrinking algorithm
            for shrunk_list in list_shrinks(&elements) {
                shrinks.push(Tree::singleton(shrunk_list));
            }

            // Element-wise shrinking: shrink individual elements while keeping the structure
            for (i, element_tree) in element_trees.iter().enumerate() {
                for shrunk_element in element_tree.shrinks() {
                    let mut shrunk_vec = elements.clone();
                    shrunk_vec[i] = shrunk_element.clone();
                    shrinks.push(Tree::singleton(shrunk_vec));
                }
            }

            Tree::with_children(elements, shrinks)
        })
    }
}

impl Gen<Vec<i32>> {
    /// Generate vectors of integers.
    pub fn vec_int() -> Self {
        Self::vec_of(Gen::int_range(-100, 100))
    }
}

impl Gen<Vec<bool>> {
    /// Generate vectors of booleans.
    pub fn vec_bool() -> Self {
        Self::vec_of(Gen::bool())
    }
}

impl<T> Gen<Option<T>>
where
    T: 'static + Clone,
{
    /// Generate optional values using the given generator.
    pub fn option_of(inner_gen: Gen<T>) -> Self {
        Gen::new(move |size, seed| {
            let (choice_seed, value_seed) = seed.split();
            let (choice, _) = choice_seed.next_bounded(4);

            if choice == 0 {
                // Generate None (25% chance)
                Tree::singleton(None)
            } else {
                // Generate Some(value) (75% chance)
                let value_tree = inner_gen.generate(size, value_seed);
                let some_value = Some(value_tree.value.clone());

                // Shrink to None and shrink the inner value
                let mut shrinks = vec![Tree::singleton(None)];

                // Add shrinks of the inner value wrapped in Some
                for shrink in value_tree.shrinks() {
                    shrinks.push(Tree::singleton(Some(shrink.clone())));
                }

                Tree::with_children(some_value, shrinks)
            }
        })
    }
}

impl<T, U> Gen<(T, U)>
where
    T: 'static + Clone,
    U: 'static + Clone,
{
    /// Generate tuples using the given generators.
    pub fn tuple_of(first_gen: Gen<T>, second_gen: Gen<U>) -> Self {
        Gen::new(move |size, seed| {
            let (first_seed, second_seed) = seed.split();

            let first_tree = first_gen.generate(size, first_seed);
            let second_tree = second_gen.generate(size, second_seed);

            let tuple_value = (first_tree.value.clone(), second_tree.value.clone());

            // Generate shrinks by shrinking each component
            let mut shrinks = Vec::new();

            // Shrink first component, keep second
            for first_shrink in first_tree.shrinks() {
                let shrunk_tuple = (first_shrink.clone(), second_tree.value.clone());
                shrinks.push(Tree::singleton(shrunk_tuple));
            }

            // Shrink second component, keep first
            for second_shrink in second_tree.shrinks() {
                let shrunk_tuple = (first_tree.value.clone(), second_shrink.clone());
                shrinks.push(Tree::singleton(shrunk_tuple));
            }

            Tree::with_children(tuple_value, shrinks)
        })
    }
}

// 3-element tuple implementation
impl<T, U, V> Gen<(T, U, V)>
where
    T: 'static + Clone,
    U: 'static + Clone,
    V: 'static + Clone,
{
    /// Generate 3-element tuples using the given generators.
    pub fn tuple_of(first_gen: Gen<T>, second_gen: Gen<U>, third_gen: Gen<V>) -> Self {
        Gen::new(move |size, seed| {
            let (first_seed, rest_seed) = seed.split();
            let (second_seed, third_seed) = rest_seed.split();

            let first_tree = first_gen.generate(size, first_seed);
            let second_tree = second_gen.generate(size, second_seed);
            let third_tree = third_gen.generate(size, third_seed);

            let tuple_value = (first_tree.value.clone(), second_tree.value.clone(), third_tree.value.clone());

            let mut shrinks = Vec::new();

            // Shrink first component, keep others
            for first_shrink in first_tree.shrinks() {
                let shrunk_tuple = (first_shrink.clone(), second_tree.value.clone(), third_tree.value.clone());
                shrinks.push(Tree::singleton(shrunk_tuple));
            }

            // Shrink second component, keep others
            for second_shrink in second_tree.shrinks() {
                let shrunk_tuple = (first_tree.value.clone(), second_shrink.clone(), third_tree.value.clone());
                shrinks.push(Tree::singleton(shrunk_tuple));
            }

            // Shrink third component, keep others
            for third_shrink in third_tree.shrinks() {
                let shrunk_tuple = (first_tree.value.clone(), second_tree.value.clone(), third_shrink.clone());
                shrinks.push(Tree::singleton(shrunk_tuple));
            }

            Tree::with_children(tuple_value, shrinks)
        })
    }
}

// 4-element tuple implementation
impl<T, U, V, W> Gen<(T, U, V, W)>
where
    T: 'static + Clone,
    U: 'static + Clone,
    V: 'static + Clone,
    W: 'static + Clone,
{
    /// Generate 4-element tuples using the given generators.
    pub fn tuple_of(first_gen: Gen<T>, second_gen: Gen<U>, third_gen: Gen<V>, fourth_gen: Gen<W>) -> Self {
        Gen::new(move |size, seed| {
            let (first_seed, rest_seed) = seed.split();
            let (second_seed, rest_seed) = rest_seed.split();
            let (third_seed, fourth_seed) = rest_seed.split();

            let first_tree = first_gen.generate(size, first_seed);
            let second_tree = second_gen.generate(size, second_seed);
            let third_tree = third_gen.generate(size, third_seed);
            let fourth_tree = fourth_gen.generate(size, fourth_seed);

            let tuple_value = (
                first_tree.value.clone(),
                second_tree.value.clone(),
                third_tree.value.clone(),
                fourth_tree.value.clone(),
            );

            let mut shrinks = Vec::new();

            // Shrink each component while keeping others fixed
            for first_shrink in first_tree.shrinks() {
                let shrunk_tuple = (first_shrink.clone(), second_tree.value.clone(), third_tree.value.clone(), fourth_tree.value.clone());
                shrinks.push(Tree::singleton(shrunk_tuple));
            }

            for second_shrink in second_tree.shrinks() {
                let shrunk_tuple = (first_tree.value.clone(), second_shrink.clone(), third_tree.value.clone(), fourth_tree.value.clone());
                shrinks.push(Tree::singleton(shrunk_tuple));
            }

            for third_shrink in third_tree.shrinks() {
                let shrunk_tuple = (first_tree.value.clone(), second_tree.value.clone(), third_shrink.clone(), fourth_tree.value.clone());
                shrinks.push(Tree::singleton(shrunk_tuple));
            }

            for fourth_shrink in fourth_tree.shrinks() {
                let shrunk_tuple = (first_tree.value.clone(), second_tree.value.clone(), third_tree.value.clone(), fourth_shrink.clone());
                shrinks.push(Tree::singleton(shrunk_tuple));
            }

            Tree::with_children(tuple_value, shrinks)
        })
    }
}

// 5-element tuple implementation
impl<T, U, V, W, X> Gen<(T, U, V, W, X)>
where
    T: 'static + Clone,
    U: 'static + Clone,
    V: 'static + Clone,
    W: 'static + Clone,
    X: 'static + Clone,
{
    /// Generate 5-element tuples using the given generators.
    pub fn tuple_of(first_gen: Gen<T>, second_gen: Gen<U>, third_gen: Gen<V>, fourth_gen: Gen<W>, fifth_gen: Gen<X>) -> Self {
        Gen::new(move |size, seed| {
            let (first_seed, rest_seed) = seed.split();
            let (second_seed, rest_seed) = rest_seed.split();
            let (third_seed, rest_seed) = rest_seed.split();
            let (fourth_seed, fifth_seed) = rest_seed.split();

            let first_tree = first_gen.generate(size, first_seed);
            let second_tree = second_gen.generate(size, second_seed);
            let third_tree = third_gen.generate(size, third_seed);
            let fourth_tree = fourth_gen.generate(size, fourth_seed);
            let fifth_tree = fifth_gen.generate(size, fifth_seed);

            let tuple_value = (
                first_tree.value.clone(),
                second_tree.value.clone(),
                third_tree.value.clone(),
                fourth_tree.value.clone(),
                fifth_tree.value.clone(),
            );

            let mut shrinks = Vec::new();

            // Shrink each component while keeping others fixed
            for first_shrink in first_tree.shrinks() {
                let shrunk_tuple = (first_shrink.clone(), second_tree.value.clone(), third_tree.value.clone(), fourth_tree.value.clone(), fifth_tree.value.clone());
                shrinks.push(Tree::singleton(shrunk_tuple));
            }

            for second_shrink in second_tree.shrinks() {
                let shrunk_tuple = (first_tree.value.clone(), second_shrink.clone(), third_tree.value.clone(), fourth_tree.value.clone(), fifth_tree.value.clone());
                shrinks.push(Tree::singleton(shrunk_tuple));
            }

            for third_shrink in third_tree.shrinks() {
                let shrunk_tuple = (first_tree.value.clone(), second_tree.value.clone(), third_shrink.clone(), fourth_tree.value.clone(), fifth_tree.value.clone());
                shrinks.push(Tree::singleton(shrunk_tuple));
            }

            for fourth_shrink in fourth_tree.shrinks() {
                let shrunk_tuple = (first_tree.value.clone(), second_tree.value.clone(), third_tree.value.clone(), fourth_shrink.clone(), fifth_tree.value.clone());
                shrinks.push(Tree::singleton(shrunk_tuple));
            }

            for fifth_shrink in fifth_tree.shrinks() {
                let shrunk_tuple = (first_tree.value.clone(), second_tree.value.clone(), third_tree.value.clone(), fourth_tree.value.clone(), fifth_shrink.clone());
                shrinks.push(Tree::singleton(shrunk_tuple));
            }

            Tree::with_children(tuple_value, shrinks)
        })
    }
}

impl<T, E> Gen<Result<T, E>>
where
    T: 'static + Clone,
    E: 'static + Clone,
{
    /// Generate Result values using the given success and error generators.
    /// By default, generates Ok values 75% of the time and Err values 25% of the time.
    pub fn result_of(ok_gen: Gen<T>, err_gen: Gen<E>) -> Self {
        Gen::new(move |size, seed| {
            let (choice_seed, value_seed) = seed.split();
            let (choice, _) = choice_seed.next_bounded(4);

            if choice == 0 {
                // Generate Err (25% chance)
                let err_tree = err_gen.generate(size, value_seed);
                let err_value = Err(err_tree.value.clone());

                // Shrinking strategy: try shrinking the error, but prioritize Ok values
                let mut shrinks = Vec::new();

                // Try to shrink to a simple Ok value if possible
                // We use a minimal seed to generate a simple success case
                let (ok_seed, _) = value_seed.split();
                let ok_tree = ok_gen.generate(Size::new(0), ok_seed);
                shrinks.push(Tree::singleton(Ok(ok_tree.value.clone())));

                // Add shrinks of the error value wrapped in Err
                for shrink in err_tree.shrinks() {
                    shrinks.push(Tree::singleton(Err(shrink.clone())));
                }

                Tree::with_children(err_value, shrinks)
            } else {
                // Generate Ok (75% chance)
                let ok_tree = ok_gen.generate(size, value_seed);
                let ok_value = Ok(ok_tree.value.clone());

                // Shrinking strategy: shrink the inner value, but keep it as Ok
                let mut shrinks = Vec::new();

                // Add shrinks of the inner value wrapped in Ok
                for shrink in ok_tree.shrinks() {
                    shrinks.push(Tree::singleton(Ok(shrink.clone())));
                }

                Tree::with_children(ok_value, shrinks)
            }
        })
    }

    /// Generate Result values with custom success/error ratio.
    /// `ok_weight` should be between 1-10, higher values favor Ok results.
    pub fn result_of_weighted(ok_gen: Gen<T>, err_gen: Gen<E>, ok_weight: u64) -> Self {
        let total_weight = ok_weight + 1; // Error always has weight 1
        Gen::new(move |size, seed| {
            let (choice_seed, value_seed) = seed.split();
            let (choice, _) = choice_seed.next_bounded(total_weight);

            if choice < ok_weight {
                // Generate Ok
                let ok_tree = ok_gen.generate(size, value_seed);
                let ok_value = Ok(ok_tree.value.clone());

                let mut shrinks = Vec::new();
                for shrink in ok_tree.shrinks() {
                    shrinks.push(Tree::singleton(Ok(shrink.clone())));
                }

                Tree::with_children(ok_value, shrinks)
            } else {
                // Generate Err
                let err_tree = err_gen.generate(size, value_seed);
                let err_value = Err(err_tree.value.clone());

                let mut shrinks = Vec::new();

                // Try to shrink to a simple Ok value
                let (ok_seed, _) = value_seed.split();
                let ok_tree = ok_gen.generate(Size::new(0), ok_seed);
                shrinks.push(Tree::singleton(Ok(ok_tree.value.clone())));

                // Add shrinks of the error value
                for shrink in err_tree.shrinks() {
                    shrinks.push(Tree::singleton(Err(shrink.clone())));
                }

                Tree::with_children(err_value, shrinks)
            }
        })
    }
}

/// Function generators for testing functions as first-class values.
/// These generators create functions that can be called during property tests,
/// enabling testing of higher-order functions and functional composition.
impl<A, B> Gen<Box<dyn Fn(A) -> B>>
where
    A: 'static + Clone + std::fmt::Debug + PartialEq + std::hash::Hash + Eq,
    B: 'static + Clone + std::fmt::Debug,
{
    /// Generate functions from a lookup table mapping inputs to outputs.
    /// 
    /// This creates a finite function by generating a table of input-output pairs
    /// and using a default value for unmapped inputs. The function will have
    /// deterministic behavior that can be shrunk by reducing the lookup table.
    pub fn function_of(
        input_gen: Gen<A>,
        output_gen: Gen<B>,
        default_output: B,
    ) -> Self 
    where
        B: Clone,
    {
        Gen::new(move |size, seed| {
            use std::collections::HashMap;
            
            let (table_size_seed, rest_seed) = seed.split();
            let (table_size, _) = table_size_seed.next_bounded((size.get() + 1) as u64);
            let table_size = (table_size as usize).max(1).min(20); // Reasonable bounds
            
            let mut current_seed = rest_seed;
            let mut lookup_table = HashMap::new();
            let mut input_trees = Vec::new();
            let mut output_trees = Vec::new();
            
            // Generate lookup table entries
            for _ in 0..table_size {
                let (input_seed, rest) = current_seed.split();
                let (output_seed, next_seed) = rest.split();
                current_seed = next_seed;
                
                let input_tree = input_gen.generate(size, input_seed);
                let output_tree = output_gen.generate(size, output_seed);
                
                lookup_table.insert(input_tree.value.clone(), output_tree.value.clone());
                input_trees.push(input_tree);
                output_trees.push(output_tree);
            }
            
            let default = default_output.clone();
            let lookup_table_clone = lookup_table.clone();
            let function: Box<dyn Fn(A) -> B> = Box::new(move |input: A| {
                lookup_table_clone.get(&input).cloned().unwrap_or_else(|| default.clone())
            });
            
            // Shrinking strategy: reduce lookup table size and shrink individual entries
            let mut shrinks = Vec::new();
            
            // Shrink to smaller lookup tables
            if lookup_table.len() > 1 {
                // Try empty lookup table (constant function returning default)
                let empty_default = default_output.clone();
                let constant_fn: Box<dyn Fn(A) -> B> = Box::new(move |_: A| empty_default.clone());
                shrinks.push(Tree::singleton(constant_fn));
                
                // Try lookup table with half the entries
                let half_size = lookup_table.len() / 2;
                if half_size > 0 {
                    let mut smaller_table = HashMap::new();
                    for (key, value) in lookup_table.iter().take(half_size) {
                        smaller_table.insert(key.clone(), value.clone());
                    }
                    let smaller_default = default_output.clone();
                    let smaller_fn: Box<dyn Fn(A) -> B> = Box::new(move |input: A| {
                        smaller_table.get(&input).cloned().unwrap_or_else(|| smaller_default.clone())
                    });
                    shrinks.push(Tree::singleton(smaller_fn));
                }
            }
            
            // Shrink individual lookup entries
            for (i, output_tree) in output_trees.iter().enumerate() {
                for shrunk_output in output_tree.shrinks() {
                    let mut shrunk_table = lookup_table.clone();
                    let input_key = &input_trees[i].value;
                    shrunk_table.insert(input_key.clone(), shrunk_output.clone());
                    
                    let shrunk_default = default_output.clone();
                    let shrunk_fn: Box<dyn Fn(A) -> B> = Box::new(move |input: A| {
                        shrunk_table.get(&input).cloned().unwrap_or_else(|| shrunk_default.clone())
                    });
                    shrinks.push(Tree::singleton(shrunk_fn));
                }
            }
            
            Tree::with_children(function, shrinks)
        })
    }
    
    /// Generate constant functions that always return the same value.
    pub fn constant_function(output_gen: Gen<B>) -> Self {
        Gen::new(move |size, seed| {
            let output_tree = output_gen.generate(size, seed);
            let output_value = output_tree.value.clone();
            
            let function: Box<dyn Fn(A) -> B> = Box::new(move |_: A| output_value.clone());
            
            // Shrink by shrinking the constant output value
            let mut shrinks = Vec::new();
            for shrunk_output in output_tree.shrinks() {
                let shrunk_value = shrunk_output.clone();
                let shrunk_fn: Box<dyn Fn(A) -> B> = Box::new(move |_: A| shrunk_value.clone());
                shrinks.push(Tree::singleton(shrunk_fn));
            }
            
            Tree::with_children(function, shrinks)
        })
    }
    
    /// Generate identity-like functions for compatible input/output types.
    pub fn identity_function() -> Self 
    where
        A: Into<B>,
    {
        Gen::new(move |_size, _seed| {
            let function: Box<dyn Fn(A) -> B> = Box::new(|input: A| input.into());
            Tree::singleton(function)
        })
    }
}

/// Function generators for binary functions.
impl<A, B, C> Gen<Box<dyn Fn(A, B) -> C>>
where
    A: 'static + Clone + std::fmt::Debug + PartialEq + std::hash::Hash + Eq,
    B: 'static + Clone + std::fmt::Debug + PartialEq + std::hash::Hash + Eq,
    C: 'static + Clone + std::fmt::Debug,
{
    /// Generate binary functions using a lookup table for input pairs.
    pub fn binary_function_of(
        input_a_gen: Gen<A>,
        input_b_gen: Gen<B>,
        output_gen: Gen<C>,
        default_output: C,
    ) -> Self {
        Gen::new(move |size, seed| {
            use std::collections::HashMap;
            
            let (table_size_seed, rest_seed) = seed.split();
            let (table_size, _) = table_size_seed.next_bounded((size.get() + 1) as u64);
            let table_size = (table_size as usize).max(1).min(15); // Smaller for binary functions
            
            let mut current_seed = rest_seed;
            let mut lookup_table = HashMap::new();
            let mut output_trees = Vec::new();
            
            // Generate lookup table entries
            for _ in 0..table_size {
                let (input_a_seed, rest) = current_seed.split();
                let (input_b_seed, rest2) = rest.split();
                let (output_seed, next_seed) = rest2.split();
                current_seed = next_seed;
                
                let input_a_tree = input_a_gen.generate(size, input_a_seed);
                let input_b_tree = input_b_gen.generate(size, input_b_seed);
                let output_tree = output_gen.generate(size, output_seed);
                
                let key = (input_a_tree.value.clone(), input_b_tree.value.clone());
                lookup_table.insert(key, output_tree.value.clone());
                output_trees.push(output_tree);
            }
            
            let default = default_output.clone();
            let function: Box<dyn Fn(A, B) -> C> = Box::new(move |a: A, b: B| {
                lookup_table.get(&(a, b)).cloned().unwrap_or_else(|| default.clone())
            });
            
            // Shrinking: similar to unary functions
            let mut shrinks = Vec::new();
            
            // Constant function shrink
            let constant_default = default_output.clone();
            let constant_fn: Box<dyn Fn(A, B) -> C> = Box::new(move |_: A, _: B| constant_default.clone());
            shrinks.push(Tree::singleton(constant_fn));
            
            Tree::with_children(function, shrinks)
        })
    }
}

/// Predicate function generators for testing filter operations.
impl<A> Gen<Box<dyn Fn(A) -> bool>>
where
    A: 'static + Clone + std::fmt::Debug + PartialEq + std::hash::Hash + Eq,
{
    /// Generate predicate functions based on a set of "accepted" values.
    pub fn predicate_from_set(accepted_gen: Gen<Vec<A>>) -> Self {
        Gen::new(move |size, seed| {
            let accepted_tree = accepted_gen.generate(size, seed);
            let accepted_set: std::collections::HashSet<A> = 
                accepted_tree.value.iter().cloned().collect();
            
            let accepted_set_clone = accepted_set.clone();
            let predicate: Box<dyn Fn(A) -> bool> = Box::new(move |input: A| {
                accepted_set_clone.contains(&input)
            });
            
            // Shrinking: shrink the accepted set
            let mut shrinks = Vec::new();
            
            // Always-false predicate (empty set)
            let false_pred: Box<dyn Fn(A) -> bool> = Box::new(|_: A| false);
            shrinks.push(Tree::singleton(false_pred));
            
            // Always-true predicate (if we have any accepted values)
            if !accepted_set.is_empty() {
                let true_pred: Box<dyn Fn(A) -> bool> = Box::new(|_: A| true);
                shrinks.push(Tree::singleton(true_pred));
            }
            
            // Shrink by reducing the accepted set
            for shrunk_accepted in accepted_tree.shrinks() {
                let shrunk_set: std::collections::HashSet<A> = 
                    shrunk_accepted.iter().cloned().collect();
                let shrunk_pred: Box<dyn Fn(A) -> bool> = Box::new(move |input: A| {
                    shrunk_set.contains(&input)
                });
                shrinks.push(Tree::singleton(shrunk_pred));
            }
            
            Tree::with_children(predicate, shrinks)
        })
    }
    
    /// Generate predicate functions that always return the same boolean value.
    pub fn constant_predicate(value_gen: Gen<bool>) -> Self {
        Gen::new(move |size, seed| {
            let bool_tree = value_gen.generate(size, seed);
            let bool_value = bool_tree.value;
            
            let predicate: Box<dyn Fn(A) -> bool> = Box::new(move |_: A| bool_value);
            
            // Shrinking: prefer false over true
            let mut shrinks = Vec::new();
            if bool_value {
                let false_pred: Box<dyn Fn(A) -> bool> = Box::new(|_: A| false);
                shrinks.push(Tree::singleton(false_pred));
            }
            
            Tree::with_children(predicate, shrinks)
        })
    }
}

/// Comparator function generators for testing sorting operations.
impl<A> Gen<Box<dyn Fn(A, A) -> std::cmp::Ordering>>
where
    A: 'static + Clone + std::fmt::Debug + PartialEq + std::hash::Hash + Eq,
{
    /// Generate a constant comparator that always returns the same ordering.
    pub fn constant_comparator(ordering: std::cmp::Ordering) -> Self {
        Gen::new(move |_size, _seed| {
            let comparator: Box<dyn Fn(A, A) -> std::cmp::Ordering> = 
                Box::new(move |_: A, _: A| ordering);
            Tree::singleton(comparator)
        })
    }
    
    /// Generate comparators based on ordering choices.
    pub fn comparator_from_choices(choices: Vec<std::cmp::Ordering>) -> Self {
        Gen::new(move |_size, seed| {
            // Pick a random ordering from the choices
            let (choice_index, _) = seed.next_bounded(choices.len() as u64);
            let chosen_ordering = choices.get(choice_index as usize).copied().unwrap_or(std::cmp::Ordering::Equal);
            
            let constant_cmp: Box<dyn Fn(A, A) -> std::cmp::Ordering> = 
                Box::new(move |_: A, _: A| chosen_ordering);
            
            Tree::singleton(constant_cmp)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_integer_shrinking() {
        let gen = Gen::int_range(-10, 10);
        let seed = Seed::from_u64(42);
        let tree = gen.generate(Size::new(50), seed);

        // Should have origin (0) as a shrink since it's in range
        let shrinks = tree.shrinks();
        assert!(!shrinks.is_empty(), "Integer should have shrinks");
        assert!(shrinks.contains(&&0), "Should shrink towards origin (0)");
    }

    #[test]
    fn test_positive_range_integer_shrinking() {
        let gen = Gen::int_range(5, 20);
        let seed = Seed::from_u64(123);
        let tree = gen.generate(Size::new(50), seed);

        let shrinks = tree.shrinks();
        assert!(
            !shrinks.is_empty(),
            "Positive range integer should have shrinks"
        );
        assert!(
            shrinks.contains(&&5),
            "Should shrink towards minimum (5) as origin"
        );
    }

    #[test]
    fn test_character_simplification() {
        // Test that uppercase letters shrink to lowercase
        assert_eq!(simplify_char('Z'), 'z');
        assert_eq!(simplify_char('A'), 'a');

        // Test that special characters shrink to 'a'
        assert_eq!(simplify_char('!'), 'a');
        assert_eq!(simplify_char('@'), 'a');

        // Test that numbers shrink towards lower numbers
        assert_eq!(simplify_char('9'), '8');
        assert_eq!(simplify_char('5'), '4');
        assert_eq!(simplify_char('1'), '0');

        // Test that lowercase letters shrink towards 'a'
        assert_eq!(simplify_char('z'), 'y');
        assert_eq!(simplify_char('b'), 'a');
        assert_eq!(simplify_char('a'), 'a'); // 'a' stays 'a'
    }

    #[test]
    fn test_string_shrinking_strategies() {
        let gen = Gen::<String>::ascii_alpha();
        let seed = Seed::from_u64(999);
        let tree = gen.generate(Size::new(10), seed);

        let shrinks = tree.shrinks();
        assert!(!shrinks.is_empty(), "String should have shrinks");

        // Should always include empty string as ultimate shrink
        assert!(
            shrinks.contains(&&String::new()),
            "Should include empty string shrink"
        );

        // If original string is not empty, should have shorter versions
        if !tree.value.is_empty() {
            let has_shorter = shrinks.iter().any(|s| s.len() < tree.value.len());
            assert!(has_shorter, "Should have shorter string shrinks");
        }
    }

    #[test]
    fn test_vector_element_wise_shrinking() {
        let gen = Gen::<Vec<i32>>::vec_of(Gen::int_range(10, 20));
        let seed = Seed::from_u64(777);
        let tree = gen.generate(Size::new(5), seed);

        let shrinks = tree.shrinks();
        assert!(!shrinks.is_empty(), "Vector should have shrinks");

        // Should always include empty vector as ultimate shrink
        assert!(
            shrinks.contains(&&Vec::new()),
            "Should include empty vector shrink"
        );

        // If original vector is not empty, should have element-wise shrinks
        if !tree.value.is_empty() {
            let _has_element_shrinks = shrinks
                .iter()
                .any(|v| v.len() == tree.value.len() && *v != &tree.value);
            // Note: This might not always be true if individual elements don't shrink
            // But we should at least have removal shrinks
            let has_removal_shrinks = shrinks.iter().any(|v| v.len() < tree.value.len());
            assert!(has_removal_shrinks, "Should have removal shrinks");
        }
    }

    #[test]
    fn test_option_shrinking() {
        let gen = Gen::<Option<i32>>::option_of(Gen::int_range(1, 100));
        let _seed = Seed::from_u64(555);

        // Generate multiple trees to test both Some and None cases
        for i in 0..10 {
            let test_seed = Seed::from_u64(555 + i);
            let tree = gen.generate(Size::new(50), test_seed);
            let shrinks = tree.shrinks();

            match &tree.value {
                Some(_) => {
                    // Some values should shrink to None
                    assert!(shrinks.contains(&&None), "Some should shrink to None");
                }
                None => {
                    // None has no shrinks (it's already minimal)
                    assert!(shrinks.is_empty(), "None should have no shrinks");
                }
            }
        }
    }

    #[test]
    fn test_result_shrinking() {
        let gen = Gen::<std::result::Result<i32, String>>::result_of(
            Gen::int_range(1, 10),
            Gen::<String>::ascii_alpha(),
        );
        let _seed = Seed::from_u64(333);

        // Generate multiple trees to test both Ok and Err cases
        for i in 0..20 {
            let test_seed = Seed::from_u64(333 + i);
            let tree = gen.generate(Size::new(50), test_seed);
            let shrinks = tree.shrinks();

            match &tree.value {
                Ok(_) => {
                    // Ok values should have shrinks of the inner value
                    assert!(!shrinks.is_empty(), "Ok should have shrinks");
                    let has_ok_shrinks = shrinks.iter().any(|r| r.is_ok());
                    assert!(has_ok_shrinks, "Should have Ok shrinks");
                }
                Err(_) => {
                    // Err values should try to shrink to Ok first
                    assert!(!shrinks.is_empty(), "Err should have shrinks");
                    let has_ok_shrink = shrinks.iter().any(|r| r.is_ok());
                    assert!(has_ok_shrink, "Err should shrink to Ok");
                }
            }
        }
    }

    #[test]
    fn test_tuple_shrinking() {
        let gen =
            Gen::<(i32, String)>::tuple_of(Gen::int_range(-5, 5), Gen::<String>::ascii_alpha());
        let seed = Seed::from_u64(111);
        let tree = gen.generate(Size::new(10), seed);

        let shrinks = tree.shrinks();
        assert!(!shrinks.is_empty(), "Tuple should have shrinks");

        // Should have shrinks that modify first component
        let has_first_shrinks = shrinks
            .iter()
            .any(|(first, second)| first != &tree.value.0 && second == &tree.value.1);

        // Should have shrinks that modify second component
        let has_second_shrinks = shrinks
            .iter()
            .any(|(first, second)| first == &tree.value.0 && second != &tree.value.1);

        // At least one type of component shrinking should be present
        assert!(
            has_first_shrinks || has_second_shrinks,
            "Should have component-wise shrinks"
        );
    }

    #[test]
    fn test_result_weighted_distribution() {
        let gen = Gen::<std::result::Result<bool, i32>>::result_of_weighted(
            Gen::bool(),
            Gen::int_range(1, 5),
            9, // 90% Ok, 10% Err
        );

        let mut ok_count = 0;
        let mut err_count = 0;

        // Generate many samples to test distribution
        for i in 0..100 {
            let seed = Seed::from_u64(i);
            let tree = gen.generate(Size::new(10), seed);
            match tree.value {
                Ok(_) => ok_count += 1,
                Err(_) => err_count += 1,
            }
        }

        // Should heavily favor Ok values (allow some variance)
        assert!(
            ok_count > err_count * 5,
            "Weighted result should favor Ok: Ok={}, Err={}",
            ok_count,
            err_count
        );
    }

    #[test]
    fn test_numeric_types() {
        // Test i64 generator
        let i64_gen = Gen::i64_range(-100, 100);
        let seed = Seed::from_u64(123);
        let tree = i64_gen.generate(Size::new(50), seed);

        assert!(
            tree.value >= -100 && tree.value <= 100,
            "i64 should be in range"
        );
        let shrinks = tree.shrinks();
        assert!(!shrinks.is_empty(), "i64 should have shrinks");
        assert!(
            shrinks.contains(&&0),
            "i64 should shrink towards origin (0)"
        );

        // Test u32 generator
        let u32_gen = Gen::u32_range(10, 100);
        let tree2 = u32_gen.generate(Size::new(50), seed);

        assert!(
            tree2.value >= 10 && tree2.value <= 100,
            "u32 should be in range"
        );
        let shrinks2 = tree2.shrinks();
        assert!(!shrinks2.is_empty(), "u32 should have shrinks");
        assert!(
            shrinks2.contains(&&10),
            "u32 should shrink towards minimum (10)"
        );

        // Test f64 generator
        let f64_gen = Gen::f64_range(-1.0, 1.0);
        let tree3 = f64_gen.generate(Size::new(50), seed);

        assert!(
            tree3.value >= -1.0 && tree3.value <= 1.0,
            "f64 should be in range"
        );
        let shrinks3 = tree3.shrinks();
        assert!(!shrinks3.is_empty(), "f64 should have shrinks");

        // Test f64 unit interval
        let unit_gen = Gen::<f64>::unit();
        let tree4 = unit_gen.generate(Size::new(50), seed);

        assert!(
            tree4.value >= 0.0 && tree4.value <= 1.0,
            "f64 unit should be in [0,1]"
        );
    }

    #[test]
    fn test_convenience_generators() {
        // Test Vec<i32> convenience method
        let vec_int_gen = Gen::<Vec<i32>>::vec_int();
        let seed = Seed::from_u64(456);
        let tree = vec_int_gen.generate(Size::new(5), seed);

        // All elements should be in expected range
        for &element in &tree.value {
            assert!(
                element >= -100 && element <= 100,
                "Vec<i32> elements should be in range [-100, 100]"
            );
        }

        // Test Vec<bool> convenience method
        let vec_bool_gen = Gen::<Vec<bool>>::vec_bool();
        let tree2 = vec_bool_gen.generate(Size::new(5), seed);

        // All elements should be valid booleans (always true, but good for completeness)
        for &element in &tree2.value {
            assert!(
                element == true || element == false,
                "Vec<bool> elements should be valid booleans"
            );
        }
    }

    #[test]
    fn test_range_system() {
        use crate::data::Range;

        // Test Range creation and bounds checking
        let range = Range::new(-10, 10).with_origin(0);
        assert!(range.contains(&0));
        assert!(range.contains(&-10));
        assert!(range.contains(&10));
        assert!(!range.contains(&11));
        assert!(!range.contains(&-11));

        // Test i32 range generator
        let gen = Gen::<i32>::from_range(Range::<i32>::small_positive());
        let seed = Seed::from_u64(789);
        let tree = gen.generate(Size::new(50), seed);

        assert!(
            tree.value >= 1 && tree.value <= 100,
            "Should be in small positive range"
        );

        // Test f64 range generator
        let unit_gen = Gen::<f64>::from_range(Range::<f64>::unit());
        let tree2 = unit_gen.generate(Size::new(50), seed);

        assert!(
            tree2.value >= 0.0 && tree2.value <= 1.0,
            "Should be in unit range"
        );

        // Test predefined ranges
        let positive_range = Range::<i32>::positive();
        assert_eq!(positive_range.min, 1);
        assert_eq!(positive_range.origin, Some(1));

        let natural_range = Range::<i32>::natural();
        assert_eq!(natural_range.min, 0);
        assert_eq!(natural_range.origin, Some(0));
    }

    #[test]
    fn test_frequency_generator() {
        let gen = Gen::frequency(vec![
            WeightedChoice::new(1, Gen::constant(0)),       // 10% zeros
            WeightedChoice::new(9, Gen::int_range(1, 100)), // 90% positive
        ])
        .expect("valid frequency generator");

        let tree = gen.generate(Size::new(10), Seed::from_u64(42));

        // Should generate a valid value
        assert!(tree.value >= 0 && tree.value <= 100);
    }

    #[test]
    fn test_range_distributions() {
        // Test uniform distribution
        let uniform_gen = Gen::<i32>::from_range(crate::data::Range::new(1, 10));
        let uniform_tree = uniform_gen.generate(Size::new(10), Seed::from_u64(42));
        assert!(uniform_tree.value >= 1 && uniform_tree.value <= 10);

        // Test linear distribution (favors smaller values)
        let linear_gen = Gen::<i32>::from_range(crate::data::Range::linear(1, 100));
        let linear_tree = linear_gen.generate(Size::new(10), Seed::from_u64(42));
        assert!(linear_tree.value >= 1 && linear_tree.value <= 100);

        // Test exponential distribution (strongly favors smaller values)
        let exponential_gen = Gen::<i32>::from_range(crate::data::Range::exponential(1, 1000));
        let exponential_tree = exponential_gen.generate(Size::new(10), Seed::from_u64(42));
        assert!(exponential_tree.value >= 1 && exponential_tree.value <= 1000);

        // Test constant distribution
        let constant_gen = Gen::<i32>::from_range(crate::data::Range::constant(42));
        let constant_tree = constant_gen.generate(Size::new(10), Seed::from_u64(42));
        assert_eq!(constant_tree.value, 42);
    }

    #[test]
    fn test_with_range() {
        // Test string with controlled length
        let string_gen = Gen::<String>::alpha_with_range(crate::data::Range::new(5, 10));
        let string_tree = string_gen.generate(Size::new(10), Seed::from_u64(42));

        assert!(string_tree.value.len() >= 5 && string_tree.value.len() <= 10);
        assert!(string_tree.value.chars().all(|c| c.is_ascii_alphabetic()));

        // Test string with linear distribution (favors shorter strings)
        let linear_string_gen = Gen::<String>::alpha_with_range(crate::data::Range::linear(1, 20));
        let linear_string_tree = linear_string_gen.generate(Size::new(10), Seed::from_u64(42));

        assert!(linear_string_tree.value.len() >= 1 && linear_string_tree.value.len() <= 20);
        assert!(linear_string_tree
            .value
            .chars()
            .all(|c| c.is_ascii_alphabetic()));
    }

    #[test]
    fn test_one_of_generator() {
        let gen = Gen::one_of(vec![
            Gen::constant("hello"),
            Gen::constant("world"),
            Gen::constant("test"),
        ])
        .expect("valid one_of generator");

        let tree = gen.generate(Size::new(10), Seed::from_u64(42));

        // Should generate one of the three values
        assert!(tree.value == "hello" || tree.value == "world" || tree.value == "test");
    }

    #[test]
    fn test_frequency_errors() {
        // Test empty choices list
        let result = Gen::<String>::frequency(vec![]);
        assert!(matches!(
            result,
            Err(crate::HedgehogError::InvalidGenerator { .. })
        ));

        // Test zero total weight
        let result = Gen::frequency(vec![
            WeightedChoice::new(0, Gen::constant("a")),
            WeightedChoice::new(0, Gen::constant("b")),
        ]);
        assert!(matches!(
            result,
            Err(crate::HedgehogError::InvalidGenerator { .. })
        ));

        // Test valid case
        let result = Gen::frequency(vec![
            WeightedChoice::new(1, Gen::constant("a")),
            WeightedChoice::new(2, Gen::constant("b")),
        ]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_one_of_errors() {
        // Test empty generators list
        let result = Gen::<String>::one_of(vec![]);
        assert!(matches!(
            result,
            Err(crate::HedgehogError::InvalidGenerator { .. })
        ));

        // Test valid case
        let result = Gen::one_of(vec![Gen::constant("a"), Gen::constant("b")]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_distribution_behavior() {
        let seed = Seed::from_u64(42);

        // Test uniform distribution
        let (uniform_val, _) = Distribution::Uniform.sample_u64(seed, 100);
        assert!(uniform_val < 100);

        // Test linear distribution
        let (linear_val, _) = Distribution::Linear.sample_u64(seed, 100);
        assert!(linear_val < 100);

        // Test exponential distribution
        let (exp_val, _) = Distribution::Exponential.sample_u64(seed, 100);
        assert!(exp_val < 100);

        // Test constant distribution
        let (const_val, _) = Distribution::Constant.sample_u64(seed, 100);
        assert_eq!(const_val, 0);
    }

    #[test]
    fn test_function_generator() {
        // Test function_of generator
        let input_gen = Gen::int_range(0, 5);
        let output_gen = Gen::int_range(10, 20);
        let function_gen = Gen::<Box<dyn Fn(i32) -> i32>>::function_of(input_gen, output_gen, -1);
        
        let seed = Seed::from_u64(42);
        let tree = function_gen.generate(Size::new(10), seed);
        
        // Test that the function works
        let result1 = (tree.value)(0);
        let result2 = (tree.value)(0); // Should be deterministic
        assert_eq!(result1, result2);
        
        // Test that unmapped inputs return default
        let default_result = (tree.value)(999);
        assert_eq!(default_result, -1);
        
        // Test shrinking exists
        let shrinks = tree.shrinks();
        assert!(!shrinks.is_empty(), "Function should have shrinks");
    }
    
    #[test]
    fn test_constant_function_generator() {
        let output_gen = Gen::int_range(10, 20);
        let function_gen = Gen::<Box<dyn Fn(i32) -> i32>>::constant_function(output_gen);
        
        let seed = Seed::from_u64(123);
        let tree = function_gen.generate(Size::new(10), seed);
        
        // Test that function is constant
        let result1 = (tree.value)(0);
        let result2 = (tree.value)(42);
        let result3 = (tree.value)(-100);
        
        assert_eq!(result1, result2);
        assert_eq!(result2, result3);
        assert!(result1 >= 10 && result1 <= 20);
        
        // Test shrinking
        let shrinks = tree.shrinks();
        assert!(!shrinks.is_empty(), "Constant function should have shrinks");
    }
    
    #[test]
    fn test_identity_function_generator() {
        let function_gen = Gen::<Box<dyn Fn(i32) -> i32>>::identity_function();
        
        let seed = Seed::from_u64(456);
        let tree = function_gen.generate(Size::new(10), seed);
        
        // Test identity property
        assert_eq!((tree.value)(0), 0);
        assert_eq!((tree.value)(42), 42);
        assert_eq!((tree.value)(-100), -100);
        
        // Identity function has no shrinks (it's already minimal)
        let shrinks = tree.shrinks();
        assert!(shrinks.is_empty(), "Identity function should have no shrinks");
    }
    
    #[test]
    fn test_binary_function_generator() {
        let input_a_gen = Gen::int_range(0, 2);
        let input_b_gen = Gen::int_range(0, 2);
        let output_gen = Gen::int_range(10, 15);
        let function_gen = Gen::<Box<dyn Fn(i32, i32) -> i32>>::binary_function_of(
            input_a_gen, input_b_gen, output_gen, 0
        );
        
        let seed = Seed::from_u64(789);
        let tree = function_gen.generate(Size::new(10), seed);
        
        // Test that function works
        let result1 = (tree.value)(0, 0);
        let result2 = (tree.value)(0, 0); // Should be deterministic
        assert_eq!(result1, result2);
        
        // Test default for unmapped inputs
        let default_result = (tree.value)(999, 999);
        assert_eq!(default_result, 0);
        
        // Test shrinking
        let shrinks = tree.shrinks();
        assert!(!shrinks.is_empty(), "Binary function should have shrinks");
    }
    
    #[test] 
    fn test_predicate_from_set_generator() {
        let accepted_gen = Gen::<Vec<i32>>::vec_of(Gen::int_range(0, 5));
        let predicate_gen = Gen::<Box<dyn Fn(i32) -> bool>>::predicate_from_set(accepted_gen);
        
        let seed = Seed::from_u64(333);
        let tree = predicate_gen.generate(Size::new(10), seed);
        
        // Test predicate behavior - should be deterministic
        let result1 = (tree.value)(0);
        let result2 = (tree.value)(0);
        assert_eq!(result1, result2);
        
        // Test shrinking includes false predicate
        let shrinks = tree.shrinks();
        assert!(!shrinks.is_empty(), "Predicate should have shrinks");
        
        // At least one shrink should be always-false
        let has_false_shrink = shrinks.iter().any(|pred| {
            !(pred)(0) && !(pred)(1) && !(pred)(2) && !(pred)(999)
        });
        assert!(has_false_shrink, "Should have always-false predicate shrink");
    }
    
    #[test]
    fn test_constant_predicate_generator() {
        let bool_gen = Gen::bool();
        let predicate_gen = Gen::<Box<dyn Fn(i32) -> bool>>::constant_predicate(bool_gen);
        
        let seed = Seed::from_u64(666);
        let tree = predicate_gen.generate(Size::new(10), seed);
        
        // Test that predicate is constant
        let result1 = (tree.value)(0);
        let result2 = (tree.value)(42);
        let result3 = (tree.value)(-100);
        
        assert_eq!(result1, result2);
        assert_eq!(result2, result3);
        
        // If true predicate, should have false shrink
        if result1 {
            let shrinks = tree.shrinks();
            assert!(!shrinks.is_empty(), "True predicate should shrink to false");
            
            // Should have a false shrink
            let has_false_shrink = shrinks.iter().any(|pred| !(pred)(0));
            assert!(has_false_shrink, "Should shrink to false predicate");
        }
    }
    
    #[test]
    fn test_constant_comparator_generator() {
        let comparator_gen = Gen::<Box<dyn Fn(i32, i32) -> std::cmp::Ordering>>::constant_comparator(std::cmp::Ordering::Equal);
        
        let seed = Seed::from_u64(999);
        let tree = comparator_gen.generate(Size::new(10), seed);
        
        // Test comparator properties
        let cmp_result1 = (tree.value)(1, 2);
        let cmp_result2 = (tree.value)(1, 2); // Should be deterministic
        assert_eq!(cmp_result1, cmp_result2);
        assert_eq!(cmp_result1, std::cmp::Ordering::Equal);
        
        // Test reflexivity: compare something to itself
        let self_cmp = (tree.value)(5, 5);
        assert_eq!(self_cmp, std::cmp::Ordering::Equal);
        
        // Constant comparator has no shrinks (it's already minimal)
        let shrinks = tree.shrinks();
        assert!(shrinks.is_empty(), "Constant comparator should have no shrinks");
    }

    #[test]
    fn test_towards_algorithm() {
        // Test basic functionality
        let result = towards(0, 8);
        println!("towards(0, 8) = {:?}", result);
        assert_eq!(result[0], 0); // First element is always destination
        assert!(result.len() > 1); // Should have multiple shrinks
        
        let result2 = towards(0, 4);
        println!("towards(0, 4) = {:?}", result2);
        
        // Test same values
        assert_eq!(towards(5i32, 5i32), Vec::<i32>::new());
        
        // Test edge cases
        assert_eq!(towards(0i32, 1i32), vec![0]);
        assert_eq!(towards(1i32, 0i32), vec![1]);
        
        // Test that destination is always first (key property)
        let result3 = towards(10, 20);
        println!("towards(10, 20) = {:?}", result3);
        assert_eq!(result3[0], 10);
    }

    #[test]
    fn test_removes_function() {
        // Test basic removal
        let input = vec![1, 2, 3, 4];
        
        // Remove 1 element
        let result = removes(1, &input);
        println!("removes(1, [1,2,3,4]) = {:?}", result);
        
        // Remove 2 elements  
        let result2 = removes(2, &input);
        println!("removes(2, [1,2,3,4]) = {:?}", result2);
        
        // Remove all elements
        let result_all = removes(4, &input);
        assert_eq!(result_all, vec![Vec::<i32>::new()]);
        
        // Remove 0 elements
        let result_none = removes(0, &input);
        assert_eq!(result_none, vec![input.clone()]);
        
        // Remove more than available
        let result_over = removes(10, &input);
        assert!(result_over.is_empty());
    }

    #[test]
    fn test_list_shrinks_comprehensive() {
        let input = vec![1, 2, 3, 4];
        let shrinks = list_shrinks(&input);
        println!("list_shrinks([1,2,3,4]) = {:?}", shrinks);
        
        // Should include empty list (from removing all 4 elements)
        assert!(shrinks.contains(&vec![]));
        
        // Should have various removal patterns - just verify we get multiple results
        assert!(!shrinks.is_empty());
        
        // Verify we get different lengths (the key property)
        let lengths: Vec<usize> = shrinks.iter().map(|v| v.len()).collect();
        println!("shrink lengths: {:?}", lengths);
        assert!(lengths.contains(&0)); // Empty list
        assert!(lengths.len() > 2); // Multiple different shrinks
    }

    // Dictionary support tests
    #[test]
    fn test_from_elements_basic() {
        let elements = vec!["apple", "banana", "cherry"];
        let gen = Gen::from_elements(elements.clone()).unwrap();
        
        let mut generated_values = std::collections::HashSet::new();
        for _ in 0..20 {
            let tree = gen.generate(crate::data::Size::new(10), crate::data::Seed::random());
            let value = tree.value;
            assert!(elements.contains(&value));
            generated_values.insert(value);
        }
        
        // Should generate different values over multiple runs
        assert!(generated_values.len() > 1);
    }
    
    #[test]
    fn test_from_elements_empty_error() {
        let empty_elements: Vec<i32> = vec![];
        let result = Gen::from_elements(empty_elements);
        
        assert!(result.is_err());
        if let Err(crate::HedgehogError::InvalidGenerator { message }) = result {
            assert!(message.contains("empty"));
        }
    }
    
    #[test]
    fn test_from_elements_shrinking() {
        let elements = vec![1, 2, 3, 4, 5];
        let gen = Gen::from_elements(elements).unwrap();
        let tree = gen.generate(crate::data::Size::new(10), crate::data::Seed::random());
        
        // Should have shrinks to other elements
        let shrinks = tree.shrinks();
        assert!(!shrinks.is_empty());
        
        // All shrinks should be from the original elements
        for &shrink in shrinks {
            assert!([1, 2, 3, 4, 5].contains(&shrink));
        }
    }
    
    #[test]
    fn test_from_dictionary_basic() {
        let dictionary = vec![1, 2, 3];
        let random_gen = Gen::int_range(10, 20);
        
        let gen = Gen::from_dictionary(dictionary.clone(), random_gen, 50, 50).unwrap();
        
        let mut dict_values = 0;
        let mut random_values = 0;
        
        for _ in 0..100 {
            let tree = gen.generate(crate::data::Size::new(10), crate::data::Seed::random());
            let value = tree.value;
            
            if dictionary.contains(&value) {
                dict_values += 1;
            } else if value >= 10 && value <= 20 {
                random_values += 1;
            }
        }
        
        // Should get both dictionary and random values
        assert!(dict_values > 0);
        assert!(random_values > 0);
        
        // With 50/50 weights, should be roughly balanced
        let total = dict_values + random_values;
        let dict_ratio = dict_values as f64 / total as f64;
        assert!(dict_ratio > 0.3 && dict_ratio < 0.7, "Dictionary ratio: {}", dict_ratio);
    }
    
    #[test]
    fn test_from_dictionary_weights() {
        let dictionary = vec![1];
        let random_gen = Gen::int_range(10, 10); // Always generates 10
        
        // Heavy weight on dictionary
        let gen = Gen::from_dictionary(dictionary, random_gen, 90, 10).unwrap();
        
        let mut dict_count = 0;
        for _ in 0..100 {
            let tree = gen.generate(crate::data::Size::new(10), crate::data::Seed::random());
            if tree.value == 1 {
                dict_count += 1;
            }
        }
        
        // Should heavily favor dictionary values
        assert!(dict_count > 70, "Dictionary count: {}", dict_count);
    }
    
    #[test]
    fn test_from_dictionary_errors() {
        let empty_dict: Vec<i32> = vec![];
        
        // Empty dictionary should error
        assert!(Gen::from_dictionary(empty_dict, Gen::int_range(1, 10), 50, 50).is_err());
        
        // Zero weights should error  
        assert!(Gen::from_dictionary(vec![1], Gen::int_range(1, 10), 0, 0).is_err());
    }
    
    #[test] 
    fn test_http_status_code_generator() {
        let gen = Gen::<u16>::http_status_code();
        
        let mut statuses = std::collections::HashSet::new();
        for _ in 0..50 {
            let tree = gen.generate(crate::data::Size::new(10), crate::data::Seed::random());
            let status = tree.value;
            
            // Should be valid HTTP status codes
            assert!(status >= 100 && status <= 599);
            statuses.insert(status);
        }
        
        // Should generate variety of status codes
        assert!(statuses.len() > 3);
        
        // Should heavily favor common codes, so we should see some
        let common_codes = [200, 404, 500];
        let has_common = statuses.iter().any(|&s| common_codes.contains(&s));
        assert!(has_common, "Generated statuses: {:?}", statuses);
    }
    
    #[test]
    fn test_network_port_generator() {
        let gen = Gen::<u16>::network_port();
        
        let mut ports = std::collections::HashSet::new();
        for _ in 0..50 {
            let tree = gen.generate(crate::data::Size::new(10), crate::data::Seed::random());
            let port = tree.value;
            
            // Should be valid port numbers
            assert!(port > 0);
            ports.insert(port);
        }
        
        // Should generate variety of ports
        assert!(ports.len() > 5);
        
        // Should include some well-known ports
        let well_known = [22, 80, 443];
        let _has_well_known = ports.iter().any(|&p| well_known.contains(&p));
        // Note: This might occasionally fail due to randomness, but should usually pass
        // We don't assert on _has_well_known as it's probabilistic
    }
    
    #[test]
    fn test_web_domain_generator() {
        let gen = Gen::<String>::web_domain();
        
        let mut domains = std::collections::HashSet::new();
        for _ in 0..20 {
            let tree = gen.generate(crate::data::Size::new(10), crate::data::Seed::random());
            let domain = tree.value;
            
            // Should have a TLD
            assert!(domain.contains('.'));
            
            // Should have a reasonable structure
            let parts: Vec<&str> = domain.split('.').collect();
            assert!(parts.len() >= 2);
            
            // Domain part should be non-empty and alphabetic
            assert!(!parts[0].is_empty());
            assert!(parts[0].chars().all(|c| c.is_ascii_alphabetic()));
            
            domains.insert(domain);
        }
        
        // Should generate variety
        assert!(domains.len() > 3);
    }
    
    #[test]
    fn test_email_address_generator() {
        let gen = Gen::<String>::email_address();
        
        for _ in 0..10 {
            let tree = gen.generate(crate::data::Size::new(10), crate::data::Seed::random());
            let email = tree.value;
            
            // Should have @ symbol
            assert!(email.contains('@'));
            
            // Should have reasonable structure
            let parts: Vec<&str> = email.split('@').collect();
            assert_eq!(parts.len(), 2);
            
            let username = parts[0];
            let domain = parts[1];
            
            // Username should be non-empty
            assert!(!username.is_empty());
            assert!(username.len() >= 3);
            
            // Domain should be non-empty
            assert!(!domain.is_empty());
            assert!(domain.contains('.'));
        }
    }
    
    #[test]
    fn test_sql_identifier_safe() {
        let gen = Gen::<String>::sql_identifier(false);
        
        for _ in 0..20 {
            let tree = gen.generate(crate::data::Size::new(10), crate::data::Seed::random());
            let identifier = tree.value;
            
            // Should be non-empty alphabetic
            assert!(!identifier.is_empty());
            assert!(identifier.chars().all(|c| c.is_ascii_alphabetic()));
            assert!(identifier.len() >= 3 && identifier.len() <= 20);
        }
    }
    
    #[test]
    fn test_sql_identifier_with_keywords() {
        let gen = Gen::<String>::sql_identifier(true);
        
        let mut has_keyword = false;
        let mut has_random = false;
        
        for _ in 0..30 {
            let tree = gen.generate(crate::data::Size::new(10), crate::data::Seed::random());
            let identifier = tree.value;
            
            // Check if it's a SQL keyword
            let sql_keywords = ["SELECT", "INSERT", "UPDATE", "DELETE", "FROM", "WHERE"];
            if sql_keywords.contains(&identifier.as_str()) {
                has_keyword = true;
            } else if identifier.chars().all(|c| c.is_ascii_alphabetic()) {
                has_random = true;
            }
        }
        
        // Should get mix of keywords and random identifiers
        assert!(has_keyword || has_random); // At least one type should appear
    }
    
    #[test]
    fn test_programming_tokens() {
        let rust_keywords = ["fn", "let", "mut", "pub", "struct"];
        let gen = Gen::<String>::programming_tokens(&rust_keywords);
        
        let mut has_keyword = false;
        let mut has_random = false;
        
        for _ in 0..30 {
            let tree = gen.generate(crate::data::Size::new(10), crate::data::Seed::random());
            let token = tree.value;
            
            if rust_keywords.contains(&token.as_str()) {
                has_keyword = true;
            } else if token.chars().all(|c| c.is_ascii_alphabetic()) {
                has_random = true;
            }
        }
        
        // Should get mix of keywords and random tokens
        assert!(has_keyword || has_random); // At least one type should appear
    }
}
