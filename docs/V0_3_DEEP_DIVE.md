# Karma v0.3 — Ownership & Resource Model Deep Dive

This guide explains Karma v0.3 from first principles. It is intentionally written for someone who understands ordinary variables and functions but has never designed a memory model or ownership checker.

---

## 1. Why v0.3 exists

v0.1 taught Karma how to read and run code.

```text
source -> lexer -> parser -> AST -> interpreter
```

v0.2 taught Karma how to reject many invalid programs before they run.

```text
source -> lexer -> parser -> AST -> type checker -> Typed AST -> interpreter
```

But a typed program can still have a deeper problem: **who owns memory and resources?**

Imagine this code:

```karma
let first: String = "Karma";
let second: String = first;
```

What exactly should happen?

Possible answers include:

1. copy all bytes from `first` into `second`;
2. make both names point at the same object and run a garbage collector later;
3. increase a reference count;
4. transfer ownership from `first` to `second`;
5. let the programmer manually free one or both values.

The language must define one answer. If it does not, performance and safety become unpredictable.

Karma v0.3 chooses an ownership-oriented model.

---

## 2. The baby analogy: one physical book

Imagine you own one physical book.

```text
Piyush owns Book A
```

If you **give** the physical book to another person, you cannot honestly say you still possess it.

```text
Piyush ----gives----> Guduli

Before: Piyush owns book
After:  Guduli owns book
```

That is a **move**.

But someone can temporarily read the book without becoming its owner.

```text
Owner: Piyush
          |
          +---- temporary read-only view ----> Reader
```

That is a **borrow**.

And if you deliberately photocopy the book, there are now two independent logical copies.

That is a **clone**.

Karma uses the same three ideas:

```text
Move   = transfer ownership
Borrow = temporary read-only access
Clone  = intentionally create another logical owner/value
```

---

## 3. Why not just copy everything?

For `Int`, copying is cheap.

```text
42
```

An `Int` is a small fixed-size value. Copying 8 bytes is inexpensive.

But consider a 500 MB buffer:

```text
500 MB request body
```

If this line silently copies it:

```karma
let second = first;
```

then a harmless-looking assignment could unexpectedly allocate another 500 MB and copy all data.

That is terrible for:

- memory efficiency;
- latency;
- cache behavior;
- containers;
- serverless workloads;
- old machines;
- large buffers;
- enterprise services under load.

Karma therefore wants expensive ownership changes to be explicit.

---

## 4. Copy values vs owned values

v0.3 creates two broad categories.

### Copy

```text
Int
Bool
Unit
```

Copy values can be duplicated automatically.

```karma
let a: Int = 10;
let b: Int = a;

print(a);
print(b);
```

`a` remains valid.

### Owned

```text
String
```

`String` is the first owned value in Karma.

```karma
let a: String = "Karma";
let b: String = a;
```

Ownership moves from `a` to `b`.

Afterward:

```text
binding a -> MOVED
binding b -> owns "Karma"
```

So this is invalid:

```karma
print(a);
```

The compiler should reject it before execution.

---

## 5. Why String is our first owned type

We need at least one type whose storage is not conceptually a tiny scalar.

A String can contain:

```text
"Karma"
```

or millions of characters.

It is therefore a useful first example for ownership semantics.

Later, the same rules can apply to more serious resources:

```text
ByteBuffer
File
Socket
DatabaseConnection
MappedMemory
GPUBuffer
MessagePayload
```

The value of v0.3 is not only String safety. It is establishing rules that future resource types can reuse.

---

## 6. What a move means to the compiler

Source:

```karma
let first: String = "Karma";
let second: String = first;
```

The raw AST only knows that `first` is referenced.

```text
Binding second
   initializer
      Variable(first)
```

The ownership checker asks:

```text
What type is first?
    |
    v
String
    |
Is String Copy?
    |
   no
    |
Does this context need ownership?
    |
   yes
    |
Access = MOVE
```

The Typed AST records that decision.

Conceptually:

```text
Variable {
    name: "first",
    access: Move,
    type: String
}
```

This is extremely important for the future compiler.

KIR does not need to rediscover whether `first` was meant to move. The semantic checker has already decided.

---

## 7. The three Typed-AST access modes

v0.3 adds:

```text
Copy
Move
Borrow
```

### Copy

Used for small Copy values.

```karma
let b: Int = a;
```

### Move

Used when an owned value transfers ownership.

```karma
let b: String = a;
```

### Borrow

Used when code only needs temporary read access.

```karma
print(a);
```

`print` should inspect the String, not become its owner.

---

## 8. Why print must borrow

Suppose `print` consumed String arguments.

```karma
let name: String = "Karma";
print(name);
print(name);
```

The second print would fail because the first print took ownership.

That would be technically consistent but terrible ergonomically.

Printing is naturally a read-only operation.

Therefore Karma defines:

```text
print(x)
```

as borrowing `x`.

The owner remains available afterward.

---

## 9. Function parameters: owned by default

Consider:

```karma
fn store(text: String) -> Unit {
    ...
}
```

In v0.3, a normal non-Copy parameter is an **owning parameter**.

Calling:

```karma
store(name);
```

means:

```text
caller owns name
      |
      | move
      v
function parameter owns String
```

After the call, the caller cannot use `name` again unless it had cloned it first.

This is a useful default for functions that take responsibility for a value.

---

## 10. Read-only parameters use `borrow`

Many functions do not need ownership.

For example:

```karma
fn show(text: borrow String) -> Unit {
    print(text);
}
```

At the call site:

```karma
let name: String = "Karma";
show(name);
print(name);
```

This is valid.

The signature already communicates the ownership contract:

```text
text: String         -> function takes ownership
text: borrow String  -> function temporarily reads it
```

That makes function APIs self-documenting.

---

## 11. Why Karma does not use `&String` in v0.3

We could copy Rust syntax:

```text
&String
```

But that immediately raises larger questions:

```text
Can references be stored?
Can they be returned?
Can structs contain them?
What are their lifetimes?
Can they be mutable?
How are lifetime parameters written?
```

We do not need all that complexity yet.

Karma v0.3 chooses a narrower concept:

```karma
borrow String
```

means:

> This function parameter is a read-only view valid for this call.

The borrow cannot escape the call.

This is deliberately less expressive, but much easier to reason about.

---

## 12. Call-scoped borrowing

Think of a borrow as existing between function entry and function exit.

```text
Caller owns String
       |
       | temporary borrow begins
       v
+--------------------+
| function executes  |
| read-only access   |
+--------------------+
       |
       | borrow ends
       v
Caller still owns String
```

There is no storable `Reference<String>` value in v0.3.

That is intentional.

---

## 13. Borrowed values cannot escape

This function is invalid:

```karma
fn steal(text: borrow String) -> String {
    return text;
}
```

Why?

The function promised only temporary access.

But returning `text` as an owned String would effectively say:

```text
borrowed view -> magically becomes owner
```

That breaks ownership.

Karma rejects it.

A programmer who genuinely needs an owned result can explicitly clone:

```karma
fn duplicate(text: borrow String) -> String {
    return clone(text);
}
```

Now the ownership change is intentional and visible.

---

## 14. `clone` means intentional duplication

Source:

```karma
let first: String = "Karma";
let second: String = clone(first);
```

The compiler treats the argument to `clone` as a borrow.

So `first` remains available.

Afterward:

```text
first  -> logical owner
second -> logical owner
```

Both can be used.

This does **not** require the runtime to physically duplicate bytes every time.

Logical semantics and physical representation are separate concerns.

---

## 15. Logical clone vs physical clone

Language rule:

```text
clone(x) creates another logical value/owner
```

Implementation choices could include:

```text
deep copy
copy-on-write
immutable shared backing
reference-counted backing
small-string optimization
```

As long as observable Karma semantics remain correct, the runtime/compiler can choose an efficient representation.

The v0.3 bootstrap interpreter uses shared immutable String backing with Rust `Rc<str>`.

That means borrowing/cloning a runtime String handle does not automatically duplicate every byte.

This is an implementation optimization, not the final Karma ABI.

---

## 16. `drop` means release ownership early

Normally an owned local value lives until:

- it is moved elsewhere; or
- its scope ends.

Sometimes a programmer wants to say:

> I am finished with this value now.

Karma provides:

```karma
drop(value);
```

For an owned String, this consumes ownership.

```karma
let temp: String = "temporary";
drop(temp);
print(temp); // error
```

This becomes more valuable later with resources such as:

```text
large buffer
file handle
socket
lock guard
GPU allocation
```

---

## 17. Deterministic cleanup

Imagine this block:

```karma
{
    let a: String = "A";
    let b: String = "B";
}
```

At block exit, the scope disappears.

Therefore values still owned by the scope are released.

Conceptually:

```text
enter scope
   |
   +-- own a
   +-- own b
   |
exit scope
   |
   +-- release b
   +-- release a
```

v0.3 does not yet expose user-defined destructors, but the ownership rule is established now.

That lets future File/Socket types rely on deterministic cleanup instead of requiring a global tracing GC.

---

## 18. Runtime environment now has moved slots

In v0.2 a runtime binding was roughly:

```text
Binding {
    value,
    mutable
}
```

v0.3 changes the idea to:

```text
Binding {
    value: Some(value) | None,
    mutable,
    movable
}
```

### Some(value)

The binding currently contains a value.

### None

Ownership was moved out.

This means the runtime can independently reject accidental use-after-move.

---

## 19. Why keep runtime checks if the compiler checks first?

Because compilers have bugs too.

Our intended path is:

```text
static checker proves invariant
            +
runtime bootstrap checks invariant again
```

This is defense in depth.

Later, after compiler maturity, optimized native code can remove redundant checks when proof is strong enough.

But during language development, internal invariant checks are valuable.

---

## 20. Moving from a binding internally

Suppose:

```karma
let first: String = "Karma";
let second: String = first;
```

The Typed AST says the read of `first` is `Move`.

The interpreter therefore calls conceptually:

```text
environment.take("first")
```

Before:

```text
first -> Some("Karma")
```

After:

```text
first -> None
```

The returned value becomes the initializer for `second`.

So:

```text
second -> Some("Karma")
```

There is only one active owner slot.

---

## 21. Borrowing from a binding internally

For:

```karma
print(first);
```

Typed AST says:

```text
access = Borrow
```

The interpreter uses read-only `get`, not `take`.

The binding remains:

```text
first -> Some("Karma")
```

With `Rc<str>` backing, the bootstrap interpreter only duplicates a small handle rather than all String bytes.

---

## 22. Why assignment now returns Unit

In v0.2 assignment was treated like an expression with the variable's type.

Example:

```karma
value = newValue
```

If `newValue` is owned, what would it mean for assignment also to return that same owned value?

We would create a problem:

```text
one value stored in target
        +
what is the returned owner?
```

Karma v0.3 removes the ambiguity.

Assignment stores the value and evaluates to:

```text
Unit
```

So ownership has exactly one destination.

---

## 23. Reinitializing a moved mutable binding

Consider:

```karma
mut first: String = "one";
let second: String = first;
```

Now:

```text
first = moved
second owns "one"
```

Can `first` ever be used again?

Yes, because it is `mut` and can be assigned a completely new owned value.

```karma
first = "two";
```

Now:

```text
first owns "two"
second owns "one"
```

This is not recovering the old value. It is reinitializing the empty binding.

---

## 24. Why immutable bindings cannot be reinitialized

With:

```karma
let first: String = "one";
let second: String = first;
```

`first` becomes moved.

But `let` means the binding is immutable.

So:

```karma
first = "two";
```

is not allowed.

If reinitialization is part of the intended design, the binding should have been declared:

```karma
mut first: String = "one";
```

This preserves Karma's explicit-mutation philosophy.

---

## 25. String concatenation consumes owned operands

Consider:

```karma
let a: String = "Kar";
let b: String = "ma";
let c: String = a + b;
```

The `+` operation creates a new owned String.

v0.3 treats its String operands as consumed.

Afterward:

```text
a -> moved
b -> moved
c -> owns "Karma"
```

Why?

This gives a future optimizer freedom to reuse storage from `a` or `b` rather than preserving them unnecessarily.

If the programmer needs the originals:

```karma
let c: String = clone(a) + clone(b);
```

The cost/intent is visible.

---

## 26. Equality borrows instead of consuming

Comparison is naturally observational.

```karma
let a: String = "Karma";
let b: String = "Karma";
let same: Bool = a == b;

print(a);
print(b);
```

This should work.

Therefore equality checks borrow operands.

Conceptually:

```text
a --borrow--+
             +--> equality -> Bool
b --borrow--+
```

Neither owner is consumed.

---

## 27. Ownership is context-sensitive

The same source variable can mean different access modes depending on context.

```karma
print(name);
```

`name` -> Borrow

```karma
let other: String = name;
```

`name` -> Move

```karma
let n2: Int = n;
```

`n` -> Copy

This is why ownership belongs in semantic analysis, not in the lexer or parser.

The parser sees only syntax.

The type/ownership checker understands meaning.

---

## 28. The ownership state machine

For an owned binding, the checker tracks a small state machine.

```text
          declaration
              |
              v
         +-----------+
         | Available |
         +-----------+
              |
           move
              |
              v
         +-----------+
         |   Moved   |
         +-----------+
              |
     assignment if mutable
              |
              v
         +-----------+
         | Available |
         +-----------+
```

A read while state is `Moved` is a compile-time error.

---

## 29. Why control flow makes ownership harder

Straight-line code is easy.

```karma
let a = ...;
let b = a;
print(a); // obviously invalid
```

But consider:

```karma
if condition {
    drop(a);
}

print(a);
```

If `condition` is false, `a` exists.

If `condition` is true, `a` was moved/dropped.

So after the `if`:

```text
a is MAYBE available
```

A safe compiler cannot allow a guaranteed read from a maybe-available owner.

---

## 30. How v0.3 merges `if` states

Before branch:

```text
a = Available
```

Branch A:

```text
a = Moved
```

Branch B:

```text
a = Available
```

Merge:

```text
Available only if BOTH branches guarantee Available
```

So result:

```text
a = Moved/unavailable
```

This is conservative but safe.

---

## 31. If both branches reinitialize, availability can return

Suppose `a` is mutable and both branches assign a new String.

```karma
mut a: String = "start";
let old: String = a;

if condition {
    a = "yes";
} else {
    a = "no";
}

print(a);
```

Before the `if`, `a` is moved.

But every branch reinitializes it.

So after the merge:

```text
a = Available
```

This illustrates why ownership analysis is data-flow analysis, not just a boolean flag attached forever.

---

## 32. Why loops are even harder

Consider:

```karma
let a: String = "Karma";
while condition {
    drop(a);
}
```

First iteration:

```text
a exists -> drop(a)
```

Second iteration:

```text
a is already moved
```

Unsafe.

The checker must reason about the value flowing from the end of one iteration back to the beginning of the next.

That is called a loop-carried state.

---

## 33. v0.3's conservative loop rule

If an owned value defined outside a `while` loop is:

```text
Available at loop entry
```

but:

```text
Moved at body exit
```

v0.3 rejects the loop.

Because another iteration might run.

This is true even if a human sees:

```karma
while false {
    drop(a);
}
```

The v0.3 checker deliberately does not perform constant-folding proof here.

Safety analysis remains simple and predictable.

---

## 34. Move + reinitialize inside a loop

This can be safe:

```karma
mut value: String = "first";

while condition {
    drop(value);
    value = "next";
}
```

At body end:

```text
value = Available
```

So the next iteration begins with an owner again.

The loop may also run zero times, in which case the original owner remains.

Therefore after the loop, the binding is still guaranteed available.

---

## 35. Function ownership isolation in v0.3

v0.2's symbol scopes could let a function accidentally refer to an earlier top-level variable.

Ownership makes that much more complicated.

Example:

```karma
let global: String = "x";

fn f() -> Unit {
    // what happens if this moves global?
}
```

Now questions appear:

- can `f()` be called twice?
- is global still available?
- can recursion move it?
- can another function borrow it simultaneously later?

Rather than define sloppy semantics, v0.3 isolates function ownership analysis from top-level runtime bindings.

Functions see:

```text
parameters
+
locals
```

Future module constants/statics will get explicit rules.

---

## 36. Why this is good language engineering

A common mistake in language design is to make a feature work in simple examples and postpone the hard semantics.

Karma is doing the opposite.

Before introducing:

```text
File
Socket
DatabaseConnection
async tasks
threads
```

we establish:

```text
who owns a value
when ownership transfers
when read-only access is allowed
when a value becomes unavailable
when cleanup occurs
how branches/loops affect availability
```

That reduces the chance of redesigning the language later.

---

## 37. Why v0.3 does not have mutable borrowing yet

A mutable borrow introduces exclusivity rules.

Conceptually:

```text
one mutable borrower
OR
many immutable borrowers
but not both
```

That becomes important for alias safety and concurrency.

But adding it now would multiply complexity before we even have user-defined aggregates/resources.

So v0.3 supports only read-only borrowing.

Possible future syntax could be designed later, for example:

```text
borrow mut T
inout T
```

No syntax is committed yet.

---

## 38. Why no general lifetime syntax yet

Rust can express relationships such as:

```text
returned reference lives as long as input reference
```

Karma v0.3 simply does not permit borrowed values to escape calls.

That means the compiler does not need user-visible lifetime parameters yet.

This is a deliberate design constraint:

```text
less expressive now
        ->
far easier to understand and secure
        ->
add complexity only when a real use case demands it
```

---

## 39. What about arenas?

Arenas/regions remain part of Karma's memory-efficiency direction, but v0.3 does not expose arena syntax yet.

Why?

An arena matters most when we have native allocation lowering.

The future model may be:

```text
ownership analysis
      |
      v
escape analysis
      |
      +--> stack allocate
      +--> arena/region allocate
      `--> heap allocate
```

Introducing surface syntax before KIR/native allocation exists would risk freezing a poor abstraction.

So v0.3 establishes deterministic lifetimes first; arena lowering can use those lifetimes later.

---

## 40. Why this can be more memory efficient than a mandatory tracing GC

A tracing GC usually needs to discover unreachable objects by examining runtime object graphs.

Karma's long-term goal is different:

```text
compiler knows ownership/lifetime
       |
       v
insert deterministic cleanup
       |
       v
no mandatory whole-heap tracing collector
```

That can reduce:

- runtime metadata;
- background GC work;
- unpredictable pause behavior;
- baseline memory overhead.

It does not mean every program is automatically faster than every GC language. Real performance depends on workloads and implementation quality.

The goal is **predictability and controllable overhead**.

---

## 41. Why Rc is used in the bootstrap interpreter if we do not want mandatory ref-counting

This is an important distinction.

The Karma language memory model and the Rust bootstrap interpreter's internal representation are not the same thing.

Today:

```text
Karma String semantic ownership
          |
          v
Rust interpreter implementation
          |
       Rc<str>
```

Later native Karma could use:

```text
owned String = pointer + length + capacity
borrowed view = pointer + length
```

The bootstrap interpreter uses `Rc<str>` because it gives us cheap temporary read handles while we validate semantics.

It does **not** commit the final language to universal reference counting.

---

## 42. Why the interpreter still matters

You might ask:

> If v0.4 will compile native code, why invest in interpreter semantics?

Because the interpreter is our executable specification.

If:

```karma
let b = a;
```

means Move in v0.3, then both:

```text
interpreter
native compiler
```

must implement the same observable behavior.

That makes differential testing possible later:

```text
same .kr program
     |           |
     v           v
interpreter    native backend
     |           |
     +---- compare results ----+
```

This is valuable for compiler correctness.

---

## 43. What changed in the source tree

### `types.rs`

Now classifies types and defines parameter ownership mode.

```text
Type::is_copy()
Type::is_owned()
ParamMode::Owned
ParamMode::Borrowed
```

### `ast.rs`

Function parameters now carry:

```text
name
type
mode
source position
```

### `typed_ast.rs`

Variable uses now carry:

```text
Copy
Move
Borrow
```

### `type_checker.rs`

Now performs:

```text
type analysis
+
move-state analysis
+
borrow parameter checking
+
branch-state merging
+
loop-carried move checking
```

### `environment.rs`

Runtime slots can become empty after move.

### `value.rs`

Bootstrap String uses immutable shared backing.

### `interpreter.rs`

Executes Typed-AST access modes through read/take operations.

---

## 44. End-to-end example: move

Source:

```karma
let first: String = "Karma";
let second: String = first;
```

### Lexer

Produces tokens.

```text
LET IDENTIFIER COLON TYPE_STRING EQUAL STRING SEMICOLON ...
```

### Parser

Builds raw AST.

```text
Binding(first, String, "Karma")
Binding(second, String, Variable(first))
```

### Type checker

Finds:

```text
first: String
String is owned
initializer needs ownership
```

Marks:

```text
Variable(first, Move)
```

and updates symbol state:

```text
first: Available -> Moved
```

### Interpreter

Executes Move through:

```text
environment.take("first")
```

and stores returned value under `second`.

---

## 45. End-to-end example: borrow

Source:

```karma
fn show(text: borrow String) -> Unit {
    print(text);
}

show(name);
```

### Parser

Parameter becomes:

```text
Param {
  name: text,
  type: String,
  mode: Borrowed
}
```

### Function signature registry

Stores:

```text
show(String / Borrowed) -> Unit
```

### Call checker

Sees Borrowed parameter and checks `name` in Borrow context.

Typed AST:

```text
Variable(name, Borrow)
```

### Interpreter

Reads without taking owner slot.

Caller binding remains populated after the call.

---

## 46. End-to-end example: clone

Source:

```karma
let copy: String = clone(name);
```

Checker treats `clone` argument as Borrow.

```text
name remains Available
```

`clone` result has type:

```text
String
```

The new binding becomes another logical owner.

---

## 47. End-to-end example: use-after-move diagnostic

Source:

```karma
let first: String = "Karma";
let second: String = first;
print(first);
```

Checker records where the move occurred.

When `first` appears again, it can report conceptually:

```text
type error: use of moved value 'first';
ownership was previously transferred at 2:22
```

Tracking move position matters because future diagnostics can point to both:

```text
where ownership moved
where illegal reuse happened
```

---

## 48. What v0.3 guarantees before execution

For the currently supported language subset, semantic analysis now aims to prove:

- referenced variables exist;
- static types match;
- immutable bindings are not assigned;
- non-Copy owned values are not used after move;
- borrowed parameters do not transfer ownership;
- borrowed owned values cannot escape through an owning use;
- normal owned parameters do transfer ownership;
- explicit clone preserves the original owner;
- drop consumes an owned value;
- branch merges do not pretend a maybe-moved owner is definitely available;
- loops do not carry a moved outer owner into a later iteration;
- function return types remain valid.

---

## 49. What v0.3 does NOT guarantee yet

This is equally important.

v0.3 does not yet have:

- user-defined structs;
- user-defined destructors;
- mutable borrowing;
- storable references;
- lifetime parameters;
- threads;
- async tasks;
- data-race analysis;
- atomics;
- native stack/heap allocation decisions;
- arena syntax;
- file/socket resource APIs;
- native machine-code output.

We should not claim these guarantees before implementing them.

---

## 50. How v0.3 prepares v0.4

v0.4 is planned to introduce KIR and native compilation.

The critical bridge is now:

```text
Typed AST
   |
   +-- type
   +-- mutability
   +-- Copy/Move/Borrow access
   +-- control-flow validity
   |
   v
KIR
```

KIR can then represent operations such as conceptually:

```text
%1 = move_string %owner
%2 = borrow_string %owner
release %value
```

The exact KIR syntax is not yet frozen.

But ownership intent now exists early enough for native lowering.

---

## 51. Future optimization opportunities created by ownership

Once KIR exists, the compiler can investigate optimizations such as:

### Stack allocation

If a value does not escape its function:

```text
allocate on stack
```

### Move reuse

For:

```karma
let c = a + b;
```

if `a` is moved and uniquely owned, the compiler might reuse its buffer.

### Drop elimination

If ownership moves into a return value, do not release it in the old scope.

### Arena grouping

Values with the same proven lifetime may be grouped into a region.

### Reference-count elimination

If static ownership proves uniqueness, no runtime refcount is needed in native code.

These optimizations are possible because semantics are explicit.

---

## 52. Security implications

Memory/resource rules are not only performance rules.

They help prevent categories such as:

```text
use-after-free
use-after-move
multiple owners freeing same resource
dangling borrowed views
resource leaks from forgotten cleanup
```

v0.3 only implements the foundation, but the direction is security-relevant.

---

## 53. Process-friendliness implications

A deterministic ownership model supports the project's original process goals:

```text
no mandatory tracing GC thread
no required JIT compiler
no hidden global heap scan
predictable resource release
small runtime direction
```

The current interpreter is still a bootstrap tool, so native process measurements belong to later compilation releases.

---

## 54. How to validate v0.3 locally

From the v0.3 project directory:

```bash
cargo test
```

Then:

```bash
cargo build --release
```

Check version:

```bash
./target/release/karma --version
```

Expected:

```text
Karma 0.3.0
```

Run the full smoke suite:

```bash
./scripts/smoke.sh
```

Run ownership example:

```bash
./target/release/karma examples/ownership.kr
```

Check an expected failure:

```bash
./target/release/karma --check examples/move_error.kr
```

That command should fail because the example intentionally reuses a moved String.

---

## 55. What to inspect if a test fails

Use this order:

```text
1. First compiler error
2. File + line
3. Is failure lexer/parser/type/runtime?
4. Reduce source to smallest example
5. Check ownership access mode
6. Add regression test before fixing
```

Do not change semantics merely to make one test pass. If a rule is wrong, update the specification and ADR together with the implementation.

---

## 56. How to commit v0.3 after local validation

Only after the full build and smoke tests pass:

```bash
git status
git add .
git commit -m "Karma v0.3.0 ownership and resource foundation"
git tag -a v0.3.0 -m "Karma v0.3.0 - Ownership and Resource Foundation"
git push
git push origin v0.3.0
```

Before committing, confirm IDE/build output remains ignored:

```bash
git status
```

`.idea/` and `target/` should not appear as tracked additions.

---

## 57. The mental model to remember

```text
                 KARMA v0.3

          What kind of value is this?
                    |
        +-----------+-----------+
        |                       |
      Copy                    Owned
 Int / Bool / Unit            String
        |                       |
    duplicate            What context?
                                |
                    +-----------+-----------+
                    |                       |
                 owning                  read-only
                    |                       |
                  MOVE                   BORROW
                    |
             old binding moved

Intentional extra owner -> clone(...)
Finished early           -> drop(...)
Scope ends               -> deterministic cleanup
```

If you remember that picture, most v0.3 behavior follows naturally.

---

## 58. The one-sentence definition of v0.3

**Karma v0.3 teaches the compiler that not every value should be silently copied: small values copy, owned values move, read-only calls borrow, duplication is explicit, and cleanup follows ownership.**
