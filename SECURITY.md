# Karma Security Policy — Bootstrap Phase

Karma is pre-1.0 experimental software. Do not use the v0.1 interpreter for security-critical production workloads.

## v0.1 security invariants

- compiler/interpreter crate forbids Rust `unsafe` code;
- Karma programs receive no filesystem, network, process, shell, environment-variable, or FFI primitives;
- integer arithmetic is checked;
- source size is bounded by the CLI;
- interpreter execution steps and call depth are bounded;
- no third-party Rust dependencies are present.

## Reporting

Until a public repository and security contact are established, security findings should remain private to the project maintainers rather than being published with working exploit details.
