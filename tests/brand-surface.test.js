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
  /agentdock\.json/i,
  /agentdock_authed/i,
  /agentdock_must_change_pw/i,
  /disabled-by-agentdock/i,
]

const OLD_ZH_ROOT = String.fromCodePoint(0x6674, 0x8fb0)
const OLD_ZH_SIMPLIFIED_CLOUD = `${OLD_ZH_ROOT}${String.fromCodePoint(0x4e91)}`
const OLD_ZH_TRADITIONAL_CLOUD = `${OLD_ZH_ROOT}${String.fromCodePoint(0x96f2)}`
const OLD_ZH_ASSISTANT = `${OLD_ZH_ROOT}${String.fromCodePoint(0x52a9, 0x624b)}`
const OLD_QING = ['qing', 'chen'].join('')
const OLD_QING_CLOUD = `${OLD_QING}cloud`
const OLD_QING_SPACE_CLOUD = `${OLD_QING} cloud`
const OLD_PANEL = ['Claw', 'Panel'].join('')
const OLD_PANEL_DOMAIN = ['claw', 'qt', 'cool'].join('.')
const OLD_PANEL_IDENTIFIER = ['ai', 'openclaw', OLD_PANEL.toLowerCase()].join('.')
const OLD_COMPANY_SIMPLIFIED = `${String.fromCodePoint(0x6b66, 0x6c49)}${OLD_ZH_ROOT}${String.fromCodePoint(0x5929, 0x4e0b, 0x7f51, 0x7edc, 0x79d1, 0x6280, 0x6709, 0x9650, 0x516c, 0x53f8)}`
const OLD_COMPANY_TRADITIONAL = `${String.fromCodePoint(0x6b66, 0x6f22)}${OLD_ZH_ROOT}${String.fromCodePoint(0x5929, 0x4e0b, 0x7db2, 0x8def, 0x79d1, 0x6280, 0x6709, 0x9650, 0x516c, 0x53f8)}`
const OLD_COMPANY_EN = ['Wuhan ', 'Qing', 'chen Tianxia Network Technology Co., Ltd.'].join('')

function literalPattern(value, flags = '') {
  return new RegExp(value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), flags)
}

const VISIBLE_LEGACY_PATTERNS = [
  { label: 'old panel brand', pattern: literalPattern(OLD_PANEL) },
  { label: 'old simplified cloud brand', pattern: literalPattern(OLD_ZH_SIMPLIFIED_CLOUD) },
  { label: 'old traditional cloud brand', pattern: literalPattern(OLD_ZH_TRADITIONAL_CLOUD) },
  { label: 'old assistant brand', pattern: literalPattern(OLD_ZH_ASSISTANT) },
  { label: 'old latin brand', pattern: literalPattern(OLD_QING, 'i') },
  { label: 'old company simplified', pattern: literalPattern(OLD_COMPANY_SIMPLIFIED) },
  { label: 'old company traditional', pattern: literalPattern(OLD_COMPANY_TRADITIONAL) },
  { label: 'old company english', pattern: literalPattern(OLD_COMPANY_EN, 'i') },
  { label: 'old website domain', pattern: literalPattern(OLD_PANEL_DOMAIN) },
  { label: 'old owner/agentdock', pattern: literalPattern(`${OLD_QING_CLOUD}/agentdock`, 'i') },
  { label: 'old app identifier', pattern: literalPattern(OLD_PANEL_IDENTIFIER, 'i') },
]

const OLD_APP_PATTERNS = [
  literalPattern(OLD_PANEL, 'g'),
  literalPattern(OLD_PANEL_DOMAIN, 'g'),
  literalPattern(`${OLD_QING_CLOUD}/agentdock`, 'gi'),
  literalPattern(OLD_QING_SPACE_CLOUD, 'gi'),
  literalPattern(OLD_PANEL_IDENTIFIER, 'g'),
]

const RESERVED_PLACEHOLDER_PATTERNS = [
  /agentdock\.example\.com/g,
]

function stripReadmeAttribution(text) {
  return text.replace(/\n## Upstream and Referenced Projects[\s\S]*$/m, '')
}

function visibleLinesWithOldBrand(text) {
  return text
    .split(/\r?\n/)
    .map((line, index) => ({ line, index: index + 1 }))
    .filter(({ line }) => VISIBLE_LEGACY_PATTERNS.some(({ pattern }) => pattern.test(line)))
    .filter(({ line }) => !ALLOWED_LEGACY_CONTEXT.some((pattern) => pattern.test(line)))
}

test('first-run brand surfaces use AgentDock identity', () => {
  for (const file of BRAND_SURFACE_FILES) {
    const raw = fs.readFileSync(file, 'utf8')
    const text = file === 'README.md'
      ? stripReadmeAttribution(raw)
      : raw
    for (const pattern of OLD_APP_PATTERNS) {
      assert.doesNotMatch(text, pattern, `${file} still matches ${pattern}`)
    }
  }
})

test('production brand surfaces do not ship reserved placeholder domains', () => {
  for (const file of BRAND_SURFACE_FILES) {
    const raw = fs.readFileSync(file, 'utf8')
    const text = file === 'README.md'
      ? stripReadmeAttribution(raw)
      : raw
    for (const pattern of RESERVED_PLACEHOLDER_PATTERNS) {
      assert.doesNotMatch(text, pattern, `${file} still matches ${pattern}`)
    }
  }
})

test('about page does not render legacy mirror i18n keys', () => {
  const text = fs.readFileSync('src/pages/about.js', 'utf8')

  assert.doesNotMatch(
    text,
    /\babout\.domesticMirrorHint\b/,
    'about page must not render domesticMirrorHint because it carries old app mirror links'
  )
})

test('about page uses destination-accurate action labels', () => {
  const text = [
    fs.readFileSync('src/pages/about.js', 'utf8'),
    fs.readFileSync('src/main.js', 'utf8'),
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

test('app-visible copy uses AgentDock and DeepAi assistant naming', () => {
  for (const file of APP_VISIBLE_BRAND_FILES) {
    const text = fs.readFileSync(file, 'utf8')
    const offenders = visibleLinesWithOldBrand(text)
    assert.deepEqual(
      offenders,
      [],
      `${file} has visible legacy copy:\n${offenders.map(({ index, line }) => `${index}: ${line}`).join('\n')}`
    )
  }
})
