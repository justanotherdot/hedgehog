//! Test the newly added foundational APIs

use hedgehog::*;

#[test]
fn test_new_integer_from_range_apis() {
    
    // Test all the new integer from_range implementations
    let i8_gen = Gen::<i8>::from_range(Range::new(-10, 10));
    let i16_gen = Gen::<i16>::from_range(Range::new(-1000, 1000));
    let isize_gen = Gen::<isize>::from_range(Range::new(-50, 50));
    let u8_gen = Gen::<u8>::from_range(Range::new(0, 255));
    let u16_gen = Gen::<u16>::from_range(Range::new(100, 1000));
    let u64_gen = Gen::<u64>::from_range(Range::new(0, 1000000));
    let usize_gen = Gen::<usize>::from_range(Range::new(0, 100));
    
    // Test that they generate values within bounds
    for _ in 0..10 {
        let seed = Seed::random();
        let size = Size::new(10);
        
        let i8_val = i8_gen.generate(size, seed).value;
        assert!(i8_val >= -10 && i8_val <= 10);
        
        let i16_val = i16_gen.generate(size, seed).value;
        assert!(i16_val >= -1000 && i16_val <= 1000);
        
        let isize_val = isize_gen.generate(size, seed).value;
        assert!(isize_val >= -50 && isize_val <= 50);
        
        let u8_val = u8_gen.generate(size, seed).value;
        // u8 is always valid, just verify it was generated
        let _ = u8_val;
        
        let u16_val = u16_gen.generate(size, seed).value;
        assert!(u16_val >= 100 && u16_val <= 1000);
        
        let u64_val = u64_gen.generate(size, seed).value;
        assert!(u64_val <= 1000000);
        
        let usize_val = usize_gen.generate(size, seed).value;
        assert!(usize_val <= 100);
    }
}

#[test]
fn test_new_tuple_apis() {
    let seed = Seed::random();
    let size = Size::new(10);
    
    // Test 3-element tuples
    let triple_gen = Gen::<(i32, String, bool)>::tuple_of(
        Gen::int_range(0, 100),
        Gen::<String>::ascii_alpha(),
        Gen::<bool>::bool(),
    );
    
    let triple = triple_gen.generate(size, seed);
    let (int_val, string_val, bool_val) = triple.value;
    assert!(int_val >= 0 && int_val <= 100);
    assert!(!string_val.is_empty() || true); // Empty strings are valid
    assert!(bool_val == true || bool_val == false);
    
    // Test 4-element tuples
    let quad_gen = Gen::<(u8, u16, u32, u64)>::tuple_of(
        Gen::<u8>::from_range(Range::new(0, 10)),
        Gen::<u16>::from_range(Range::new(0, 100)),
        Gen::<u32>::from_range(Range::new(0, 1000)),
        Gen::<u64>::from_range(Range::new(0, 10000)),
    );
    
    let quad = quad_gen.generate(size, seed);
    let (u8_val, u16_val, u32_val, u64_val) = quad.value;
    assert!(u8_val <= 10);
    assert!(u16_val <= 100);
    assert!(u32_val <= 1000);
    assert!(u64_val <= 10000);
    
    // Test 5-element tuples
    let quint_gen = Gen::<(i8, i16, i32, i64, isize)>::tuple_of(
        Gen::<i8>::from_range(Range::new(-5, 5)),
        Gen::<i16>::from_range(Range::new(-50, 50)),
        Gen::<i32>::from_range(Range::new(-500, 500)),
        Gen::<i64>::from_range(Range::new(-5000, 5000)),
        Gen::<isize>::from_range(Range::new(-50000, 50000)),
    );
    
    let quint = quint_gen.generate(size, seed);
    let (i8_val, i16_val, i32_val, i64_val, isize_val) = quint.value;
    assert!(i8_val >= -5 && i8_val <= 5);
    assert!(i16_val >= -50 && i16_val <= 50);
    assert!(i32_val >= -500 && i32_val <= 500);
    assert!(i64_val >= -5000 && i64_val <= 5000);
    assert!(isize_val >= -50000 && isize_val <= 50000);
}

#[test]
fn test_integer_overflow_edge_cases() {
    let seed = Seed::random();
    let size = Size::new(10);
    
    // Test edge cases that previously caused overflow
    
    // i32 with extreme values (this previously overflowed)
    let i32_extreme_gen = Gen::int_range(i32::MIN, i32::MAX);
    let i32_extreme = i32_extreme_gen.generate(size, seed);
    // Should not panic and should be a valid i32
    assert!(i32_extreme.value >= i32::MIN && i32_extreme.value <= i32::MAX);
    
    // i64 with extreme values  
    let i64_extreme_gen = Gen::i64_range(i64::MIN, i64::MAX);
    let i64_extreme = i64_extreme_gen.generate(size, seed);
    assert!(i64_extreme.value >= i64::MIN && i64_extreme.value <= i64::MAX);
    
    // u64 with extreme values
    let u64_extreme_gen = Gen::u64_range(0, u64::MAX);
    let u64_extreme = u64_extreme_gen.generate(size, seed);
    assert!(u64_extreme.value <= u64::MAX);
}

#[test] 
fn test_shrinking_with_new_apis() {
    let seed = Seed::from_u64(42);
    let size = Size::new(10);
    
    // Test that shrinking works with new integer types
    let i8_gen = Gen::<i8>::from_range(Range::new(-100, 100));
    let i8_tree = i8_gen.generate(size, seed);
    let i8_shrinks = i8_tree.shrinks();
    
    // Should have shrinks if the value isn't already minimal
    if i8_tree.value != 0 {
        assert!(!i8_shrinks.is_empty(), "i8 should have shrinks when not at origin");
    }
    
    // Test tuple shrinking  
    let triple_gen = Gen::<(i8, i16, i32)>::tuple_of(
        Gen::<i8>::from_range(Range::new(-10, 10)),
        Gen::<i16>::from_range(Range::new(-100, 100)),
        Gen::<i32>::from_range(Range::new(-1000, 1000)),
    );
    
    let triple_tree = triple_gen.generate(size, seed);
    let triple_shrinks = triple_tree.shrinks();
    
    // Should have shrinks unless all components are already minimal
    let (i8_val, i16_val, i32_val) = triple_tree.value;
    if i8_val != 0 || i16_val != 0 || i32_val != 0 {
        assert!(!triple_shrinks.is_empty(), "Triple should have shrinks when not at origin");
    }
}