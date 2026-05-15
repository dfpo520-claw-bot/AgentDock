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

const RESERVED_PLACEHOLDER_PATTERNS = [
  /agentdock\.example\.com/g,
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

test('production brand surfaces do not ship reserved placeholder domains', () => {
  for (const file of BRAND_SURFACE_FILES) {
    const raw = fs.readFileSync(file, 'utf8')
    const text = file.endsWith('.js') || file.endsWith('.rs')
      ? stripJavaScriptComments(raw)
      : raw
    for (const pattern of RESERVED_PLACEHOLDER_PATTERNS) {
      assert.doesNotMatch(text, pattern, `${file} still matches ${pattern}`)
    }
  }
})

test('about page does not render legacy mirror i18n keys', () => {
  const text = stripJavaScriptComments(fs.readFileSync('src/pages/about.js', 'utf8'))

  assert.doesNotMatch(
    text,
    /\babout\.domesticMirrorHint\b/,
    'about page must not render domesticMirrorHint because it carries old app mirror links'
  )
})

test('about page uses destination-accurate action labels', () => {
  const text = [
    stripJavaScriptComments(fs.readFileSync('src/pages/about.js', 'utf8')),
    stripJavaScriptComments(fs.readFileSync('src/main.js', 'utf8')),
  ].join('\n')

  assert.doesNotMatch(
    text,
    /href="\$\{PRODUCT_IDENTITY\.releaseUrl\}"[\s\S]*?\$\{t\('about\.downloadFromGitHub'\)\}/,
    'release links must use a generic releases label, not downloadFromGitHub'
  )

  assert.doesNotMatch(
    text,
    /href="\$\{PRODUCT_IDENTITY\.(?:repositoryUrl|supportUrl)\}"[\s\S]*?\$\{t\('about\.(?:submitPR|contributeGuide|viewIssues)'\)\}/,
    'generic repository/support links must not use PR, guide, or issues-specific labels'
  )
})
