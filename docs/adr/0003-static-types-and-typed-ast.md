# ADR 0003 — Static Types and Typed AST

## Status

Accepted for v0.2.

## Decision

Karma uses a mandatory static semantic-analysis pass before normal execution. The checker resolves symbols, validates types and mutability, and emits a Typed AST.

## Rationale

A Typed AST creates a stable semantic boundary for future memory analysis and KIR generation. It also catches invalid states before runtime.

## Consequence

The bootstrap interpreter now consumes Typed AST rather than raw AST.
