# AgentDock UI And Brand Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor the frontend into the selected A Modern Ops Console direction while preserving existing AgentDock functionality and replacing visible legacy brand copy with AgentDock and DeepAi assistant naming.

**Architecture:** Keep the Vite + vanilla JavaScript + CSS architecture. Implement the refactor in thin, verifiable slices: brand guardrails, product copy, design tokens, app shell, shared primitives, core routes, brand assets, then secondary route cleanup. Preserve Tauri IPC contracts, route names, storage compatibility keys, and backend command behavior.

**Tech Stack:** Vite, vanilla JavaScript modules, global CSS, Tauri v2, Rust command layer, Node test runner, Chrome-based UI smoke.

---

## File Structure And Ownership

Core files for this plan:

- `tests/brand-surface.test.js`: visible brand regression tests.
- `tests/ui-modernization.test.js`: new static tests for the A direction token/component/page-shell contract.
- `src/lib/product-identity.js`: centralized product identity and assistant naming constants.
- `src/locales/modules/sidebar.js`: navigation labels, especially `DeepAi助手`.
- `src/locales/modules/setup.js`: first-run and install guidance copy.
- `src/locales/modules/about.js`: About page copy, release/support labels, attribution copy.
- `src/locales/modules/assistant.js`: DeepAi assistant labels and guide text.
- `src/locales/modules/chat.js`, `dashboard.js`, `common.js`, `settings.js`, `notifications.js`, `services.js`, `engine.js`, `engagement.js`: visible product copy cleanup.
- `src/pages/assistant.js`: system prompt and visible assistant identity.
- `src/pages/setup.js`: setup logo alt text and product copy.
- `src/pages/about.js`: product-owned About layout and release links.
- `src/pages/dashboard.js`: first reference route for the new page header, stat cards, quick actions, and signal panels.
- `src/pages/services.js`: dense operational control layout.
- `src/pages/settings.js`: grouped form layout.
- `src/pages/logs.js`: source selector and log viewer layout.
- `public/push-sw.js`: browser notification title/tag.
- `src/lib/push-web.js`: browser notification fallback title.
- `src/style/variables.css`: Modern Ops Console design tokens.
- `src/style/layout.css`: app shell, sidebar, content frame.
- `src/style/components.css`: shared UI primitives.
- `src/style/pages.css`: route-level layout primitives.
- `src/style/assistant.css`: DeepAi assistant layout polish.
- `src/components/sidebar.js`: shell navigation and brand surface.
- `src/components/modal.js`, `src/components/toast.js`, `src/components/ai-drawer.js`: shared overlay/notification/global assistant polish.
- `docs/agentdock-icon.png`, `docs/agentdock-logo-brand.png`, `public/images/logo-brand.png`, `public/images/logo.png`, `public/favicon.ico`, `src-tauri/icons/*`: brand assets.
- `docs/superpowers/specs/2026-05-15-agentdock-ui-brand-refactor-design.md`: source spec.

Files intentionally not renamed in this plan:

- `agentdock.json` compatibility references.
- `agentdock_authed`, `agentdock_must_change_pw`, and existing local/session storage keys.
- `.disabled-by-agentdock-*` quarantine marker behavior.
- Backend command names and Tauri IPC payloads.

---

## Task 1: Brand And UI Guardrail Tests

**Files:**
- Modify: `tests/brand-surface.test.js`
- Create: `tests/ui-modernization.test.js`

- [ ] **Step 1: Extend visible brand surface test**

Modify `tests/brand-surface.test.js` so it separately checks user-facing app files and allows explicit compatibility references.

Add these constants after `BRAND_SURFACE_FILES`:

```js
const APP_VISIBLE_BRAND_FILES = [
  'public/push-sw.js',
  'src/lib/push-web.js',
  'src/pages/assistant.js',
  'src/pages/setup.js',
  'src/locales/modules/sidebar.js',
  'src/locales/modules/setup.js',
  'src/locales/modules/about.js',
  'src/locales/modules/assistant.js',
  'src/locales/modules/chat.js',
  'src/locales/modules/dashboard.js',
  'src/locales/modules/common.js',
  'src/locales/modules/settings.js',
  'src/locales/modules/notifications.js',
  'src/locales/modules/services.js',
  'src/locales/modules/engine.js',
  'src/locales/modules/engagement.js',
]

const ALLOWED_LEGACY_CONTEXT = [
  /legacy/i,
  /compat/i,
  /compatibility/i,
  /upstream/i,
  /referenced/i,
  /agentdock\.json/i,
  /agentdock_authed/i,
  /agentdock_must_change_pw/i,
  /disabled-by-agentdock/i,
]
```

Add this helper after `stripReadmeAttribution`:

```js
function visibleLinesWithOldBrand(text) {
  return text
    .split(/\r?\n/)
    .map((line, index) => ({ line, index: index + 1 }))
    .filter(({ line }) => /AgentDock|DeepAiepAi助手/.test(line))
    .filter(({ line }) => !ALLOWED_LEGACY_CONTEXT.some((pattern) => pattern.test(line)))
}
```

Add this test after the existing destination label test:

```js
test('app-visible copy uses AgentDock and DeepAi assistant naming', () => {
  for (const file of APP_VISIBLE_BRAND_FILES) {
    const raw = fs.readFileSync(file, 'utf8')
    const text = file.endsWith('.js') ? stripJavaScriptComments(raw) : raw
    const offenders = visibleLinesWithOldBrand(text)
    assert.deepEqual(
      offenders,
      [],
      `${file} has visible legacy copy:\n${offenders.map(({ index, line }) => `${index}: ${line}`).join('\n')}`
    )
  }
})
```

- [ ] **Step 2: Run the new brand test and verify it fails before implementation**

Run:

```powershell
node --test tests\brand-surface.test.js
```

Expected before implementation:

```text
not ok ... app-visible copy uses AgentDock and DeepAi assistant naming
```

The failure should list visible `AgentDock` and `DeepAiepAi助手` references.

- [ ] **Step 3: Add static UI modernization contract tests**

Create `tests/ui-modernization.test.js`:

```js
import test from 'node:test'
import assert from 'node:assert/strict'
import fs from 'node:fs'

function read(file) {
  return fs.readFileSync(file, 'utf8')
}

test('modern ops design tokens are exposed in variables.css', () => {
  const css = read('src/style/variables.css')

  assert.match(css, /--surface-0:/)
  assert.match(css, /--surface-1:/)
  assert.match(css, /--surface-2:/)
  assert.match(css, /--accent-ops:/)
  assert.match(css, /--accent-release:/)
  assert.match(css, /--radius-control:/)
  assert.match(css, /--shadow-panel:/)
  assert.match(css, /--content-max-form:/)
})

test('shared component primitives include modern ops classes', () => {
  const css = read('src/style/components.css')

  for (const className of [
    'ad-button',
    'ad-button-primary',
    'ad-icon-button',
    'ad-status-pill',
    'ad-data-table',
    'ad-form-field',
    'ad-segmented-control',
    'ad-empty-state',
    'ad-code-block',
  ]) {
    assert.match(css, new RegExp(`\\.${className}\\b`), `${className} missing`)
  }
})

test('app shell exposes modern sidebar and page layout primitives', () => {
  const layout = read('src/style/layout.css')
  const pages = read('src/style/pages.css')

  assert.match(layout, /#sidebar/)
  assert.match(layout, /\.app-content-frame\b/)
  assert.match(layout, /\.sidebar-logo-mark\b/)
  assert.match(pages, /\.page-shell\b/)
  assert.match(pages, /\.page-header-actions\b/)
  assert.match(pages, /\.page-toolbar\b/)
})

test('core route files use page shell classes after refactor', () => {
  for (const file of [
    'src/pages/dashboard.js',
    'src/pages/services.js',
    'src/pages/settings.js',
    'src/pages/assistant.js',
    'src/pages/logs.js',
    'src/pages/about.js',
  ]) {
    const text = read(file)
    assert.match(text, /page-shell|ast-page|logs-page/, `${file} should expose a modern page shell class`)
  }
})
```

- [ ] **Step 4: Run UI modernization test and verify it fails before implementation**

Run:

```powershell
node --test tests\ui-modernization.test.js
```

Expected before implementation:

```text
not ok ... modern ops design tokens are exposed in variables.css
```

- [ ] **Step 5: Commit guardrail tests**

Run:

```powershell
git add tests\brand-surface.test.js tests\ui-modernization.test.js
git commit -m "test: add ui brand refactor guardrails"
```

Expected:

```text
[codex/agentdock-new <hash>] test: add ui brand refactor guardrails
```

---

## Task 2: Product Identity And Visible Copy Replacement

**Files:**
- Modify: `src/lib/product-identity.js`
- Modify: `public/push-sw.js`
- Modify: `src/lib/push-web.js`
- Modify: `src/pages/assistant.js`
- Modify: `src/pages/setup.js`
- Modify: `src/lib/openclaw-kb.js`
- Modify: `src/lib/error-diagnosis.js`
- Modify: `src/lib/term-tooltip.js`
- Modify: `src/pages/dreaming.js`
- Modify: `src/pages/settings.js`
- Modify: `src/locales/modules/sidebar.js`
- Modify: `src/locales/modules/setup.js`
- Modify: `src/locales/modules/about.js`
- Modify: `src/locales/modules/assistant.js`
- Modify: `src/locales/modules/chat.js`
- Modify: `src/locales/modules/dashboard.js`
- Modify: `src/locales/modules/common.js`
- Modify: `src/locales/modules/settings.js`
- Modify: `src/locales/modules/notifications.js`
- Modify: `src/locales/modules/services.js`
- Modify: `src/locales/modules/engine.js`
- Modify: `src/locales/modules/engagement.js`

- [ ] **Step 1: Add assistant identity constants**

Modify `src/lib/product-identity.js` to include assistant names:

```js
export const PRODUCT_IDENTITY = Object.freeze({
  id: 'agentdock',
  name: 'AgentDock',
  displayName: 'AgentDock',
  assistantNameZh: 'DeepAi助手',
  assistantNameEn: 'DeepAi Assistant',
  tagline: 'Multi-engine AI agent operations console',
  description: 'AgentDock - production desktop console for multi-engine AI agent operations',
  tauriIdentifier: 'com.agentdock.desktop',
  homepage: 'https://github.com/dfpo520-claw-bot/AgentDock',
  homepageHost: 'github.com/dfpo520-claw-bot/AgentDock',
  supportUrl: 'https://github.com/dfpo520-claw-bot/AgentDock/issues',
  repositoryUrl: 'https://github.com/dfpo520-claw-bot/AgentDock',
  releaseUrl: 'https://github.com/dfpo520-claw-bot/AgentDock/releases',
  updateManifestUrl: 'https://raw.githubusercontent.com/dfpo520-claw-bot/AgentDock/main/update/latest.json',
  legacyProductName: 'AgentDock',
})
```

Update `tests/product-identity.test.js` to assert the new constants:

```js
assert.equal(PRODUCT_IDENTITY.assistantNameZh, 'DeepAi助手')
assert.equal(PRODUCT_IDENTITY.assistantNameEn, 'DeepAi Assistant')
```

Add both fields to the `visible` array in `visible identity fields no longer use the fork app name`.

- [ ] **Step 2: Replace service worker and push notification visible names**

In `public/push-sw.js`:

- Change comment title to `AgentDock Web Push Service Worker`.
- Change default notification title:

```js
const title = payload.title || 'AgentDock'
```

- Change default tag:

```js
tag: payload.tag || 'agentdock',
```

- Update visible comments from "AgentDock" to "AgentDock".

In `src/lib/push-web.js`, change fallback title:

```js
title: title || 'AgentDock',
```

Keep RPC names and OpenClaw integration untouched.

- [ ] **Step 3: Replace sidebar and setup assistant labels**

In `src/locales/modules/sidebar.js`, change the `assistant` entry to:

```js
assistant: _('DeepAi助手', 'DeepAi Assistant', 'DeepAi助手', 'DeepAi Assistant', 'DeepAi Assistant', 'DeepAi Assistant', 'DeepAi Assistant', 'DeepAi Assistant', 'DeepAi Assistant', 'DeepAi Assistant', 'DeepAi Assistant'),
```

In `src/locales/modules/setup.js`:

- `headerTitle`: use `欢迎使用 AgentDock`, `Welcome to AgentDock`.
- `aiAssistant`: use `DeepAi助手`, `DeepAi Assistant`.
- Replace visible `AgentDock` product mentions with `AgentDock`.
- Keep `agentdock.json` mentions unchanged if they refer to compatibility config filename.
- Replace `AgentDock Web 版` with `AgentDock Web mode` or Chinese `AgentDock Web 版`.

- [ ] **Step 4: Replace assistant module visible labels**

In `src/locales/modules/assistant.js`:

- `defaultName`: use `DeepAi助手` and `DeepAi Assistant`.
- `guideTitle`: use `这是 AgentDock 内置的 DeepAi助手` and `This is AgentDock's built-in DeepAi Assistant`.
- Replace visible `AgentDock` product mentions with `AgentDock`.
- Replace `DeepAiepAi助手` with `DeepAi助手`.

- [ ] **Step 5: Replace page and locale visible AgentDock copy**

For these locale modules:

- `src/locales/modules/about.js`
- `src/locales/modules/chat.js`
- `src/locales/modules/dashboard.js`
- `src/locales/modules/common.js`
- `src/locales/modules/settings.js`
- `src/locales/modules/notifications.js`
- `src/locales/modules/services.js`
- `src/locales/modules/engine.js`
- `src/locales/modules/engagement.js`

Apply these replacements only to visible product copy:

```text
AgentDock -> AgentDock
AgentDock's -> AgentDock's
AgentDock desktop -> AgentDock desktop
AgentDock Web -> AgentDock Web
DeepAiepAi助手 -> DeepAi助手
```

Do not change:

```text
agentdock.json
disabled-by-agentdock
@DeepAi助手/openclaw-zh
DeepAi助手/openclaw
```

- [ ] **Step 6: Replace visible assistant prompt identity**

In `src/pages/assistant.js`, update the system prompt copy:

- Replace "你是 AgentDock 内置的智能助手" with "你是 AgentDock 内置的 DeepAi助手".
- Replace "AgentDock 官网" with "AgentDock 项目主页".
- Replace old GitHub links with `https://github.com/dfpo520-claw-bot/AgentDock` or `https://github.com/dfpo520-claw-bot/AgentDock/issues/new`.
- Keep technical references to `agentdock.json` as compatibility config filename.
- Replace "AgentDock 工具能力" with "AgentDock 工具能力".

Also import `PRODUCT_IDENTITY` if it helps keep links centralized, but do not change tool execution logic.

- [ ] **Step 7: Replace visible setup page alt and command copy**

In `src/pages/setup.js`:

- Change logo alt text from `AgentDock` to `AgentDock`.
- Change visible app launch command from `/Applications/AgentDock.app` to `/Applications/AgentDock.app`.
- Do not change runtime command names.

- [ ] **Step 8: Replace visible helper copy in supporting modules**

Update visible copy in:

- `src/lib/openclaw-kb.js`: product support links and product name to AgentDock; keep OpenClaw knowledge intact.
- `src/lib/error-diagnosis.js`: visible hints to AgentDock.
- `src/lib/term-tooltip.js`: visible explanations to AgentDock.
- `src/pages/dreaming.js`: note string to `Dreaming settings updated from AgentDock.`
- `src/pages/settings.js`: file header comment may be changed to AgentDock when touching the file.

- [ ] **Step 9: Run brand guardrails**

Run:

```powershell
node --test tests\product-identity.test.js tests\brand-surface.test.js
```

Expected:

```text
pass
```

- [ ] **Step 10: Run targeted search**

Run:

```powershell
rg "AgentDock|DeepAiepAi助手" src public tests -n
```

Expected:

- Remaining matches are compatibility comments, `legacyProductName`, `agentdock.json`, local storage keys, quarantine markers, or upstream attribution.
- No remaining visible app UI copy says `AgentDock` or `DeepAiepAi助手`.

- [ ] **Step 11: Commit brand copy replacement**

Run:

```powershell
git add src public tests
git commit -m "feat: replace visible app brand copy"
```

Expected:

```text
[codex/agentdock-new <hash>] feat: replace visible app brand copy
```

---

## Task 3: Modern Ops Design Tokens

**Files:**
- Modify: `src/style/variables.css`
- Modify: `tests/ui-modernization.test.js`

- [ ] **Step 1: Update global design tokens**

Modify `src/style/variables.css` to keep existing variable names and add Modern Ops aliases.

For light theme, use this shape:

```css
:root, [data-theme="light"] {
  --surface-0: #f4f7fb;
  --surface-1: #ffffff;
  --surface-2: #eef3f8;
  --surface-3: #e5edf5;
  --surface-elevated: #ffffff;

  --bg-primary: var(--surface-0);
  --bg-secondary: var(--surface-1);
  --bg-tertiary: var(--surface-2);
  --bg-card: #ffffff;
  --bg-card-hover: #f8fbfe;
  --bg-glass: rgba(15, 23, 42, 0.035);
  --bg-glass-hover: rgba(15, 23, 42, 0.065);
  --bg-hover: rgba(15, 118, 110, 0.08);

  --border-primary: rgba(23, 32, 51, 0.11);
  --border-secondary: rgba(23, 32, 51, 0.07);
  --border: var(--border-primary);
  --border-focus: rgba(15, 118, 110, 0.45);

  --text-primary: #172033;
  --text-secondary: #526174;
  --text-tertiary: #8a98aa;
  --text-inverse: #ffffff;

  --accent-ops: #0f766e;
  --accent-release: #2563eb;
  --accent: var(--accent-ops);
  --accent-hover: #0b625c;
  --accent-muted: rgba(15, 118, 110, 0.11);
  --primary: var(--accent);

  --success: #148a46;
  --success-muted: rgba(20, 138, 70, 0.11);
  --warning: #b86b00;
  --warning-muted: rgba(184, 107, 0, 0.12);
  --error: #c2413a;
  --error-muted: rgba(194, 65, 58, 0.11);
  --info: #2563eb;
  --info-muted: rgba(37, 99, 235, 0.11);

  --shadow-sm: 0 1px 2px rgba(15, 23, 42, 0.05);
  --shadow-md: 0 8px 24px rgba(15, 23, 42, 0.08);
  --shadow-lg: 0 18px 48px rgba(15, 23, 42, 0.12);
  --shadow-panel: 0 14px 34px rgba(15, 23, 42, 0.08);
  --shadow-glow: 0 0 0 3px rgba(15, 118, 110, 0.12);
}
```

For dark theme, use this shape:

```css
[data-theme="dark"] {
  --surface-0: #0f1720;
  --surface-1: #141d29;
  --surface-2: #1a2533;
  --surface-3: #243244;
  --surface-elevated: #182232;

  --bg-primary: var(--surface-0);
  --bg-secondary: var(--surface-1);
  --bg-tertiary: var(--surface-2);
  --bg-card: #182232;
  --bg-card-hover: #1d2a3a;
  --bg-glass: rgba(255, 255, 255, 0.045);
  --bg-glass-hover: rgba(255, 255, 255, 0.075);
  --bg-hover: rgba(45, 212, 191, 0.11);

  --border-primary: rgba(226, 232, 240, 0.12);
  --border-secondary: rgba(226, 232, 240, 0.075);
  --border: var(--border-primary);
  --border-focus: rgba(45, 212, 191, 0.48);

  --text-primary: #edf4fb;
  --text-secondary: #b6c2d1;
  --text-tertiary: #8290a3;
  --text-inverse: #0f1720;

  --accent-ops: #2dd4bf;
  --accent-release: #60a5fa;
  --accent: var(--accent-ops);
  --accent-hover: #5eead4;
  --accent-muted: rgba(45, 212, 191, 0.14);
  --primary: var(--accent);

  --success: #22c55e;
  --success-muted: rgba(34, 197, 94, 0.14);
  --warning: #f59e0b;
  --warning-muted: rgba(245, 158, 11, 0.14);
  --error: #f87171;
  --error-muted: rgba(248, 113, 113, 0.14);
  --info: #60a5fa;
  --info-muted: rgba(96, 165, 250, 0.14);

  --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.28);
  --shadow-md: 0 10px 28px rgba(0, 0, 0, 0.34);
  --shadow-lg: 0 24px 60px rgba(0, 0, 0, 0.42);
  --shadow-panel: 0 16px 42px rgba(0, 0, 0, 0.32);
  --shadow-glow: 0 0 0 3px rgba(45, 212, 191, 0.14);
}
```

In the shared `:root` block, add:

```css
--radius-control: 7px;
--radius-panel: 8px;
--content-max-form: 920px;
--content-max-readable: 1120px;
--sidebar-width: 236px;
--sidebar-collapsed: 64px;
--header-height: 56px;
```

Keep existing `--radius-*`, `--space-*`, and `--font-*` variables for compatibility.

- [ ] **Step 2: Run token test**

Run:

```powershell
node --test tests\ui-modernization.test.js
```

Expected:

- Token test passes.
- Other tests in the file may still fail until later tasks.

- [ ] **Step 3: Run frontend build**

Run:

```powershell
node node_modules\vite\bin\vite.js build
```

Expected:

```text
✓ built
```

The existing `i18n` chunk size warning may remain.

- [ ] **Step 4: Commit tokens**

Run:

```powershell
git add src\style\variables.css tests\ui-modernization.test.js
git commit -m "style: add modern ops design tokens"
```

Expected:

```text
[codex/agentdock-new <hash>] style: add modern ops design tokens
```

---

## Task 4: Shared Component Primitives

**Files:**
- Modify: `src/style/components.css`
- Modify: `src/components/modal.js`
- Modify: `src/components/toast.js`
- Modify: `src/components/ai-drawer.js`

- [ ] **Step 1: Add modern primitive classes to components.css**

Append this block near the shared component section in `src/style/components.css`:

```css
.ad-button,
.btn {
  min-height: 34px;
  border: 1px solid transparent;
  border-radius: var(--radius-control);
  font-weight: 650;
  line-height: 1;
}

.ad-button-primary,
.btn-primary {
  background: var(--accent);
  color: var(--text-inverse);
  box-shadow: 0 1px 0 rgba(255,255,255,0.16) inset;
}

.ad-button-secondary,
.btn-secondary {
  background: var(--surface-elevated);
  border-color: var(--border-primary);
  color: var(--text-primary);
}

.ad-button-ghost,
.btn-ghost {
  background: transparent;
  color: var(--text-secondary);
}

.ad-button-danger,
.btn-danger {
  background: var(--error-muted);
  color: var(--error);
  border-color: color-mix(in srgb, var(--error) 22%, transparent);
}

.ad-icon-button {
  width: 34px;
  height: 34px;
  padding: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-control);
  border: 1px solid var(--border-primary);
  background: var(--surface-elevated);
  color: var(--text-secondary);
}

.ad-status-pill {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  min-height: 24px;
  padding: 0 9px;
  border-radius: 999px;
  border: 1px solid var(--border-primary);
  background: var(--surface-elevated);
  color: var(--text-secondary);
  font-size: var(--font-size-xs);
  font-weight: 700;
}

.ad-status-pill.success { color: var(--success); background: var(--success-muted); border-color: color-mix(in srgb, var(--success) 20%, transparent); }
.ad-status-pill.warning { color: var(--warning); background: var(--warning-muted); border-color: color-mix(in srgb, var(--warning) 20%, transparent); }
.ad-status-pill.error { color: var(--error); background: var(--error-muted); border-color: color-mix(in srgb, var(--error) 20%, transparent); }
.ad-status-pill.info { color: var(--info); background: var(--info-muted); border-color: color-mix(in srgb, var(--info) 20%, transparent); }

.ad-data-table {
  width: 100%;
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-panel);
  overflow: hidden;
  background: var(--surface-elevated);
}

.ad-data-row {
  display: grid;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border-bottom: 1px solid var(--border-secondary);
}

.ad-data-row:last-child { border-bottom: 0; }
.ad-data-row.header {
  min-height: 36px;
  background: var(--surface-2);
  color: var(--text-tertiary);
  font-size: var(--font-size-xs);
  font-weight: 750;
}

.ad-form-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: var(--space-lg);
}

.ad-form-field .form-label {
  margin-bottom: 0;
}

.ad-field-description {
  color: var(--text-tertiary);
  font-size: var(--font-size-xs);
  line-height: 1.5;
}

.ad-segmented-control {
  display: inline-flex;
  padding: 3px;
  border-radius: var(--radius-control);
  background: var(--surface-2);
  border: 1px solid var(--border-secondary);
}

.ad-segmented-control > button {
  min-height: 28px;
  padding: 0 10px;
  border-radius: 5px;
}

.ad-segmented-control > button.active,
.ad-segmented-control > button[aria-pressed="true"] {
  background: var(--surface-elevated);
  color: var(--text-primary);
  box-shadow: var(--shadow-sm);
}

.ad-empty-state {
  padding: 28px;
  border: 1px dashed var(--border-primary);
  border-radius: var(--radius-panel);
  color: var(--text-tertiary);
  text-align: center;
  background: color-mix(in srgb, var(--surface-1) 68%, transparent);
}

.ad-code-block {
  display: block;
  padding: 12px;
  border-radius: var(--radius-control);
  background: var(--surface-2);
  border: 1px solid var(--border-secondary);
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  overflow: auto;
}
```

- [ ] **Step 2: Modernize modal focus and destructive surfaces**

In `src/components/modal.js`, keep all function names and event behavior unchanged. Add or preserve classes so rendered modal buttons use existing `.btn` variants and destructive confirmations use `.btn-danger`.

If a modal action button currently has no semantic class, map it as:

```js
const className = btn.className || (btn.danger ? 'btn btn-danger' : 'btn btn-secondary')
```

Do not change modal return values.

- [ ] **Step 3: Modernize toast class shape**

In `src/components/toast.js`, keep the `toast(message, type, opts)` API unchanged.

Ensure toast root still includes:

```js
el.className = `toast ${type}`
```

If adding structure, do not remove `.toast.success`, `.toast.error`, `.toast.info`, `.toast.warning` compatibility.

- [ ] **Step 4: Modernize DeepAi global entry styles without changing behavior**

In `src/components/ai-drawer.js`, keep `initAIFab`, `openAIDrawerWithError`, and `registerPageContext` unchanged.

Only update visible title usage through `t('sidebar.assistant')`, which should now resolve to DeepAi naming.

- [ ] **Step 5: Run component tests**

Run:

```powershell
node --test tests\ui-modernization.test.js
node node_modules\vite\bin\vite.js build
```

Expected:

- `shared component primitives include modern ops classes` passes.
- Build passes with only known `i18n` chunk size warning.

- [ ] **Step 6: Commit shared primitives**

Run:

```powershell
git add src\style\components.css src\components\modal.js src\components\toast.js src\components\ai-drawer.js tests\ui-modernization.test.js
git commit -m "style: add modern shared ui primitives"
```

Expected:

```text
[codex/agentdock-new <hash>] style: add modern shared ui primitives
```

---

## Task 5: App Shell And Sidebar

**Files:**
- Modify: `src/style/layout.css`
- Modify: `src/style/pages.css`
- Modify: `src/components/sidebar.js`

- [ ] **Step 1: Add app content frame and sidebar logo classes**

Modify `src/style/layout.css`:

- Keep `#sidebar`, `.sidebar-collapsed`, `.sidebar-nav`, and `.sidebar-footer` selectors.
- Add `.app-content-frame`:

```css
.app-content-frame {
  flex: 1;
  min-width: 0;
  min-height: 100vh;
  background: var(--bg-primary);
  display: flex;
  flex-direction: column;
}
```

- Update `#sidebar` background to:

```css
background: var(--surface-elevated);
box-shadow: 1px 0 0 var(--border-secondary);
```

- Add:

```css
.sidebar-logo-mark {
  width: 30px;
  height: 30px;
  border-radius: 8px;
  display: grid;
  place-items: center;
  color: #fff;
  font-size: 12px;
  font-weight: 850;
  background: linear-gradient(135deg, var(--accent-ops), var(--accent-release));
  letter-spacing: 0;
}
```

- Keep `.sidebar-logo img` support for generated assets.

- [ ] **Step 2: Add page shell classes**

Modify `src/style/pages.css`:

Add near the top:

```css
.page-shell,
.page {
  width: 100%;
  min-width: 0;
}

.page {
  padding: var(--space-xl);
}

.page-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-lg);
  margin-bottom: var(--space-xl);
}

.page-header-main {
  min-width: 0;
}

.page-title {
  margin: 0;
  font-size: 22px;
  line-height: 1.25;
  font-weight: 760;
  letter-spacing: 0;
}

.page-desc {
  margin: 6px 0 0;
  max-width: 760px;
  color: var(--text-secondary);
  line-height: 1.55;
}

.page-header-actions {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  flex-wrap: wrap;
  justify-content: flex-end;
}

.page-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-md);
  margin-bottom: var(--space-lg);
  padding: 10px;
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-panel);
  background: var(--surface-elevated);
}

.page-grid {
  display: grid;
  gap: var(--space-lg);
}

.page-grid.two {
  grid-template-columns: minmax(0, 1.4fr) minmax(320px, 0.8fr);
}

.page-form-shell {
  max-width: var(--content-max-form);
}

@media (max-width: 980px) {
  .page {
    padding: var(--space-lg);
  }
  .page-header,
  .page-toolbar,
  .page-grid.two {
    grid-template-columns: 1fr;
    flex-direction: column;
    align-items: stretch;
  }
  .page-header-actions {
    justify-content: flex-start;
  }
}
```

- [ ] **Step 3: Update sidebar brand mark**

In `src/components/sidebar.js`, replace the logo block so it renders a compact mark with image fallback.

Target markup:

```js
<div class="sidebar-logo">
  <div class="sidebar-logo-mark" aria-hidden="true">AD</div>
</div>
<span class="sidebar-title">${PRODUCT_IDENTITY.name}</span>
```

If an image is already required by tests, keep it as:

```js
<img src="/images/logo.png" alt="${PRODUCT_IDENTITY.name}" onerror="this.replaceWith(Object.assign(document.createElement('div'), { className: 'sidebar-logo-mark', textContent: 'AD' }))">
```

Use the simpler text mark if no test depends on an image.

- [ ] **Step 4: Preserve sidebar interactions**

Do not change:

- Collapse/expand local storage key.
- `renderSidebar(sidebar)`.
- `openMobileSidebar()`.
- Engine switching behavior.
- Language switching behavior.
- Theme switching behavior.

- [ ] **Step 5: Run shell tests**

Run:

```powershell
node --test tests\ui-modernization.test.js tests\brand-surface.test.js
node node_modules\vite\bin\vite.js build
```

Expected:

- `app shell exposes modern sidebar and page layout primitives` passes.
- Build passes.

- [ ] **Step 6: Commit app shell**

Run:

```powershell
git add src\style\layout.css src\style\pages.css src\components\sidebar.js tests\ui-modernization.test.js
git commit -m "style: modernize app shell and sidebar"
```

Expected:

```text
[codex/agentdock-new <hash>] style: modernize app shell and sidebar
```

---

## Task 6: Core Route Layout Pass

**Files:**
- Modify: `src/pages/dashboard.js`
- Modify: `src/pages/services.js`
- Modify: `src/pages/settings.js`
- Modify: `src/pages/assistant.js`
- Modify: `src/pages/logs.js`
- Modify: `src/pages/about.js`
- Modify: `src/style/pages.css`
- Modify: `src/style/assistant.css`

- [ ] **Step 1: Dashboard page shell**

In `src/pages/dashboard.js`, keep all existing IDs and event targets.

Change:

```js
page.className = 'page'
```

to:

```js
page.className = 'page page-shell dashboard-page'
```

Update the page header markup to include actions:

```html
<div class="page-header">
  <div class="page-header-main">
    <h1 class="page-title">${t('dashboard.title')}</h1>
    <p class="page-desc">${t('dashboard.desc')}</p>
  </div>
  <div class="page-header-actions">
    <button class="btn btn-secondary" id="btn-check-update">${t('dashboard.checkUpdate')}</button>
    <button class="btn btn-secondary" id="btn-create-backup">${t('dashboard.createBackup')}</button>
  </div>
</div>
```

Remove the duplicate `btn-check-update` and `btn-create-backup` buttons from `.quick-actions`, leaving `btn-restart-gw` and `btn-open-glossary`.

Do not change event binding functions; existing selectors must still find the buttons.

- [ ] **Step 2: Services page shell**

In `src/pages/services.js`, set:

```js
page.className = 'page page-shell services-page'
```

Wrap the service action controls in `.page-toolbar` where practical, but keep all existing button IDs and data attributes.

Any service-changing button must retain its confirmation flow.

- [ ] **Step 3: Settings page shell**

In `src/pages/settings.js`, set:

```js
page.className = 'page page-shell settings-page page-form-shell'
```

Use `.ad-form-field` around newly touched form groups. Do not change config read/write calls.

- [ ] **Step 4: Assistant page DeepAi shell**

In `src/pages/assistant.js`, preserve existing `ast-*` selectors and IDs.

Ensure the root page class includes:

```js
page.className = 'ast-page page-shell assistant-page'
```

Update visible title fallback from old assistant naming to `DeepAi助手` through i18n keys.

In `src/style/assistant.css`, apply Modern Ops surfaces:

```css
.ast-page {
  background: var(--bg-primary);
}

.ast-sidebar,
.ast-header {
  background: var(--surface-elevated);
}

.ast-msg-bubble {
  border: 1px solid var(--border-secondary);
}
```

Do not change message persistence or tool execution logic.

- [ ] **Step 5: Logs page shell**

In `src/pages/logs.js`, set the root class to include:

```js
page.className = 'page page-shell logs-page'
```

Keep log source IDs, refresh buttons, auto-scroll controls, and export buttons unchanged.

- [ ] **Step 6: About page product shell**

In `src/pages/about.js`, set:

```js
page.className = 'page page-shell about-page'
```

Use a modern header:

```html
<div class="page-header about-hero">
  <div class="page-header-main about-identity">
    <img src="/images/logo-brand.png" alt="${PRODUCT_IDENTITY.name}" title="${PRODUCT_IDENTITY.name}" class="about-logo">
    <div>
      <h1 class="page-title">${PRODUCT_IDENTITY.name}</h1>
      <p class="page-desc">${PRODUCT_IDENTITY.tagline} - <a href="${PRODUCT_IDENTITY.homepage}" target="_blank" rel="noopener">${PRODUCT_IDENTITY.homepageHost}</a></p>
    </div>
  </div>
  <div class="page-header-actions">
    <a class="btn btn-secondary" href="${PRODUCT_IDENTITY.releaseUrl}" target="_blank" rel="noopener">${RELEASES_LABEL}</a>
    <a class="btn btn-secondary" href="${PRODUCT_IDENTITY.supportUrl}" target="_blank" rel="noopener">${SUPPORT_LABEL}</a>
  </div>
</div>
```

Keep version loading functions and update checks unchanged.

- [ ] **Step 7: Add route-specific polish CSS**

In `src/style/pages.css`, add:

```css
.about-hero {
  align-items: center;
}

.about-identity {
  display: flex;
  align-items: center;
  gap: var(--space-lg);
}

.about-logo {
  width: 52px;
  height: 52px;
  object-fit: contain;
  border-radius: 12px;
}

.dashboard-page .quick-actions {
  margin-bottom: var(--space-lg);
}

.services-page .service-card,
.settings-page .config-section,
.logs-page .config-section,
.about-page .config-section {
  background: var(--surface-elevated);
  border-color: var(--border-primary);
  box-shadow: var(--shadow-sm);
}
```

- [ ] **Step 8: Run core route static test**

Run:

```powershell
node --test tests\ui-modernization.test.js
```

Expected:

```text
pass
```

- [ ] **Step 9: Run build**

Run:

```powershell
node node_modules\vite\bin\vite.js build
```

Expected:

```text
✓ built
```

- [ ] **Step 10: Commit core route layout pass**

Run:

```powershell
git add src\pages\dashboard.js src\pages\services.js src\pages\settings.js src\pages\assistant.js src\pages\logs.js src\pages\about.js src\style\pages.css src\style\assistant.css tests\ui-modernization.test.js
git commit -m "style: modernize core route layouts"
```

Expected:

```text
[codex/agentdock-new <hash>] style: modernize core route layouts
```

---

## Task 7: Brand Assets Final Replacement

**Files:**
- Modify: `docs/agentdock-icon.png`
- Modify: `docs/agentdock-logo-brand.png`
- Modify: `public/images/logo-brand.png`
- Modify: `public/images/logo.png`
- Modify: `public/favicon.ico`
- Modify: `src-tauri/icons/*`
- Modify: `README.md`
- Modify: `README.en.md`

- [ ] **Step 1: Generate source brand images**

Create final AgentDock assets using the selected concept:

- AD monogram.
- Dock/ring motif.
- Deep neutral blue-gray background.
- Teal operational accent.
- No decorative blobs or generic AI clip art.

Use existing asset generation flow if available:

```powershell
npm run brand:assets
```

If the script still generates temporary assets, update `scripts/generate-brand-assets.ps1` in this task to generate the final Modern Ops assets.

- [ ] **Step 2: Regenerate Tauri icons**

Run:

```powershell
cargo tauri icon docs\agentdock-icon.png -o src-tauri\icons
```

Expected:

- `src-tauri/icons` contains refreshed icon outputs.

- [ ] **Step 3: Sync favicon**

Run:

```powershell
Copy-Item src-tauri\icons\icon.ico public\favicon.ico -Force
```

- [ ] **Step 4: Update README logo references**

If README files display old assets, update them to use:

```markdown
![AgentDock](docs/agentdock-logo-brand.png)
```

Only add the image if it renders cleanly and does not bloat the README opening.

- [ ] **Step 5: Run asset surface checks**

Run:

```powershell
node --test tests\brand-surface.test.js tests\product-identity.test.js
node node_modules\vite\bin\vite.js build
```

Expected:

- Tests pass.
- Build passes.

- [ ] **Step 6: Commit brand assets**

Run:

```powershell
git add docs\agentdock-icon.png docs\agentdock-logo-brand.png public\images\logo-brand.png public\images\logo.png public\favicon.ico src-tauri\icons README.md README.en.md scripts\generate-brand-assets.ps1
git commit -m "feat: replace final agentdock brand assets"
```

Expected:

```text
[codex/agentdock-new <hash>] feat: replace final agentdock brand assets
```

If `scripts/generate-brand-assets.ps1` was not changed, omit it from `git add`.

---

## Task 8: Secondary Routes And Hermes Visual Sweep

**Files:**
- Modify: remaining route files under `src/pages`
- Modify: selected Hermes route files under `src/engines/hermes/pages`
- Modify: `src/engines/hermes/style/hermes.css`
- Modify: `src/engines/xintian/style/xintian.css` only if visible shell conflicts remain

- [ ] **Step 1: Find remaining visible legacy product copy**

Run:

```powershell
rg "AgentDock|DeepAiepAi助手" src public -n
```

Classify each result:

- Visible UI copy: replace.
- Compatibility filename/path/key: keep.
- Comment only: optionally clean if touching nearby code.
- Upstream/reference context: keep if explicit.

- [ ] **Step 2: Apply page-shell class to remaining OpenClaw pages**

For each remaining page in `src/pages/*.js`, if it creates `page.className = 'page'`, change it to:

```js
page.className = 'page page-shell <route-name>-page'
```

Examples:

```js
page.className = 'page page-shell models-page'
page.className = 'page page-shell agents-page'
page.className = 'page page-shell channels-page'
```

Do not change IDs used by event handlers.

- [ ] **Step 3: Apply shared primitives to obvious repeated blocks**

Where a page has repeated button/card/table patterns, add compatible shared classes without removing old ones:

```html
<button class="btn btn-secondary ad-button ad-button-secondary">
<div class="card ad-panel">
<div class="ad-data-table">
```

Use this as a visual convergence pass, not a rewrite.

- [ ] **Step 4: Hermes visual sweep**

In `src/engines/hermes/style/hermes.css`:

- Align Hermes surfaces with new token values.
- Avoid adding new purple/blue gradient dominance.
- Keep existing Hermes-specific layout and route behavior.

In Hermes page files, only replace visible AgentDock copy with AgentDock where it is not an upstream/compatibility reference.

- [ ] **Step 5: Run search and build**

Run:

```powershell
rg "AgentDock|DeepAiepAi助手" src public -n
node node_modules\vite\bin\vite.js build
```

Expected:

- Remaining search hits are documented compatibility/reference cases.
- Build passes.

- [ ] **Step 6: Commit secondary sweep**

Run:

```powershell
git add src
git commit -m "style: align secondary routes with modern shell"
```

Expected:

```text
[codex/agentdock-new <hash>] style: align secondary routes with modern shell
```

---

## Task 9: UI Smoke, Screenshots, And Release Docs

**Files:**
- Modify: `docs/release/phase-5-ui-smoke-2026-05-15.md` or create a new dated smoke note
- Modify: `docs/release/ui-smoke-2026-05-15/*` or create a new dated screenshot directory
- Modify: `docs/release/phase-5-release-handoff.md`
- Modify: `docs/release/phase-5-release-checklist.md`

- [ ] **Step 1: Run full tests**

Run:

```powershell
node --test tests\*.test.js
cargo check --manifest-path src-tauri\Cargo.toml
node node_modules\vite\bin\vite.js build
```

Expected:

```text
tests pass
cargo check pass
vite build pass
```

The known `i18n` size warning may remain.

- [ ] **Step 2: Run route-level UI smoke**

Start the static server:

```powershell
node scripts\serve.js --host 127.0.0.1 --port 1421
```

In another PowerShell:

```powershell
$env:AGENTDOCK_SMOKE_PASSWORD = (Get-Content "$HOME\.openclaw\agentdock.json" -Raw | ConvertFrom-Json).accessPassword
node scripts\smoke-ui-routes.mjs --base-url http://127.0.0.1:1421 --password $env:AGENTDOCK_SMOKE_PASSWORD --out-dir docs\release\ui-smoke-2026-05-15
```

Expected:

```text
UI smoke passed for 6 routes
PASS /dashboard
PASS /settings
PASS /services
PASS /assistant
PASS /logs
PASS /about
```

- [ ] **Step 3: Manually inspect screenshots**

Open screenshots under:

```text
docs\release\ui-smoke-2026-05-15
```

Check:

- Sidebar uses AgentDock brand.
- Assistant route shows DeepAi naming.
- No obvious text overlap.
- Dashboard, Settings, Services, Logs, and About follow the Modern Ops direction.
- Gateway/security banners remain visible when present.

- [ ] **Step 4: Update release docs**

In `docs/release/phase-5-release-handoff.md`, add a completed note:

```markdown
- Modern Ops Console UI/brand refactor completed for app shell and core routes; route-level UI smoke screenshots refreshed.
```

In `docs/release/phase-5-release-checklist.md`, add under Full Smoke:

```markdown
- Confirm AgentDock final brand assets, DeepAi assistant naming, and release links are visible in the installed app.
```

- [ ] **Step 5: Commit smoke and release docs**

Run:

```powershell
git add docs\release
git commit -m "docs: record ui brand refactor smoke"
```

Expected:

```text
[codex/agentdock-new <hash>] docs: record ui brand refactor smoke
```

---

## Task 10: Final Full Verification And Handoff

**Files:**
- Read only unless a verification gap is found.

- [ ] **Step 1: Confirm working tree**

Run:

```powershell
git status --short
git log --oneline -10
```

Expected:

- Only intentional untracked files remain, if any.
- Recent commits correspond to plan tasks.

- [ ] **Step 2: Run full verification**

Run:

```powershell
node --test tests\*.test.js
cargo check --manifest-path src-tauri\Cargo.toml
node node_modules\vite\bin\vite.js build
node scripts\verify-release-smoke.mjs --bundle-dir src-tauri\target\x86_64-pc-windows-msvc\release\bundle --platform windows
```

Expected:

- Node tests pass.
- Cargo check passes.
- Vite build passes with only known `i18n` size warning.
- Release smoke passes for Windows bundle if the artifact exists.

- [ ] **Step 3: Final brand search**

Run:

```powershell
rg "AgentDock|DeepAiepAi助手" src public README.md README.en.md docs -n
```

Expected:

- No visible production UI copy remains.
- Remaining hits are one of:
  - legacy config path references such as `agentdock.json`.
  - upstream/reference documentation.
  - release smoke historical excerpts.
  - compatibility notes.

- [ ] **Step 4: Final handoff summary**

Prepare final summary with:

- Completed UI direction.
- Changed core surfaces.
- Remaining known warning: `i18n` chunk size.
- Remaining deferred items: formal signing certificate, manual installed-app smoke, CI/release automation, license/upstream strategy.

Do not claim completion unless Step 2 verification passed in the current turn.

---

## Self-Review

Spec coverage:

- Selected A Modern Ops Console direction: Tasks 3, 4, 5, 6, 8.
- Product and assistant naming: Tasks 1, 2, 10.
- Brand assets: Task 7.
- Release entry points: Tasks 2, 6, 9.
- Compatibility exceptions: Tasks 2, 8, 10.
- Testing and screenshots: Tasks 1, 6, 9, 10.

No placeholders remain. Tasks are intentionally staged so each commit keeps the app buildable.
