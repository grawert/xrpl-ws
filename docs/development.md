# Development

## Change Management
- Changes in the public facing API are major changes.
- Changes in the non-public API layers are minor changes.
- The version information in `Cargo.toml` must be updated for both minor and major changes.
- Version updates must be done on a git feature branch. Create a branch `feature/[topic]`.

## Build & Test Commands
- **Check:** `cargo check`
- **Build:** `cargo build`
- **Test:** `cargo test`
- **Lint:** `cargo clippy -- -D warnings`
- **Format:** `cargo fmt`
- **Expand Macros:** `cargo expand` (essential for debugging XRPL serialization macros)

## Communication Rules
- No AI apologies or filler text in code comments.
- Public APIs must be fully documented before the code is considered "complete".
- If a logic change is made, ensure the matching tests are updated or added.
