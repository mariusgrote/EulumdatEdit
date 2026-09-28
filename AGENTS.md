# AGENTS.md

EulumdatEdit is a Tauri 2 desktop app with a Svelte 5 frontend. The frontend calls Rust commands through `src/lib/api.ts` and `src-tauri/src/commands.rs`.

To run the desktop app, use `pnpm tauri dev`. Tauri starts the Vite server through `beforeDevCommand`; do not start `pnpm dev` separately.

The `eulumdat-core` Git dependency uses Rust edition 2024. Use Rust 1.85 or newer when building the app.
