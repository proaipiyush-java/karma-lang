# ADR 0007 — Assignment Has Type `Unit`

## Status

Accepted for v0.3.

## Decision

An assignment such as:

```karma
value = expression;
```

stores ownership in the target binding and evaluates to `Unit`.

## Why

If assignment also returned the assigned owned value, the language would need to define whether it copied, borrowed, or duplicated ownership. Returning Unit keeps ownership singular and explicit.

## Consequences

Assignment cannot be used as an expression that yields the assigned value. This is an intentional pre-1.0 breaking change from the v0.2 interpreter behavior.
