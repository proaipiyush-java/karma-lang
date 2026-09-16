# Karma Programming Language — v0.2 Static Type System

Karma is an experimental programming language focused on security, memory efficiency, predictable process behavior, native compilation, and long-term system compatibility.

**v0.2 adds compile-time semantic analysis and a real Typed AST.**

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
Type Checker / Symbol Resolution
   ↓
Typed AST
   ↓
Interpreter (bootstrap backend)
   ↓
Output
```

The interpreter is still the execution backend in v0.2. The Typed AST is the foundation for KIR and native compilation in later releases.

## New in v0.2

- Static primitive types: `Int`, `Bool`, `String`, `Unit`
- Optional local type inference
- Typed function parameters and return types
- `let` is immutable by default
- `mut` explicitly opts into mutation
- Compile-time symbol resolution
- Compile-time operator type checking
- Compile-time function argument checking
- Compile-time return type checking
- Boolean-only `if` and `while` conditions
- Basic all-path return analysis for non-`Unit` functions
- A real Typed AST
- `karma --check file.kr` to type-check without execution
- Source positions carried into type errors
- Runtime mutability checks retained as defense-in-depth

## Example

```karma
fn factorial(n: Int) -> Int {
    if n <= 1 {
        return 1;
    }
    return n * factorial(n - 1);
}

let language: String = "Karma";
mut count: Int = 0;
count = count + 1;

print(language);
print(factorial(10));
print(count);
```

## Compile-time failures

This is invalid:

```karma
let age: Int = "thirty eight";
```

Karma rejects it before the interpreter runs:

```text
type error: initializer for 'age' expects Int, found String
```

This is also invalid because `let` is immutable:

```karma
let count: Int = 0;
count = 1;
```

Use explicit mutation:

```karma
mut count: Int = 0;
count = 1;
```

## Build and test

```bash
cargo test
cargo build --release
./scripts/smoke.sh
```

## Type-check only

```bash
./target/release/karma --check examples/hello.kr
```

Expected:

```text
Karma check: OK
```

## Run

```bash
./target/release/karma examples/functions.kr
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
karma --check examples/functions.kr
karma examples/functions.kr
```

## Version direction

```text
v0.1  Lexer + parser + AST + interpreter
v0.2  Static type system + Typed AST       ← current
v0.3  Memory/resource model
v0.4  KIR + native compilation
```

Read `docs/V0_2_DEEP_DIVE.md` for a beginner-first explanation of every new compiler concept.
