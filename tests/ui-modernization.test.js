import test from 'node:test'
import assert from 'node:assert/strict'
import fs from 'node:fs'

function read(file) {
  return fs.readFileSync(file, 'utf8')
}

function cssClassSelectorPattern(className) {
  return new RegExp(`\\.${className}(?=[\\s,.#:{\\[])`)
}

function assertModernRootClass(file, expectedClass) {
  const text = read(file)
  const assignment = text.match(/page\.className\s*=\s*['"`]([^'"`]*)['"`]/)
  assert.ok(assignment, `${file} should assign route root page.className`)
  assert.ok(
    assignment[1].split(/\s+/).includes(expectedClass),
    `${file} should assign ${expectedClass} as an exact route root class token`
  )
}

test('modern ops design tokens are exposed in variables.css', () => {
  const css = read('src/style/variables.css')

  for (const token of [
    '--surface-0',
    '--surface-1',
    '--surface-2',
    '--accent-ops',
    '--accent-release',
    '--radius-control',
    '--shadow-panel',
    '--content-max-form',
  ]) {
    assert.match(css, new RegExp(`${token}:`), `${token} missing from Modern Ops token contract`)
  }
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
    assert.match(css, cssClassSelectorPattern(className), `${className} missing as a concrete CSS selector`)
  }
})

test('app shell exposes modern sidebar and page layout primitives', () => {
  const layout = read('src/style/layout.css')
  const pages = read('src/style/pages.css')

  assert.match(layout, /#sidebar/, '#sidebar shell selector missing')
  assert.match(layout, cssClassSelectorPattern('app-content-frame'), 'app-content-frame shell selector missing')
  assert.match(layout, cssClassSelectorPattern('sidebar-logo-mark'), 'sidebar-logo-mark selector missing')
  assert.match(pages, cssClassSelectorPattern('page-shell'), 'page-shell route layout selector missing')
  assert.match(pages, cssClassSelectorPattern('page-header-actions'), 'page-header-actions selector missing')
  assert.match(pages, cssClassSelectorPattern('page-toolbar'), 'page-toolbar selector missing')
})

test('engine dropdown expands within sidebar flow instead of covering navigation', () => {
  const layout = read('src/style/layout.css')
  const dropdown = layout.match(/\.engine-dropdown\s*{(?<body>[^}]*)}/)

  assert.ok(dropdown?.groups?.body, 'engine dropdown CSS block missing')
  assert.doesNotMatch(dropdown.groups.body, /position:\s*absolute\b/, 'engine dropdown must not overlay sidebar navigation')
  assert.match(dropdown.groups.body, /position:\s*(static|relative)\b/, 'engine dropdown should participate in sidebar layout')
  assert.match(dropdown.groups.body, /width:\s*100%/, 'engine dropdown should keep the engine switcher width')
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
    assertModernRootClass(file, file === 'src/pages/assistant.js' ? 'ast-page' : 'page-shell')
  }
})

test('assistant settings hides signup promo without disabling core API controls', () => {
  const assistant = read('src/pages/assistant.js')

  assert.doesNotMatch(assistant, /id="ast-qtcool-promo"/, 'assistant settings should not render the qtcool signup promo card')

  const resultIndex = assistant.indexOf("const resultEl = overlay.querySelector('#ast-test-result')")
  assert.ok(resultIndex >= 0, 'assistant settings should initialize the API test result area')

  for (const selector of [
    '#ast-btn-test',
    '#ast-btn-models',
    '#ast-btn-import',
  ]) {
    const buttonIndex = assistant.indexOf(`id="${selector.slice(1)}"`)
    const bindingIndex = assistant.indexOf(`querySelector('${selector}')`)
    assert.ok(buttonIndex >= 0, `${selector} button should still render`)
    assert.ok(bindingIndex > resultIndex, `${selector} event binding should stay on the active settings path`)
  }
})

test('assistant settings defaults to the DeepAi-only API surface', () => {
  const assistant = read('src/pages/assistant.js')

  assert.doesNotMatch(assistant, /id="ast-provider-presets"/, 'assistant settings should not render the generic provider preset strip')
  assert.match(assistant, /id="ast-deepai-provider"/, 'assistant settings should render the dedicated DeepAi provider surface')
  assert.match(assistant, /获取 API Key/, 'assistant settings should render the API key acquisition link text')
  assert.match(assistant, /https:\/\/api\.deepai\.wang\/v1/, 'assistant settings should include the DeepAi default API base URL')
  assert.match(assistant, /https:\/\/api\.deepai\.wang\//, 'assistant settings should include the DeepAi API key link target')
  assert.match(assistant, /id="ast-baseurl"/, 'assistant settings should keep the base URL input')
  assert.match(assistant, /id="ast-apikey"/, 'assistant settings should keep the API key input')
  assert.match(assistant, /id="ast-model"/, 'assistant settings should keep the model input')
})

test('about page no longer renders the community exchange section', () => {
  const about = read('src/pages/about.js')

  assert.doesNotMatch(about, /id="community-section"/, 'about page should not render the community section container')
  assert.doesNotMatch(about, /t\('about\.sectionCommunity'\)/, 'about page should not render the community section title')
  assert.doesNotMatch(about, /renderCommunity\(page\)/, 'about page should not call the community renderer')
})

test('hermes engine listener callbacks use the declared listener collection', () => {
  const hermes = read('src/engines/hermes/index.js')

  assert.match(hermes, /let\s+_listeners\s*=\s*\[\]/, 'Hermes should declare its listener collection')
  assert.doesNotMatch(hermes, /\b_stateListeners\b/, 'Hermes must not reference undeclared state listener storage')
  assert.doesNotMatch(hermes, /\b_readyListeners\b/, 'Hermes must not reference undeclared ready listener storage')
  assert.match(hermes, /onStateChange\(fn\)\s*{\s*_listeners\.push\(fn\)/s, 'onStateChange should register with the declared listener collection')
  assert.match(hermes, /onReadyChange\(fn\)\s*{\s*_listeners\.push\(fn\)/s, 'onReadyChange should register with the declared listener collection')
})

test('engine chooser detects OpenClaw and Hermes installation status on first render', () => {
  const page = read('src/pages/engine-select.js')
  const css = read('src/style/pages.css')

  assert.match(page, /from '..\/lib\/engine-manager\.js'[\s\S]*getEngine/, 'engine chooser should access registered engines')
  assert.match(page, /ENGINE_STATUS_IDS\s*=\s*\[[\s\S]*'openclaw'[\s\S]*'hermes'[\s\S]*\]/, 'engine chooser should track both installable engines')
  assert.match(page, /refreshEngineStatuses\(page\)/, 'engine chooser should refresh status after first render')
  assert.match(page, /Promise\.all\(ENGINE_STATUS_IDS\.map/, 'engine chooser should detect both engines in parallel')
  assert.match(page, /\.detect\(\)/, 'engine chooser should call each engine detect contract')
  assert.match(css, cssClassSelectorPattern('es-install-status'), 'engine chooser should expose visible install status styling')
})

test('sidebar engine switcher does not perform installation detection', () => {
  const sidebar = read('src/components/sidebar.js')
  const layout = read('src/style/layout.css')

  assert.doesNotMatch(sidebar, /\bgetEngine\b/, 'sidebar should not access install-detection engine contracts')
  assert.doesNotMatch(sidebar, /\bENGINE_STATUS_IDS\b/, 'sidebar should not track install status')
  assert.doesNotMatch(sidebar, /refreshEngineSwitcherStatuses/, 'sidebar should not run install detection after render')
  assert.doesNotMatch(sidebar, /\.detect\(\)/, 'sidebar should not call engine detect')
  assert.doesNotMatch(layout, cssClassSelectorPattern('engine-status'), 'sidebar should not expose install status styling')
})

test('dashboard owns OpenClaw install detection and keeps version checks non-blocking', () => {
  const dashboard = read('src/pages/dashboard.js')
  const criticalIndex = dashboard.indexOf('const criticalP = Promise.allSettled')
  const versionIndex = dashboard.indexOf('const versionP =')
  const firstRenderIndex = dashboard.indexOf('renderStatCards(page, services, version, [], config, panelConfig)')
  const awaitVersionIndex = dashboard.indexOf('const versionRes = await versionP')

  assert.match(dashboard, /api\.checkInstallation\(\)/, 'dashboard should detect whether OpenClaw is installed')
  assert.ok(criticalIndex >= 0, 'dashboard should split critical service/config loading from slow tasks')
  assert.ok(versionIndex >= 0, 'dashboard should isolate version loading into a separate promise')
  assert.ok(firstRenderIndex >= 0, 'dashboard should render stat cards from critical data first')
  assert.ok(awaitVersionIndex >= 0, 'dashboard should await version loading only after first stat render')
  assert.ok(firstRenderIndex < awaitVersionIndex, 'dashboard first stat render must happen before awaiting version result')
  assert.doesNotMatch(dashboard, /versionLoadFail/, 'dashboard should not show a user-facing error when optional version probing fails')
})

test('initial boot defaults to OpenClaw instead of forcing the engine chooser', () => {
  const manager = read('src/lib/engine-manager.js')
  const main = read('src/main.js')

  assert.match(manager, /let\s+mode\s*=\s*'openclaw'/, 'OpenClaw should remain the initial engine default')
  assert.doesNotMatch(manager, /_needsInitialEngineChoice\s*=\s*!hasChoice/, 'missing legacy engine choice should not force the chooser')
  assert.doesNotMatch(main, /needsInitialEngineChoice\(\)\s*\|\|\s*isEngineSetupDeferred\(\)[\s\S]{0,120}setDefaultRoute\('\/engine-select'\)/, 'startup should not point the default route at the engine chooser')
  assert.doesNotMatch(main, /needsInitialEngineChoice\(\)\s*&&\s*!engine\.isReady\(\)[\s\S]{0,120}navigate\('\/engine-select'\)/, 'OpenClaw not-ready startup should stay in the OpenClaw flow')
})
