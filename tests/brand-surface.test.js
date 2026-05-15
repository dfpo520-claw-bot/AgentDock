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
  /clawpanel\.json/i,
  /clawpanel_authed/i,
  /clawpanel_must_change_pw/i,
  /disabled-by-clawpanel/i,
]

const VISIBLE_LEGACY_PATTERNS = [
  { label: 'ClawPanel', pattern: /ClawPanel/ },
  { label: '晴辰助手', pattern: /晴辰助手/ },
  { label: 'mojibake 晴辰助手', pattern: /鏅磋景鍔╂墜/ },
  { label: 'claw.qt.cool', pattern: /claw\.qt\.cool/ },
  { label: 'qingchencloud/clawpanel', pattern: /qingchencloud\/clawpanel/i },
  { label: 'ai.openclaw.clawpanel', pattern: /ai\.openclaw\.clawpanel/i },
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
