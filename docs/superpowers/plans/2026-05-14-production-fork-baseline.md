# Production Fork Baseline Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Produce the first runnable production-branded fork baseline while preserving the current Tauri v2, Vite, and Rust command architecture.

**Architecture:** Keep the existing app shell and command layer intact, add a centralized product identity module, replace first-run visible brand surfaces, regenerate temporary brand assets, and verify frontend/Rust/desktop build health. OpenClaw and Hermes remain available as engines in this baseline.

**Tech Stack:** Vite 6, vanilla JavaScript modules, Node.js `node:test`, Tauri v2, Rust 2021, PowerShell asset generation.

---

## Scope

This plan implements Phase 1 from `docs/superpowers/specs/2026-05-14-production-fork-refactor-design.md`.

Use `AgentDock` as the temporary production identity:

- Product name: `AgentDock`
- Package id: `agentdock`
- Tauri identifier: `com.agentdock.desktop`
- Homepage: `https://agentdock.example.com`
- Support URL: `https://agentdock.example.com/support`
- Repository URL: `https://agentdock.example.com/source`
- Update manifest URL: `https://agentdock.example.com/update/latest.json`

The plan intentionally does not rename OpenClaw, Hermes Agent, ClawApp, cftunnel, or other integration names when they describe external engines or projects. This baseline replaces the app shell and distribution identity, not the engine ecosystem.

## File Structure

Create:

- `src/lib/product-identity.js` - single frontend source of truth for temporary product identity.
- `tests/product-identity.test.js` - verifies identity constants are complete and not still set to the fork brand.
- `tests/brand-surface.test.js` - verifies first-run brand surfaces no longer expose old app identity.
- `scripts/generate-brand-assets.ps1` - generates temporary PNG brand assets for docs, public images, favicon source, and Tauri icon source.
- `docs/agentdock-icon.png` - generated source icon for Tauri icon regeneration.
- `docs/agentdock-logo-brand.png` - generated temporary documentation logo.
- `public/favicon-source.png` - generated temporary favicon PNG source retained for repeatable favicon regeneration.

Modify:

- `package.json` - npm package metadata and scripts.
- `src-tauri/tauri.conf.json` - product name, identifier, window title, icon paths remain generated in `src-tauri/icons`.
- `src-tauri/Cargo.toml` - Rust package metadata and lib name.
- `src-tauri/Cargo.lock` - Rust package lock metadata after the package rename.
- `src-tauri/src/main.rs` - call the renamed Rust library crate.
- `index.html` - title, splash product name, user-facing links, old app name in splash diagnostics.
- `src/components/sidebar.js` - sidebar logo alt/title and footer links use product identity.
- `src/main.js` - login overlay title/footer links and boot failure copy use product identity.
- `src/pages/about.js` - primary About page identity, product cards, release links, and support links.
- `README.md` - concise Chinese production README for AgentDock.
- `README.en.md` - concise English production README for AgentDock.
- `package-lock.json` - package name metadata after `npm install --package-lock-only`.

Verification:

- `node --test tests/*.test.js`
- `npm run build`
- `cargo check --manifest-path src-tauri/Cargo.toml`
- `npm run tauri -- info`

## Task 1: Establish A Clean Baseline

**Files:**
- Read: `docs/superpowers/specs/2026-05-14-production-fork-refactor-design.md`
- Read: `package.json`
- Read: `src-tauri/tauri.conf.json`
- Read: `src-tauri/Cargo.toml`
- Read: `src-tauri/src/main.rs`
- Read: `index.html`
- Read: `src/components/sidebar.js`
- Read: `src/main.js`
- Read: `src/pages/about.js`
- Read: `README.md`
- Read: `README.en.md`

- [ ] **Step 1: Check the current branch and working tree**

Run:

```powershell
git -c safe.directory=D:/workSpace/ohter/clawpanel branch --show-current
git -c safe.directory=D:/workSpace/ohter/clawpanel status --short
```

Expected:

```text
codex/clawpanel-new
 M docs/superpowers/specs/2026-05-14-production-fork-refactor-design.md
?? docs/superpowers/plans/2026-05-14-production-fork-baseline.md
```

If additional unrelated files appear, do not revert them. Record them in the task notes and avoid editing them.

- [ ] **Step 2: Run existing JavaScript tests before changing code**

Run:

```powershell
node --test tests/*.test.js
```

Expected:

```text
# pass
```

If the command fails because dependencies are missing, run:

```powershell
npm install
node --test tests/*.test.js
```

Expected after dependency install:

```text
# pass
```

- [ ] **Step 3: Run the current frontend build**

Run:

```powershell
npm run build
```

Expected:

```text
vite v6
built
```

- [ ] **Step 4: Run the current Rust check**

Run:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected:

```text
Finished `dev` profile
```

- [ ] **Step 5: Commit the baseline plan only**

Run:

```powershell
git -c safe.directory=D:/workSpace/ohter/clawpanel add docs/superpowers/plans/2026-05-14-production-fork-baseline.md
git -c safe.directory=D:/workSpace/ohter/clawpanel commit -m "docs: add production fork baseline plan"
```

Expected:

```text
[codex/clawpanel-new <hash>] docs: add production fork baseline plan
```

## Task 2: Add Product Identity Constants

**Files:**
- Create: `src/lib/product-identity.js`
- Create: `tests/product-identity.test.js`

- [ ] **Step 1: Write the failing product identity test**

Create `tests/product-identity.test.js` with:

```javascript
import test from 'node:test'
import assert from 'node:assert/strict'

import {
  PRODUCT_IDENTITY,
  productTitle,
} from '../src/lib/product-identity.js'

test('PRODUCT_IDENTITY exposes the production fork identity', () => {
  assert.equal(PRODUCT_IDENTITY.id, 'agentdock')
  assert.equal(PRODUCT_IDENTITY.name, 'AgentDock')
  assert.equal(PRODUCT_IDENTITY.displayName, 'AgentDock')
  assert.equal(PRODUCT_IDENTITY.tauriIdentifier, 'com.agentdock.desktop')
  assert.equal(PRODUCT_IDENTITY.homepage, 'https://agentdock.example.com')
  assert.equal(PRODUCT_IDENTITY.supportUrl, 'https://agentdock.example.com/support')
  assert.equal(PRODUCT_IDENTITY.repositoryUrl, 'https://agentdock.example.com/source')
  assert.equal(PRODUCT_IDENTITY.updateManifestUrl, 'https://agentdock.example.com/update/latest.json')
})

test('visible identity fields no longer use the fork app name', () => {
  const visible = [
    PRODUCT_IDENTITY.name,
    PRODUCT_IDENTITY.displayName,
    PRODUCT_IDENTITY.tagline,
    PRODUCT_IDENTITY.description,
    PRODUCT_IDENTITY.homepage,
    PRODUCT_IDENTITY.supportUrl,
    PRODUCT_IDENTITY.repositoryUrl,
    PRODUCT_IDENTITY.updateManifestUrl,
  ].join('\n')

  assert.doesNotMatch(visible, /ClawPanel/)
  assert.doesNotMatch(visible, /claw\.qt\.cool/)
  assert.doesNotMatch(visible, /qingchencloud\/clawpanel/)
})

test('productTitle formats document and window titles', () => {
  assert.equal(productTitle(), 'AgentDock')
  assert.equal(productTitle('Settings'), 'AgentDock - Settings')
})
```

- [ ] **Step 2: Run the new test and verify it fails**

Run:

```powershell
node --test tests/product-identity.test.js
```

Expected:

```text
Error [ERR_MODULE_NOT_FOUND]
```

- [ ] **Step 3: Create the product identity implementation**

Create `src/lib/product-identity.js` with:

```javascript
export const PRODUCT_IDENTITY = Object.freeze({
  id: 'agentdock',
  name: 'AgentDock',
  displayName: 'AgentDock',
  tagline: 'Multi-engine AI agent operations console',
  description: 'AgentDock - production desktop console for multi-engine AI agent operations',
  tauriIdentifier: 'com.agentdock.desktop',
  homepage: 'https://agentdock.example.com',
  supportUrl: 'https://agentdock.example.com/support',
  repositoryUrl: 'https://agentdock.example.com/source',
  updateManifestUrl: 'https://agentdock.example.com/update/latest.json',
  legacyProductName: 'ClawPanel',
})

export function productTitle(suffix = '') {
  const clean = String(suffix || '').trim()
  return clean ? `${PRODUCT_IDENTITY.name} - ${clean}` : PRODUCT_IDENTITY.name
}
```

- [ ] **Step 4: Run the identity test and verify it passes**

Run:

```powershell
node --test tests/product-identity.test.js
```

Expected:

```text
# pass
```

- [ ] **Step 5: Commit the identity module**

Run:

```powershell
git -c safe.directory=D:/workSpace/ohter/clawpanel add src/lib/product-identity.js tests/product-identity.test.js
git -c safe.directory=D:/workSpace/ohter/clawpanel commit -m "feat: add product identity constants"
```

Expected:

```text
[codex/clawpanel-new <hash>] feat: add product identity constants
```

## Task 3: Replace Package And Desktop Metadata

**Files:**
- Modify: `package.json`
- Modify: `package-lock.json`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Write the failing metadata surface test**

Create `tests/brand-surface.test.js` with:

```javascript
import test from 'node:test'
import assert from 'node:assert/strict'
import fs from 'node:fs'

const BRAND_SURFACE_FILES = [
  'package.json',
  'src-tauri/tauri.conf.json',
  'src-tauri/Cargo.toml',
  'src-tauri/src/main.rs',
  'index.html',
  'src/components/sidebar.js',
  'src/main.js',
  'src/pages/about.js',
  'README.md',
  'README.en.md',
]

const OLD_APP_PATTERNS = [
  /ClawPanel/g,
  /claw\.qt\.cool/g,
  /qingchencloud\/clawpanel/g,
  /ai\.openclaw\.clawpanel/g,
]

function stripJavaScriptComments(text) {
  return text
    .replace(/\/\*[\s\S]*?\*\//g, '')
    .replace(/(^|[^:])\/\/.*$/gm, '$1')
}

test('first-run brand surfaces use AgentDock identity', () => {
  for (const file of BRAND_SURFACE_FILES) {
    const raw = fs.readFileSync(file, 'utf8')
    const text = file.endsWith('.js') || file.endsWith('.rs')
      ? stripJavaScriptComments(raw)
      : raw
    for (const pattern of OLD_APP_PATTERNS) {
      assert.doesNotMatch(text, pattern, `${file} still matches ${pattern}`)
    }
  }
})
```

- [ ] **Step 2: Run the brand surface test and verify it fails**

Run:

```powershell
node --test tests/brand-surface.test.js
```

Expected:

```text
not ok
package.json still matches /ClawPanel/g
```

- [ ] **Step 3: Update `package.json` metadata**

Edit the top-level metadata in `package.json` to:

```json
{
  "name": "agentdock",
  "version": "0.15.3",
  "private": true,
  "description": "AgentDock - production desktop console for multi-engine AI agent operations",
  "type": "module",
  "author": "AgentDock Team",
  "license": "TBD",
  "homepage": "https://agentdock.example.com",
  "repository": {
    "type": "git",
    "url": "https://agentdock.example.com/source.git"
  },
  "bugs": {
    "url": "https://agentdock.example.com/support"
  },
  "keywords": [
    "ai-agent",
    "tauri",
    "desktop-app",
    "management-panel",
    "agentdock"
  ]
}
```

Keep the existing `scripts`, `dependencies`, and `devDependencies` sections unchanged.

- [ ] **Step 4: Update `package-lock.json` package name**

Run:

```powershell
npm install --package-lock-only
```

Expected:

```text
up to date
```

Then verify:

```powershell
node -e "const p=require('./package-lock.json'); if(p.name!=='agentdock') throw new Error(p.name); if(p.packages[''].name!=='agentdock') throw new Error(p.packages[''].name); console.log('package-lock ok')"
```

Expected:

```text
package-lock ok
```

- [ ] **Step 5: Update `src-tauri/tauri.conf.json` desktop metadata**

Change these fields:

```json
{
  "productName": "AgentDock",
  "version": "0.15.3",
  "identifier": "com.agentdock.desktop",
  "app": {
    "windows": [
      {
        "title": "AgentDock"
      }
    ]
  }
}
```

Keep all other existing fields in `src-tauri/tauri.conf.json` unchanged.

- [ ] **Step 6: Update Rust package metadata**

Change the top of `src-tauri/Cargo.toml` to:

```toml
[package]
name = "agentdock"
version = "0.15.3"
edition = "2021"
description = "AgentDock - production desktop console for multi-engine AI agent operations"
authors = ["AgentDock Team"]
repository = "https://agentdock.example.com/source"
homepage = "https://agentdock.example.com"
license = "TBD"

[lib]
name = "agentdock_lib"
crate-type = ["lib", "cdylib", "staticlib"]
```

Keep the existing dependencies unchanged.

- [ ] **Step 7: Update the Tauri binary entrypoint**

Change `src-tauri/src/main.rs` to:

```rust
// AgentDock entrypoint
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    agentdock_lib::run()
}
```

- [ ] **Step 8: Verify metadata tests still fail only on UI/docs files**

Run:

```powershell
node --test tests/brand-surface.test.js
```

Expected:

```text
not ok
index.html still matches /ClawPanel/g
```

The test should no longer report `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, or `src-tauri/src/main.rs`.

- [ ] **Step 9: Verify Rust metadata still compiles**

Run:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected:

```text
Finished `dev` profile
```

- [ ] **Step 10: Confirm `src-tauri/Cargo.lock` reflects the package rename**

Run:

```powershell
Select-String -Path src-tauri/Cargo.lock -Pattern 'name = "agentdock"'
```

Expected:

```text
name = "agentdock"
```

- [ ] **Step 11: Commit metadata changes**

Run:

```powershell
git -c safe.directory=D:/workSpace/ohter/clawpanel add package.json package-lock.json src-tauri/tauri.conf.json src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/main.rs tests/brand-surface.test.js
git -c safe.directory=D:/workSpace/ohter/clawpanel commit -m "chore: rename package metadata to AgentDock"
```

Expected:

```text
[codex/clawpanel-new <hash>] chore: rename package metadata to AgentDock
```

## Task 4: Replace First-Run Shell Branding

**Files:**
- Modify: `index.html`
- Modify: `src/components/sidebar.js`
- Modify: `src/main.js`

- [ ] **Step 0: Find all remaining app identity matches in shell files**

Run:

```powershell
rg "ClawPanel|claw\.qt\.cool|qingchencloud/clawpanel|ai\.openclaw\.clawpanel" index.html src/components/sidebar.js src/main.js
```

Replace or remove every match in these shell files unless the match is inside a comment documenting historical context that is not part of runtime app identity. Do not keep old app identity in file headers, mobile topbar labels, release/download URLs, login surfaces, footer links, or boot-failure copy.

- [ ] **Step 1: Update `index.html` static splash identity**

Replace these values in `index.html`:

```html
<title>AgentDock</title>
```

```html
<span class="sp-name">AgentDock</span>
```

```html
<a href="https://agentdock.example.com" target="_blank" rel="noopener">agentdock.example.com</a>
```

In the Chinese splash strings, replace the timeout diagnostic with:

```javascript
'diag.timeout': '<strong>可尝试的处理方式</strong> · 刷新当前窗口、重启 AgentDock、检查 WebView2 Runtime，或前往官网下载最新版本。',
```

In the English splash strings, replace the timeout diagnostic with:

```javascript
'diag.timeout': '<strong>Things to try</strong> · refresh this window, restart AgentDock, verify WebView2 Runtime, or download the latest build from the official website.',
```

In splash action links, replace:

```javascript
+ '<a class="sp-btn" href="https://agentdock.example.com" target="_blank" rel="noopener">' + t('btn.site') + '</a>'
```

Keep `clawpanel_lang` localStorage keys unchanged in Phase 1 because changing persisted keys belongs to the configuration ownership phase.

- [ ] **Step 2: Update sidebar branding imports**

At the top of `src/components/sidebar.js`, add:

```javascript
import { PRODUCT_IDENTITY } from '../lib/product-identity.js'
```

- [ ] **Step 3: Update sidebar logo/title/footer**

In `renderSidebar`, replace the sidebar header and meta identity with:

```javascript
    <div class="sidebar-header">
      <div class="sidebar-logo">
        <img src="/images/logo.png" alt="${PRODUCT_IDENTITY.name}">
      </div>
      <span class="sidebar-title">${PRODUCT_IDENTITY.name}</span>
      <button class="sidebar-collapse-btn" id="btn-sidebar-collapse" title="${t('sidebar.collapse')}">${collapsed ? '禄' : '芦'}</button>
      <button class="sidebar-close-btn" id="btn-sidebar-close" title="${t('sidebar.closeMenu')}">&times;</button>
    </div>
```

Replace sidebar meta with:

```javascript
      <div class="sidebar-meta">
        <a href="${PRODUCT_IDENTITY.homepage}" target="_blank" rel="noopener" class="sidebar-link">agentdock.example.com</a>
        <span class="sidebar-version">v${APP_VERSION}</span>
      </div>
```

- [ ] **Step 4: Update `src/main.js` imports**

Add this import near other local imports:

```javascript
import { PRODUCT_IDENTITY } from './lib/product-identity.js'
```

- [ ] **Step 5: Update login overlay title and footer links**

In `src/main.js`, replace visible login title/footer literals with:

```javascript
      <div class="login-title">${PRODUCT_IDENTITY.name}</div>
```

```javascript
        <a href="${PRODUCT_IDENTITY.homepage}" target="_blank" rel="noopener" style="color:#aaa;text-decoration:none">agentdock.example.com</a>
```

Apply the same footer replacement in `showBackendDownOverlay()` and `showLoginOverlay()`.

- [ ] **Step 6: Update boot failure support link**

In the boot failure markup in `src/main.js`, replace the issues link with:

```javascript
<a href="${PRODUCT_IDENTITY.supportUrl}" target="_blank" style="color:#6366f1">${PRODUCT_IDENTITY.supportUrl}</a>
```

- [ ] **Step 7: Run brand surface test and verify About/README remain**

Run:

```powershell
node --test tests/brand-surface.test.js
```

Expected:

```text
not ok
src/pages/about.js still matches /ClawPanel/g
```

If `src/main.js` still appears in the failure output, return to Step 0 and remove the remaining app-identity match before committing Task 4.

- [ ] **Step 8: Commit shell branding**

Run:

```powershell
git -c safe.directory=D:/workSpace/ohter/clawpanel add index.html src/components/sidebar.js src/main.js
git -c safe.directory=D:/workSpace/ohter/clawpanel commit -m "feat: apply AgentDock shell branding"
```

Expected:

```text
[codex/clawpanel-new <hash>] feat: apply AgentDock shell branding
```

## Task 5: Replace About Page And Public README Branding

**Files:**
- Modify: `src/pages/about.js`
- Modify: `README.md`
- Modify: `README.en.md`

- [ ] **Step 0: Find all remaining app identity matches in About and README files**

Run:

```powershell
rg "ClawPanel|claw\.qt\.cool|qingchencloud/clawpanel|ai\.openclaw\.clawpanel" src/pages/about.js README.md README.en.md
```

Replace or remove every match that describes this app's identity, documentation, release entry points, support URLs, repository URLs, download URLs, or user-facing product labels. Keep external integration names only when they identify external projects, engines, or dependencies rather than this app. Comments in `src/pages/about.js` may retain historical context only if they are not user-facing and not part of runtime app identity; `tests/brand-surface.test.js` strips JavaScript comments before scanning.

- [ ] **Step 1: Update About page imports**

Add this import to `src/pages/about.js`:

```javascript
import { PRODUCT_IDENTITY } from '../lib/product-identity.js'
```

- [ ] **Step 2: Replace About page primary identity**

Replace the header logo/title/description block in `src/pages/about.js` with:

```javascript
    <div style="display:flex;align-items:center;gap:14px;margin-bottom:20px">
      <img src="/images/logo-brand.png" alt="${PRODUCT_IDENTITY.name}" style="height:48px;width:auto">
      <div>
        <h1 class="page-title" style="margin:0">${PRODUCT_IDENTITY.name}</h1>
        <p class="page-desc" style="margin:0">${PRODUCT_IDENTITY.tagline} · <a href="${PRODUCT_IDENTITY.homepage}" target="_blank" rel="noopener" style="color:var(--primary)">agentdock.example.com</a></p>
      </div>
    </div>
```

Replace stat labels that show the app name with:

```javascript
<span class="stat-card-label">${PRODUCT_IDENTITY.name}</span>
```

- [ ] **Step 3: Replace About page download and support URLs**

Replace website download anchors with:

```javascript
<a class="btn btn-primary btn-sm" href="${PRODUCT_IDENTITY.homepage}" target="_blank" rel="noopener" style="${btnSm}">${t('about.downloadFromWebsite')}</a>
```

Replace release fallback URLs with:

```javascript
${info.url || PRODUCT_IDENTITY.repositoryUrl}
```

Replace contribution/support buttons with:

```javascript
      <a class="btn btn-primary btn-sm" href="${PRODUCT_IDENTITY.supportUrl}" target="_blank" rel="noopener">${t('about.submitIssue')}</a>
      <a class="btn btn-secondary btn-sm" href="${PRODUCT_IDENTITY.repositoryUrl}" target="_blank" rel="noopener">${t('about.submitPR')}</a>
      <a class="btn btn-secondary btn-sm" href="${PRODUCT_IDENTITY.repositoryUrl}" target="_blank" rel="noopener">${t('about.contributeGuide')}</a>
      <a class="btn btn-secondary btn-sm" href="${PRODUCT_IDENTITY.supportUrl}" target="_blank" rel="noopener">${t('about.viewIssues')}</a>
```

Replace contact links in the footer area with:

```javascript
<a href="${PRODUCT_IDENTITY.homepage}" target="_blank" rel="noopener" style="color:var(--accent)">agentdock.example.com</a>
```

- [ ] **Step 4: Replace the product card entry**

Change the product card object for this app to:

```javascript
  {
    name: PRODUCT_IDENTITY.name,
    desc: PRODUCT_IDENTITY.description,
    url: PRODUCT_IDENTITY.repositoryUrl,
    gitee: PRODUCT_IDENTITY.homepage,
  },
```

Do not rename external project cards for ClawApp or cftunnel in this task.

- [ ] **Step 5: Replace `README.md` with concise Chinese production README**

Replace `README.md` with:

```markdown
# AgentDock

AgentDock 是一个生产级多引擎 AI Agent 桌面控制台，基于 Tauri v2、Vite 和 Rust command 层构建。

## 当前阶段

本仓库当前处于生产 Fork 基线阶段：

- 保留现有 OpenClaw 与 Hermes Agent 引擎能力。
- 保留 Tauri 桌面壳、Web 模式、服务管理、模型管理、Agent 管理、聊天、日志、诊断、备份和扩展模块。
- 先替换产品身份、图标、文档、应用元数据和发布入口。
- 后续逐步重构配置归属、模块边界和后端命令实现。

## 技术栈

- 前端：Vite + Vanilla JavaScript
- 桌面端：Tauri v2
- 后端能力层：Rust Tauri commands
- 构建：Node.js、Cargo、Tauri CLI

## 本地开发

```bash
npm install
npm run dev
```

桌面开发：

```bash
npm run tauri dev
```

生产构建：

```bash
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
npm run tauri build
```

## 项目状态

第一阶段目标是获得可运行、可打包、可继续重构的 AgentDock 基线版本。
```

- [ ] **Step 6: Replace `README.en.md` with concise English production README**

Replace `README.en.md` with:

```markdown
# AgentDock

AgentDock is a production desktop console for multi-engine AI agent operations, built with Tauri v2, Vite, and a Rust command layer.

## Current Phase

This repository is in the production fork baseline phase:

- Keep the existing OpenClaw and Hermes Agent engine capabilities.
- Keep the Tauri desktop shell, Web mode, service management, model management, agent management, chat, logs, diagnostics, backup, and extension modules.
- Replace product identity, icons, documentation, app metadata, and release entry points first.
- Gradually refactor configuration ownership, module boundaries, and backend command implementations afterward.

## Tech Stack

- Frontend: Vite + vanilla JavaScript
- Desktop: Tauri v2
- Capability layer: Rust Tauri commands
- Build: Node.js, Cargo, Tauri CLI

## Local Development

```bash
npm install
npm run dev
```

Desktop development:

```bash
npm run tauri dev
```

Production build:

```bash
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
npm run tauri build
```

## Project Status

The first milestone is a runnable and packageable AgentDock baseline that can be safely refactored in later phases.
```

- [ ] **Step 7: Run brand surface test and verify it passes**

First run the full app-identity scan across the files covered by this baseline:

```powershell
rg "ClawPanel|claw\.qt\.cool|qingchencloud/clawpanel|ai\.openclaw\.clawpanel" index.html src/components/sidebar.js src/main.js src/pages/about.js README.md README.en.md package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml src-tauri/src/main.rs
```

Remove or replace app-identity matches unless they are explicitly external integration names that identify external projects, engines, or dependencies rather than this app. For JavaScript files, non-runtime comments may retain historical context; `tests/brand-surface.test.js` strips JavaScript comments before scanning.

Run:

```powershell
node --test tests/brand-surface.test.js
```

Expected:

```text
# pass
```

If the test fails for `src/pages/about.js`, run the Step 0 `rg` command again and replace any remaining app-identity match unless it is an external integration name or a non-runtime comment. If the test fails for README files, replace the old identity because comments are not stripped in Markdown documentation.

- [ ] **Step 8: Commit About and README branding**

Run:

```powershell
git -c safe.directory=D:/workSpace/ohter/clawpanel add src/pages/about.js README.md README.en.md
git -c safe.directory=D:/workSpace/ohter/clawpanel commit -m "feat: update about page and readme branding"
```

Expected:

```text
[codex/clawpanel-new <hash>] feat: update about page and readme branding
```

## Task 6: Generate Temporary Brand Assets

**Files:**
- Create: `scripts/generate-brand-assets.ps1`
- Create: `docs/agentdock-icon.png`
- Create: `docs/agentdock-logo-brand.png`
- Create: `public/favicon-source.png`
- Modify: `public/favicon.ico`
- Modify: `public/images/logo.png`
- Modify: `public/images/logo-brand.png`
- Modify: `src-tauri/icons/*`

- [ ] **Step 1: Create the asset generator script**

Create `scripts/generate-brand-assets.ps1` with:

```powershell
Add-Type -AssemblyName System.Drawing

$Root = Split-Path -Parent $PSScriptRoot

function New-AgentDockIcon {
  param(
    [string]$Path,
    [int]$Size,
    [switch]$Wide
  )

  $width = if ($Wide) { [int]($Size * 3.2) } else { $Size }
  $height = $Size
  $bmp = New-Object System.Drawing.Bitmap $width, $height
  $gfx = [System.Drawing.Graphics]::FromImage($bmp)
  $gfx.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
  $gfx.Clear([System.Drawing.Color]::FromArgb(0, 0, 0, 0))

  $bg = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
    (New-Object System.Drawing.Rectangle 0, 0, $width, $height),
    [System.Drawing.Color]::FromArgb(22, 28, 45),
    [System.Drawing.Color]::FromArgb(38, 96, 139),
    45
  )
  $radius = [Math]::Max(8, [int]($Size * 0.16))
  $rect = New-Object System.Drawing.Rectangle 0, 0, ($height - 1), ($height - 1)
  if ($Wide) {
    $rect = New-Object System.Drawing.Rectangle 0, 0, ($height - 1), ($height - 1)
  }
  $gfx.FillEllipse($bg, $rect)

  $pen = New-Object System.Drawing.Pen ([System.Drawing.Color]::FromArgb(226, 248, 255)), ([Math]::Max(3, [int]($Size * 0.055)))
  $nodeBrush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(245, 255, 255))
  $accentBrush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(83, 220, 180))

  $cx = [int]($height / 2)
  $cy = [int]($height / 2)
  $r = [int]($Size * 0.24)
  $left = New-Object System.Drawing.Point ([int]($cx - $r), [int]($cy + $r * 0.55))
  $right = New-Object System.Drawing.Point ([int]($cx + $r), [int]($cy + $r * 0.55))
  $top = New-Object System.Drawing.Point $cx, ([int]($cy - $r * 0.9))

  $gfx.DrawLine($pen, $top, $left)
  $gfx.DrawLine($pen, $top, $right)
  $gfx.DrawLine($pen, $left, $right)

  $dot = [Math]::Max(9, [int]($Size * 0.11))
  foreach ($pt in @($left, $right, $top)) {
    $gfx.FillEllipse($nodeBrush, ($pt.X - $dot / 2), ($pt.Y - $dot / 2), $dot, $dot)
  }
  $gfx.FillEllipse($accentBrush, ($top.X - $dot / 3), ($top.Y - $dot / 3), ($dot * 0.66), ($dot * 0.66))

  if ($Wide) {
    $fontSize = [Math]::Max(18, [int]($Size * 0.34))
    $font = New-Object System.Drawing.Font "Segoe UI", $fontSize, ([System.Drawing.FontStyle]::Bold)
    $textBrush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(238, 248, 255))
    $gfx.DrawString("AgentDock", $font, $textBrush, ([int]($height * 1.12)), ([int]($height * 0.26)))
    $font.Dispose()
    $textBrush.Dispose()
  }

  $dir = Split-Path -Parent $Path
  if (!(Test-Path $dir)) { New-Item -ItemType Directory -Path $dir -Force | Out-Null }
  $bmp.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png)

  $accentBrush.Dispose()
  $nodeBrush.Dispose()
  $pen.Dispose()
  $bg.Dispose()
  $gfx.Dispose()
  $bmp.Dispose()
}

New-AgentDockIcon -Path (Join-Path $Root "docs/agentdock-icon.png") -Size 1024
New-AgentDockIcon -Path (Join-Path $Root "public/images/logo.png") -Size 256
New-AgentDockIcon -Path (Join-Path $Root "public/images/logo-brand.png") -Size 160 -Wide
New-AgentDockIcon -Path (Join-Path $Root "docs/agentdock-logo-brand.png") -Size 220 -Wide
New-AgentDockIcon -Path (Join-Path $Root "public/favicon-source.png") -Size 256

Write-Host "Generated AgentDock brand assets."
```

- [ ] **Step 2: Run the asset generator**

Run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/generate-brand-assets.ps1
```

Expected:

```text
Generated AgentDock brand assets.
```

- [ ] **Step 3: Regenerate Tauri icons from the new source icon**

Run:

```powershell
npm run tauri -- icon docs/agentdock-icon.png -o src-tauri/icons
```

Expected:

```text
Generating icons
```

- [ ] **Step 4: Replace favicon if Tauri icon command did not update it**

If `public/favicon.ico` is unchanged after Step 3, copy the generated Windows icon:

```powershell
Copy-Item -Path src-tauri/icons/icon.ico -Destination public/favicon.ico -Force
```

Expected:

```text
```

- [ ] **Step 5: Verify asset files exist**

Run:

```powershell
Test-Path docs/agentdock-icon.png
Test-Path docs/agentdock-logo-brand.png
Test-Path public/images/logo.png
Test-Path public/images/logo-brand.png
Test-Path public/favicon-source.png
Test-Path src-tauri/icons/icon.ico
```

Expected:

```text
True
True
True
True
True
True
```

- [ ] **Step 6: Commit generated brand assets**

Run:

```powershell
git -c safe.directory=D:/workSpace/ohter/clawpanel add scripts/generate-brand-assets.ps1 docs/agentdock-icon.png docs/agentdock-logo-brand.png public/favicon-source.png public/favicon.ico public/images/logo.png public/images/logo-brand.png src-tauri/icons
git -c safe.directory=D:/workSpace/ohter/clawpanel commit -m "feat: generate AgentDock brand assets"
```

Expected:

```text
[codex/clawpanel-new <hash>] feat: generate AgentDock brand assets
```

## Task 7: Verify Production Baseline

**Files:**
- Read: `package.json`
- Read: `src-tauri/tauri.conf.json`
- Read: `src-tauri/Cargo.toml`
- Read: `index.html`
- Read: `src/components/sidebar.js`
- Read: `src/main.js`
- Read: `src/pages/about.js`
- Read: `README.md`
- Read: `README.en.md`

- [ ] **Step 1: Run all JavaScript tests**

Run:

```powershell
node --test tests/*.test.js
```

Expected:

```text
# pass
```

- [ ] **Step 2: Run the frontend production build**

Run:

```powershell
npm run build
```

Expected:

```text
vite v6
built
```

- [ ] **Step 3: Run Rust check**

Run:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected:

```text
Finished `dev` profile
```

- [ ] **Step 4: Verify Tauri project metadata**

Run:

```powershell
npm run tauri -- info
```

Expected:

```text
Environment
Packages
App
```

- [ ] **Step 5: Run a non-packaging desktop smoke check**

Run:

```powershell
npm run tauri -- build --debug
```

Expected:

```text
Finished
```

If this fails due to missing Rust targets, Visual Studio Build Tools, WebView2, signing, or bundler requirements, do not hide the failure. Capture the exact error in `docs/superpowers/plans/2026-05-14-production-fork-baseline.md` under a new `Execution Notes` section before the final commit.

- [ ] **Step 6: Scan first-run brand surfaces**

Run:

```powershell
node --test tests/brand-surface.test.js
```

Expected:

```text
# pass
```

- [ ] **Step 7: Commit verification notes if added**

If Step 5 added execution notes, run:

```powershell
git -c safe.directory=D:/workSpace/ohter/clawpanel add docs/superpowers/plans/2026-05-14-production-fork-baseline.md
git -c safe.directory=D:/workSpace/ohter/clawpanel commit -m "docs: record production baseline verification notes"
```

Expected when notes were added:

```text
[codex/clawpanel-new <hash>] docs: record production baseline verification notes
```

If no notes were added, skip this commit.

### Execution Notes

- `node --test tests/*.test.js` passed with 51 tests.
- `node --test tests/brand-surface.test.js` passed with 3 tests.
- `node_modules\\.bin\\vite.cmd build` passed and produced `dist/`.
- `cargo check --manifest-path src-tauri/Cargo.toml` passed.
- `node_modules\\.bin\\tauri.cmd info` passed and reported the expected Environment, Packages, and App sections.
- `node_modules\\.bin\\tauri.cmd build --debug` failed in this environment because `beforeBuildCommand` runs `npm run build`, and `npm` is not available in the PowerShell PATH here.
- Follow-up: rerun the desktop debug smoke in a shell where `npm` or `npm.cmd` is available on PATH, using `node_modules\\.bin\\tauri.cmd build --debug` or `npm run tauri -- build --debug`.
- `cargo tauri icon docs/agentdock-icon.png -o src-tauri/icons` succeeded and regenerated the Tauri icon set from the new AgentDock source icon.
- `public/favicon.ico` was synchronized from `src-tauri/icons/icon.ico`.
- Later Phase 5 release hardening produced a Windows NSIS release artifact and recorded install/launch/uninstall smoke evidence in `docs/release/phase-5-windows-smoke-2026-05-15.md`.
- Browser-driven UI route smoke passed for the core routes recorded in `docs/release/phase-5-ui-smoke-2026-05-15.md`.
- Windows signing execution and verification are wired, but formal certificate hookup is deferred until a real certificate and thumbprint are available.
- Rust product-config helper warnings and Vite mixed dynamic/static import warnings were cleaned; the remaining frontend build warning is the real `i18n` chunk size warning.

## Task 8: Final Review And Handoff

**Files:**
- Read: all changed files from Tasks 2-7

- [ ] **Step 1: Review the final diff**

Run:

```powershell
git -c safe.directory=D:/workSpace/ohter/clawpanel diff --stat HEAD~5..HEAD
git -c safe.directory=D:/workSpace/ohter/clawpanel log --oneline -8
```

Expected:

```text
feat: generate AgentDock brand assets
feat: update about page and readme branding
feat: apply AgentDock shell branding
chore: rename package metadata to AgentDock
feat: add product identity constants
```

- [ ] **Step 2: Confirm the working tree is clean except user-owned spec edits**

Run:

```powershell
git -c safe.directory=D:/workSpace/ohter/clawpanel status --short
```

Expected:

```text
 M docs/superpowers/specs/2026-05-14-production-fork-refactor-design.md
```

The spec edit is user-owned and should not be reverted. If the user wants it committed, commit it separately with:

```powershell
git -c safe.directory=D:/workSpace/ohter/clawpanel add docs/superpowers/specs/2026-05-14-production-fork-refactor-design.md
git -c safe.directory=D:/workSpace/ohter/clawpanel commit -m "docs: refine production fork refactor design"
```

- [ ] **Step 3: Prepare the next plan boundary**

Create no code in this step. Record the next plan topic in the final handoff:

```text
Next implementation plan: Phase 2 configuration ownership. It should introduce product-owned config constants, migration detection, and compatibility aliases for existing fork paths.
```
