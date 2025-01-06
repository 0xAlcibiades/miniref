---
id: monads-in-rust
title: Understanding Monads through Rust's Result and Option
tags:
  - rust
  - functional-programming
  - category-theory
  - monads
references:
  - rust-option-type
  - rust-result-type
  - category-theory-basics
  - functor-pattern
---

In Rust, we can understand monads through familiar types like `Option` and `Result`. A monad is a type that implements a specific set of operations with certain guarantees.

## Mathematical Definition

In category theory, a monad consists of three components:

1. A type constructor $M$
2. A unit function $\eta : A \rightarrow M(A)$
3. A bind operation $\mu : M(M(A)) \rightarrow M(A)$

These must satisfy the following laws:

$\mu \circ \eta_M = \mu \circ M\eta = id_M \quad \text{(unit laws)}$

and

$\mu \circ M\mu = \mu \circ \mu_M \quad \text{(associativity)}$

## Rust Implementation

In Rust, we can see this pattern in `Option`:

```rust
// The type constructor M is Option<T>
enum Option<T> {
    Some(T),
    None,
}

impl<T> Option<T> {
    // η (unit) - wrap a value
    fn unit(x: T) -> Option<T> {
        Some(x)
    }

    // µ (bind) - chain computations
    fn bind<U, F>(self, f: F) -> Option<U>
    where
        F: FnOnce(T) -> Option<U>
    {
        match self {
            Some(x) => f(x),
            None => None,
        }
    }
}
```

## Practical Example

Here's how we use this in practice:

```rust
fn divide(x: i32, y: i32) -> Option<i32> {
    if y == 0 {
        None
    } else {
        Some(x / y)
    }
}

fn add_one(x: i32) -> Option<i32> {
    Some(x + 1)
}

// Using bind to chain operations
let result = Some(10)
    .bind(|x| divide(x, 2))    // Some(5)
    .bind(|x| add_one(x));     // Some(6)
```

## Connection to Category Theory

The monad laws in Rust terms:

1. **Left Unit**: `Option::unit(x).bind(f) == f(x)`
2. **Right Unit**: `m.bind(Option::unit) == m`
3. **Associativity**: `m.bind(f).bind(g) == m.bind(|x| f(x).bind(g))`

## Notes

- The `?` operator in Rust is syntactic sugar for monadic bind with `Result`
- Similar patterns appear in other languages:
  - Haskell's `Maybe` type
  - Scala's `Option`
  - Swift's optionals

## Code Examples

Here's how we can chain multiple fallible operations:

```rust
fn complex_computation() -> Option<i32> {
    Some(42)
        .bind(|x| divide(x, 2))
        .bind(|x| if x > 10 { Some(x) } else { None })
        .bind(add_one)
}
```
