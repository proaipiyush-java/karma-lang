# Karma v0.3 Architecture

## Purpose

v0.3 extends semantic analysis into ownership/resource analysis. The compiler now classifies each relevant variable use as Copy, Move or Borrow before the bootstrap interpreter executes it.

```text
.kr source
    |
    v
+---------+
|  Lexer  |
+---------+
    |
  Tokens
    |
    v
+---------+
| Parser  |
+---------+
    |
 Raw AST
    |
    v
+-----------------------+
| Type + Ownership Check|
| + symbols             |
| + mutability          |
| + move state          |
| + borrow mode         |
| + control-flow merge  |
+-----------------------+
    |
 Typed AST
 Copy / Move / Borrow
    |
    v
+-----------------------+
| Bootstrap Interpreter |
| runtime guard rails   |
+-----------------------+
    |
  Output
```

## Semantic boundary

The Typed AST is the contract between language semantics and execution backends.

Example source:

```karma
let a: String = "Karma";
let b: String = a;
```

The checker annotates the second use of `a` as a `Move`. A later backend does not have to rediscover that ownership rule.

```text
Raw AST variable use: a
          |
          v
Ownership checker
          |
          v
Typed variable use: a / Move
```

For:

```karma
print(a);
```

`print` is a borrowing built-in, so the Typed AST marks the access as `Borrow`.

## Runtime representation

The interpreter environment stores a binding as conceptually:

```text
Binding
  value: Some(Value) | None
  mutable: Bool
  movable: Bool
```

- `Some(value)` means the binding currently owns/contains a value.
- `None` means an owned value was moved out.
- `movable = false` is used for borrowed parameters.

This lets the runtime detect a use-after-move even if a compiler invariant is accidentally violated.

## String implementation in the bootstrap interpreter

`String` uses `Rc<str>` backing storage in v0.3's interpreter. This is not the final native ABI. It has two useful bootstrap properties:

1. borrowing a String can clone a small reference-counted handle rather than duplicate all bytes;
2. logical ownership rules remain testable before KIR/native memory layout exists.

Future native lowering can represent owned String differently, for example pointer + length + capacity, while a borrowed call can lower to a read-only pointer/length view.

## Control-flow ownership

### If merge

```text
             before: value available
                     |
             +-------+-------+
             |               |
          branch A        branch B
          moves value     keeps value
             |               |
             +-------+-------+
                     |
          after: not guaranteed available
```

Karma conservatively marks the value moved after the `if`.

### While analysis

An outer owner that is moved and remains moved at the end of one loop body would be unavailable on the next iteration. v0.3 rejects that pattern.

If a mutable binding is moved and definitely reinitialized before body exit, its next iteration starts available and the pattern is accepted.

## Deterministic cleanup

Lexical scopes own their local bindings. When the scope exits, values still present in the environment are dropped with the environment. Moved values have already left their original slot.

This is the semantic foundation for later deterministic cleanup of files, sockets, locks and other resources.

## Future boundary

```text
Typed AST
   |
   v
  KIR
   |
   +-> ownership-aware optimization
   +-> escape analysis
   +-> stack vs heap placement
   +-> deterministic drop insertion
   +-> optional arena lowering
   |
   v
native backend
   |
   +-> ARM64
   +-> x86-64
   `-> future targets
```
