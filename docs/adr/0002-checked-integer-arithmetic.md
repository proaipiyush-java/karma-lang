# ADR-0002: Checked integer arithmetic by default

- Status: Accepted for v0.1
- Decision: `Int` arithmetic reports overflow instead of wrapping silently.

Future versions may introduce explicit wrapping, saturating, or unchecked operations with visible syntax/APIs, but the safe default remains checked.
