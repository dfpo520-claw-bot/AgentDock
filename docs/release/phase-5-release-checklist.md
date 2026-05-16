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
- Windows Authenticode signing must run `npm run release:sign:windows -- --file <installer>` with `WINDOWS_CODESIGN_CERT_THUMBPRINT` or `--thumbprint <sha1>` before the publish verification step.
- Windows local smoke may run `npm run release:signing:windows -- --file <installer> --allow-unsigned`, but publish candidates must run the same command without `--allow-unsigned`.
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
- Confirm AgentDock final brand assets, DeepAi assistant naming, and release links are visible in the installed app.
- Confirm uninstall removes the application while preserving user data unless the installer explicitly asks for removal.
- Browser-driven UI route smoke has already passed for `/dashboard`, `/settings`, `/services`, `/assistant`, `/logs`, and `/about`; evidence is recorded in `docs/release/phase-5-ui-smoke-2026-05-15.md`.
- Refreshed UI smoke screenshots should show the AgentDock sidebar brand, security banner without header overlap or shell clipping, and Modern Ops route layout on Dashboard, Settings, Services, Logs, Assistant, and About.
- The remaining manual smoke items should focus on installed-app actions that mutate local state or services: Gateway restart, install/upgrade/uninstall flows, Settings writes, assistant tool execution, log export, and uninstall UX.

## Known Warnings To Recheck

- Vite may report the existing `i18n` chunk size warning; record it in release notes if it remains.
- Rust product config helper warnings were cleaned in the Phase 5 hardening pass; re-run `cargo check --manifest-path src-tauri/Cargo.toml` before publishing.
