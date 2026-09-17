# ADR 0008 — Shared Immutable String Backing in the Bootstrap Interpreter

## Status

Accepted for v0.3 bootstrap interpreter only.

## Decision

The Rust interpreter represents Karma `String` values using `Rc<str>`.

## Why

Read-only borrow operations should not deep-copy string bytes merely because the bootstrap interpreter needs a temporary runtime handle. Immutable shared backing makes borrowing cheap while ownership rules are enforced independently by the type checker and environment move state.

## Consequences

This is not the native Karma String ABI. KIR/native compilation may later use pointer/length/capacity or another representation. Language-level clone semantics remain logical semantics, not a promise about physical allocation strategy.
