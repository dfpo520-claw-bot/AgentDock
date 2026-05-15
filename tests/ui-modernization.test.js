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
