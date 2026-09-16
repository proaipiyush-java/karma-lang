# ADR-0001: Bootstrap Karma in Rust

- Status: Accepted for v0.1
- Decision: Implement the bootstrap compiler/interpreter in Rust.

## Rationale

Rust provides strong implementation-level memory safety, native binaries, low-level control, and a practical path to LLVM integration while avoiding a JVM runtime dependency. The implementation language does not define Karma's execution model: Karma's long-term target is native AOT compilation through KIR and a native backend.

## Constraints

- v0.1 core forbids `unsafe` Rust.
- v0.1 uses no external crates.
- self-hosting remains a long-term goal after the language becomes expressive and stable enough.
