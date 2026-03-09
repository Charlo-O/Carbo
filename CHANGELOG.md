# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

## 1.5.1 - 2026-03-09

- Fix: repair GitHub Release workflow by pinning `tauri-apps/tauri-action@v0.6.0`.
- Release: retrigger desktop packaging for Windows and unsigned macOS bundles.

## 1.5.0 - 2026-03-09

- Fix: make file drag-and-drop work on Tauri v2 and preserve real filesystem paths by switching to `getCurrentWebview().onDragDropEvent`.
- Security: add Tauri v2 capability file (`src-tauri/capabilities/default.json`) to allow required core event listening permissions.
- Fix: read dropped markdown files via a Rust command (`read_text_file`) instead of `fetch(convertFileSrc(...))` to avoid asset protocol scope limitations.
- Fix: render local images reliably by copying dropped images into AppData (`copy_image_to_app_data`) and enabling `assetProtocol` scope for `$APPDATA/carbo-assets/images/**`.
- Config: update CSP to include `http://asset.localhost` and enable `app.security.assetProtocol`.
- Feat(MVP): add GitHub image bed via Contents API with user-configurable repo/branch/pathPrefix/token; upload runs in a sequential queue and replaces local image URLs with `raw.githubusercontent.com` links.

### Verification

- `npm run build`
- `npm run tauri:dev` (drag/drop `.md` and image files)
- `cargo check` (in `src-tauri`)
