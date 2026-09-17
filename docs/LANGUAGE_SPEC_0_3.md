# Karma Language Specification v0.3 (Draft)

## 1. Purpose

v0.3 defines the first enforceable Karma ownership/resource rules. The goal is to prevent use-after-move and accidental hidden duplication of owned values while keeping the language easier to read than a general reference/lifetime system.

## 2. Value categories

### Copy values

```text
Int
Bool
Unit
```

Reading a Copy value duplicates its value semantics. The source binding remains available.

### Owned values

```text
String
```

`String` is the first non-Copy owned value. Future files, sockets, byte buffers and user-defined resource types are expected to use the same ownership principles.

## 3. Move semantics

An owned value used in an owning context transfers ownership.

```karma
let a: String = "Karma";
let b: String = a;
```

After this statement, `a` is moved and cannot be read again unless a mutable binding is reinitialized.

## 4. Borrowed function parameters

Function parameters are owning by default:

```karma
fn consume(text: String) -> Unit {
    print(text);
}
```

A read-only call-scoped parameter uses `borrow`:

```karma
fn inspect(text: borrow String) -> Unit {
    print(text);
}
```

Calling a borrowed parameter does not transfer ownership from the caller.

```karma
let name: String = "Karma";
inspect(name);
print(name);
```

### v0.3 borrow restriction

Borrowed values are not first-class reference values in v0.3. They cannot be stored in variables, returned from functions, or escape the function call that created the borrow.

This deliberately avoids user-visible lifetime syntax while the core model is stabilized.

## 5. Borrowed parameter cannot become an owner

Invalid:

```karma
fn steal(text: borrow String) -> String {
    return text;
}
```

A borrowed parameter may be inspected or borrowed again, but not moved into an owning destination.

## 6. Explicit clone

```karma
let a: String = "Karma";
let b: String = clone(a);
```

`clone` reads its argument without moving the original and returns a new logical owner.

The bootstrap interpreter may share immutable physical backing for efficiency. Logical ownership semantics remain independent of the chosen runtime representation.

## 7. Explicit drop

```karma
let temp: String = "temporary";
drop(temp);
```

`drop` consumes the value immediately. A later use is a static error.

Copy values may be passed to `drop`, but because they are copied rather than moved, their source binding remains usable.

## 8. Assignment semantics

Assignment stores a value into a mutable binding and has type `Unit` in v0.3.

```karma
mut name: String = "old";
name = "new";
```

The assignment expression does not produce another owned copy of the value being stored.

A mutable owned binding may be reinitialized after its old value was moved:

```karma
mut a: String = "one";
let b: String = a;
a = "two";
```

## 9. Operators and ownership

### Consuming operations

`String + String` consumes owned operands and returns a new owned `String`.

```karma
let a: String = "Kar";
let b: String = "ma";
let c: String = a + b;
```

Afterward, `a` and `b` are moved.

### Read-only comparisons

Equality and comparison operations are read-only. Comparing strings does not consume them.

```karma
let a: String = "Karma";
let b: String = "Karma";
let same: Bool = a == b;
print(a);
print(b);
```

## 10. Control-flow ownership

### If

If an owned value may be moved on any branch, it is considered unavailable after the `if` unless every branch leaves it available.

```karma
let value: String = "Karma";
if condition {
    drop(value);
}
// value is not guaranteed to exist here
```

### While

A loop body may execute more than once. v0.3 rejects a move of an outer owned value if that value is not definitely reinitialized before the end of the loop body.

Safe pattern:

```karma
mut value: String = "one";
while condition {
    drop(value);
    value = "next";
}
```

## 11. Scope cleanup

Values still owned by a lexical scope are released deterministically when that scope ends.

v0.3 does not yet expose user-defined destructors. The rule is intentionally established before files/sockets/resources are added.

## 12. Function capture restriction

v0.3 functions do not capture top-level runtime bindings. Function ownership analysis starts from its parameters and local bindings. Future module constants/statics will receive explicit semantics rather than accidental capture behavior.

## 13. Built-ins

```text
print(x) -> Unit
clone(x) -> T
drop(x)  -> Unit
```

- `print` borrows its argument.
- `clone` borrows its argument and returns a new logical owner/value.
- `drop` consumes its argument.
