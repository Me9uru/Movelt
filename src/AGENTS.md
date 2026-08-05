# Vue Frontend Guidelines

## Scope and Architecture

These instructions apply to the Vue 3 and TypeScript frontend under `src/`.

- Keep `components/` focused on rendering and user interaction.
- Put reusable stateful behavior in `composables/`; name composables with a `use` prefix.
- Keep source-independent types in `domain/`, Tauri transport wrappers in `services/`, and provider-to-domain conversion in `sources/`.
- The reader UI consumes `ReaderDocument` and `ReaderBlock` only. Do not expose Bilinovel, EPUB, TXT, HTTP, or Tauri response details to reader components.
- Add new content providers by implementing or adapting to `ReaderSource`, not by branching on providers in the UI.

## Vue and TypeScript Style

- Use Vue Composition API with `<script setup lang="ts">` for components.
- Use two-space indentation, double quotes, and semicolons.
- Name Vue components in PascalCase and TypeScript modules in camelCase or concise descriptive lowercase.
- Prefer typed props, emits, return values, and boundary payloads. Avoid `any`; use `unknown` plus validation when external data is not trusted.
- Keep derived values in `computed` state and side effects explicit. Clean up timers, listeners, and subscriptions created by composables or components.
- Preserve loading, empty, error, and success states for asynchronous flows. Do not silently discard rejected Tauri invocations.

## Data and Persistence Boundaries

- Keep command names and request/response mapping in `services/`. Components and composables should call typed service functions instead of invoking Tauri directly.
- Normalize provider-specific content in `sources/` before returning it to the reader.
- Treat persisted library data and reader settings as compatibility-sensitive. Use stable identifiers and handle missing older fields with safe defaults.
- Keep the development invoke bridge development-only; never add production behavior that depends on it.

## Verification

- Run `pnpm build` after frontend changes; it performs TypeScript checking before the Vite production build.
- No frontend test runner is currently configured. Manually exercise affected loading, error, empty, and reader interaction paths with `pnpm dev` or `pnpm tauri dev` as appropriate.
- For UI changes, verify both narrow and wide window layouts and confirm keyboard/scroll reader behavior is unchanged unless intentionally modified.

