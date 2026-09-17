# Migrating Karma Source from v0.2 to v0.3

v0.3 introduces ownership semantics for `String`.

## 1. Read-only String parameters

v0.2:

```karma
fn show(text: String) -> Unit {
    print(text);
}

let name: String = "Karma";
show(name);
print(name);
```

In v0.3 a normal String parameter owns/consumes its argument. If the function only reads it, change the signature:

```karma
fn show(text: borrow String) -> Unit {
    print(text);
}
```

## 2. Intentional duplication

v0.2 code may have relied on implicit runtime cloning:

```karma
let second: String = first;
print(first);
```

v0.3 requires intent:

```karma
let second: String = clone(first);
print(first);
```

Or transfer ownership and stop using `first`.

## 3. String concatenation

v0.3 consumes owned String variable operands:

```karma
let c: String = a + b;
```

Do not use `a` or `b` afterward unless you cloned what must be preserved.

## 4. Assignment result

Assignment now returns `Unit` rather than the assigned type. Normal statement assignment is unchanged:

```karma
mut count: Int = 0;
count = 1;
```

Avoid relying on the value of an assignment expression.

## 5. Top-level variable capture

Functions cannot capture top-level runtime bindings in v0.3. Pass data explicitly:

```karma
let name: String = "Karma";

fn show(text: borrow String) -> Unit {
    print(text);
}

show(name);
```
