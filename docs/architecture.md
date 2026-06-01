# Style & Architecture

## Project Context
This is a **public-facing Rust library** for the XRP Ledger (XRPL).
The code must be high-quality, performant, and maintainable by the open-source community.
The code is layered in order to keep the public facing API as stable as possible, but
still be open to changes in the layers not directly exposed as public API.

## Principles
- **Lean Functions:** Write simple, small, and focused functions. This aids the borrow checker and improves readability.
- **DRY (Don't Repeat Yourself):** Avoid logic duplication for core protocols, but prioritize API stability over extreme code reduction.
- **Trait-Based Outsourcing:** When a function handles too much state or protocol logic, use traits to "outsource" that behavior (e.g., the `SessionHandler` pattern). Meaning when a function exceeds 50 lines.
- **Composition over Complexity:** For internal organization, prefer breaking large structs into smaller ones (composition) rather than creating excessive layers of abstraction.

## Module Organization
- Split large domains into focused submodules (e.g., `types/transactions/payment.rs`, `types/transactions/escrow.rs`) rather than a single monolithic file.
- Re-export from the parent module so public import paths stay stable as the internal structure evolves.

## Public Enums
- Apply `#[non_exhaustive]` to any public enum that maps to an external protocol (e.g., transaction types, ledger entry types).
- Always include an `Unknown` catch-all variant to handle protocol amendments without breaking downstream users.

## Wire Format Serialization
- When the protocol's flat JSON structure does not map naturally to Rust types, implement `Serialize` and `Deserialize` manually rather than deriving them.
- The serialized form must match the XRPL binary protocol field names exactly (e.g., `TransactionType`, `Account`, `Fee`).
- `Option` fields must be omitted entirely when `None` — never serialized as `null`.

## Builder Pattern
- Builders are type aliases over a generic `TransactionBuilder<T>`, where `T` is the shared domain struct (e.g. `pub type CheckCreateBuilder = TransactionBuilder<CheckCreate>`).
- Never duplicate fields in a local staging struct — reuse the domain struct directly. This keeps the builder and the transaction type in sync automatically.

## Rust API Design Guidelines
- **Prefer Borrowing:** Functions should take `&str`, `&[T]`, or `&Path` instead of owned collections unless ownership is strictly required.
- **Flexible Inputs:** Use `impl Into<T>` or `impl AsRef<T>` for public APIs to allow users to pass various compatible types (e.g., `&str` or `String`) seamlessly.
- **Explicit Ownership:** If the function needs to own the data (e.g., to store it in a struct), take the type by value (`T`). This lets the user choose whether to `clone()` or move their data.
- **Smart Lifetimes:** Use explicit lifetimes when returning references to internal data to avoid unnecessary allocations, but prioritize `'static` or owned data for "Builder" patterns to keep the API ergonomic.
