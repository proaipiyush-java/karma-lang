# Karma v0.2 — Static Type System Deep Dive

This document explains v0.2 from first principles. It assumes you understand basic programming, but it does **not** assume compiler knowledge.

---

## 1. What problem are we solving?

Karma v0.1 could understand and execute code, but many mistakes were discovered only when that code actually ran.

Example:

```karma
let age = "hello";
print(age - 1);
```

In an interpreter-only design, the program may begin running and fail only when `age - 1` executes.

For production systems, we want a stronger rule:

> If the compiler can prove that a program is invalid, the program must not start.

That is the purpose of Karma v0.2.

---

## 2. The v0.1 pipeline

```text
SOURCE
  ↓
LEXER
  ↓
TOKENS
  ↓
PARSER
  ↓
AST
  ↓
INTERPRETER
  ↓
OUTPUT
```

The parser could tell whether syntax was structurally valid, but it did not know enough about *meaning*.

It knew this was syntactically shaped like addition:

```karma
1 + "hello"
```

but syntax alone does not answer whether `Int + String` should be legal.

---

## 3. The v0.2 pipeline

We insert a semantic phase:

```text
SOURCE
  ↓
LEXER
  ↓
TOKENS
  ↓
PARSER
  ↓
RAW AST
  ↓
SYMBOL RESOLUTION + TYPE CHECKING
  ↓
TYPED AST
  ↓
INTERPRETER
  ↓
OUTPUT
```

This is one of the most important architectural changes in the language so far.

---

## 4. Syntax versus semantics

### Syntax asks:

> Is the program written in a structurally valid form?

For example:

```karma
let age: Int = 38;
```

The parser checks things such as:

- `let` is followed by a variable name
- `:` is followed by a type
- `=` is present
- the statement ends in `;`

### Semantics asks:

> Does the program *make sense* according to Karma's rules?

For example:

```karma
let age: Int = "hello";
```

This is syntactically valid.

But semantically invalid because:

```text
expected: Int
actual:   String
```

That distinction is why a type checker exists after the parser.

---

## 5. The first Karma static types

v0.2 introduces four compiler-level types:

```text
Int
Bool
String
Unit
```

### Int

Signed 64-bit integer in the v0.2 runtime.

```karma
let age: Int = 38;
```

### Bool

Exactly `true` or `false`.

```karma
let ready: Bool = true;
```

### String

Text data.

```karma
let name: String = "Karma";
```

### Unit

Represents "no meaningful result".

A function that performs an action but does not return a value can declare:

```karma
fn greet(name: String) -> Unit {
    print("Hello, " + name);
}
```

`Unit` is conceptually similar to "nothing useful to return", but it is still a real type.

---

## 6. Why types exist separately from runtime values

This distinction is critical.

At compile time:

```text
Type::Int
```

means:

> The compiler has proven that this expression produces an integer.

At runtime:

```text
Value::Int(38)
```

means:

> The executing program currently contains the concrete integer value 38.

So:

```text
TYPE = compile-time knowledge
VALUE = runtime data
```

We deliberately created `src/types.rs` separately from `src/value.rs` so these concepts never become confused.

---

## 7. Explicit types and inferred types

Karma supports explicit annotation:

```karma
let age: Int = 38;
```

The compiler checks:

```text
annotation  = Int
initializer = Int
match       = yes
```

Karma v0.2 also allows simple local inference:

```karma
let age = 38;
```

The compiler sees the integer literal and records:

```text
age : Int
```

Why support inference?

Because this:

```karma
let answer: Int = 42;
```

and this:

```karma
let answer = 42;
```

contain the same information in simple local cases.

But public function boundaries remain explicit in v0.2:

```karma
fn add(a: Int, b: Int) -> Int
```

This makes APIs easy to understand and simplifies recursive type checking.

---

## 8. Immutable by default

This is a major Karma design principle.

```karma
let count: Int = 0;
```

means:

```text
name       = count
type       = Int
mutable    = false
```

This is invalid:

```karma
count = 1;
```

The type checker rejects it.

If state genuinely needs to change:

```karma
mut count: Int = 0;
count = count + 1;
```

Why make immutability the default?

Because mutation increases the number of possible states a program can have.

Compare:

```text
immutable x
x always means the same value in that scope
```

versus:

```text
mutable x
x could have changed at many different program points
```

Immutability improves reasoning, concurrency safety, optimization opportunities, and future ownership analysis.

---

## 9. Defense-in-depth mutability

We enforce mutability twice.

### Compile time

The type checker rejects assignment to `let`.

### Runtime

The environment stores:

```text
Binding {
    value,
    mutable
}
```

and refuses an illegal assignment even if a compiler bug somehow allowed one through.

This is intentional.

```text
Type checker  = primary safety barrier
Runtime       = defensive invariant barrier
```

For a security-oriented language, important invariants should not rely unnecessarily on one layer.

---

## 10. Symbols

Consider:

```karma
let age: Int = 38;
print(age);
```

When the type checker sees `age` in `print(age)`, it needs to know what `age` means.

A symbol entry contains information such as:

```text
name     = age
type     = Int
mutable  = false
```

The compiler stores these entries in symbol scopes.

This process is called **symbol resolution**.

---

## 11. Symbol scopes

Example:

```karma
let x: Int = 10;

{
    let y: Int = 20;
    print(x);
    print(y);
}
```

Conceptually:

```text
GLOBAL SCOPE
┌───────────────┐
│ x : Int       │
└───────┬───────┘
        │ parent lookup
        ▼
CHILD SCOPE
┌───────────────┐
│ y : Int       │
└───────────────┘
```

Looking up `y` finds it immediately.

Looking up `x` searches the current scope, then the parent.

Looking up `z` reaches the top without finding anything and becomes a compile-time error.

---

## 12. Why duplicate variables are rejected in the same scope

This is rejected:

```karma
let x: Int = 1;
let x: Int = 2;
```

because it creates ambiguity and makes accidental shadowing easy.

Nested scopes may later define their own names according to the language's shadowing policy, but a single scope should have one unambiguous binding per name.

---

## 13. Typed function signatures

v0.1 allowed:

```karma
fn add(a, b) {
    return a + b;
}
```

v0.2 requires:

```karma
fn add(a: Int, b: Int) -> Int {
    return a + b;
}
```

The compiler now knows before entering the body:

```text
function add
parameters:
    a : Int
    b : Int
returns:
    Int
```

This creates a compile-time contract.

---

## 14. Function signature collection happens before body checking

This matters for recursion.

Consider:

```karma
fn factorial(n: Int) -> Int {
    if n <= 1 {
        return 1;
    }
    return n * factorial(n - 1);
}
```

While checking the body, the compiler encounters `factorial(...)` again.

So before checking any function body, Karma first registers all top-level function signatures.

Conceptually:

```text
PASS 1
collect signatures

factorial : (Int) -> Int

PASS 2
check bodies
```

Without that first pass, recursion would look like a call to an unknown function.

---

## 15. Function arguments are checked before execution

Given:

```karma
fn add(a: Int, b: Int) -> Int {
    return a + b;
}
```

this is valid:

```karma
add(1, 2)
```

but this is rejected:

```karma
add(1, "two")
```

The checker compares:

```text
parameter 1 expected Int  actual Int     ✓
parameter 2 expected Int  actual String  ✗
```

No function call happens at runtime.

---

## 16. Return types

For:

```karma
fn answer() -> Int {
    return 42;
}
```

Karma checks:

```text
declared return type = Int
returned expression  = Int
```

This fails:

```karma
fn answer() -> Int {
    return "forty two";
}
```

because:

```text
expected Int
found String
```

---

## 17. All-path return checking

A function can have a correct return expression and still be unsafe.

Example:

```karma
fn value(ok: Bool) -> Int {
    if ok {
        return 1;
    }
}
```

What happens when `ok == false`?

The function reaches the end without producing an `Int`.

Karma v0.2 therefore performs simple control-flow return analysis.

This is accepted:

```karma
fn value(ok: Bool) -> Int {
    if ok {
        return 1;
    } else {
        return 2;
    }
}
```

because both branches guarantee a return.

The v0.2 analysis is intentionally conservative. Loops are not considered guaranteed returns even if a programmer believes they run forever.

---

## 18. Boolean conditions only

v0.1 had a truthiness model.

v0.2 removes that ambiguity for control flow.

Valid:

```karma
if age > 18 {
    print("adult");
}
```

because `age > 18` has type `Bool`.

Invalid:

```karma
if 1 {
    print("wrong");
}
```

Karma does not decide that integer `1` should secretly mean `true`.

This is safer and easier to reason about.

---

## 19. Operator typing rules

### Addition

Allowed:

```text
Int    + Int    -> Int
String + String -> String
```

Rejected:

```text
Int + String
Bool + Bool
```

### Arithmetic

```text
Int - Int -> Int
Int * Int -> Int
Int / Int -> Int
```

### Comparison

```text
Int <  Int -> Bool
Int <= Int -> Bool
Int >  Int -> Bool
Int >= Int -> Bool
```

### Equality

Operands must currently have the same type:

```text
Int == Int       -> Bool
Bool == Bool     -> Bool
String == String -> Bool
```

---

## 20. Why integer overflow is still a runtime concern

Static typing proves that this is integer arithmetic:

```karma
x + y
```

but it usually cannot know the concrete values that will exist at runtime.

So there are two different safety questions:

```text
TYPE SAFETY
Are x and y both Int?

VALUE SAFETY
Will x + y overflow the allowed Int range?
```

The type checker handles the first.

Checked arithmetic in the interpreter handles the second.

---

## 21. Raw AST versus Typed AST

This is perhaps the most important v0.2 concept.

### Raw AST

The parser can produce something like:

```text
Binary(+)
├── Variable(x)
└── Integer(1)
```

The AST knows the structure, but not necessarily the proven type.

### Typed AST

After semantic analysis:

```text
Binary(+) : Int
├── Variable(x) : Int
└── Integer(1)  : Int
```

Every expression now carries its resolved type.

This is extremely valuable for later compilation.

---

## 22. Why Typed AST matters for native compilation

Future KIR generation should not need to rediscover language meaning.

Instead:

```text
Parser
  ↓
Raw AST
  ↓
Semantic analysis
  ↓
Typed AST   ← language rules proven here
  ↓
KIR         ← lower-level representation
  ↓
Optimizer
  ↓
Machine code
```

By the time KIR is generated, the compiler can assume many semantic properties have already been verified.

This creates clean separation of responsibilities.

---

## 23. Source positions

v0.2 carries line and column information into AST nodes.

Why?

A compiler error saying:

```text
type mismatch
```

is far less useful than:

```text
type error at 4:18: initializer for 'age' expects Int, found String
```

Good diagnostics are part of language design, not decoration.

---

## 24. The `--check` command

v0.2 adds:

```bash
karma --check application.kr
```

The pipeline becomes:

```text
read file
  ↓
lex
  ↓
parse
  ↓
type-check
  ↓
STOP
```

No application code executes.

This will later be useful for IDEs, CI pipelines, editors and build systems.

---

## 25. `karma file.kr` now always type-checks first

Normal execution becomes:

```text
read
 ↓
lex
 ↓
parse
 ↓
type-check
 ↓
Typed AST
 ↓
execute
```

There is no normal "skip type checking" execution path.

That is an intentional security and correctness choice.

---

## 26. Runtime checks are not simply removed

Once a program has passed type checking, many runtime type failures should be impossible.

However the interpreter still checks important invariants.

For example, if a non-Boolean condition somehow reaches the runtime, the interpreter reports an internal invariant violation instead of silently guessing what to do.

This helps us detect compiler bugs during development.

---

## 27. Why function parameters are immutable in v0.2

Parameters are inserted into the function's symbol scope as immutable bindings.

So:

```karma
fn f(x: Int) -> Int {
    x = x + 1;
    return x;
}
```

is rejected in v0.2.

This keeps the default mental model simple.

If we later want mutable parameters, we should introduce them explicitly rather than allowing mutation accidentally.

---

## 28. Top-level functions only in v0.2

The parser can structurally see a function declaration inside a block, but the semantic checker rejects it.

Why?

Nested functions immediately introduce questions about closures, captured variables, lifetimes and allocation.

Those interact with the future memory model.

Rather than implement them incorrectly, v0.2 says:

```text
functions = top level only
```

This is deliberate scope control, not a permanent limitation.

---

## 29. Security consequences of static typing

Static typing does not make software automatically secure, but it removes categories of invalid states before deployment.

Examples now caught before runtime include:

- assigning `String` to an `Int`
- using an undefined variable
- assigning to an immutable binding
- calling an undefined function
- passing the wrong number of arguments
- passing the wrong argument types
- returning the wrong type
- using a non-Boolean condition
- performing invalid operator combinations

Every error found before execution is one less runtime state that production must survive.

---

## 30. Memory implications

v0.2 does **not** define the final Karma memory model. That is intentionally deferred to v0.3.

But v0.2 prepares for it by making types and mutability explicit.

Why does that matter?

A future ownership analyzer needs facts like:

```text
What type is this value?
Can this binding change?
Where was this value introduced?
Which scope owns the name?
What does this function accept and return?
```

v0.2 creates that semantic foundation.

---

## 31. Compiler architecture after v0.2

```text
                    KARMA v0.2

              Source file (.kr)
                     │
                     ▼
                   Lexer
                     │
                     ▼
                   Tokens
                     │
                     ▼
                   Parser
                     │
                     ▼
                  Raw AST
                     │
                     ▼
          ┌────────────────────┐
          │ Semantic Analysis  │
          │                    │
          │ • symbol scopes    │
          │ • static types     │
          │ • mutability       │
          │ • calls            │
          │ • returns          │
          │ • operators        │
          └──────────┬─────────┘
                     │
                     ▼
                  Typed AST
                     │
                     ▼
        Bootstrap Interpreter (v0.2)
                     │
                     ▼
                   Output
```

Later:

```text
Typed AST
   ↓
KIR
   ↓
Optimization
   ↓
Native backend
   ↓
ARM64 / x86-64
```

---

## 32. Files added or changed in v0.2

### New

```text
src/types.rs
src/source.rs
src/typed_ast.rs
src/type_checker.rs
```

### Major changes

```text
src/token.rs
src/lexer.rs
src/ast.rs
src/parser.rs
src/environment.rs
src/interpreter.rs
src/main.rs
```

### New validation examples

```text
examples/type_error.kr
examples/immutable_error.kr
```

---

## 33. How to validate v0.2 locally

```bash
cargo test
cargo build --release
./scripts/smoke.sh
```

Then:

```bash
./target/release/karma --version
```

Expected:

```text
Karma 0.2.0
```

Check a valid program without running it:

```bash
./target/release/karma --check examples/functions.kr
```

Expected:

```text
Karma check: OK
```

Try the intentional type error:

```bash
./target/release/karma --check examples/type_error.kr
```

It must fail.

Try the intentional immutability error:

```bash
./target/release/karma --check examples/immutable_error.kr
```

It must fail.

---

## 34. What v0.2 proves

v0.1 proved:

> Karma can understand and run programs.

v0.2 should prove:

> Karma can understand enough about a program's meaning to reject important invalid programs before they execute.

That is the first major step from a small interpreter toward a production compiler.

---

## 35. What comes next

Karma v0.3 will answer a deeper question:

> Who owns memory and resources, when are they released, and how can Karma prove resource safety without requiring a heavyweight tracing GC?

v0.2's type/symbol/mutability infrastructure is what makes that analysis possible.
