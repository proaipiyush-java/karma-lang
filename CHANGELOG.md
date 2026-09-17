# Karma Changelog

## v0.3.0 — Ownership & Resource Foundation

- Added Copy vs owned type classification.
- Added String move semantics and static use-after-move checking.
- Added `borrow` parameter mode for call-scoped read-only access.
- Added Typed AST Copy/Move/Borrow access annotations.
- Added `clone` and `drop` built-ins.
- Added deterministic lexical-scope cleanup semantics.
- Added conservative `if` ownership merge.
- Added loop-carried move analysis.
- Added mutable-binding reinitialization after move.
- Changed assignment expression type to Unit.
- Isolated function ownership analysis from top-level runtime bindings.
- Changed bootstrap String backing to `Rc<str>`.

## v0.2.0 — Static Type System

- Added static primitive types, symbol resolution, immutable-by-default bindings, explicit mutation, typed functions, Typed AST and `--check`.

## v0.1.0 — Foundation

- Added lexer, parser, AST, interpreter, CLI, control flow, functions, recursion and core runtime limits.
