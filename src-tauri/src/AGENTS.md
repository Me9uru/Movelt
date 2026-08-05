# Rust Backend Guidelines

## Scope and Architecture

These instructions apply to the Rust backend under `src-tauri/src/`.

- Keep `lib.rs` limited to application setup, managed state registration, plugins, and Tauri command registration.
- Tauri commands should validate input and orchestrate work. Keep HTTP fetching, parsing, persistence, and domain conversion out of command bodies.
- Keep source-independent novel types in `novel/domain.rs`, provider code under its provider module, and reusable error mapping in `novel/error.rs`.
- Within a provider, keep transport concerns in `client.rs` and HTML/content extraction in `parser.rs`. Parsers should accept fetched input and remain deterministic where practical.
- Keep SQLite ownership and bookshelf/progress operations in the library layer. Do not share database connections or mutable state outside their managed-state abstraction without a clear synchronization strategy.

## Rust and API Style

- Follow `cargo fmt`; use `snake_case` for modules and functions and `PascalCase` for types.
- Return structured, serializable domain values from Tauri commands. Keep frontend-facing field names and command payloads stable unless both sides are migrated together.
- Use `Result` and typed errors for recoverable failures. Avoid `unwrap`, `expect`, and panics in request, parsing, and persistence paths; startup-only failures may include actionable context.
- Validate user-controlled identifiers, query text, page numbers, URLs, and other bounds before network or database work.
- Do not hold blocking mutex guards across `.await`. Keep blocking filesystem or SQLite work out of async network sections, and use an appropriate blocking boundary when operations become expensive.
- Never log credentials, full sensitive content, or machine-specific paths.

## Parsing and Provider Changes

- Treat remote HTML as untrusted and subject to layout drift. Return descriptive errors for missing required fields and tolerate optional metadata where possible.
- Resolve and validate links deliberately; do not accept unsupported schemes or silently cross provider boundaries.
- Convert provider output into the shared novel domain before it reaches Tauri command wiring.
- Add focused fixtures or inline samples for parser behavior. Tests must not depend on a live remote site unless explicitly designed as ignored integration tests.

## Tests and Verification

- Place unit tests beside implementations in `#[cfg(test)]` modules and name them after observable behavior.
- Add regression tests for parser fixes, input validation, error mapping, and persistence changes.
- Run from `src-tauri/`:
  - `cargo fmt --check`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
- When a Rust API or serialized type changes, also run `pnpm build` from the repository root and manually verify the corresponding Tauri flow.

## Security and Capabilities

- Keep the debug invoke bridge localhost-only and development-only.
- Capability or permission changes belong under `src-tauri/capabilities/` or `src-tauri/permissions/`; grant the narrowest permission required and call out the change during review.
- Never commit credentials, session tokens, or machine-specific service endpoints.
