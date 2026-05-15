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

test('main delegates gateway banner app-shell ownership to a dedicated module', () => {
  const main = fs.readFileSync('src/main.js', 'utf8')
  const gatewayBanner = fs.readFileSync('src/app-shell/gateway-banner.js', 'utf8')

  assert.match(main, /from '\.\/app-shell\/gateway-banner\.js'/)
  assert.match(main, /createGatewayBannerController\(/)
  assert.doesNotMatch(main, /function setupGatewayBanner\s*\(/)
  assert.doesNotMatch(main, /function showGuardianRecovery\s*\(/)
  assert.match(gatewayBanner, /export function createGatewayBannerController/)
})

test('app config commands move behind a dedicated Rust boundary module', () => {
  assert.equal(fs.existsSync('src-tauri/src/commands/app_config.rs'), true)

  const configRs = fs.readFileSync('src-tauri/src/commands/config.rs', 'utf8')
  const appConfigRs = fs.readFileSync('src-tauri/src/commands/app_config.rs', 'utf8')
  const modRs = fs.readFileSync('src-tauri/src/commands/mod.rs', 'utf8')

  assert.match(modRs, /pub mod app_config;/)

  for (const name of [
    'get_openclaw_dir',
    'read_panel_config',
    'write_panel_config',
    'detect_legacy_config_migration',
    'apply_legacy_config_migration',
    'get_npm_registry',
    'set_npm_registry',
    'invalidate_path_cache',
  ]) {
    assert.match(appConfigRs, new RegExp(`pub fn ${name}\\b`))
    assert.match(configRs, new RegExp(`super::app_config::${name}\\(`))
  }
})

test('gateway runtime commands move out of config.rs ownership', () => {
  assert.equal(fs.existsSync('src-tauri/src/commands/gateway_runtime.rs'), true)

  const configRs = fs.readFileSync('src-tauri/src/commands/config.rs', 'utf8')
  const gatewayRuntimeRs = fs.readFileSync('src-tauri/src/commands/gateway_runtime.rs', 'utf8')
  const modRs = fs.readFileSync('src-tauri/src/commands/mod.rs', 'utf8')
  const libRs = fs.readFileSync('src-tauri/src/lib.rs', 'utf8')

  assert.match(modRs, /pub mod gateway_runtime;/)

  for (const name of [
    'reload_gateway',
    'restart_gateway',
    'doctor_fix',
    'doctor_check',
    'install_gateway',
    'uninstall_gateway',
  ]) {
    assert.match(gatewayRuntimeRs, new RegExp(`pub (async )?fn ${name}\\b`))
    assert.match(libRs, new RegExp(`gateway_runtime::${name}`))
    assert.doesNotMatch(configRs, new RegExp(`pub (async )?fn ${name}\\b`))
  }

  assert.match(configRs, /gateway_runtime::do_reload_gateway/)
})
