import test from 'node:test'
import assert from 'node:assert/strict'
import fs from 'node:fs'

const ALLOWED_LEGACY_FILES = new Set([
  'docs/superpowers/specs/2026-05-14-configuration-ownership-design.md',
  'docs/superpowers/plans/2026-05-14-configuration-ownership.md',
  'src-tauri/src/product_config.rs',
  'src-tauri/src/commands/mod.rs',
  'src-tauri/src/commands/config.rs',
  'src/lib/product-config.js',
  'src/main.js',
  'src/pages/settings.js',
  'src/pages/setup.js',
  'src/locales/modules/settings.js',
  'src/locales/modules/setup.js',
])

const SCANNED_FILES = [
  'src-tauri/src/product_config.rs',
  'src-tauri/src/commands/mod.rs',
  'src-tauri/src/commands/config.rs',
  'src/lib/product-config.js',
  'src/lib/tauri-api.js',
  'src/main.js',
  'src/pages/settings.js',
  'src/pages/setup.js',
  'src/locales/modules/settings.js',
  'src/locales/modules/setup.js',
]

function stripCodeComments(text) {
  return text
    .replace(/\/\*[\s\S]*?\*\//g, '')
    .replace(/(^|[^:])\/\/.*$/gm, '$1')
}

test('product config surfaces declare AgentDock-owned config names', () => {
  const frontend = fs.readFileSync('src/lib/product-config.js', 'utf8')
  const rust = fs.readFileSync('src-tauri/src/product_config.rs', 'utf8')

  assert.match(frontend, /panelConfigFile:\s*'agentdock\.json'/)
  assert.match(frontend, /productDataDirName:\s*'\.agentdock'/)
  assert.match(rust, /PRODUCT_CONFIG_FILENAME:\s*&str\s*=\s*"agentdock\.json"/)
  assert.match(rust, /PRODUCT_DATA_DIR_NAME:\s*&str\s*=\s*"\.agentdock"/)
})

test('legacy panel config literals stay in declared compatibility files', () => {
  for (const file of SCANNED_FILES) {
    const raw = fs.readFileSync(file, 'utf8')
    const text = file.endsWith('.js') || file.endsWith('.rs')
      ? stripCodeComments(raw)
      : raw
    if (!text.includes('clawpanel.json') && !text.includes('.openclaw')) continue
    assert.equal(
      ALLOWED_LEGACY_FILES.has(file),
      true,
      `${file} uses legacy config names outside the compatibility boundary`,
    )
  }
})
