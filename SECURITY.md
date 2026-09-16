# Karma Security Policy — Bootstrap Phase

Karma is pre-1.0 experimental software. Do not use v0.2 for security-critical production workloads.

## v0.2 security invariants

- the Rust bootstrap crate forbids `unsafe` code;
- normal execution always passes through static semantic analysis first;
- `let` bindings are immutable by default;
- mutation requires explicit `mut` and is checked both statically and at runtime;
- undefined symbols and invalid type combinations are rejected before execution;
- function arguments and returns are statically checked;
- control-flow conditions must be Bool;
- Karma programs receive no filesystem, network, process, shell, environment-variable, or FFI primitives;
- integer arithmetic is checked for overflow;
- source size is bounded by the CLI;
- interpreter execution steps and call depth are bounded;
- no third-party Rust dependencies are required.

## Important limitation

The v0.2 interpreter and type checker have not received an independent security audit. The ownership/resource-safety model is scheduled for v0.3.

## Reporting

Security findings should be reported privately to the project maintainer. Do not publish working exploit details before a fix is available.
