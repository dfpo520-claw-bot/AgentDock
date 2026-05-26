# DeepAi Assistant Settings Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Simplify the chat-page assistant settings modal so only the DeepAi assistant provider is shown while preserving API Key, Base URL, model, connection test, model fetch, and sync behavior.

**Architecture:** Keep the existing Vite + vanilla JavaScript modal structure and current assistant config object. Implement the change in two thin slices: a regression test that locks the desired UI contract, then a small template/default-value update in `src/pages/assistant.js` plus minimal locale copy support if needed.

**Tech Stack:** Vite, vanilla JavaScript modules, Tauri desktop shell, Node test runner.

---

## File Structure And Ownership

- `docs/superpowers/specs/2026-05-26-deepai-assistant-settings-design.md`: approved scope and UX contract.
- `tests/ui-modernization.test.js`: static regression coverage for the assistant settings surface.
- `src/pages/assistant.js`: settings modal template and default Base URL behavior.
- `src/locales/modules/assistant.js`: optional localized copy for the DeepAi-only provider surface.

---

### Task 1: Lock The New UI Contract With A Failing Test

**Files:**
- Modify: `tests/ui-modernization.test.js`
- Spec: `docs/superpowers/specs/2026-05-26-deepai-assistant-settings-design.md`

- [ ] **Step 1: Write the failing test**

Add a new test near the existing assistant settings regression:

```js
test('assistant settings defaults to the DeepAi-only API surface', () => {
  const assistant = read('src/pages/assistant.js')

  assert.doesNotMatch(assistant, /id="ast-provider-presets"/, 'assistant settings should not render the generic provider preset strip')
  assert.match(assistant, /DeepAi助手/, 'assistant settings should render the DeepAi assistant provider surface')
  assert.match(assistant, /https:\/\/api\.deepai\.wang\/v1/, 'assistant settings should include the DeepAi default API base URL')
  assert.match(assistant, /https:\/\/api\.deepai\.wang\//, 'assistant settings should include the DeepAi API key link target')
  assert.match(assistant, /id="ast-baseurl"/, 'assistant settings should keep the base URL input')
  assert.match(assistant, /id="ast-apikey"/, 'assistant settings should keep the API key input')
  assert.match(assistant, /id="ast-model"/, 'assistant settings should keep the model input')
})
```

- [ ] **Step 2: Run the test to verify it fails**

Run:

```powershell
node --test tests\ui-modernization.test.js
```

Expected before implementation:

```text
not ok ... assistant settings defaults to the DeepAi-only API surface
```

The failure should show that `ast-provider-presets` is still present or that the DeepAi-only surface is missing.

---

### Task 2: Implement The DeepAi-Only API Surface

**Files:**
- Modify: `src/pages/assistant.js`
- Modify if needed: `src/locales/modules/assistant.js`
- Test: `tests/ui-modernization.test.js`

- [ ] **Step 1: Add the minimal implementation**

In `src/pages/assistant.js`:

- Add a constant for the DeepAi default base URL:

```js
const DEEPAI_DEFAULT_BASE_URL = 'https://api.deepai.wang/v1'
```

- In `showSettings()`, compute the rendered base URL with:

```js
const resolvedBaseUrl = (c.baseUrl || '').trim() || DEEPAI_DEFAULT_BASE_URL
```

- Replace the generic provider preset strip with a compact DeepAi-only surface and API key link.

- Keep the existing input ids:

```html
id="ast-baseurl"
id="ast-apikey"
id="ast-model"
```

- Keep existing event bindings and save/test handlers attached to the same ids.

- [ ] **Step 2: Run the focused test to verify it passes**

Run:

```powershell
node --test tests\ui-modernization.test.js
```

Expected:

```text
# pass ...
ok ...
```

- [ ] **Step 3: Run the production build as a regression check**

Run:

```powershell
node node_modules\vite\bin\vite.js build
```

Expected:

```text
✓ built in ...
```
