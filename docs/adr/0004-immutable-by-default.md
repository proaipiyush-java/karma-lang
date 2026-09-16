# ADR 0004 — Immutable Bindings by Default

## Status

Accepted for v0.2.

## Decision

`let` creates an immutable binding. Mutation must be explicitly requested with `mut`.

## Rationale

Immutability reduces state complexity, improves reasoning, supports future concurrency guarantees, and provides stronger input to ownership/resource analysis.

## Defense in depth

Both the type checker and runtime environment enforce the mutability flag.
