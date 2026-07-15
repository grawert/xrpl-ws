# Coding Standards

## Tone
Professional, technical, and objective in all documentation and comments.

## Naming
Use self-explaining function, variable, and field names in order to reduce the need for extra comments.
Prefer defining variables for a type over passing types on the fly to functions. To improve readability.

## Comments
- Use `///` for public API documentation with clear examples.
- Avoid conversational or "chatty" meta-comments (e.g., "I'm doing this because...").
- Focus documentation on what it is good for, not what it does.
- Add short code examples for structs/objects.

## Serde Conventions
- Apply `#[serde(rename_all = "PascalCase")]` at the struct level on all public protocol structs.
- Apply `#[skip_serializing_none]` on any struct with `Option` fields — never serialize `None` as `null`.
- For XRPL field names that do not survive PascalCase conversion (e.g. `CheckID`, `NFTokenID`, `InvoiceID`), keep an explicit `#[serde(rename = "...")]` on the field.

## Errors
Use `thiserror` for library-level error definitions and `anyhow` for tests/examples.

## Diagnostic Output
Avoid writing to stdout/stderr in library code. Do not use `println!`.

Use `eprintln!` only for unrecoverable error conditions inside async tasks where the error cannot be propagated (e.g. a deserialization failure inside a WebSocket receive loop).

For debug/trace output use the `jsondump` feature flag and the `json_dump!` macro. This keeps the diagnostic footprint opt-in and dependency-free:

```rust
#[cfg(feature = "jsondump")]
macro_rules! json_dump { ... }

#[cfg(not(feature = "jsondump"))]
macro_rules! json_dump { ... } // no-op

// Usage
json_dump!("REQUEST", &payload, Some(id));
```

## Safety
Avoid `unsafe` unless required for performance; if used, document with a `// SAFETY:` block.

## Code Structure
- Prefer `match` over `if/else` except for `if let` when it improves readability.
- Use macros when it makes the code structure easier to read. Also offer macros to the user when it serves convenience.
