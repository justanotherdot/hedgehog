# Trophy Case 🏆

This document records bugs and issues found through property-based testing with Hedgehog.

## Bugs Found in Hedgehog Itself

### Integer Overflow in Range Generation

**Found**: 2025-08-09  
**Location**: `hedgehog-core/src/gen.rs` in `impl_numeric_gen!` macro  
**Issue**: Using extreme values like `i32::MAX` as upper bound caused integer overflow in `(max - min + 1) as u64` calculation  
**Symptom**: Tests panicked with "attempt to add with overflow"  
**Impact**: Affects users who use maximum integer values as range bounds  
**Discovery Method**: Meta testing framework validation  
**Status**: **FIXED** - Replaced overflow-prone arithmetic with saturating operations

### API Completeness Gaps

**Found**: 2025-08-09  
**Location**: Generator API surface  
**Issue**: Missing `tuple_of` for 3+ element tuples, no `from_range` for u64/usize types  
**Impact**: Makes certain generator patterns more verbose and requires workarounds  
**Discovery Method**: Meta testing framework implementation  
**Status**: **FIXED** - Added `tuple_of` for 3/4/5-element tuples and `from_range` for all integer types

### Map/Bind Combinator Recursion Issue

**Found**: 2025-08-10  
**Location**: Generator map/bind combinators  
**Issue**: Infinite recursion in Tree shrinking when using map/bind operations, causing "reached the recursion limit" compiler errors  
**Symptom**: Meta tests for combinator properties fail to compile with recursion limit exceeded  
**Impact**: Prevents effective use of map/bind combinators in property tests  
**Discovery Method**: Meta testing framework for combinator laws  
**Status**: **FIXED** - Added Clone bound to bind method and fixed reference passing

### Filter Implementation Bug

**Found**: 2025-08-10  
**Location**: `hedgehog-core/src/gen.rs` in `filter` method  
**Issue**: Filter fallback returns original unfiltered value when predicate fails, bypassing filter entirely  
**Symptom**: Filtered generators produce values that don't match the filter predicate  
**Impact**: Critical - makes filter combinator unreliable and unsafe  
**Discovery Method**: Meta testing framework for combinator properties  
**Status**: **FIXED** - Replaced unsafe fallback with proper retry mechanism using seed splitting and discard limits

### Additional Integer Overflow in Shrinking

**Found**: 2025-08-10  
**Location**: `hedgehog-core/src/gen.rs` in `towards` function  
**Issue**: Additional overflow issues in shrinking calculations when dealing with extreme integer values like `i32::MIN`  
**Symptom**: "attempt to subtract with overflow" in shrinking operations  
**Impact**: Prevented safe generation of extreme integer values  
**Discovery Method**: Meta testing after fixing the bind recursion issue  
**Status**: **FIXED** - Added safe subtraction with overflow detection and fallback mechanisms

## Instructions for Contributors

When you find a bug using Hedgehog, add it to this trophy case with:

- Date found
- Location in code
- Description of the issue
- How it was discovered
- Impact/severity
- Current status

This helps track the effectiveness of property-based testing and builds confidence in the approach.