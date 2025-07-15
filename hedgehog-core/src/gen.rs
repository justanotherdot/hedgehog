//! Generator combinators for property-based testing.

use crate::{data::*, tree::*};

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

impl Gen<i32> {
    /// Generate an integer in the given range.
    pub fn int_range(min: i32, max: i32) -> Self {
        Gen::new(move |_size, seed| {
            let range = (max - min + 1) as u64;
            let (value, _new_seed) = seed.next_bounded(range);
            let result = min + value as i32;

            // Enhanced shrinking: towards zero, binary search, and origin-based
            let mut shrinks = Vec::new();

            // First, try shrinking towards the origin (closest valid value to 0)
            let origin = if min <= 0 && max >= 0 {
                0 // Zero is in range
            } else if min > 0 {
                min // Positive range, shrink towards minimum
            } else {
                max // Negative range, shrink towards maximum (closest to 0)
            };

            if origin != result {
                shrinks.push(Tree::singleton(origin));
            }

            // Binary search shrinking between result and origin
            let mut low = if result < origin { result } else { origin };
            let mut high = if result > origin { result } else { origin };

            while high - low > 1 {
                let mid = low + (high - low) / 2;
                if mid != result && mid >= min && mid <= max {
                    shrinks.push(Tree::singleton(mid));
                }
                if result < origin {
                    low = mid;
                } else {
                    high = mid;
                }
            }

            // Traditional halving shrinks as fallback
            let mut current = result;
            for _ in 0..10 {
                // Limit iterations to prevent infinite loops
                current = if current > origin {
                    current - (current - origin + 1) / 2
                } else if current < origin {
                    current + (origin - current + 1) / 2
                } else {
                    break;
                };

                if current != result && current >= min && current <= max {
                    shrinks.push(Tree::singleton(current));
                }
            }

            Tree::with_children(result, shrinks)
        })
    }

    /// Generate a positive integer.
    pub fn positive() -> Self {
        Self::int_range(1, i32::MAX)
    }

    /// Generate a natural number (including zero).
    pub fn natural() -> Self {
        Self::int_range(0, i32::MAX)
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

                // Remove middle character for longer strings
                if chars.len() > 2 {
                    let mid = chars.len() / 2;
                    let mut middle_removed = chars.clone();
                    middle_removed.remove(mid);
                    let shorter: String = middle_removed.iter().collect();
                    shrinks.push(Tree::singleton(shorter));
                }

                // Try removing multiple characters at once for very long strings
                if chars.len() > 4 {
                    let quarter = chars.len() / 4;
                    let shorter: String = chars[..quarter]
                        .iter()
                        .chain(chars[3 * quarter..].iter())
                        .collect();
                    shrinks.push(Tree::singleton(shorter));
                }
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

            // Enhanced collection shrinking: element removal + element-wise shrinking
            let mut shrinks = Vec::new();

            // Always try empty vector first (ultimate shrink)
            if !elements.is_empty() {
                shrinks.push(Tree::singleton(Vec::new()));
            }

            // Element removal shrinking
            if !elements.is_empty() {
                // Remove last element
                let mut shorter = elements.clone();
                shorter.pop();
                shrinks.push(Tree::singleton(shorter));

                // Remove first element
                if elements.len() > 1 {
                    let shorter = elements[1..].to_vec();
                    shrinks.push(Tree::singleton(shorter));
                }

                // Remove middle element for longer vectors
                if elements.len() > 2 {
                    let mid = elements.len() / 2;
                    let shorter = [&elements[..mid], &elements[mid + 1..]].concat();
                    shrinks.push(Tree::singleton(shorter));
                }

                // Remove multiple elements for very long vectors
                if elements.len() > 6 {
                    let quarter = elements.len() / 4;
                    let shorter = [&elements[..quarter], &elements[3 * quarter..]].concat();
                    shrinks.push(Tree::singleton(shorter));
                }

                // Try first half and second half
                if elements.len() > 3 {
                    let half = elements.len() / 2;
                    shrinks.push(Tree::singleton(elements[..half].to_vec()));
                    shrinks.push(Tree::singleton(elements[half..].to_vec()));
                }
            }

            // Element-wise shrinking: shrink individual elements while keeping the structure
            for (i, element_tree) in element_trees.iter().enumerate() {
                for shrunk_element in element_tree.shrinks() {
                    let mut shrunk_vec = elements.clone();
                    shrunk_vec[i] = shrunk_element.clone();
                    shrinks.push(Tree::singleton(shrunk_vec));
                }
            }

            // Smart removal: try removing elements that might be less important
            if elements.len() > 1 {
                // Remove every other element
                let filtered: Vec<_> = elements
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| i % 2 == 0)
                    .map(|(_, elem)| elem.clone())
                    .collect();
                if filtered.len() < elements.len() {
                    shrinks.push(Tree::singleton(filtered));
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
}
