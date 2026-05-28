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

test('Hermes dashboard and setup config surfaces are DeepAi-only', () => {
  const dashboard = read('src/engines/hermes/pages/dashboard.js')
  const setup = read('src/engines/hermes/pages/setup.js')
  const engineLocale = read('src/locales/modules/engine.js')

  assert.match(dashboard, /const DEFAULT_HERMES_MODEL = 'gpt-5\.5'/, 'dashboard should declare the shared Hermes default model')
  assert.match(setup, /const DEFAULT_HERMES_MODEL = 'gpt-5\.5'/, 'setup should declare the shared Hermes default model')

  for (const [file, text, apiKeyLabel] of [
    ['dashboard', dashboard, "t\\('engine\\.dashApiKey'\\)"],
    ['setup', setup, "t\\('engine\\.configApiKey'\\)"],
  ]) {
    assert.match(text, /QTCOOL\.baseUrl/, `${file} should use the shared DeepAi default API base URL`)
    assert.match(text, /QTCOOL\.site/, `${file} should link to the DeepAi site from the API key affordance`)
    assert.match(text, new RegExp(`${apiKeyLabel}[\\s\\S]*href="\\$\\{QTCOOL\\.site\\}"`), `${file} should render a DeepAi site link next to the API key field`)
    assert.doesNotMatch(text, /hermesProviderGroupIntl|hermesProviderGroupCn|hermesProviderGroupAggregator|hermesProviderGroupOAuth/, `${file} should not render the old multi-provider grouping UI`)
    assert.match(text, /matched\?\.id === 'deepai'\s*\?\s*'openai-api'\s*:\s*\(matched\?\.id \|\| 'custom'\)/, `${file} should persist the DeepAi-only preset through Hermes' native OpenAI API provider on save`)
  }

  assert.match(dashboard, /formBaseUrl\s*=\s*hermesConfig\.base_url\s*\|\|\s*QTCOOL\.baseUrl/, 'dashboard should seed DeepAi base URL when Hermes config does not define one')
  assert.match(dashboard, /let formModel = DEFAULT_HERMES_MODEL/, 'dashboard should seed the DeepAi default model before config loads')
  assert.match(dashboard, /formModel = hermesConfig\.model \|\| DEFAULT_HERMES_MODEL/, 'dashboard should retain the DeepAi default model when config.yaml omits model.default')
  assert.match(setup, /api\.hermesReadConfig\(\)/, 'setup should hydrate the wizard form from the saved Hermes config')
  assert.match(setup, /configDraft/, 'setup should track a draft Hermes config separate from the saved config')
  assert.match(setup, /t\('engine\.configFetchDraftOnly'\)/, 'setup should explain that fetching models does not mutate the installed Hermes config')
  assert.match(setup, /t\('engine\.configSaveWritesFiles'\)/, 'setup should explain that only saving writes to config.yaml and .env')
  assert.match(setup, /t\('engine\.configSavedNextStep'\)/, 'setup should guide the user to start Gateway after saving config')
  assert.match(setup, /value="\$\{esc\(configDraft\.baseUrl\)\}"/, 'setup should prefill the configuration form from the current Hermes draft base URL')
  assert.match(setup, /value="\$\{esc\(configDraft\.model\)\}"/, 'setup should prefill the configuration form from the current Hermes draft model')
  assert.match(engineLocale, /configFetchDraftOnly:/, 'engine locale should define the fetch-models draft-only helper copy')
  assert.match(engineLocale, /configSaveWritesFiles:/, 'engine locale should define the save-to-Hermes helper copy')
  assert.match(engineLocale, /configSavedNextStep:/, 'engine locale should define the post-save Gateway guidance copy')
})

test('OpenClaw models DeepAi entry uses production API links without check-in copy', () => {
  const modelsPage = read('src/pages/models.js')
  const modelPresets = read('src/lib/model-presets.js')
  const modelLocale = read('src/locales/modules/models.js')

  assert.doesNotMatch(modelsPage, /qtcoolCheckin/, 'models page should not render the daily check-in button')
  assert.doesNotMatch(modelsPage, /QTCOOL\.checkinUrl/, 'models page should not link to the old check-in URL')
  assert.doesNotMatch(modelsPage, /qtcoolDesc/, 'models page should not render the old free-credit promo sentence')
  assert.match(modelsPage, /href="\$\{QTCOOL\.site\}"[\s\S]*qtcoolMore/, 'learn-more link should remain wired to the DeepAi site')
  assert.match(modelsPage, /href="\$\{QTCOOL\.usageUrl\}"[\s\S]*qtcoolDashboard/, 'dashboard link should remain wired to the DeepAi console')
  assert.match(modelsPage, /providerKey\s*===\s*QTCOOL\.providerKey[\s\S]*p\.baseUrl\s*=\s*QTCOOL\.baseUrl/, 'saved DeepAi providers should migrate to the production API base URL')

  assert.match(modelPresets, /baseUrl:\s*'https:\/\/api\.deepai\.wang\/v1'/, 'DeepAi model fetch should use the production API base URL')
  assert.match(modelPresets, /site:\s*'https:\/\/api\.deepai\.wang\/'/, 'learn-more should point to the DeepAi site')
  assert.match(modelPresets, /usageUrl:\s*'https:\/\/api\.deepai\.wang\/console'/, 'user dashboard should point to the DeepAi console')

  for (const legacyCopy of ['每日签到', '签到页', '免费模型测试额度', '邀请好友', 'Daily check-in', 'check-in page']) {
    assert.doesNotMatch(modelLocale, new RegExp(legacyCopy), `models locale should not keep legacy promo copy: ${legacyCopy}`)
  }
})

test('OpenClaw models page hides generic provider creation entry', () => {
  const modelsPage = read('src/pages/models.js')
  const modelLocale = read('src/locales/modules/models.js')

  assert.doesNotMatch(modelsPage, /id="btn-add-provider"/, 'models page should not render the generic add-provider button')
  assert.doesNotMatch(modelsPage, /querySelector\('#btn-add-provider'\)/, 'models page should not bind the removed add-provider button')
  assert.doesNotMatch(modelsPage, /function\s+addProvider\(/, 'models page should not keep the generic add-provider modal implementation')
  assert.doesNotMatch(modelsPage, /models\.addProviderTitle/, 'models page should not reference the add-provider modal title')
  assert.doesNotMatch(modelLocale, /点击「\+ 添加服务商」|Click "\+ Add Provider"/, 'empty-state copy should not point to the removed add-provider action')
  assert.match(modelsPage, /id="btn-undo"/, 'models page should keep undo controls')
  assert.match(modelsPage, /id="btn-qtcool-oneclick"/, 'models page should keep the DeepAi model list action')
  assert.match(modelsPage, /function\s+editProvider\(/, 'models page should keep provider editing')
  assert.match(modelsPage, /function\s+addModel\(/, 'models page should keep model adding')
})

test('DeepAi system model ids use deepai while migrating legacy qtcool config', () => {
  const modelsPage = read('src/pages/models.js')
  const assistant = read('src/pages/assistant.js')

  assert.match(modelsPage, /replaceLegacyDeepAiModelId/, 'models page should normalize legacy qtcool model ids')
  assert.match(modelsPage, /legacyProviderKeys/, 'models page should read the DeepAi legacy provider key list')
  assert.match(modelsPage, /primary\s*=\s*replaceLegacyDeepAiModelId\(primary\)/, 'primary model should migrate legacy DeepAi ids')
  assert.match(modelsPage, /fallbacks\.map\(replaceLegacyDeepAiModelId\)/, 'fallback model chain should migrate legacy DeepAi ids')
  assert.match(modelsPage, /defaults\.models\s*=\s*migrateLegacyDeepAiModelMap\(defaults\.models\)/, 'default model metadata map should migrate legacy DeepAi ids')
  assert.doesNotMatch(assistant, new RegExp('models\\.providers\\.qtcool'), 'assistant sync should not write a new qtcool provider')
  assert.doesNotMatch(assistant, /'qtcool\/'\s*\+/, 'assistant sync should not write qtcool primary model ids')
  assert.match(assistant, /providers\[QTCOOL\.providerKey\]/, 'assistant sync should write the current DeepAi provider key')
  assert.match(assistant, /QTCOOL\.providerKey\s*\+\s*'\/'/, 'assistant sync should compose primary model ids from the DeepAi provider key')
})

test('OpenClaw config pages ignore detached route loads before surfacing errors', () => {
  const router = read('src/router.js')
  assert.match(router, /markRouteDisposed/, 'router should mark old route DOM as disposed before replacement')
  assert.match(router, /__agentdockRouteDisposed\s*=\s*true/, 'router should expose a disposed marker to async route loaders')

  for (const file of [
    'src/pages/models.js',
    'src/pages/gateway.js',
    'src/pages/communication.js',
  ]) {
    const text = read(file)
    assert.match(text, /__agentdockRouteDisposed/, `${file} should consult the route disposed marker`)
    assert.match(text, /if\s*\(\s*isRouteDisposed\(page\)\s*\)\s*return/, `${file} should ignore stale async load results after route switches`)
  }
})

test('gateway page imports the helper modules it calls during render and save', () => {
  const gateway = read('src/pages/gateway.js')

  assert.match(gateway, /from '\.\.\/lib\/term-tooltip\.js'[\s\S]*termHelpHtml[\s\S]*attachTermTooltips/, 'gateway page should import term tooltip helpers before rendering the auth token help affordance')
  assert.match(gateway, /from '\.\.\/lib\/config-schema\.js'[\s\S]*validateField/, 'gateway page should import validateField before save-time schema validation')
})

test('agents page ignores stale overlapping list reloads', () => {
  const agents = read('src/pages/agents.js')

  assert.match(agents, /state\._loadAgentsRequestId\s*=\s*\(state\._loadAgentsRequestId\s*\|\|\s*0\)\s*\+\s*1/, 'agents page should version each list reload request')
  assert.match(agents, /const requestId = state\._loadAgentsRequestId/, 'agents page should capture the active reload version')
  assert.match(agents, /if\s*\(\s*requestId\s*!==\s*state\._loadAgentsRequestId\s*\|\|\s*isRouteDisposed\(page\)\s*\)\s*return[\s\S]*renderAgents\(page,\s*state\)/, 'agents page should ignore stale or detached results before re-rendering the list')
})

test('agents list helpers normalize wrapped payloads across runtimes', () => {
  const compat = read('src/lib/api-compat.js')
  const agents = read('src/pages/agents.js')

  assert.match(compat, /export function normalizeAgentListPayload\(raw\)/, 'api compat should expose a shared agent-list payload normalizer')
  assert.match(compat, /Array\.isArray\(raw\)/, 'agent-list payload normalizer should accept bare arrays')
  assert.match(compat, /raw\?\.(items|agents)/, 'agent-list payload normalizer should unwrap wrapped agent-list payloads')
  assert.match(compat, /const list = normalizeAgentListPayload\(await api\.listAgents\(\)\)/, 'listAgentsCompat should normalize the raw list_agents payload before decorating agents')
  assert.match(agents, /normalizeAgentListPayload\(createdAgentsRaw\)/, 'agents page should normalize add_agent results before deciding whether it can render immediately')
})

test('agents page uses the add_agent result to refresh the list immediately', () => {
  const agents = read('src/pages/agents.js')

  assert.match(agents, /const createdAgentsRaw = await api\.addAgent\(/, 'agents page should capture the fresh add_agent payload')
  assert.match(agents, /const createdAgents = normalizeAgentListPayload\(createdAgentsRaw\)/, 'agents page should normalize add_agent results before updating local state')
  assert.match(agents, /if\s*\(\s*createdAgents\.length\s*\)\s*{[\s\S]*state\.agents\s*=\s*normalizeAgentList\(createdAgents\)/s, 'agents page should update local state from normalized add_agent data before any follow-up reload')
  assert.match(agents, /renderAgents\(page,\s*state\)[\s\S]*await loadAgents\(page,\s*state\)/, 'agents page should re-render immediately, then reconcile with an awaited reload')
})

test('agents page ignores async writes after route disposal', () => {
  const agents = read('src/pages/agents.js')

  assert.match(agents, /function isRouteDisposed\(page\)/, 'agents page should define a route-disposal guard')
  assert.match(agents, /if\s*\(\s*isRouteDisposed\(page\)\s*\)\s*return/, 'agents page should bail out before mutating detached route DOM')
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

test('main boot no longer registers the legacy xintian engine entry', () => {
  const main = read('src/main.js')

  assert.doesNotMatch(main, /import\s+xintianEngine\s+from\s+'\.\/engines\/xintian\/index\.js'/, 'main should not import the removed xintian engine')
  assert.doesNotMatch(main, /registerEngine\(xintianEngine\)/, 'main should not register the removed xintian engine')
  assert.doesNotMatch(main, /'\.\/engines\/xintian\/style\/xintian\.css'/, 'main should not load xintian-only styles')
})

test('Hermes installs include web dependencies so the api_server gateway can boot', () => {
  const setup = read('src/engines/hermes/pages/setup.js')
  const hermes = read('src-tauri/src/commands/hermes.rs')

  assert.match(setup, /installHermes\('uv-tool',\s*\['web'\]\)/, 'Hermes setup should explicitly install the web extra for dashboard/gateway support')
  assert.match(hermes, /let pkg = if extras\.is_empty\(\)\s*{\s*format!\("hermes-agent\[web\] @ \{\}", HERMES_GIT_URL\)/s, 'uv tool installs should default to the web extra when no explicit extras are supplied')
  assert.match(hermes, /let pkg = if extras\.is_empty\(\)\s*{\s*format!\("hermes-agent\[web\] @ \{\}", HERMES_GIT_URL\)/s, 'uv pip installs should default to the web extra when no explicit extras are supplied')
  assert.match(hermes, /aiohttp==3\.13\.3/, 'Hermes installs should explicitly add aiohttp because the upstream web extra does not include the api_server dependency')
})
