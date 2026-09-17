# Karma Design Constitution

These are project-level constraints. A release proposal that violates one must include an explicit architecture decision record and justification.

1. **Security by construction.** Safe Karma code must not gain arbitrary memory, filesystem, network, process, FFI, or secret access implicitly.
2. **Memory efficiency is a language concern.** Allocation behavior must be measurable and, where practical, visible or inferable from source semantics.
3. **No mandatory JVM, JIT, or heavyweight VM.** The production target is ahead-of-time native code; WASM may be an additional backend.
4. **No mandatory tracing GC.** The final memory model should favor deterministic reclamation; any GC facility must be explicit/optional if introduced.
5. **No hidden concurrency.** A program that does not request concurrency should not acquire worker threads merely by using the language runtime.
6. **Unsafe operations are explicit and auditable.** The safe language cannot silently fall through to unchecked memory behavior.
7. **Checked behavior by default.** Integer overflow, invalid conversions, resource errors, and bounds violations must have defined semantics.
8. **Small trusted core.** Minimize compiler/runtime code that must be trusted for memory and security guarantees.
9. **Capability-oriented I/O.** Filesystem, network, process, environment, and other ambient authority should become explicit capabilities.
10. **Old and new systems matter.** Define conservative CPU/OS baselines and permit optimized dispatch on newer hardware without changing source code.
11. **Predictability before cleverness.** Startup time, resident memory, allocation count, and latency are tracked alongside throughput.
12. **Compatibility is a product feature.** Language, package, ABI, and serialized-data compatibility policies must be written before 1.0.
13. **Reproducible builds are a target.** Toolchain, dependencies, and build inputs must be lockable and verifiable.
14. **Diagnostics are part of correctness.** Security and type errors should explain what invariant was violated and how to fix it.
15. **Enterprise features build on language invariants.** HTTP, database, messaging, AI, and framework work cannot bypass the core safety model.
16. **Ownership transfer is explicit in semantics.** Non-Copy resources must not be silently duplicated merely because a variable is assigned or passed.
17. **Borrowing starts narrow.** Read-only borrowing is call-scoped in v0.3; general references/lifetimes are added only when concrete requirements justify the complexity.
18. **Resource cleanup is deterministic by default.** Values/resources still owned by a lexical scope are reclaimed when ownership ends; future resource APIs must preserve this principle.
