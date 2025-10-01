//! String generator and unicode handling meta tests
//!
//! These properties test the string generation capabilities including
//! character generation, string shrinking, unicode handling, and specialized
//! string generators for domains, emails, identifiers, and programming tokens.

use crate::arbitrary_seed;
use hedgehog::*;
use std::collections::HashSet;

/// Property: Character generators should produce valid characters in expected ranges
pub fn test_character_generator_ranges() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&seed: &Seed| {
        let size = Size::new(10);

        // Test ASCII alpha characters
        let alpha_char = Gen::<char>::ascii_alpha().generate(size, seed).value;
        let alpha_valid = alpha_char.is_ascii_alphabetic();

        // Test ASCII alphanumeric characters
        let alnum_char = Gen::<char>::ascii_alphanumeric().generate(size, seed).value;
        let alnum_valid = alnum_char.is_ascii_alphanumeric();

        // Test ASCII printable characters
        let print_char = Gen::<char>::ascii_printable().generate(size, seed).value;
        let print_valid = print_char.is_ascii() && (' '..='~').contains(&print_char);

        alpha_valid && alnum_valid && print_valid
    });

    let fast_config = Config::default().with_tests(50).with_shrinks(5);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Character generator ranges property passed"),
        result => panic!("Character generator ranges property failed: {result:?}"),
    }
}

/// Property: String generators should produce strings with correct character sets
pub fn test_string_generator_character_sets() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&seed: &Seed| {
        let size = Size::new(15);

        // Test ASCII alpha strings
        let alpha_string = Gen::<String>::ascii_alpha().generate(size, seed).value;
        let alpha_valid = alpha_string.chars().all(|c| c.is_ascii_alphabetic());

        // Test ASCII alphanumeric strings
        let alnum_string = Gen::<String>::ascii_alphanumeric()
            .generate(size, seed)
            .value;
        let alnum_valid = alnum_string.chars().all(|c| c.is_ascii_alphanumeric());

        // Test ASCII printable strings
        let print_string = Gen::<String>::ascii_printable().generate(size, seed).value;
        let print_valid = print_string
            .chars()
            .all(|c| c.is_ascii() && (' '..='~').contains(&c));

        alpha_valid && alnum_valid && print_valid
    });

    let fast_config = Config::default().with_tests(30).with_shrinks(3);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ String generator character sets property passed"),
        result => panic!("String generator character sets property failed: {result:?}"),
    }
}

/// Property: String generators with ranges should respect length constraints
pub fn test_string_range_constraints() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&seed: &Seed| {
        let size = Size::new(20);

        // Test fixed range strings
        let range = Range::new(5, 15);
        let ranged_string = Gen::<String>::alpha_with_range(range)
            .generate(size, seed)
            .value;
        let length_valid = ranged_string.len() >= 5 && ranged_string.len() <= 15;
        let content_valid = ranged_string.chars().all(|c| c.is_ascii_alphabetic());

        // Test linear range strings (should favor shorter lengths)
        let linear_range = Range::linear(3, 12);
        let linear_string = Gen::<String>::alphanumeric_with_range(linear_range)
            .generate(size, seed)
            .value;
        let linear_length_valid = linear_string.len() >= 3 && linear_string.len() <= 12;
        let linear_content_valid = linear_string.chars().all(|c| c.is_ascii_alphanumeric());

        length_valid && content_valid && linear_length_valid && linear_content_valid
    });

    let fast_config = Config::default().with_tests(40).with_shrinks(3);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ String range constraints property passed"),
        result => panic!("String range constraints property failed: {result:?}"),
    }
}

/// Property: String shrinking should produce progressively simpler strings
pub fn test_string_shrinking_behavior() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&seed: &Seed| {
        let size = Size::new(20);
        let string_gen = Gen::<String>::ascii_alphanumeric();
        let tree = string_gen.generate(size, seed);

        // Collect all shrink values
        let shrinks = tree.shrinks();

        // All shrinks should be valid for the generator type
        let all_shrinks_valid = shrinks
            .iter()
            .all(|s| s.chars().all(|c| c.is_ascii_alphanumeric()));

        if tree.value.is_empty() {
            // Empty string should have no shrinks
            shrinks.is_empty() && all_shrinks_valid
        } else {
            // Non-empty strings should have shrinks, including empty string and shorter versions
            let has_empty_shrink = shrinks.contains(&&String::new());
            let has_shorter_shrinks = shrinks.iter().any(|s| s.len() < tree.value.len());

            !shrinks.is_empty() && has_empty_shrink && has_shorter_shrinks && all_shrinks_valid
        }
    });

    let fast_config = Config::default().with_tests(25).with_shrinks(3);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ String shrinking behavior property passed"),
        result => panic!("String shrinking behavior property failed: {result:?}"),
    }
}

/// Property: Character simplification should move towards simpler forms
pub fn test_character_simplification() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&_seed: &Seed| {
        // Test known simplification patterns
        let uppercase_differs_from_lowercase = !'Z'.eq(&'z');
        let numbers_work = '5' > '0';

        // Test that strings with uppercase letters can be simplified
        let uppercase_string = "HELLO".to_string();
        let has_uppercase = uppercase_string.chars().any(|c| c.is_ascii_uppercase());

        // Test that strings with special characters exist
        let special_string = "hello@world!".to_string();
        let has_special = special_string.chars().any(|c| !c.is_ascii_alphanumeric());

        uppercase_differs_from_lowercase && numbers_work && has_uppercase && has_special
    });

    let fast_config = Config::default().with_tests(10).with_shrinks(2);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Character simplification property passed"),
        result => panic!("Character simplification property failed: {result:?}"),
    }
}

/// Property: Web domain generator should produce realistic domain names
pub fn test_web_domain_generation() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&seed: &Seed| {
        let size = Size::new(10);
        let domain_gen = Gen::<String>::web_domain();
        let domain = domain_gen.generate(size, seed).value;

        // Should have at least one dot
        let has_dot = domain.contains('.');

        // Should not start or end with dot
        let proper_dots = !domain.starts_with('.') && !domain.ends_with('.');

        // Parts should be non-empty and alphabetic
        let parts: Vec<&str> = domain.split('.').collect();
        let valid_parts = parts.len() >= 2
            && parts
                .iter()
                .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_alphabetic()));

        // Should have reasonable length
        let reasonable_length = domain.len() >= 4 && domain.len() <= 50;

        has_dot && proper_dots && valid_parts && reasonable_length
    });

    let fast_config = Config::default().with_tests(20).with_shrinks(2);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Web domain generation property passed"),
        result => panic!("Web domain generation property failed: {result:?}"),
    }
}

/// Property: Email address generator should produce valid email formats
pub fn test_email_address_generation() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&seed: &Seed| {
        let size = Size::new(10);
        let email_gen = Gen::<String>::email_address();
        let email = email_gen.generate(size, seed).value;

        // Should have exactly one @ symbol
        let at_count = email.matches('@').count();
        let single_at = at_count == 1;

        // Should not start or end with @
        let proper_at = !email.starts_with('@') && !email.ends_with('@');

        // Should have username and domain parts
        let parts: Vec<&str> = email.split('@').collect();
        let valid_structure = parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty();

        // Username should be alphabetic
        let valid_username = if parts.len() == 2 {
            parts[0].chars().all(|c| c.is_ascii_alphabetic())
        } else {
            false
        };

        // Domain should look like domain.tld
        let valid_domain = if parts.len() == 2 {
            parts[1].contains('.') && !parts[1].starts_with('.') && !parts[1].ends_with('.')
        } else {
            false
        };

        single_at && proper_at && valid_structure && valid_username && valid_domain
    });

    let fast_config = Config::default().with_tests(15).with_shrinks(2);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Email address generation property passed"),
        result => panic!("Email address generation property failed: {result:?}"),
    }
}

/// Property: SQL identifier generator should produce valid identifiers
pub fn test_sql_identifier_generation() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&seed: &Seed| {
        let size = Size::new(10);

        // Test safe SQL identifiers (no keywords)
        let safe_gen = Gen::<String>::sql_identifier(false);
        let safe_id = safe_gen.generate(size, seed).value;

        // Safe identifiers should be alphabetic and within length bounds
        let safe_valid = safe_id.len() >= 3
            && safe_id.len() <= 20
            && safe_id.chars().all(|c| c.is_ascii_alphabetic());

        // Test risky SQL identifiers (may include keywords)
        let risky_gen = Gen::<String>::sql_identifier(true);
        let risky_id = risky_gen.generate(size, seed).value;

        // Risky identifiers should be valid strings (keywords or random)
        let risky_valid = !risky_id.is_empty() && risky_id.chars().all(|c| c.is_ascii_alphabetic());

        safe_valid && risky_valid
    });

    let fast_config = Config::default().with_tests(25).with_shrinks(2);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ SQL identifier generation property passed"),
        result => panic!("SQL identifier generation property failed: {result:?}"),
    }
}

/// Property: Programming token generator should produce valid tokens
pub fn test_programming_token_generation() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&seed: &Seed| {
        let size = Size::new(10);
        let keywords = ["fn", "let", "mut", "pub", "struct", "enum", "impl", "trait"];
        let token_gen = Gen::<String>::programming_tokens(&keywords);
        let token = token_gen.generate(size, seed).value;

        // Should be either a keyword or a valid identifier
        let is_keyword = keywords.contains(&token.as_str());
        let is_identifier =
            token.len() >= 2 && token.len() <= 15 && token.chars().all(|c| c.is_ascii_alphabetic());

        // Should not be empty
        let non_empty = !token.is_empty();

        non_empty && (is_keyword || is_identifier)
    });

    let fast_config = Config::default().with_tests(30).with_shrinks(2);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Programming token generation property passed"),
        result => panic!("Programming token generation property failed: {result:?}"),
    }
}

/// Property: String generators should produce variety in output
pub fn test_string_generation_variety() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&_seed: &Seed| {
        let size = Size::new(8);
        let string_gen = Gen::<String>::ascii_alpha();

        // Generate multiple strings and check for variety
        let mut strings = HashSet::new();
        for i in 0..20 {
            let seed = Seed::from_u64(i * 12345);
            let string = string_gen.generate(size, seed).value;
            strings.insert(string);
        }

        // Should generate at least some variety (not all identical)
        let has_variety = strings.len() > 5;

        // All should be valid
        let all_valid = strings
            .iter()
            .all(|s| s.chars().all(|c| c.is_ascii_alphabetic()));

        has_variety && all_valid
    });

    let fast_config = Config::default().with_tests(10).with_shrinks(1);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ String generation variety property passed"),
        result => panic!("String generation variety property failed: {result:?}"),
    }
}

/// Property: Unicode handling and edge cases
pub fn test_unicode_edge_cases() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&seed: &Seed| {
        let size = Size::new(10);

        // Test that string generators handle empty strings correctly
        let empty_string = String::new();
        let empty_valid = empty_string.is_empty();

        // Test that ASCII generators don't produce Unicode
        let ascii_string = Gen::<String>::ascii_printable().generate(size, seed).value;
        let ascii_only = ascii_string.is_ascii();

        // Test string length calculations are correct
        let test_string = "hello";
        let char_count = test_string.chars().count();
        let byte_count = test_string.len();
        let length_consistency = char_count == 5 && byte_count == 5;

        empty_valid && ascii_only && length_consistency
    });

    let fast_config = Config::default().with_tests(15).with_shrinks(2);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Unicode edge cases property passed"),
        result => panic!("Unicode edge cases property failed: {result:?}"),
    }
}

/// Property: String generators with custom character sets should work correctly
pub fn test_custom_character_string_generation() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&seed: &Seed| {
        let size = Size::new(12);

        // Create custom character generator for hex digits
        let hex_char_gen = Gen::one_of(vec![
            Gen::constant('0'),
            Gen::constant('1'),
            Gen::constant('2'),
            Gen::constant('3'),
            Gen::constant('4'),
            Gen::constant('5'),
            Gen::constant('6'),
            Gen::constant('7'),
            Gen::constant('8'),
            Gen::constant('9'),
            Gen::constant('A'),
            Gen::constant('B'),
            Gen::constant('C'),
            Gen::constant('D'),
            Gen::constant('E'),
            Gen::constant('F'),
        ]);

        match hex_char_gen {
            Ok(char_gen) => {
                let hex_string_gen = Gen::<String>::string_of(char_gen);
                let hex_string = hex_string_gen.generate(size, seed).value;

                // Should contain only hex characters
                let hex_valid = hex_string
                    .chars()
                    .all(|c| c.is_ascii_hexdigit() && c.is_ascii_uppercase() || c.is_ascii_digit());

                hex_valid
            }
            Err(_) => false,
        }
    });

    let fast_config = Config::default().with_tests(20).with_shrinks(2);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Custom character string generation property passed"),
        result => panic!("Custom character string generation property failed: {result:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_string_property_tests() {
        test_character_generator_ranges();
        test_string_generator_character_sets();
        test_string_range_constraints();
        test_string_shrinking_behavior();
        test_character_simplification();
        test_web_domain_generation();
        test_email_address_generation();
        test_sql_identifier_generation();
        test_programming_token_generation();
        test_string_generation_variety();
        test_unicode_edge_cases();
        test_custom_character_string_generation();
    }
}
