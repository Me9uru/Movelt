# Repository Guidelines

## Project Structure & Module Organization

Movel is a Tauri 2 reader with a Vue 3/TypeScript presentation layer and a Rust application backend. Frontend code lives in `src/`: feature pages in `src/pages/`, reusable UI in `src/components/`, thin Tauri-command adapters in `src/services/`, and UI view models in `src/domain/`. Static assets are in `public/`; helper scripts are in `scripts/`.

The native application is in `src-tauri/`. It owns outbound HTTP/WebSocket (SignalR) connections, authentication/session handling, response decompression, API-envelope validation, official payload parsing, and mapping into typed command DTOs. Organize this code by responsibility (for example `api/`, `auth/`, `commands/`, `dto/`, and `error/`) rather than putting it in `lib.rs`. Tauri configuration and capabilities live beside it.

## Official API Contract

[`api.md`](api.md) is the authoritative, repository-local reference for the official LightNovelShelf Web API. It was organized from the official Web project's actual calls and documents the HTTP endpoints, SignalR hub methods, authentication, payload casing, response envelope, gzip behavior, and official bookshelf semantics. Read it before changing any API-facing code.

Rust connects directly to the official service (`https://api.lightnovel.life`, or its documented Cloudflare alternative). The frontend calls explicit, typed Tauri commands and renders their DTOs; it does not own official API URLs, HTTP/WebSocket connections, authentication state, request construction, response parsing, decompression, or upstream-to-domain mapping. Authentication, bookshelf data, reading positions, and content remain owned by the official service; the Rust backend is the application's official-client boundary.

Keep the API contract at the boundary: send the documented PascalCase payload fields, handle `{ Success, Response, Status, Msg }` centrally, support the documented gzip response behavior, and keep SignalR connection/reconnect/token-refresh behavior in Rust. Commands must expose application DTOs and structured project errors rather than raw upstream envelopes, transport errors, or untyped JSON. Keep command inputs narrow and validate them before creating outbound requests. Persist refresh credentials only through an OS-backed secure credential facility; never return access or refresh tokens to the frontend.

Novel and manga are separate product domains: use novel methods for `BOOK` data and comic methods for `COMIC` data. The UI has independent novel and manga browsing/reading flows, while the unified bookshelf exposes separate 小说 / 漫画 tabs backed by the corresponding official shelf item type.

## Build, Test, and Development Commands

- `pnpm install` installs JavaScript dependencies.
- `pnpm dev` starts the Vite frontend with the Android-entry sync step.
- `pnpm tauri dev` runs the complete native app for local development.
- `pnpm build` type-checks with `vue-tsc` and creates a frontend production build.
- `pnpm tauri build` creates distributable native bundles.
- `cd src-tauri && cargo test` runs Rust unit tests.
- `cd src-tauri && cargo fmt --check` verifies Rust formatting.
- `cd src-tauri && cargo clippy --all-targets -- -D warnings` treats Rust lint warnings as errors.

## Coding Style & Naming Conventions

Use TypeScript with Vue Composition API and `<script setup lang="ts">`. Follow the existing two-space indentation, double quotes, trailing commas, and `camelCase`; components use `PascalCase` filenames such as `BookGrid.vue`. Keep feature types in `src/domain/`; `src/services/` may contain only typed wrappers around `@tauri-apps/api/core` `invoke`, with no official API URL, protocol, authentication, parsing, or business mapping logic. Frontend request types use the command's camelCase DTO contract; only Rust translates them to the official PascalCase fields described in `api.md`.

Format Rust with `cargo fmt`. Use `snake_case` for modules, functions, and tests; `PascalCase` for types; and narrow visibility (`pub(crate)`/`pub(super)`) where possible. Use `serde` DTOs with explicit rename rules at the upstream boundary, keep transport models separate from command DTOs, and return project errors rather than panicking in production paths. Register only named domain commands; do not add a generic URL/method/body invoke bridge.

### Visual Spacing

Use an 8px spacing grid for UI layout. Prefer the `--space-*` tokens for margins, padding, gaps, dimensions, and positional offsets; new control, card, and panel radii must use the shared 8px-based radius tokens. A 4px value is allowed only for small optical adjustments. One-pixel borders, typography, shadows, content-driven values (for example percentages, `auto`, and `clamp()`), and accessibility touch targets are exempt. Keep corresponding novel and manga screens visually aligned by reusing the same tokens and component patterns.

## Testing Guidelines

There is no frontend test runner, so run `pnpm build` after UI or TypeScript changes and manually exercise affected Tauri flows. Unit-test Rust request serialization, envelope/gzip decoding, DTO mapping, and command input validation without live credentials. For API changes, verify the relevant authenticated official flow (login, bookshelf, novel/comic reading, or read-position saving) when credentials are available. Run `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` for backend changes.

## Commit & Pull Request Guidelines

Recent history uses concise conventional-style prefixes: `feat:`, `fix:`, `refactor:`, `perf:`, and `chore:`. Keep commits focused and imperative. PRs should describe user-visible changes, list checks run, link issues when applicable, and include screenshots or recordings for UI changes. Call out schema, capability, or source-adapter changes.

## Security & Configuration

Keep Tauri capability changes minimal, and never commit credentials or generated `target/` output. Register narrowly scoped domain commands and do not log passwords, tokens, authorization headers, full upstream payloads, or reader content in production.
