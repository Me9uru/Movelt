# Novel

A Tauri 2 + Vue 3 desktop light-novel reader. Bilinovel search, detail,
catalogue and chapter loading run inside the Tauri Rust process through IPC
commands. Normal and release builds require no local HTTP server or extra
process; debug builds also expose the commands on localhost for Chrome testing.

## Run

```bash
pnpm install
pnpm tauri dev
```

## Debug in Chrome

This project uses an HTTP invoke bridge for browser debugging. Keep the Tauri
development process running so the Rust backend and the bridge are available:

```bash
pnpm tauri dev
```

Then open <http://localhost:1420> in Chrome. Calls made with
`@tauri-apps/api/core`'s `invoke()` are forwarded to the Tauri command handlers
through `http://127.0.0.1:3030`, so they appear in Chrome DevTools' Network tab.
The native Tauri window continues to use the normal IPC transport.

The bridge server binds only to `127.0.0.1` and starts only in Rust debug
builds. It allows browser origins through CORS, so do not expose or reuse it as
a production API. If port `3030` is occupied, stop the conflicting process
before starting `tauri dev`.

The data-source API defaults to `https://lnovel.animes.garden/`. It can be
overridden for development before starting Tauri:

```bash
BILINOVEL_API_BASE_URL=https://example.com/ pnpm tauri dev
```

The standalone [`gateway/`](gateway/README.md) remains as a debugging/reference
implementation, but the desktop application does not start or depend on it.

## Checks

```bash
pnpm build
cd src-tauri
cargo test
cargo clippy --all-targets -- -D warnings
```
