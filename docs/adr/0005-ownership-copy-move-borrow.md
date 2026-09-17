# ADR 0005 — Ownership with Copy, Move and Borrow Access Modes

## Status

Accepted for v0.3.

## Decision

Karma distinguishes Copy values from owned values. `Int`, `Bool` and `Unit` are Copy in v0.3. `String` is owned. An owned value used in an owning context is moved. Read-only contexts borrow it.

The Typed AST records `ValueAccess::Copy`, `ValueAccess::Move` or `ValueAccess::Borrow` so later backends do not need to rediscover ownership intent.

## Why

This avoids hidden deep copies, establishes deterministic ownership before native compilation, and creates a model that can later apply to files, sockets and buffers.

## Consequences

Use-after-move becomes a static error. Some v0.2 String programs become invalid and must borrow, clone or restructure ownership.
