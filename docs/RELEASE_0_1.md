# Karma v0.1 — Foundation Release

## Release objective

Establish a real, runnable Karma language pipeline with deliberately small semantics and explicit safety boundaries.

## Included

- `.kr` source files
- lexer/tokenizer
- recursive-descent parser
- AST
- interpreter backend
- lexical variable scopes
- `let` bindings and reassignment
- `Int`, `Bool`, `String`
- checked arithmetic
- comparisons and equality
- `if` / `else`
- `while`
- top-level `fn` functions
- recursion
- `return`
- `print()` built-in
- CLI runner
- structured lexer/parser/runtime diagnostics
- bootstrap execution/source safety limits
- unit tests in lexer, parser, and interpreter modules

## Exit criteria

- [x] Project structure defined
- [x] Language grammar documented
- [x] Lexer implemented
- [x] Parser implemented
- [x] AST implemented
- [x] Interpreter implemented
- [x] CLI implemented
- [x] Example programs included
- [x] Checked integer arithmetic enforced
- [x] `unsafe` implementation code forbidden at crate level
- [x] External Rust dependencies avoided
- [ ] Compile and execute the test suite on a Rust-enabled machine
- [ ] Record baseline binary size / startup / RSS / execution benchmarks

## Verification commands

```bash
cargo test
cargo run -- examples/hello.kr
cargo run -- examples/control_flow.kr
cargo run -- examples/functions.kr
```

Expected results:

```text
Hello from Karma!
```

```text
0
1
2
3
4
done
```

```text
3628800
```

## Definition of done

v0.1 becomes complete only after the code compiles cleanly, all tests pass, the three smoke examples produce the documented output, and baseline measurements are recorded.

The next release must not begin by adding enterprise libraries. v0.2 starts with the static type-system specification.
