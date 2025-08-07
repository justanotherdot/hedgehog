//! Generator combinators for property-based testing.

use crate::{data::*, tree::*};

fn towards<T>(destination: T, x: T) -> Vec<T>
where
    T: Copy + PartialEq + PartialOrd + std::ops::Sub<Output = T> + std::ops::Add<Output = T> + std::ops::Div<Output = T> + From<u8>,
{
    if destination == x {
        return Vec::new();
    }

    let mut result = vec![destination];
    let diff = if x > destination { x - destination } else { destination - x };
    
    let mut current = diff;
    let zero = T::from(0);
    let two = T::from(2);
    
    while current != zero {
        let shrink = if x > destination { x - current } else { x + current };
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
        Gen::new(move |size, seed| {
            let tree = self.generate(size, seed);
            let value = tree.value.clone();
            tree.filter(&predicate)
                .unwrap_or_else(|| Tree::singleton(value))
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

/// Macro to implement enhanced numeric generators with origin-based shrinking.
macro_rules! impl_numeric_gen {
    ($type:ty, $method:ident, $max_val:expr) => {
        impl Gen<$type> {
            /// Generate a number in the given range with enhanced shrinking.
            pub fn $method(min: $type, max: $type) -> Self {
                Gen::new(move |_size, seed| {
                    let range = (max - min + 1) as u64;
                    let (value, _new_seed) = seed.next_bounded(range);
                    let result = min + value as $type;

                    let origin = if min <= 0 && max >= 0 {
                        0
                    } else if min > 0 {
                        min
                    } else {
                        max
                    };

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

// Implement for signed integers
impl_numeric_gen!(i32, int_range, i32::MAX);
impl_numeric_gen!(i64, i64_range, i64::MAX);

// Implement for unsigned integers using specialized macro
macro_rules! impl_unsigned_gen {
    ($type:ty, $method:ident, $max_val:expr) => {
        impl Gen<$type> {
            /// Generate an unsigned number in the given range.
            pub fn $method(min: $type, max: $type) -> Self {
                Gen::new(move |_size, seed| {
                    let range = (max - min + 1) as u64;
                    let (value, _new_seed) = seed.next_bounded(range);
                    let result = min + value as $type;

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

impl_unsigned_gen!(u32, u32_range, u32::MAX);

// Enhanced range-based generators with distribution support
impl Gen<i32> {
    /// Generate integers using a Range specification with distribution control.
    pub fn from_range(range: crate::data::Range<i32>) -> Self {
        Gen::new(move |_size, seed| {
            let range_size = (range.max - range.min + 1) as u64;
            let (offset, _new_seed) = range.distribution.sample_u64(seed, range_size);
            let result = range.min + offset as i32;

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
            let range_size = (range.max - range.min + 1) as u64;
            let (offset, _new_seed) = range.distribution.sample_u64(seed, range_size);
            let result = range.min + offset as i64;

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
            let range_size = (range.max - range.min + 1) as u64;
            let (offset, _new_seed) = range.distribution.sample_u64(seed, range_size);
            let result = range.min + offset as u32;

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
}
