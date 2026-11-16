# AGENTS.md — Instructions for Adding or Updating Aidoku Extensions

This repository ports Mihon (`keiyoushi/extensions-source`) extensions to Aidoku (`aidoku-rs` / WebAssembly `.aix` packages).

## Repository Layout Rules
- **Do NOT create a root `Cargo.toml` workspace.** Each source in `sources/<id>/` is a standalone Cargo crate with its own `Cargo.lock`, `.cargo/config.toml` (`target = "wasm32-unknown-unknown"`), `[profile.dev]`, and `[profile.release]`.
- Keep `extensions.json` up to date whenever adding, updating, or removing a source.
- Every `.aix` archive in `public/sources/` is a ZIP containing `Payload/main.wasm`, `Payload/source.json`, `Payload/icon.png`, `Payload/filters.json`, and optional `Payload/settings.json`.

## Adding a New Extension from Mihon
1. Run:
   ```bash
   node scripts/mihon-to-aidoku.mjs <mihon-folder-name> [--lang <lang>]
   ```
   - Inspect `sources/<id>/MIHON_SOURCE.md` for all extracted Mihon Kotlin endpoints, CSS selectors, filters, and custom headers.
   - If the source was scaffolded (`status: "scaffold"` in `extensions.json`), complete `sources/<id>/src/lib.rs`, `res/filters.json`, and `res/source.json` using `MIHON_SOURCE.md` and the `aidoku` v0.3.0 / `aidoku-rs` API, then set `"status": "ready"` in `extensions.json`.
   - If the source was imported with a prebuilt `.aix`, run `node scripts/build-repo.mjs --verify` right away.
2. Commit and push to `main`. GitHub Actions (`.github/workflows/build.yaml`) will compile all `sources/*/Cargo.toml` crates via `aidoku package`, rebuild `public/index.min.json` via `aidoku build`, verify with `node scripts/build-repo.mjs --verify`, and push the output to the `gh-pages` branch.

## Verification
Always validate manifest and `.aix` package integrity before pushing:
```bash
node scripts/build-repo.mjs --verify
```
