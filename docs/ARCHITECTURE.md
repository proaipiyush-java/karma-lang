# Karma v0.2 Architecture

## Purpose

v0.2 introduces semantic analysis. The compiler now proves basic type, symbol, mutability, call, condition, and return invariants before execution.

```text
.kr source
    |
    v
+---------+
|  Lexer  |
+---------+
    |
  Tokens
    |
    v
+---------+
| Parser  |
+---------+
    |
 Raw AST
    |
    v
+----------------+
|  Type Checker  |
| + symbols      |
| + mutability   |
| + returns      |
+----------------+
    |
 Typed AST
    |
    v
+-------------+
| Interpreter |
+-------------+
    |
  Output
```

## Source layout

```text
src/
  main.rs          CLI, source-file boundary, front-end orchestration
  source.rs        source positions
  token.rs         token model
  lexer.rs         source -> tokens
  types.rs         compiler-level static types
  ast.rs           raw syntax tree
  parser.rs        recursive-descent parser
  typed_ast.rs     semantically checked syntax tree
  type_checker.rs  symbols + static semantic rules
  value.rs         runtime values
  environment.rs   runtime lexical bindings + mutability
  interpreter.rs   Typed AST evaluator + runtime limits
  error.rs         diagnostics
```

## Compiler boundaries

```text
Source
  -> Lexer
  -> Parser
  -> Raw AST
  -> Type Checker
  -> Typed AST
       |
       +-> Interpreter (v0.2)
       |
       `-> KIR -> native backend (future)
```

The Typed AST is now the semantic boundary. Future native-code work should lower from Typed AST rather than re-deriving type rules in the backend.

## Static guarantees in v0.2

Before normal execution Karma verifies:

- referenced variables exist;
- assignment only targets mutable bindings;
- binding annotations match initializer types;
- operator operands have legal types;
- control-flow conditions are Bool;
- called functions exist;
- call arity matches;
- call argument types match;
- returned values match declared return types;
- non-Unit functions conservatively return on all paths;
- nested functions are rejected in v0.2.

## Runtime defense in depth

The runtime retains checks for invariants that should already be statically valid, including immutable assignment rejection. It also checks value-dependent failures such as integer overflow, division by zero, maximum execution steps, and maximum call depth.

## Future boundary

```text
Typed AST
   |
   v
  KIR
   |
 optimization
   |
   v
native backend
   |
   +-> ARM64
   +-> x86-64
   `-> future targets
```
