# Karma v0.3.0 — Ownership & Resource Foundation

## Objective

Introduce the first enforceable memory/resource model before native compilation. v0.3 must make ownership transfer visible to the compiler, reject use-after-move, allow read-only calls without transferring ownership, and define deterministic scope cleanup semantics.

## Added

- Copy vs owned type classification
- `String` as the first owned/non-Copy type
- Move-state tracking for variables
- Use-after-move diagnostics
- `borrow` function parameter mode
- Call-scoped borrowing
- Typed-AST `ValueAccess::{Copy, Move, Borrow}`
- Built-in `clone`
- Built-in `drop`
- Assignment returns `Unit`
- Mutable binding reinitialization after move
- Conservative ownership merging across `if`
- Loop-carried move detection for `while`
- Runtime moved-slot representation using `Option<Value>`
- Runtime move guards for defense in depth
- `Rc<str>` String backing in the bootstrap interpreter to avoid deep byte copies on read-only borrows
- Ownership-isolated function analysis

## Deliberately not added

- User-visible reference values
- Lifetime annotations
- Mutable borrowing
- User-defined destructors
- Arena syntax
- File/socket resource types
- Native code generation

Those features should build on the rules established here rather than precede them.

## Breaking changes from v0.2

1. `String` variable uses may move ownership.
2. Assignment expressions now have type `Unit`.
3. Functions cannot capture top-level runtime bindings in v0.3.
4. A non-Copy argument passed to a normal parameter is consumed.
5. Read-only parameters should be declared with `borrow`.

## Example

```karma
fn show(text: borrow String) -> Unit {
    print(text);
}

let language: String = "Karma";
show(language);
print(language);
```

## Exit criteria

- `cargo test` passes on the development machine
- `cargo build --release` passes
- `./scripts/smoke.sh` passes
- release binary reports `Karma 0.3.0`
- String use-after-move is rejected statically
- Copy values remain usable after assignment/pass
- borrowed parameters preserve caller ownership
- borrowed parameters cannot escape as owned returns
- clone preserves original availability
- drop invalidates owned source
- conditional move is conservatively tracked
- unsafe loop-carried move is rejected
- move + definite reinitialization in a mutable loop binding is accepted
