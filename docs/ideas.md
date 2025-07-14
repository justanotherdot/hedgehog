# Hedgehog Rust Port Implementation Ideas

## Closure-Based Generators

The core insight is that generators are naturally functions, and closures might be the right abstraction for Rust.

### Why Closures Could Work

1. **Generators are functions** - At its core, a generator is `Size -> Seed -> Tree<T>`, which is naturally a closure
2. **Composability** - Closures can be composed: `Gen::map(gen, |x| f(x))` stores the closure for later execution  
3. **Lazy by nature** - Closures don't execute until called, giving us the lazy evaluation we need
4. **First-class values** - Closures are values you can pass around, store, and compose

### Potential Structure

```rust
struct Gen<T> {
    // Something like: Box<dyn Fn(Size, Seed) -> Tree<T>>
}

struct Tree<T> {
    value: T,
    children: Box<dyn Fn() -> Vec<Tree<T>>>,  // Lazy children via closure
}
```

### Why the Previous Rust Port Failed with Closures

- They used `Rc<RefCell<T>>` for sharing
- They tried to make everything lazy with complex lifetime management  
- They fought the ownership system instead of working with it

### What Might Work Better

- **Owned closures** (`Box<dyn Fn>`) instead of shared ones
- **Simple value generation, lazy shrinking** - don't over-complicate
- **Embrace move semantics** instead of complex sharing

### Key Insight

Use **closures for the generator functions themselves, not for complex lazy data structures**. 

The generator is the closure, the tree structure can be simpler:

```rust
struct Gen<T> {
    generator: Box<dyn Fn(Size, Seed) -> Tree<T>>,
}

impl<T> Gen<T> {
    fn map<U>(self, f: impl Fn(T) -> U + 'static) -> Gen<U> {
        Gen {
            generator: Box::new(move |size, seed| {
                let tree = (self.generator)(size, seed);
                Tree {
                    value: f(tree.value),
                    children: Box::new(move || {
                        tree.children().into_iter()
                            .map(|child| Tree {
                                value: f(child.value),
                                children: child.children,
                            })
                            .collect()
                    }),
                }
            }),
        }
    }
}
```

This preserves:
- **Explicit generators** as first-class values
- **Compositional** nature through combinator methods
- **Lazy evaluation** through closure-based children
- **No type-directed magic** - you explicitly choose and compose generators

## Tree Structure Ideas

### Option 1: Closure-Based Lazy Children
```rust
struct Tree<T> {
    value: T,
    children: Box<dyn Fn() -> Vec<Tree<T>>>,
}
```

### Option 2: Iterator-Based Lazy Children
```rust
struct Tree<T> {
    value: T,
    children: Box<dyn Iterator<Item = Tree<T>>>,
}
```

### Option 3: Streaming Children
```rust
struct Tree<T> {
    value: T,
    children: Box<dyn FnMut() -> Option<Tree<T>>>,
}
```

## Generation Strategy

### Size and Seed Handling
- Pass `Size` and `Seed` explicitly through the generator chain
- Use `Size` for recursive structure scaling
- Use `Seed` splitting for deterministic randomness

### Shrinking Strategy
- Build shrink trees during generation, not afterwards
- Use closure-based lazy evaluation for shrink children
- Preserve generator invariants during shrinking

## Traits with Generics Alternative

Could we use traits with generics to avoid type-direction while still getting abstraction?

### Approach 1: Generic Trait Methods
```rust
trait Generator {
    fn generate<T>(&self, size: Size, seed: Seed) -> Tree<T>;
    fn map<T, U>(self, f: impl Fn(T) -> U) -> Self where Self: Sized;
}

struct IntGen { range: Range<i32> }
struct ListGen<G> { element_gen: G, length: Range<usize> }

// Usage - still explicit!
let gen = IntGen { range: 1..=10 }
    .map(|x| x * 2)
    .into_list(0..=5);
```

### Approach 2: Parameterized Generators
```rust
trait Generator<T> {
    fn generate(&self, size: Size, seed: Seed) -> Tree<T>;
}

struct RangeGen<T> { range: Range<T> }
struct MapGen<G, T, U> { inner: G, f: fn(T) -> U }

impl<T> Generator<T> for RangeGen<T> {
    fn generate(&self, size: Size, seed: Seed) -> Tree<T> { /* ... */ }
}

// Usage - still explicit, but typed
let gen: RangeGen<i32> = RangeGen { range: 1..=10 };
let mapped = MapGen { inner: gen, f: |x| x * 2 };
```

### Approach 3: Generator Combinators
```rust
trait Generator<T> {
    fn generate(&self, size: Size, seed: Seed) -> Tree<T>;
    
    fn map<U>(self, f: impl Fn(T) -> U + 'static) -> MapGen<Self, T, U> 
    where Self: Sized 
    {
        MapGen { inner: self, f: Box::new(f) }
    }
    
    fn bind<U>(self, f: impl Fn(T) -> Box<dyn Generator<U>> + 'static) -> BindGen<Self, T, U>
    where Self: Sized
    {
        BindGen { inner: self, f: Box::new(f) }
    }
}

// Usage
let gen = RangeGen { range: 1..=10 }
    .map(|x| x * 2)
    .bind(|x| Box::new(RangeGen { range: 0..=x }));
```

### Does This Avoid Type-Direction?

**Yes, potentially!** The key differences from QuickCheck:

1. **Explicit construction** - You still explicitly create `RangeGen { range: 1..=10 }`
2. **No inference magic** - No hidden type class instances
3. **Multiple generators per type** - Can have `RangeGen<i32>` and `ChoiceGen<i32>` 
4. **Compositional** - Built through explicit combinator chains

### Trade-offs

**Pros:**
- Zero-cost abstractions (no `Box<dyn Fn>`)
- Type safety with explicit generators
- Familiar Rust patterns
- Good performance (static dispatch)

**Cons:**
- More complex types (`MapGen<BindGen<RangeGen<i32>, i32, String>, String, bool>`)
- Lifetime management with closures in traits
- May need trait objects for storage anyway

### Hybrid Approach?

Maybe combine both:
```rust
trait Generator<T> {
    fn generate(&self, size: Size, seed: Seed) -> Tree<T>;
}

struct Gen<T> {
    generator: Box<dyn Generator<T>>,
}

impl<T> Gen<T> {
    fn from_range(range: Range<T>) -> Self {
        Gen { generator: Box::new(RangeGen { range }) }
    }
    
    fn map<U>(self, f: impl Fn(T) -> U + 'static) -> Gen<U> {
        Gen { generator: Box::new(MapGen { inner: self.generator, f: Box::new(f) }) }
    }
}

// Usage - explicit and ergonomic
let gen = Gen::from_range(1..=10)
    .map(|x| x * 2)
    .bind(|x| Gen::from_range(0..=x));
```

## Open Questions

1. **Lifetime management** - How do we handle lifetimes with closures?
2. **Performance** - Will `Box<dyn Fn>` be efficient enough?
3. **Composition** - How do we handle complex generator composition?
4. **Determinism** - How do we ensure reproducible generation with closures?
5. **Trait approach** - Do traits with generics give us the best of both worlds?

## Next Steps

1. Prototype a simple closure-based generator
2. Test performance compared to type-directed approaches
3. Implement basic combinators (map, bind, choice)
4. Build a simple shrinking system
5. Compare with QuickCheck for ergonomics and performance