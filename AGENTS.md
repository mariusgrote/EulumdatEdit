# AGENTS.md

## Cursor Cloud specific instructions

`eulumdat-edit` is a **Tauri 2 + SvelteKit 5 desktop app** (an EULUMDAT `.ldt` photometric file editor). The Rust backend (`src-tauri/`) and SvelteKit frontend (`src/`) run **in one process**; they communicate via Tauri `invoke` commands (see `src/lib/api.ts` ↔ `src-tauri/src/commands.rs`). There is no web server, database, or external service.

### Running the app
- Run everything with a single command: `pnpm tauri dev` (its `beforeDevCommand` auto-starts the Vite dev server on fixed port `1420`; do **not** start `pnpm dev` separately). Standard scripts live in `package.json`.
- The window needs a display. A desktop session is available on `DISPLAY=:1`; export it before launching (`export DISPLAY=:1`). Run in a tmux/background session since it is long-running.
- `libEGL warning: DRI3 error ...` lines at startup are harmless software-rendering fallback messages, not failures.
- The app window title bar is intentionally empty (`tauri.conf.json` sets `"title": ""`); identify the window by its UI, not its title.

### Lint / test / build (matches `.github/workflows/ci.yml`)
- Frontend check: `pnpm check` (runs `svelte-kit sync` + `svelte-check`).
- Rust (run inside `src-tauri/`): `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo check --all-targets`, `cargo test`.

### Non-obvious gotchas
- **Rust toolchain**: the crate's git dependency `eulumdat-core` requires `edition2024`, so **Rust ≥ 1.85 is mandatory** (`rustup default stable`). An older pinned default (e.g. 1.83) fails with `feature edition2024 is required`.
- The first Rust build fetches `eulumdat-core` from GitHub (`Cargo.toml` git dependency), so initial compilation needs network access.
- Tauri needs Linux system libs (`libwebkit2gtk-4.1-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `patchelf`); these are preinstalled in the cloud snapshot.
