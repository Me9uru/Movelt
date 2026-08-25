# Rust Backend Guidelines

## Scope and Structure

This file governs the Rust/Tauri backend in `src-tauri/`. For Vue and TypeScript code under `src/`, follow [`src/AGENTS.md`](src/AGENTS.md).

Movel is a Tauri 2 reader. The native application owns the official-service boundary: outbound HTTP/WebSocket (SignalR) connections, authentication and session handling, response decompression, API-envelope validation, official payload parsing, and mapping into typed command DTOs. Organize backend code by responsibility (for example `api/`, `auth/`, `commands/`, `dto/`, and `error/`) rather than accumulating it in `lib.rs`. Tauri configuration and capabilities live beside the backend.

## Official API Contract

[`api.md`](api.md) is the authoritative repository-local reference for the official LightNovelShelf Web API. Read it before changing API-facing code. It documents endpoints, SignalR hub methods, authentication, payload casing, response envelopes, gzip behavior, and official bookshelf semantics.

Rust connects directly to the official service (`https://api.lightnovel.life`, or its documented Cloudflare alternative). Keep official API URLs, HTTP/WebSocket connections, authentication state, request construction, response parsing, decompression, and upstream-to-domain mapping in Rust. Authentication, bookshelf data, reading positions, and content remain owned by the official service.

Send the documented PascalCase payload fields, handle `{ Success, Response, Status, Msg }` centrally, support documented gzip response behavior, and keep SignalR connection, reconnection, and token refresh in Rust. Commands must expose application DTOs and structured project errors rather than raw upstream envelopes, transport errors, or untyped JSON. Keep command inputs narrow and validate them before creating outbound requests. Persist refresh credentials only through an OS-backed secure credential facility; never return access or refresh tokens to the frontend.

Novel and manga are separate product domains: use novel methods for `BOOK` data and comic methods for `COMIC` data. The unified bookshelf has separate 小说 and 漫画 item types.

## Build, Test, and Development Commands

- `pnpm tauri dev` runs the complete native app for local development.
- `pnpm tauri build` creates distributable native bundles.
- `cd src-tauri && cargo test` runs Rust unit tests.
- `cd src-tauri && cargo fmt --check` verifies Rust formatting.
- `cd src-tauri && cargo clippy --all-targets -- -D warnings` treats Rust lint warnings as errors.

## Coding Style

Format Rust with `cargo fmt`. Use `snake_case` for modules, functions, and tests; `PascalCase` for types; and narrow visibility (`pub(crate)`/`pub(super)`) where possible. Use `serde` DTOs with explicit rename rules at the upstream boundary, keep transport models separate from command DTOs, and return project errors rather than panicking in production paths. Register only named domain commands; do not add a generic URL/method/body invoke bridge.

## Testing

Unit-test request serialization, envelope/gzip decoding, DTO mapping, and command-input validation without live credentials. For API changes, verify the relevant authenticated official flow (login, bookshelf, novel/comic reading, or read-position saving) when credentials are available. Run `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` for backend changes.

## Security and Configuration

Keep Tauri capability changes minimal, and never commit credentials or generated `target/` output. Register narrowly scoped domain commands and do not log passwords, tokens, authorization headers, full upstream payloads, or reader content in production.
