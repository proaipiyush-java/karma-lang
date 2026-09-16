# Karma Language Specification v0.2 (Draft)

## Primitive types

```text
Int
Bool
String
Unit
```

## Bindings

Immutable binding:

```karma
let name: Type = expression;
```

Type inference is allowed for a binding initializer:

```karma
let count = 10;
```

Mutable binding:

```karma
mut count: Int = 0;
count = count + 1;
```

Assignment to an immutable binding is a static error.

## Functions

```karma
fn name(parameter: Type, ...) -> ReturnType {
    ...
}
```

Parameter and return types are mandatory in v0.2.

Functions are top-level declarations in v0.2.

Parameters are immutable.

## Conditions

`if` and `while` conditions must have type `Bool`.

## Operators

```text
Int + Int       -> Int
String + String -> String
Int - Int       -> Int
Int * Int       -> Int
Int / Int       -> Int

Int < Int       -> Bool
Int <= Int      -> Bool
Int > Int       -> Bool
Int >= Int      -> Bool

T == T          -> Bool
T != T          -> Bool

!Bool           -> Bool
-Int            -> Int
```

## Returns

A return expression must match the function's declared return type.

A non-Unit function must conservatively prove that every control-flow path returns a value.

A Unit function may fall through or use `return;`.

## Built-ins

`print(x)` accepts one argument of any current v0.2 value type and returns `Unit`.
