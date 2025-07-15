//! Demo of tree rendering capabilities.

use hedgehog_core::*;

fn main() {
    println!("Tree Rendering Demo");
    println!("===================");
    println!();

    // Example 1: Integer with shrinking
    println!("1. Integer Generator (range 1-20)");
    let int_gen = Gen::int_range(1, 20);
    let seed = Seed::from_u64(42);
    let tree = int_gen.generate(Size::new(10), seed);

    println!("Generated: {}", tree.value);
    println!();
    println!("Full tree structure:");
    print!("{}", tree.render());
    println!("Compact: {}", tree.render_compact());
    println!("Shrinks: {}", tree.render_shrinks());
    println!("Numbered:");
    print!("{}", tree.render_numbered());
    println!();

    // Example 2: String with character shrinking
    println!("2. String Generator");
    let string_gen = Gen::<String>::ascii_alpha();
    let string_tree = string_gen.generate(Size::new(4), Seed::from_u64(1));

    println!("Generated: '{}'", string_tree.value);
    println!("Compact: {}", string_tree.render_compact());
    println!("Shrinks: {}", string_tree.render_shrinks());
    println!();

    // Example 3: Vector with element shrinking
    println!("3. Vector Generator");
    let vec_gen = Gen::<Vec<i32>>::vec_of(Gen::int_range(5, 15));
    let vec_tree = vec_gen.generate(Size::new(3), Seed::from_u64(456));

    println!("Generated: {:?}", vec_tree.value);
    println!("Shrink count: {}", vec_tree.shrinks().len());
    // Note: Vec<i32> doesn't implement Display so we can't use render() methods
    println!();

    // Example 4: Show different numeric types
    println!("4. Different Numeric Types");

    // f64 with simple value shrinking
    let f64_gen = Gen::f64_range(-2.0, 2.0);
    let f64_tree = f64_gen.generate(Size::new(10), Seed::from_u64(789));
    println!("f64: {} -> {}", f64_tree.value, f64_tree.render_shrinks());

    // u32 shrinking towards minimum
    let u32_gen = Gen::u32_range(10, 50);
    let u32_tree = u32_gen.generate(Size::new(10), Seed::from_u64(999));
    println!("u32: {} -> {}", u32_tree.value, u32_tree.render_shrinks());
    println!();

    // Example 5: Boolean with no shrinking (singleton)
    println!("5. Boolean Generator");
    let bool_gen = Gen::bool();
    let bool_tree = bool_gen.generate(Size::new(10), Seed::from_u64(123));
    println!("Boolean: {}", bool_tree.render_shrinks());
    println!();

    // Example 6: Character with simplification
    println!("6. Character Generator");
    let char_gen = Gen::<char>::ascii_alphanumeric();
    let char_tree = char_gen.generate(Size::new(10), Seed::from_u64(456));
    println!("Character: {}", char_tree.render_shrinks());
    println!();

    println!("Demo complete!");
}
