# Karma v0.1 Architecture

## Purpose

v0.1 is the bootstrap foundation. It proves the language pipeline without prematurely committing to the final native backend.

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
   AST
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
  main.rs          CLI + source-file safety boundary
  token.rs         token model
  lexer.rs         UTF-8 source -> tokens
  ast.rs           syntax tree
  parser.rs        recursive-descent parser
  value.rs         bootstrap runtime values
  environment.rs   lexical variable scopes
  interpreter.rs   AST evaluator + runtime limits
  error.rs         structured bootstrap diagnostics
```

## Boundaries preserved for later releases

The front end is deliberately separated so that v0.4 can replace the interpreter backend without replacing the lexer/parser/AST work:

```text
                       +--> Interpreter (v0.1)
Source -> Lexer -> Parser -> AST
                       +--> Typed AST -> KIR -> Native backend (future)
```

## Security baseline

1. The crate root uses `#![forbid(unsafe_code)]`.
2. No third-party dependencies are required for v0.1.
3. Arithmetic uses checked integer operations.
4. Source size, execution steps, call depth, and arity are bounded in the bootstrap runtime.
5. The interpreter does not expose filesystem, network, process, environment-variable, FFI, or shell capabilities to Karma code.
6. There are no background threads.

## Memory baseline

v0.1 is an interpreter and therefore is not the final memory model. However:

- functions are stored separately from variable environments, avoiding closure/environment reference cycles;
- scopes use reference-counted parent links with no child-to-parent cycle;
- no global tracing GC is introduced;
- there are no hidden worker pools or JIT structures;
- the implementation remains small enough to profile before ownership semantics are designed in v0.3.

## Deliberate non-goals

v0.1 does not define the final:

- static type system;
- ownership/borrowing model;
- KIR representation;
- ABI;
- native code generator;
- async runtime;
- package manager;
- enterprise libraries.

Those decisions will be introduced only after their invariants are documented.
