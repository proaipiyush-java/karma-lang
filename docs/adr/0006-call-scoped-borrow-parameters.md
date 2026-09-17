# ADR 0006 — Call-Scoped `borrow` Parameters

## Status

Accepted for v0.3.

## Decision

Karma v0.3 expresses read-only borrowing at the function-parameter boundary:

```karma
fn inspect(value: borrow String) -> Unit { ... }
```

The call site does not require `&` syntax. The compiler knows from the signature that ownership is not transferred.

Borrows are call-scoped and are not first-class values in v0.3. They cannot be stored or returned.

## Why

This gives common read-only API calls an ownership-safe model without exposing general lifetime parameters before the language needs them.

## Consequences

The model is intentionally less expressive than Rust references. Mutable borrowing and storable references remain future design work.
