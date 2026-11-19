# AGENTS.md — Instructions for Adding or Updating Aidoku Extensions

This repository ports Mihon (`keiyoushi/extensions-source`) extensions to Aidoku (`aidoku-rs` / WebAssembly `.aix` packages).

## Repository Layout Rules
- **Do NOT create a root `Cargo.toml` workspace.** Each source in `sources/<id>/` is a standalone Cargo crate with its own `Cargo.lock`, `.cargo/config.toml` (`target = "wasm32-unknown-unknown"`), `[profile.dev]`, and `[profile.release]`.
- Keep `extensions.json` up to date whenever adding, updating, or removing a source.
- Every `.aix` archive in `public/sources/` is a ZIP containing `Payload/main.wasm`, `Payload/source.json`, `Payload/icon.png`, `Payload/filters.json`, and optional `Payload/settings.json`.

## Adding or Updating an Extension from Mihon
Use the unified CLI (`node scripts/mhaid.mjs` or `npm run ...`):
1. Run:
   ```bash
   node scripts/mhaid.mjs port <mihon-folder-name> [--lang <lang>] [--watch] [--community]
   ```
   - Inspect `sources/<id>/MIHON_SOURCE.md` and `.cache/briefs/<id>.md` for extracted Mihon Kotlin/JS code and Aidoku 0.8.4 compatibility rules.
   - Other useful commands: `list`, `diff <id>`, `watch <id...>`, `unwatch <id...>`, `sync [--dry-run] [--id <id>] [--all]`, `bump <id>`, `pull-pages`, `verify`.
   - Automated CI sync (`.github/workflows/sync-upstream.yaml`) only checks extensions marked `"autoSync": true` in `extensions.json`.
   - For `minAppVersion < 0.9.0`, never combine `DynamicSettings` with `LoginSetting` (Postcard ABI breaks Settings on Aidoku 0.8.4; declare `login` settings in static `res/settings.json`), and avoid 0.9.0-only `WebView` methods (`eval_async`, `add_user_script`, `set_rule_list`).
2. Commit and push to `main`. GitHub Actions (`.github/workflows/build.yaml`) will compile all `sources/*/Cargo.toml` crates via `aidoku package`, rebuild `public/index.min.json` via `aidoku build`, verify with `node scripts/build-repo.mjs --verify`, and push the output to `gh-pages`.
3. Pull the built artifacts back into `main`:
   ```bash
   node scripts/mhaid.mjs pull-pages
   ```

## Verification
Always validate manifest, settings/filters schemas, 0.8.4 compatibility, and `.aix` package integrity before pushing:
```bash
node scripts/mhaid.mjs verify
```
