# Karma Security Policy — Bootstrap Phase

Karma is pre-1.0 experimental software. **Do not use v0.3 for security-critical production workloads.** The ownership model is now enforceable for the current language subset, but the compiler/interpreter has not received an independent security audit.

## v0.3 security invariants

- the Rust bootstrap crate forbids `unsafe` code;
- normal execution always passes through type + ownership analysis first;
- `let` bindings are immutable by default;
- mutation requires explicit `mut`;
- `Int`, `Bool`, and `Unit` use Copy semantics;
- `String` is owned and use-after-move is rejected statically;
- read-only `borrow` parameters do not transfer caller ownership;
- borrowed owned values cannot be moved into owning destinations;
- `if` ownership states are conservatively merged;
- unsafe loop-carried ownership moves are rejected;
- runtime binding slots independently detect use-after-move;
- borrowed runtime parameters cannot be moved by the interpreter;
- undefined symbols and invalid type combinations are rejected before execution;
- function arguments and returns are statically checked;
- control-flow conditions must be Bool;
- Karma programs receive no filesystem, network, process, shell, environment-variable, or FFI primitives in v0.3;
- integer arithmetic is checked for overflow;
- source size is bounded by the CLI;
- interpreter execution steps and call depth are bounded;
- no third-party Rust dependencies are required.

## Important limitations

v0.3 does **not** yet provide:

- mutable borrowing;
- user-defined resource destructors;
- concurrency/data-race guarantees;
- native-code memory layout guarantees;
- sandboxed operating-system capabilities;
- independently audited compiler correctness.

The project must not claim production security until these areas and the relevant tooling/audits exist.

## Reporting

Security findings should be reported privately to the project maintainer. Do not publish working exploit details before a fix is available.
