# Repository Guidelines

## Project Structure & Module Organization

This is a Tauri 2 desktop reader with a Vue 3/TypeScript frontend.

- `src/`: frontend code. `components/` holds Vue components, `composables/` stateful UI logic, `domain/` source-independent models, `sources/` provider adapters, and `services/` transport details.
- `src-tauri/src/`: Rust backend and Tauri commands. Novel domain types and the Bilinovel client/parser live under `src-tauri/src/novel/`.
- `public/` and `src/assets/`: static assets.
- `src-tauri/capabilities/`: Tauri permission configuration.

Keep the reader source-independent: EPUB, TXT, and network providers should convert content into `ReaderDocument` rather than adding provider logic to UI components.

## Build, Test, and Development Commands

- `pnpm install`: install dependencies.
- `pnpm dev`: run only the Vite frontend for UI work.
- `pnpm tauri dev`: run the complete desktop application and Rust backend.
- `pnpm build`: run `vue-tsc --noEmit` and create a production frontend bundle.
- `pnpm preview`: serve the production bundle locally.
- `cd src-tauri && cargo test`: run Rust tests.
- `cd src-tauri && cargo clippy --all-targets -- -D warnings`: enforce Rust lint cleanliness.
- `cd src-tauri && cargo fmt --check`: verify Rust formatting.

## Coding Style & Naming Conventions

Use two-space indentation in Vue and TypeScript, double quotes, and semicolons. Name components in PascalCase (`NovelReader.vue`), composables with a `use` prefix, and TypeScript modules in camelCase or descriptive lowercase. Prefer explicit return types at data-source boundaries.

Use `cargo fmt`, `snake_case` for Rust modules/functions, and `PascalCase` for types. Keep parsing, domain models, and Tauri command wiring separate.

## Testing Guidelines

Rust tests live beside implementations in `#[cfg(test)]` modules. Name tests after observable behavior, for example `parses_images_between_paragraphs`. Add parser and adapter tests for new content sources. No frontend test runner is configured; run `pnpm build` and manually exercise affected desktop/browser flows.

## Commit & Pull Request Guidelines

No usable commit history is available to infer an established convention. Use concise, imperative subjects, optionally with Conventional Commit prefixes such as `feat: add txt source adapter` or `fix: preserve reader settings`.

Pull requests should explain user-visible and architecture changes plus verification commands. Link issues, include UI screenshots, and call out Tauri capability, endpoint, or persisted-data changes.

## Security & Configuration

Never commit credentials or machine-specific endpoints. The debug invoke bridge must remain localhost-only and development-only. Review changes to `src-tauri/capabilities/` carefully and grant only the permissions required by the feature.
