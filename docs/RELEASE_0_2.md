# Karma v0.2.0 — Static Type System

## Objective

Move invalid-program detection from runtime into a semantic analysis phase and establish a Typed AST suitable for future lowering into KIR.

## Added

- Compiler-level `Type` model
- `Int`, `Bool`, `String`, `Unit`
- Type annotations using `:`
- Function return type syntax using `->`
- Immutable `let`
- Explicit mutable `mut`
- Symbol scopes
- Function signature pre-registration
- Static function argument checking
- Static return checking
- All-path return analysis for non-Unit functions
- Boolean-only conditions
- Typed AST
- Source positions in AST/Typed AST nodes
- `karma --check`
- Runtime mutability defense-in-depth

## Breaking syntax change

v0.1:

```karma
fn add(a, b) {
    return a + b;
}
```

v0.2:

```karma
fn add(a: Int, b: Int) -> Int {
    return a + b;
}
```

Pre-1.0 Karma does not promise syntax stability. Breaking changes must be documented and intentional.

## Exit criteria

- `cargo test` passes
- `cargo build --release` passes
- valid examples pass `--check`
- valid examples execute correctly
- type mismatch example fails before execution
- immutable assignment example fails before execution
- recursive typed functions work
- release binary reports `Karma 0.2.0`
