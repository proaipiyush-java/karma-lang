# Karma Language Specification — v0.1 Draft

Status: bootstrap specification. Syntax and semantics are intentionally small and may change before v1.0.

## 1. Source files

Karma source files use the `.kr` extension and UTF-8 text.

## 2. Comments

Single-line comments begin with `//`.

```karma
// This is a comment.
let x = 10;
```

## 3. Values

v0.1 has three user-visible value kinds:

- `Int`: signed 64-bit integer.
- `Bool`: `true` or `false`.
- `String`: UTF-8 text represented by the bootstrap runtime.

Functions without an explicit return value evaluate internally to `Unit`.

### Integer safety

Integer addition, subtraction, multiplication, division, and unary negation are checked. Overflow is a runtime error in v0.1. Division by zero is a runtime error.

## 4. Variables

```karma
let name = "Karma";
let count = 0;
count = count + 1;
```

v0.1 permits reassignment. The immutable/mutable binding model is intentionally deferred to the v0.2 type-system design so it can be specified consistently with ownership.

A variable cannot be defined twice in the same lexical scope.

## 5. Operators

Arithmetic: `+`, `-`, `*`, `/`

Comparison: `<`, `<=`, `>`, `>=`

Equality: `==`, `!=`

Unary: `-`, `!`

`+` supports `Int + Int` and `String + String`. Other mixed-type arithmetic is rejected.

## 6. Conditionals

```karma
if score >= 90 {
    print("high");
} else {
    print("normal");
}
```

## 7. Loops

```karma
let i = 0;
while i < 3 {
    print(i);
    i = i + 1;
}
```

## 8. Functions

```karma
fn add(a, b) {
    return a + b;
}

print(add(2, 3));
```

v0.1 functions are top-level, named functions with at most 64 parameters. Closures and first-class function values are deliberately excluded.

## 9. Built-ins

`print(value)` writes one value followed by a newline. It accepts exactly one argument in v0.1.

## 10. Runtime safety limits

The bootstrap interpreter has protective defaults:

- source file limit: 8 MiB
- execution step limit: 1,000,000 AST evaluation/execution steps
- function call depth: 512
- function parameters/arguments: 64

These are bootstrap safety guards, not the long-term enterprise execution model.

## 11. Grammar sketch

```text
program        -> declaration* EOF ;
declaration    -> "let" IDENTIFIER "=" expression ";"
                | "fn" IDENTIFIER "(" parameters? ")" block
                | statement ;
statement      -> "if" expression block ("else" ("if" statement | block))?
                | "while" expression block
                | "return" expression? ";"
                | block
                | expression ";" ;
block          -> "{" declaration* "}" ;
expression     -> assignment ;
assignment     -> equality ("=" assignment)? ;
equality       -> comparison (("==" | "!=") comparison)* ;
comparison     -> term ((">" | ">=" | "<" | "<=") term)* ;
term           -> factor (("+" | "-") factor)* ;
factor         -> unary (("*" | "/") unary)* ;
unary          -> ("!" | "-") unary | call ;
call           -> primary ("(" arguments? ")")* ;
primary        -> INTEGER | STRING | "true" | "false" | IDENTIFIER | "(" expression ")" ;
```
