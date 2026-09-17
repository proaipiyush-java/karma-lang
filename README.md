# Karma Programming Language — v0.3 Ownership & Resource Foundation

Karma is an experimental programming language focused on security, memory efficiency, predictable process behavior, native compilation, and long-term system compatibility.

**v0.3 introduces Karma's first real memory/resource semantics: Copy values, ownership transfer, call-scoped borrowing, explicit cloning, explicit early drop, deterministic scope cleanup, and conservative ownership analysis across control flow.**

## Compiler pipeline

```text
.kr source
   ↓
Lexer
   ↓
Tokens
   ↓
Parser
   ↓
Raw AST
   ↓
Type + Ownership Checker
   ↓
Typed AST with access modes
   ↓
Interpreter (bootstrap backend)
   ↓
Output
```

The Typed AST now records whether each variable use is a `Copy`, `Move`, or `Borrow`. This is designed to become an input to KIR/native lowering later.

## New in v0.3

- `Int`, `Bool`, and `Unit` are Copy types.
- `String` is the first owned/non-Copy type.
- Using an owned value in an owning context transfers ownership.
- Use-after-move is a compile-time error.
- Function parameters are owning by default.
- `borrow` parameters read owned values without taking ownership.
- Borrows are call-scoped in v0.3: they cannot be stored or returned.
- `clone(x)` creates another logical owner without moving `x`.
- `drop(x)` explicitly consumes an owned value early.
- Scope exit deterministically releases values still owned by that scope.
- Mutable owned bindings may be reinitialized after their old value was moved.
- `if` ownership states are conservatively merged.
- `while` rejects loop-carried moves of outer owned values unless ownership is definitely restored by the end of the body.
- Runtime move/borrow guards are retained as defense in depth.
- Bootstrap String backing uses `Rc<str>` so read-only borrow operations do not duplicate string bytes.

## Why this is not Rust syntax

Karma deliberately uses parameter-level borrowing:

```karma
fn inspect(text: borrow String) -> Unit {
    print(text);
}
```

At the call site:

```karma
let name: String = "Karma";
inspect(name);
print(name); // still valid
```

The function signature tells the compiler that the argument is read-only for the duration of the call. v0.3 does not expose storable reference values or lifetime annotations.

## Move example

```karma
let first: String = "Karma";
let second: String = first;

print(second); // OK
print(first);  // compile-time error: first was moved
```

## Copy example

```karma
let a: Int = 42;
let b: Int = a;

print(a); // OK
print(b); // OK
```

## Borrow example

```karma
fn show(value: borrow String) -> Unit {
    print(value);
}

let language: String = "Karma";
show(language);
print(language); // owner remains available
```

## Explicit clone

```karma
let first: String = "Karma";
let second: String = clone(first);

print(first);
print(second);
```

## Explicit early release

```karma
let temporary: String = "temporary";
drop(temporary);

// print(temporary); // static use-after-move error
```

## Reinitialization after move

```karma
mut value: String = "old";
let previous: String = value;

value = "new";

print(previous);
print(value);
```

## Build and test

```bash
cargo test
cargo build --release
./scripts/smoke.sh
```

## Static check only

```bash
./target/release/karma --check examples/ownership.kr
```

## Run

```bash
./target/release/karma examples/ownership.kr
```

## Install locally

```bash
./scripts/install-local.sh
```

If needed:

```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

Then:

```bash
karma --version
karma --check examples/ownership.kr
karma examples/ownership.kr
```

## Version direction

```text
v0.1  Lexer + parser + AST + interpreter
v0.2  Static type system + Typed AST
v0.3  Ownership + borrowing + deterministic cleanup  ← current
v0.4  KIR + native compilation
```

Read `docs/V0_3_DEEP_DIVE.md` for a beginner-first but technically detailed explanation of the memory model and every compiler change.
