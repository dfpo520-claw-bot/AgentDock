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
  const agentRs = fs.readFileSync('src-tauri/src/commands/agent.rs', 'utf8')
  const messagingRs = fs.readFileSync('src-tauri/src/commands/messaging.rs', 'utf8')
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

  assert.doesNotMatch(configRs, /pub async fn do_reload_gateway\b/)
  assert.doesNotMatch(agentRs, /super::config::do_reload_gateway/)
  assert.doesNotMatch(messagingRs, /super::config::do_reload_gateway/)
  assert.match(agentRs, /super::gateway_runtime::do_reload_gateway/)
  assert.match(messagingRs, /super::gateway_runtime::do_reload_gateway/)
})

test('status summary command moves out of config.rs ownership', () => {
  assert.equal(fs.existsSync('src-tauri/src/commands/status_summary.rs'), true)

  const configRs = fs.readFileSync('src-tauri/src/commands/config.rs', 'utf8')
  const statusSummaryRs = fs.readFileSync('src-tauri/src/commands/status_summary.rs', 'utf8')
  const modRs = fs.readFileSync('src-tauri/src/commands/mod.rs', 'utf8')
  const libRs = fs.readFileSync('src-tauri/src/lib.rs', 'utf8')

  assert.match(modRs, /pub mod status_summary;/)
  assert.match(statusSummaryRs, /pub async fn get_status_summary\b/)
  assert.match(libRs, /status_summary::get_status_summary/)
  assert.doesNotMatch(configRs, /pub async fn get_status_summary\b/)
})

test('standalone path helpers move out of config.rs ownership', () => {
  assert.equal(fs.existsSync('src-tauri/src/standalone_paths.rs'), true)

  const configRs = fs.readFileSync('src-tauri/src/commands/config.rs', 'utf8')
  const standalonePathsRs = fs.readFileSync('src-tauri/src/standalone_paths.rs', 'utf8')
  const serviceRs = fs.readFileSync('src-tauri/src/commands/service.rs', 'utf8')
  const cliConflictRs = fs.readFileSync('src-tauri/src/commands/cli_conflict.rs', 'utf8')
  const utilsRs = fs.readFileSync('src-tauri/src/utils.rs', 'utf8')
  const commandsModRs = fs.readFileSync('src-tauri/src/commands/mod.rs', 'utf8')
  const libRs = fs.readFileSync('src-tauri/src/lib.rs', 'utf8')

  assert.match(libRs, /mod standalone_paths;/)
  assert.match(standalonePathsRs, /pub\(crate\) fn all_standalone_dirs\b/)
  assert.doesNotMatch(configRs, /pub\(crate\) fn all_standalone_dirs\b/)
  assert.doesNotMatch(serviceRs, /commands::config::all_standalone_dirs/)
  assert.doesNotMatch(cliConflictRs, /commands::config::all_standalone_dirs/)
  assert.doesNotMatch(utilsRs, /commands::config::all_standalone_dirs/)
  assert.doesNotMatch(commandsModRs, /config::all_standalone_dirs/)
})

test('runtime support helpers move out of config.rs ownership', () => {
  assert.equal(fs.existsSync('src-tauri/src/runtime_support.rs'), true)

  const configRs = fs.readFileSync('src-tauri/src/commands/config.rs', 'utf8')
  const gatewayRuntimeRs = fs.readFileSync('src-tauri/src/commands/gateway_runtime.rs', 'utf8')
  const runtimeSupportRs = fs.readFileSync('src-tauri/src/runtime_support.rs', 'utf8')
  const libRs = fs.readFileSync('src-tauri/src/lib.rs', 'utf8')

  assert.match(libRs, /mod runtime_support;/)
  assert.match(runtimeSupportRs, /pub\(crate\) struct GuardianPause/)
  assert.match(runtimeSupportRs, /pub\(crate\) fn get_uid\b/)
  assert.doesNotMatch(configRs, /pub\(crate\) struct GuardianPause/)
  assert.doesNotMatch(configRs, /pub\(crate\) fn get_uid\b/)
  assert.doesNotMatch(gatewayRuntimeRs, /super::config::GuardianPause/)
  assert.doesNotMatch(gatewayRuntimeRs, /super::config::get_uid/)
})

test('openclaw cli path helpers move out of config.rs ownership', () => {
  assert.equal(fs.existsSync('src-tauri/src/openclaw_cli_paths.rs'), true)

  const configRs = fs.readFileSync('src-tauri/src/commands/config.rs', 'utf8')
  const utilsRs = fs.readFileSync('src-tauri/src/utils.rs', 'utf8')
  const cliPathsRs = fs.readFileSync('src-tauri/src/openclaw_cli_paths.rs', 'utf8')
  const libRs = fs.readFileSync('src-tauri/src/lib.rs', 'utf8')

  assert.match(libRs, /mod openclaw_cli_paths;/)
  assert.match(cliPathsRs, /pub\(crate\) fn is_rejected_cli_path\b/)
  assert.match(cliPathsRs, /pub\(crate\) fn resolve_openclaw_cli_input_path\b/)
  assert.match(cliPathsRs, /pub\(crate\) fn resolve_openclaw_cli_input\b/)
  assert.doesNotMatch(configRs, /pub\(crate\) fn resolve_openclaw_cli_input_path\b/)
  assert.doesNotMatch(configRs, /pub\(crate\) fn resolve_openclaw_cli_input\b/)
  assert.doesNotMatch(utilsRs, /pub fn is_rejected_cli_path\b/)
  assert.doesNotMatch(utilsRs, /commands::config::resolve_openclaw_cli_input_path/)
})

test('installation status command moves out of config.rs ownership', () => {
  assert.equal(fs.existsSync('src-tauri/src/commands/installation_status.rs'), true)

  const configRs = fs.readFileSync('src-tauri/src/commands/config.rs', 'utf8')
  const installationStatusRs = fs.readFileSync('src-tauri/src/commands/installation_status.rs', 'utf8')
  const modRs = fs.readFileSync('src-tauri/src/commands/mod.rs', 'utf8')
  const libRs = fs.readFileSync('src-tauri/src/lib.rs', 'utf8')

  assert.match(modRs, /pub mod installation_status;/)
  assert.match(installationStatusRs, /pub fn check_installation\b/)
  assert.match(libRs, /installation_status::check_installation/)
  assert.doesNotMatch(configRs, /pub fn check_installation\b/)
})

test('node runtime commands move out of config.rs ownership', () => {
  assert.equal(fs.existsSync('src-tauri/src/commands/node_runtime.rs'), true)

  const configRs = fs.readFileSync('src-tauri/src/commands/config.rs', 'utf8')
  const nodeRuntimeRs = fs.readFileSync('src-tauri/src/commands/node_runtime.rs', 'utf8')
  const modRs = fs.readFileSync('src-tauri/src/commands/mod.rs', 'utf8')
  const libRs = fs.readFileSync('src-tauri/src/lib.rs', 'utf8')

  assert.match(modRs, /pub mod node_runtime;/)
  for (const name of [
    'check_node',
    'check_node_at_path',
    'scan_node_paths',
    'save_custom_node_path',
  ]) {
    assert.match(nodeRuntimeRs, new RegExp(`pub fn ${name}\\b`))
    assert.match(libRs, new RegExp(`node_runtime::${name}`))
    assert.doesNotMatch(configRs, new RegExp(`pub fn ${name}\\b`))
  }
})

test('proxy diagnostics command moves out of config.rs ownership', () => {
  assert.equal(fs.existsSync('src-tauri/src/commands/proxy_diagnostics.rs'), true)

  const configRs = fs.readFileSync('src-tauri/src/commands/config.rs', 'utf8')
  const proxyDiagnosticsRs = fs.readFileSync('src-tauri/src/commands/proxy_diagnostics.rs', 'utf8')
  const modRs = fs.readFileSync('src-tauri/src/commands/mod.rs', 'utf8')
  const libRs = fs.readFileSync('src-tauri/src/lib.rs', 'utf8')

  assert.match(modRs, /pub mod proxy_diagnostics;/)
  assert.match(proxyDiagnosticsRs, /pub async fn test_proxy\b/)
  assert.match(libRs, /proxy_diagnostics::test_proxy/)
  assert.doesNotMatch(configRs, /pub async fn test_proxy\b/)
})

test('openclaw installation scanning moves out of config.rs ownership', () => {
  assert.equal(fs.existsSync('src-tauri/src/commands/openclaw_installations.rs'), true)

  const configRs = fs.readFileSync('src-tauri/src/commands/config.rs', 'utf8')
  const openclawInstallationsRs = fs.readFileSync('src-tauri/src/commands/openclaw_installations.rs', 'utf8')
  const modRs = fs.readFileSync('src-tauri/src/commands/mod.rs', 'utf8')
  const libRs = fs.readFileSync('src-tauri/src/lib.rs', 'utf8')

  assert.match(modRs, /pub mod openclaw_installations;/)
  for (const name of [
    'scan_openclaw_paths',
    'check_openclaw_at_path',
  ]) {
    assert.match(openclawInstallationsRs, new RegExp(`pub fn ${name}\\b`))
    assert.match(libRs, new RegExp(`openclaw_installations::${name}`))
    assert.doesNotMatch(libRs, new RegExp(`config::${name}`))
    assert.doesNotMatch(configRs, new RegExp(`pub fn ${name}\\b`))
  }

  for (const name of [
    'scan_cli_identity',
    'scan_all_installations',
    'read_version_from_installation',
  ]) {
    assert.match(openclawInstallationsRs, new RegExp(`\\bfn ${name}\\b`))
    assert.doesNotMatch(configRs, new RegExp(`\\bfn ${name}\\b`))
  }
})

test('openclaw version summary moves out of config.rs ownership', () => {
  assert.equal(fs.existsSync('src-tauri/src/commands/openclaw_version.rs'), true)

  const configRs = fs.readFileSync('src-tauri/src/commands/config.rs', 'utf8')
  const openclawVersionRs = fs.readFileSync('src-tauri/src/commands/openclaw_version.rs', 'utf8')
  const modRs = fs.readFileSync('src-tauri/src/commands/mod.rs', 'utf8')
  const libRs = fs.readFileSync('src-tauri/src/lib.rs', 'utf8')

  assert.match(modRs, /pub mod openclaw_version;/)
  assert.match(openclawVersionRs, /pub async fn get_version_info\b/)
  assert.match(openclawVersionRs, /pub async fn list_openclaw_versions\b/)
  assert.match(openclawVersionRs, /pub\(crate\) fn detect_installed_source\b/)
  assert.match(openclawVersionRs, /pub\(crate\) async fn get_local_version\b/)
  assert.match(libRs, /openclaw_version::get_version_info/)
  assert.match(libRs, /openclaw_version::list_openclaw_versions/)
  assert.doesNotMatch(libRs, /config::get_version_info/)
  assert.doesNotMatch(libRs, /config::list_openclaw_versions/)

  for (const name of [
    'get_version_info',
    'list_openclaw_versions',
    'get_local_version',
    'get_latest_version_for',
    'detect_source_from_cmd_shim',
    'detect_standalone_source_from_dir',
    'detect_standalone_source_from_cli_path',
    'detect_installed_source',
  ]) {
    assert.doesNotMatch(configRs, new RegExp(`\\b(?:pub(?:\\(crate\\))?\\s+)?(?:async\\s+)?fn ${name}\\b`))
  }
})

test('openclaw install policy moves out of config.rs ownership', () => {
  assert.equal(fs.existsSync('src-tauri/src/commands/openclaw_install_policy.rs'), true)

  const configRs = fs.readFileSync('src-tauri/src/commands/config.rs', 'utf8')
  const installPolicyRs = fs.readFileSync('src-tauri/src/commands/openclaw_install_policy.rs', 'utf8')
  const openclawVersionRs = fs.readFileSync('src-tauri/src/commands/openclaw_version.rs', 'utf8')
  const modRs = fs.readFileSync('src-tauri/src/commands/mod.rs', 'utf8')

  assert.match(modRs, /pub mod openclaw_install_policy;/)

  for (const name of [
    'panel_version',
    'npm_package_name',
    'versions_match',
    'recommended_is_newer',
    'recommended_version_for',
    'r2_config',
    'standalone_config',
  ]) {
    assert.match(installPolicyRs, new RegExp(`pub\\(crate\\) fn ${name}\\b`))
    assert.doesNotMatch(configRs, new RegExp(`\\bfn ${name}\\b`))
  }

  for (const name of [
    'VersionPolicySource',
    'VersionPolicyEntry',
    'VersionPolicy',
    'R2Config',
    'StandaloneConfig',
  ]) {
    assert.match(installPolicyRs, new RegExp(`\\bstruct ${name}\\b`))
    assert.doesNotMatch(configRs, new RegExp(`\\bstruct ${name}\\b`))
  }

  assert.match(openclawVersionRs, /openclaw_install_policy::npm_package_name\(/)
  assert.match(openclawVersionRs, /openclaw_install_policy::recommended_version_for\(/)
  assert.match(openclawVersionRs, /openclaw_install_policy::recommended_is_newer\(/)
  assert.match(openclawVersionRs, /openclaw_install_policy::versions_match\(/)
  assert.match(openclawVersionRs, /openclaw_install_policy::panel_version\(/)
  assert.doesNotMatch(openclawVersionRs, /config::npm_package_name\(/)
  assert.doesNotMatch(openclawVersionRs, /config::recommended_version_for\(/)
  assert.doesNotMatch(openclawVersionRs, /config::recommended_is_newer\(/)
  assert.doesNotMatch(openclawVersionRs, /config::versions_match\(/)
  assert.doesNotMatch(openclawVersionRs, /config::panel_version\(/)
})

test('openclaw install runtime helpers move out of config.rs ownership', () => {
  assert.equal(fs.existsSync('src-tauri/src/commands/openclaw_install_runtime.rs'), true)

  const configRs = fs.readFileSync('src-tauri/src/commands/config.rs', 'utf8')
  const installRuntimeRs = fs.readFileSync('src-tauri/src/commands/openclaw_install_runtime.rs', 'utf8')
  const standaloneInstallerRs = fs.readFileSync('src-tauri/src/commands/openclaw_standalone_installer.rs', 'utf8')
  const lifecycleRs = fs.readFileSync('src-tauri/src/commands/openclaw_lifecycle.rs', 'utf8')
  const openclawVersionRs = fs.readFileSync('src-tauri/src/commands/openclaw_version.rs', 'utf8')
  const modRs = fs.readFileSync('src-tauri/src/commands/mod.rs', 'utf8')

  assert.match(modRs, /pub mod openclaw_install_runtime;/)

  for (const name of [
    'standalone_platform_key',
    'standalone_archive_ext',
    'standalone_install_dir',
    'npm_command',
    'npm_command_elevated',
    'pre_install_cleanup',
    'r2_platform_key',
    'npm_global_modules_dir',
    'npm_global_bin_dir',
  ]) {
    assert.match(installRuntimeRs, new RegExp(`pub\\(crate\\) fn ${name}\\b`))
    assert.doesNotMatch(configRs, new RegExp(`\\bfn ${name}\\b`))
  }

  assert.match(lifecycleRs, /openclaw_install_runtime::pre_install_cleanup\(\)/)
  assert.match(lifecycleRs, /openclaw_install_runtime::npm_command_elevated\(\)/)
  assert.match(installRuntimeRs, /pub\(crate\) fn standalone_install_dir\b/)
  assert.match(standaloneInstallerRs, /openclaw_install_runtime::standalone_install_dir\(\)/)
  assert.match(openclawVersionRs, /openclaw_install_runtime::npm_global_bin_dir\(\)/)
  assert.match(openclawVersionRs, /openclaw_install_runtime::npm_command\(\)/)
})

test('openclaw r2 installer moves out of config.rs ownership', () => {
  assert.equal(fs.existsSync('src-tauri/src/commands/openclaw_r2_installer.rs'), true)

  const configRs = fs.readFileSync('src-tauri/src/commands/config.rs', 'utf8')
  const r2InstallerRs = fs.readFileSync('src-tauri/src/commands/openclaw_r2_installer.rs', 'utf8')
  const modRs = fs.readFileSync('src-tauri/src/commands/mod.rs', 'utf8')

  assert.match(modRs, /pub mod openclaw_r2_installer;/)
  assert.match(r2InstallerRs, /pub\(crate\) async fn try_r2_install\b/)
  assert.doesNotMatch(configRs, /\basync fn try_r2_install\b/)
  assert.match(r2InstallerRs, /openclaw_install_policy::r2_config\(\)/)
  assert.match(r2InstallerRs, /openclaw_install_policy::versions_match\(/)
  assert.match(r2InstallerRs, /openclaw_install_runtime::r2_platform_key\(\)/)
  assert.match(r2InstallerRs, /openclaw_install_runtime::npm_command_elevated\(\)/)
  assert.match(r2InstallerRs, /openclaw_install_runtime::npm_global_bin_dir\(\)/)
})

test('openclaw standalone installer moves out of config.rs ownership', () => {
  assert.equal(fs.existsSync('src-tauri/src/commands/openclaw_standalone_installer.rs'), true)

  const configRs = fs.readFileSync('src-tauri/src/commands/config.rs', 'utf8')
  const lifecycleRs = fs.readFileSync('src-tauri/src/commands/openclaw_lifecycle.rs', 'utf8')
  const standaloneInstallerRs = fs.readFileSync('src-tauri/src/commands/openclaw_standalone_installer.rs', 'utf8')
  const modRs = fs.readFileSync('src-tauri/src/commands/mod.rs', 'utf8')

  assert.match(modRs, /pub mod openclaw_standalone_installer;/)
  assert.match(standaloneInstallerRs, /pub\(crate\) async fn try_standalone_install\b/)
  assert.doesNotMatch(configRs, /\basync fn try_standalone_install\b/)
  assert.match(lifecycleRs, /openclaw_standalone_installer::try_standalone_install\(/)
  assert.match(standaloneInstallerRs, /openclaw_install_policy::standalone_config\(\)/)
  assert.match(standaloneInstallerRs, /openclaw_install_policy::versions_match\(/)
  assert.match(standaloneInstallerRs, /openclaw_install_runtime::standalone_platform_key\(\)/)
  assert.match(standaloneInstallerRs, /openclaw_install_runtime::standalone_archive_ext\(\)/)
  assert.match(standaloneInstallerRs, /openclaw_install_runtime::standalone_install_dir\(\)/)
})

test('installation lifecycle public contract stays stable during phase 5', () => {
  const configRs = fs.readFileSync('src-tauri/src/commands/config.rs', 'utf8')
  const lifecycleRs = fs.readFileSync('src-tauri/src/commands/openclaw_lifecycle.rs', 'utf8')
  const openclawVersionRs = fs.readFileSync('src-tauri/src/commands/openclaw_version.rs', 'utf8')
  const libRs = fs.readFileSync('src-tauri/src/lib.rs', 'utf8')
  const modRs = fs.readFileSync('src-tauri/src/commands/mod.rs', 'utf8')

  assert.match(modRs, /pub mod openclaw_lifecycle;/)
  assert.match(openclawVersionRs, /pub async fn list_openclaw_versions\b/)
  assert.match(lifecycleRs, /#\[tauri::command\]\s+pub async fn upgrade_openclaw\b/)
  assert.match(lifecycleRs, /#\[tauri::command\]\s+pub async fn uninstall_openclaw\b/)
  assert.match(lifecycleRs, /\basync fn upgrade_openclaw_inner\b/)
  assert.match(lifecycleRs, /\basync fn uninstall_openclaw_inner\b/)
  assert.doesNotMatch(configRs, /pub async fn upgrade_openclaw\b/)
  assert.doesNotMatch(configRs, /pub async fn uninstall_openclaw\b/)
  assert.doesNotMatch(configRs, /\basync fn upgrade_openclaw_inner\b/)
  assert.doesNotMatch(configRs, /\basync fn uninstall_openclaw_inner\b/)

  assert.match(libRs, /openclaw_version::list_openclaw_versions/)
  assert.match(libRs, /openclaw_lifecycle::upgrade_openclaw/)
  assert.match(libRs, /openclaw_lifecycle::uninstall_openclaw/)
  assert.doesNotMatch(libRs, /config::list_openclaw_versions/)
  assert.doesNotMatch(libRs, /config::upgrade_openclaw/)
  assert.doesNotMatch(libRs, /config::uninstall_openclaw/)

  for (const eventName of [
    'upgrade-log',
    'upgrade-progress',
    'upgrade-done',
    'upgrade-error',
  ]) {
    assert.match(`${configRs}\n${lifecycleRs}`, new RegExp(`"${eventName}"`))
  }
})

test('phase 5 masks sensitive data before logs and diagnostics reach the UI', () => {
  assert.equal(fs.existsSync('src-tauri/src/commands/secret_redaction.rs'), true)

  const modRs = fs.readFileSync('src-tauri/src/commands/mod.rs', 'utf8')
  const redactionRs = fs.readFileSync('src-tauri/src/commands/secret_redaction.rs', 'utf8')
  const logsRs = fs.readFileSync('src-tauri/src/commands/logs.rs', 'utf8')
  const diagnoseRs = fs.readFileSync('src-tauri/src/commands/diagnose.rs', 'utf8')
  const assistantRs = fs.readFileSync('src-tauri/src/commands/assistant.rs', 'utf8')

  assert.match(modRs, /pub mod secret_redaction;/)
  assert.match(redactionRs, /pub\(crate\) fn redact_secrets\b/)
  assert.match(redactionRs, /token|password|api\[_-\]\?key|secret/i)
  assert.match(logsRs, /secret_redaction::redact_secrets/)
  assert.match(diagnoseRs, /secret_redaction::redact_secrets/)
  assert.match(assistantRs, /secret_redaction::redact_secrets/)
})

test('phase 5 assistant unlimited mode still confirms dangerous tools', () => {
  const assistantJs = fs.readFileSync('src/pages/assistant.js', 'utf8')

  assert.match(assistantJs, /const DANGEROUS_TOOLS = new Set\(\[/)
  assert.match(
    assistantJs,
    /unlimited:\s*\{[^}]*confirmDanger:\s*true/
  )
  assert.match(
    assistantJs,
    /mode\.confirmDanger && DANGEROUS_TOOLS\.has\(toolName\)/
  )
})

test('phase 5 masks Hermes logs on read and download surfaces', () => {
  const hermesRs = fs.readFileSync('src-tauri/src/commands/hermes.rs', 'utf8')

  assert.match(hermesRs, /pub async fn hermes_logs_read\b[\s\S]*secret_redaction::redact_secrets/)
  assert.match(hermesRs, /pub async fn hermes_logs_download\b[\s\S]*secret_redaction::redact_secrets/)
})

test('git runtime commands move out of config.rs ownership', () => {
  assert.equal(fs.existsSync('src-tauri/src/commands/git_runtime.rs'), true)

  const configRs = fs.readFileSync('src-tauri/src/commands/config.rs', 'utf8')
  const gitRuntimeRs = fs.readFileSync('src-tauri/src/commands/git_runtime.rs', 'utf8')
  const lifecycleRs = fs.readFileSync('src-tauri/src/commands/openclaw_lifecycle.rs', 'utf8')
  const modRs = fs.readFileSync('src-tauri/src/commands/mod.rs', 'utf8')
  const libRs = fs.readFileSync('src-tauri/src/lib.rs', 'utf8')

  assert.match(modRs, /pub mod git_runtime;/)
  for (const name of [
    'check_git',
    'scan_git_paths',
    'auto_install_git',
    'configure_git_https',
  ]) {
    assert.match(gitRuntimeRs, new RegExp(`pub (?:async )?fn ${name}\\b`))
    assert.match(libRs, new RegExp(`git_runtime::${name}`))
    assert.doesNotMatch(configRs, new RegExp(`pub (?:async )?fn ${name}\\b`))
  }

  for (const name of [
    'configured_git_path',
    'find_git_path',
  ]) {
    assert.match(gitRuntimeRs, new RegExp(`\\b${name}\\b`))
    assert.doesNotMatch(configRs, new RegExp(`\\b${name}\\b`))
  }

  for (const name of [
    'git_executable',
    'configure_git_https_rules',
    'apply_git_install_env',
    'check_git_impl',
    'scan_git_paths_impl',
    'auto_install_git_impl',
    'configure_git_https_impl',
  ]) {
    assert.doesNotMatch(configRs, new RegExp(`\\b${name}\\b`))
  }

  for (const name of [
    'ensure_https_rewrites',
    'apply_install_env',
    'https_rewrite_rule_count',
  ]) {
    assert.match(gitRuntimeRs, new RegExp(`\\b${name}\\b`))
    assert.match(`${configRs}\n${lifecycleRs}`, new RegExp(`git_runtime::${name}\\(`))
  }
})
