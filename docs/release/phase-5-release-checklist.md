# Phase 5 Release Checklist

This checklist closes the release and installer surface for AgentDock production builds. It is intentionally operational: every item should leave a command output, artifact, or human-verifiable note before a release is published.

## Preflight

- Confirm the release branch and working tree: `git status --short`.
- Sync versions before packaging: `npm run version:sync`.
- Run the JavaScript guardrails: `node --test tests/*.test.js`.
- Build the frontend: `npm run build`.
- Check the Tauri backend: `cargo check --manifest-path src-tauri/Cargo.toml`.

## Installer Metadata

- Verify `src-tauri/tauri.conf.json` keeps product-owned `productName`, `identifier`, `publisher`, `shortDescription`, `longDescription`, and `icon` entries.
- Verify Windows installers keep embedded WebView2 bootstrapper behavior and NSIS languages for `SimpChinese` and `English`.
- Verify `docs/update/latest.json` points at AgentDock-owned artifacts and includes the `rollback` strategy for failed web updates.

## Signing

- Windows release jobs must provide `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` before publishing installer artifacts.
- macOS release jobs must provide `APPLE_SIGNING_IDENTITY`, `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, and `APPLE_TEAM_ID` before notarized publishing.
- Linux release jobs may publish unsigned packages for the first production fork milestone, but any detached signature must be recorded beside the package and reflected in the artifact manifest.
- The generated `release-manifest.json` must show the signing status block without embedding certificate contents, passwords, tokens, or secrets.

## Build And Manifest

- Build the desktop bundle with `npm run tauri build`.
- Generate or refresh the artifact manifest with `npm run release:manifest -- --bundle-dir src-tauri/target/release/bundle`.
- Run the automated release smoke verifier with `npm run release:smoke -- --bundle-dir src-tauri/target/release/bundle`.
- Confirm `release-manifest.json` lists every installer or archive artifact with `path`, `kind`, `bytes`, and `sha256`.
- Confirm `checksums.sha256` contains one SHA256 line per release artifact and excludes generated manifest files.
- Recompute at least one installer checksum locally and compare it with `release-manifest.json`.

## Artifact Review

- Windows: verify NSIS setup and MSI artifacts are classified as `nsis` and `msi`.
- macOS: verify DMG and app archive artifacts are classified as `dmg` and `macos-app-archive`.
- Linux: verify AppImage, DEB, and RPM artifacts are classified as `appimage`, `deb`, and `rpm`.
- Archive or signature sidecars must be classified as `archive` or `signature`.

## Full Smoke

- Install the packaged app on a clean profile and confirm install succeeds.
- launch AgentDock from the installed shortcut or application entry.
- Confirm the main dashboard loads, settings read/write works, and product-owned config paths are used.
- Confirm OpenClaw installation detection, upgrade, uninstall, Node runtime detection, Git runtime detection, proxy diagnostics, and gateway restart flows still respond.
- Confirm assistant read-only, plan, and unlimited modes still enforce dangerous tool confirmation policy.
- Confirm logs, diagnostics, assistant command audit output, and Hermes log export apply secret redaction.
- Confirm update check reads `docs/update/latest.json` shape and failed update handling follows the configured rollback behavior.
- Confirm uninstall removes the application while preserving user data unless the installer explicitly asks for removal.

## Known Warnings To Recheck

- Vite may report existing dynamic/static import and chunk size warnings; record them in release notes if they remain.
- Rust may report existing unused product config helper warnings; remove those once the remaining command migration no longer needs compatibility helpers.
