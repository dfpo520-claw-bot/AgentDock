/**
 * Hermes Agent 涓€閿畨瑁?閰嶇疆鍚戝
 *
 * 鐘舵€佹満: detect 鈫?install 鈫?configure 鈫?gateway 鈫?complete
 */
import { t } from '../../../lib/i18n.js'
import { api, invalidate, isTauriRuntime } from '../../../lib/tauri-api.js'
import { toast } from '../../../components/toast.js'
import { getActiveEngine } from '../../../lib/engine-manager.js'
import { QTCOOL } from '../../../lib/model-presets.js'
import { inferProviderByBaseUrl } from '../lib/providers.js'

// SVG 鍥炬爣
const ICONS = {
  check: `<svg viewBox="0 0 24 24" fill="none" stroke="var(--accent)" stroke-width="2.5" width="16" height="16"><polyline points="20 6 9 17 4 12"/></svg>`,
  warn: `<svg viewBox="0 0 24 24" fill="none" stroke="var(--warning, #f59e0b)" stroke-width="2" width="16" height="16"><path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>`,
  error: `<svg viewBox="0 0 24 24" fill="none" stroke="var(--error, #ef4444)" stroke-width="2" width="16" height="16"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>`,
  spinner: `<svg class="hermes-spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="16" height="16"><path d="M12 2a10 10 0 0110 10"/></svg>`,
  rocket: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="18" height="18"><path d="M4.5 16.5c-1.5 1.26-2 5-2 5s3.74-.5 5-2c.71-.84.7-2.13-.09-2.91a2.18 2.18 0 00-2.91-.09z"/><path d="M12 15l-3-3a22 22 0 012-3.95A12.88 12.88 0 0122 2c0 2.72-.78 7.5-6 11a22.35 22.35 0 01-4 2z"/><path d="M9 12H4s.55-3.03 2-4c1.62-1.08 5 0 5 0"/><path d="M12 15v5s3.03-.55 4-2c1.08-1.62 0-5 0-5"/></svg>`,
  done: `<svg viewBox="0 0 24 24" fill="none" stroke="var(--accent)" stroke-width="2" width="24" height="24"><path d="M22 11.08V12a10 10 0 11-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>`,
}

// 鏍稿績瀹夎涓嶅甫 extras锛屽悗缁彲鍦ㄧ鐞嗛〉闈㈡寜闇€瀹夎

// Provider 鏁版嵁 鈥?棣栨 render 鍓嶅紓姝ュ姞杞?
const DEEPAI_HERMES_PROVIDER = Object.freeze({
  id: 'deepai',
  name: QTCOOL.brandName,
  baseUrl: QTCOOL.baseUrl,
  site: QTCOOL.site,
})

const DEFAULT_HERMES_MODEL = 'gpt-5.5'

let hermesProviders = [DEEPAI_HERMES_PROVIDER]

export function render() {
  const el = document.createElement('div')
  el.className = 'page page-shell hermes-setup-page'
  el.dataset.engine = 'hermes'

  // 鐘舵€?
  let phase = 'detect' // detect | install | configure | gateway | complete
  let pyInfo = null
  let hermesInfo = null
  let logs = []
  let installing = false
  let installError = null
  let installMode = 'local' // 'local' | 'custom'
  let customGatewayUrl = 'http://127.0.0.1:8642'
  let progress = 0
  let unlisten = null
  let savedConfig = null
  let configDraft = createConfigDraft()
  let configSaving = false
  let configSavedForGateway = false

  function createConfigDraft(source = {}) {
    return {
      baseUrl: (source.base_url || source.baseUrl || QTCOOL.baseUrl || '').trim(),
      apiKey: (source.api_key || source.apiKey || '').trim(),
      model: (source.model || source.model_raw || source.modelName || DEFAULT_HERMES_MODEL || '').trim() || DEFAULT_HERMES_MODEL,
    }
  }

  function hydrateSavedConfig(source) {
    savedConfig = source || null
    configDraft = createConfigDraft(source || {})
  }

  function syncConfigDraftFromDom() {
    const baseUrlInput = el.querySelector('#hm-baseurl')
    const apiKeyInput = el.querySelector('#hm-apikey')
    const modelInput = el.querySelector('#hm-model')
    if (baseUrlInput) configDraft.baseUrl = baseUrlInput.value.trim()
    if (apiKeyInput) configDraft.apiKey = apiKeyInput.value.trim()
    if (modelInput) configDraft.model = modelInput.value.trim()
  }

  function isConfigDraftDirty() {
    const savedDraft = createConfigDraft(savedConfig || {})
    return configDraft.baseUrl !== savedDraft.baseUrl
      || configDraft.apiKey !== savedDraft.apiKey
      || configDraft.model !== savedDraft.model
  }

  function canSaveConfig() {
    const hasRequired = Boolean(configDraft.baseUrl && configDraft.apiKey && configDraft.model)
    if (!hasRequired || configSaving) return false
    return !savedConfig?.config_exists || isConfigDraftDirty()
  }

  function getSavedConfigSummary() {
    if (!savedConfig?.config_exists) return t('engine.configNotSavedYet')
    return t('engine.configSavedState', {
      model: savedConfig.model || DEFAULT_HERMES_MODEL,
      baseUrl: savedConfig.base_url || QTCOOL.baseUrl,
    })
  }

  function refreshConfigFormUi() {
    syncConfigDraftFromDom()
    const saveBtn = el.querySelector('.hermes-config-save')
    if (saveBtn) {
      saveBtn.disabled = !canSaveConfig()
      saveBtn.textContent = configSaving ? t('engine.configSaving') : t('engine.configSaveBtn')
    }
    const draftStatus = el.querySelector('#hm-config-draft-status')
    if (draftStatus) {
      const dirty = isConfigDraftDirty()
      draftStatus.textContent = dirty ? t('engine.configDraftPending') : t('engine.configDraftClean')
      draftStatus.style.color = dirty ? 'var(--warning, #f59e0b)' : 'var(--text-tertiary)'
    }
  }

  function draw() {
    el.innerHTML = `
      <div class="page-header">
        <h1>Hermes Agent</h1>
        <p style="color:var(--text-secondary);margin-top:4px">${t('engine.hermesSetupDesc')}</p>
      </div>
      <div style="max-width:720px">
        ${renderPhaseIndicator()}
        ${phase === 'detect' ? renderDetect() : ''}
        ${phase === 'install' ? renderInstall() : ''}
        ${phase === 'configure' ? renderConfigure() : ''}
        ${phase === 'gateway' ? renderGateway() : ''}
        ${phase === 'complete' ? renderComplete() : ''}
      </div>`
    bind()
  }

  // --- 闃舵鎸囩ず鍣?---
  function renderPhaseIndicator() {
    const phases = [
      { id: 'detect', label: t('engine.hermesPhaseDetect') },
      { id: 'install', label: t('engine.hermesPhaseInstall') },
      { id: 'configure', label: t('engine.hermesPhaseConfigure') },
      { id: 'gateway', label: t('engine.hermesPhaseGateway') },
      { id: 'complete', label: t('engine.hermesPhaseComplete') },
    ]
    const idx = phases.findIndex(p => p.id === phase)
    return `<div class="hermes-phases">${phases.map((p, i) => {
      const cls = i < idx ? 'done' : i === idx ? 'active' : ''
      const clickable = i < idx ? `data-goto-phase="${p.id}" style="cursor:pointer" title="${t('engine.hermesPhaseClickHint')}"` : ''
      return `<div class="hermes-phase ${cls}" ${clickable}>
        <span class="hermes-phase-dot">${i < idx ? ICONS.check : i + 1}</span>
        <span class="hermes-phase-label">${p.label}</span>
      </div>`
    }).join('<div class="hermes-phase-line"></div>')}</div>`
  }

  // --- 妫€娴嬮樁娈?---
  function renderDetect() {
    const rows = []
    if (!pyInfo && !hermesInfo) {
      rows.push(`<div class="hermes-detect-row">${ICONS.spinner} <span>${t('engine.detecting')}</span></div>`)
    } else {
      // Python
      if (pyInfo) {
        if (pyInfo.installed && pyInfo.versionOk) {
          rows.push(`<div class="hermes-detect-row ok">${ICONS.check} <span>${t('engine.pythonFound', { version: pyInfo.version })}</span></div>`)
        } else if (pyInfo.installed && !pyInfo.versionOk) {
          rows.push(`<div class="hermes-detect-row warn">${ICONS.warn} <span>${t('engine.pythonTooOld', { version: pyInfo.version })}</span></div>`)
        } else {
          rows.push(`<div class="hermes-detect-row warn">${ICONS.warn} <span>${t('engine.pythonNotFound')}</span></div>`)
        }
        // uv
        if (pyInfo.hasUv) {
          rows.push(`<div class="hermes-detect-row ok">${ICONS.check} <span>${t('engine.uvFound')}</span></div>`)
        } else {
          rows.push(`<div class="hermes-detect-row warn">${ICONS.warn} <span>${t('engine.uvNotFound')}</span></div>`)
        }
        // git锛堜粠 GitHub 瀹夎闇€瑕侊級
        if (pyInfo.hasGit) {
          rows.push(`<div class="hermes-detect-row ok">${ICONS.check} <span>${t('engine.gitFound')}</span></div>`)
        } else {
          rows.push(`<div class="hermes-detect-row warn">${ICONS.error} <span>${t('engine.gitNotFound')}</span></div>`)
        }
      }
      // Hermes
      if (hermesInfo) {
        if (hermesInfo.installed) {
          rows.push(`<div class="hermes-detect-row ok">${ICONS.check} <span>${t('engine.hermesFound', { version: hermesInfo.version })}</span></div>`)
          if (hermesInfo.gatewayRunning) {
            rows.push(`<div class="hermes-detect-row ok">${ICONS.check} <span>${t('engine.hermesReady')}</span></div>`)
          }
        } else {
          rows.push(`<div class="hermes-detect-row">${ICONS.warn} <span>${t('engine.hermesNotFound')}</span></div>`)
        }
      }
    }
    return `<div class="card" style="margin-bottom:16px">
      <div class="card-body" style="padding:24px">
        <p style="color:var(--text-secondary);line-height:1.7;margin:0 0 16px">${t('engine.hermesSetupIntro')}</p>
        <div class="hermes-detect-list">${rows.join('')}</div>
      </div>
    </div>`
  }

  // --- 瀹夎闃舵 ---
  function renderInstall() {
    // 妯″紡鍒囨崲鎸夐挳
    const modeSwitch = `
      <div style="display:flex;gap:8px;margin-bottom:20px">
        <button class="btn btn-sm hermes-mode-btn ${installMode === 'local' ? 'btn-primary' : 'btn-secondary'}" data-mode="local">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14" style="vertical-align:-2px"><rect x="2" y="3" width="20" height="14" rx="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/></svg>
          ${t('engine.installModeLocal')}
        </button>
        <button class="btn btn-sm hermes-mode-btn ${installMode === 'custom' ? 'btn-primary' : 'btn-secondary'}" data-mode="custom">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14" style="vertical-align:-2px"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 014 10 15.3 15.3 0 01-4 10 15.3 15.3 0 01-4-10 15.3 15.3 0 014-10z"/></svg>
          ${t('engine.installModeCustom')}
        </button>
      </div>`

    if (installMode === 'custom') {
      // 鑷畾涔夋ā寮忥細杈撳叆宸叉湁 Gateway 鍦板潃
      return `<div class="card" style="margin-bottom:16px">
        <div class="card-body" style="padding:24px">
          <h3 style="margin:0 0 4px;font-size:16px">${t('engine.installTitle')}</h3>
          <p style="color:var(--text-secondary);margin:0 0 16px;font-size:13px">${t('engine.installCustomDesc')}</p>
          ${modeSwitch}
          ${installError ? `
            <div style="margin-bottom:14px;padding:10px 14px;background:var(--error-bg, #fef2f2);border:1px solid var(--error, #ef4444);border-radius:var(--radius-sm,6px);font-size:13px;color:var(--error, #ef4444)">
              ${esc(installError)}
            </div>
          ` : ''}
          <div class="hermes-form">
            <label class="hermes-field">
              <span>Gateway URL</span>
              <input type="text" id="hm-custom-url" class="input" placeholder="http://127.0.0.1:8642" value="${esc(customGatewayUrl)}">
              <div style="font-size:11px;color:var(--text-tertiary);margin-top:4px">${t('engine.installCustomHint')}</div>
            </label>
          </div>
          <div style="display:flex;gap:10px;align-items:center;margin-top:16px">
            <button class="btn btn-primary hermes-custom-connect" ${installing ? 'disabled' : ''}>${installing ? ICONS.spinner + ' ' + t('engine.installCustomTesting') : t('engine.installCustomConnect')}</button>
          </div>
        </div>
      </div>`
    }

    // 鏈湴妯″紡锛氫竴閿畨瑁?
    const btnText = installing ? `${ICONS.spinner} ${t('engine.installingBtn')}` : `${ICONS.rocket} ${t('engine.installBtn')}`
    const btnDisabled = installing ? 'disabled' : ''

    // 閿欒鎻愮ず鍧?
    const errorBlock = installError ? `
      <div style="margin-bottom:14px;padding:12px 16px;background:var(--error-bg, #fef2f2);border:1px solid var(--error, #ef4444);border-radius:var(--radius-sm,6px);font-size:13px;line-height:1.6">
        <div style="display:flex;align-items:flex-start;gap:8px">
          ${ICONS.error}
          <div>
            <div style="font-weight:600;color:var(--error, #ef4444);margin-bottom:4px">${t('engine.installFailed')}</div>
            <div style="color:var(--text-secondary);word-break:break-all">${esc(installError)}</div>
          </div>
        </div>
      </div>
    ` : ''

    // 杩涘害 + 鏃ュ織鍖猴紙瀹夎涓垨瀹夎澶辫触鍚庨兘鏄剧ず锛?
    const hasLogs = installing || logs.length > 0
    const progressBlock = hasLogs ? `
      <div class="hermes-install-status">
        <div class="hermes-progress"><div class="hermes-progress-bar${installError ? ' error' : ''}" style="width:${progress}%"></div></div>
        <div style="display:flex;justify-content:space-between;align-items:center;margin-top:6px">
          <span class="hermes-progress-text" style="font-size:12px;color:${installError ? 'var(--error, #ef4444)' : 'var(--text-tertiary)'}">${installError ? t('engine.installFailed') : progress >= 100 ? t('engine.installSuccess') : t('engine.installingBtn')}</span>
          <span style="font-size:12px;color:var(--text-tertiary);font-family:monospace">${Math.min(progress, 100)}%</span>
        </div>
      </div>
      <div class="hermes-log-panel" style="margin-top:12px">
        <div class="hermes-log-content">${logs.map(l => `<div>${esc(l)}</div>`).join('')}</div>
      </div>
    ` : `
      <div class="hermes-install-info">
        <div class="hermes-detect-row" style="margin-bottom:6px">${ICONS.check} <span>${t('engine.installInfoUv')}</span></div>
        <div class="hermes-detect-row" style="margin-bottom:6px">${ICONS.check} <span>${t('engine.installInfoCore')}</span></div>
        <div class="hermes-detect-row" style="color:var(--text-tertiary)">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="16" height="16"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
          <span>${t('engine.installInfoExtrasLater')}</span>
        </div>
      </div>
    `

    return `<div class="card" style="margin-bottom:16px">
      <div class="card-body" style="padding:24px">
        <h3 style="margin:0 0 4px;font-size:16px">${t('engine.installTitle')}</h3>
        <p style="color:var(--text-secondary);margin:0 0 16px;font-size:13px">${t('engine.installDescSimple')}</p>
        ${modeSwitch}
        ${errorBlock}
        ${progressBlock}
        <div style="display:flex;gap:10px;align-items:center;margin-top:16px">
          <button class="btn btn-primary hermes-install-btn" ${btnDisabled}>${installError ? `${ICONS.rocket} ${t('engine.retryBtn')}` : btnText}</button>
        </div>
      </div>
    </div>`
  }

  // --- 閰嶇疆闃舵 ---
  function renderConfigure() {
    const draftDirty = isConfigDraftDirty()
    const saveDisabled = canSaveConfig() ? '' : 'disabled'
    const savedSummary = getSavedConfigSummary()
    return `<div class="card" style="margin-bottom:16px">
      <div class="card-body" style="padding:24px">
        <h3 style="margin:0 0 4px;font-size:16px">${t('engine.configTitle')}</h3>
        <p style="color:var(--text-secondary);margin:0 0 20px;font-size:13px">${t('engine.configDesc')}</p>

        <div class="hermes-form">
          <div class="hermes-field">
            <span>${t('engine.configProvider')}</span>
            <div style="display:flex;flex-wrap:wrap;gap:8px;margin-top:8px">
              <button class="btn btn-sm btn-secondary hermes-preset-btn" data-key="${DEEPAI_HERMES_PROVIDER.id}" data-url="${DEEPAI_HERMES_PROVIDER.baseUrl}" data-api="${QTCOOL.api}" style="font-size:12px;padding:3px 10px;opacity:1">
                ${DEEPAI_HERMES_PROVIDER.name}
              </button>
            </div>
          </div>
          <label class="hermes-field">
            <span>${t('engine.configBaseUrl')}</span>
            <input type="text" id="hm-baseurl" class="input" value="${esc(configDraft.baseUrl)}" placeholder="${QTCOOL.baseUrl}">
          </label>
          <div class="hermes-field">
            <span style="display:flex;align-items:center;justify-content:space-between;gap:10px">
              <span>${t('engine.configApiKey')}</span>
              <a href="${QTCOOL.site}" target="_blank" rel="noreferrer">${t('engine.configGetApiKey')}</a>
            </span>
            <div style="display:flex;gap:8px;align-items:center">
              <input type="password" id="hm-apikey" class="input" value="${esc(configDraft.apiKey)}" placeholder="sk-..." autocomplete="off" style="flex:1">
              <button class="btn btn-sm btn-secondary hermes-fetch-models" style="white-space:nowrap;flex-shrink:0">${t('engine.configFetchModels')}</button>
            </div>
          </div>
          <div id="hm-fetch-result" style="font-size:12px;min-height:16px;margin:-6px 0 2px"></div>
          <div class="hermes-field">
            <span>${t('engine.configModel')}</span>
            <div style="position:relative">
              <input type="text" id="hm-model" class="input" value="${esc(configDraft.model)}" placeholder="${DEFAULT_HERMES_MODEL}" autocomplete="off">
              <div id="hm-model-dropdown" class="hermes-model-dropdown" style="display:none"></div>
            </div>
          </div>
          <div style="margin-top:14px;padding:12px 14px;background:var(--bg-tertiary);border:1px solid var(--border-primary);border-radius:var(--radius-sm,6px);font-size:12px;line-height:1.6;color:var(--text-secondary)">
            <div>${t('engine.configFetchDraftOnly')}</div>
            <div style="margin-top:6px">${t('engine.configSaveWritesFiles')}</div>
          </div>
          <div style="margin-top:12px;padding:12px 14px;background:var(--bg-secondary);border:1px solid var(--border-primary);border-radius:var(--radius-sm,6px);font-size:12px;line-height:1.7">
            <div style="color:var(--text-secondary)">${savedSummary}</div>
            <div id="hm-config-draft-status" style="margin-top:6px;color:${draftDirty ? 'var(--warning, #f59e0b)' : 'var(--text-tertiary)'}">${draftDirty ? t('engine.configDraftPending') : t('engine.configDraftClean')}</div>
          </div>
        </div>

        <div style="display:flex;gap:10px;margin-top:20px">
          <button class="btn btn-primary hermes-config-save" ${saveDisabled}>${configSaving ? t('engine.configSaving') : t('engine.configSaveBtn')}</button>
          <button class="btn-text hermes-config-skip">${t('engine.configSkipBtn')}</button>
        </div>
      </div>
    </div>`
  }

  // --- Gateway 闃舵 ---
  function renderGateway() {
    const running = hermesInfo?.gatewayRunning
    return `<div class="card" style="margin-bottom:16px">
      <div class="card-body" style="padding:24px">
        <h3 style="margin:0 0 4px;font-size:16px">${t('engine.gatewayTitle')}</h3>
        <p style="color:var(--text-secondary);margin:0 0 20px;font-size:13px">${t('engine.gatewayDesc')}</p>
        ${configSavedForGateway ? `
          <div class="hermes-detect-row ok" style="margin-bottom:12px">
            ${ICONS.check}
            <span>${t('engine.configSavedNextStep')}</span>
          </div>
        ` : ''}
        <div class="hermes-detect-row ${running ? 'ok' : ''}">
          ${running ? ICONS.check : ICONS.warn}
          <span>${running ? t('engine.gatewayRunning', { port: hermesInfo?.gatewayPort || 8642 }) : t('engine.gatewayStopped')}</span>
        </div>
        <div id="hm-gw-error" style="display:none;margin-top:12px;padding:10px 14px;background:var(--error-bg, #fef2f2);border:1px solid var(--error, #ef4444);border-radius:var(--radius-sm,6px);color:var(--error, #ef4444);font-size:13px;line-height:1.5;word-break:break-all"></div>
        <div style="display:flex;gap:10px;margin-top:16px">
          ${!running ? `<button class="btn btn-primary hermes-gw-start">${t('engine.gatewayStartBtn')}</button>` : ''}
          <button class="btn btn-primary hermes-gw-next">${running ? t('engine.goToDashboard') : t('engine.configSkipBtn')}</button>
        </div>
      </div>
    </div>`
  }

  // --- 瀹屾垚 ---
  function renderComplete() {
    return `<div class="card" style="margin-bottom:16px">
      <div class="card-body" style="padding:32px;text-align:center">
        <div style="margin-bottom:12px">${ICONS.done}</div>
        <h3 style="margin:0 0 6px;font-size:18px">${t('engine.setupComplete')}</h3>
        <p style="color:var(--text-secondary);margin:0 0 20px">${t('engine.setupCompleteDesc')}</p>
        <button class="btn btn-primary hermes-go-dashboard">${t('engine.goToDashboard')}</button>
      </div>
    </div>`
  }

  function esc(s) {
    return (s || '').replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
  }

  // --- 浜嬩欢缁戝畾 ---
  function bind() {
    // 鐐瑰嚮宸插畬鎴愮殑闃舵鎸囩ず鍣紝璺冲洖璇ยูն楠?
    el.querySelectorAll('[data-goto-phase]').forEach(dot => {
      dot.addEventListener('click', () => {
        phase = dot.dataset.gotoPhase
        draw()
      })
    })
    // 瀹夎妯″紡鍒囨崲
    el.querySelectorAll('.hermes-mode-btn').forEach(btn => {
      btn.addEventListener('click', () => {
        const mode = btn.dataset.mode
        if (mode && mode !== installMode) {
          installMode = mode
          installError = null
          draw()
        }
      })
    })
    // 瀹夎鎸夐挳锛堟湰鍦版ā寮忥級
    el.querySelector('.hermes-install-btn')?.addEventListener('click', doInstall)
    // 鑷畾涔夎繛鎺ユ寜閽?
    el.querySelector('.hermes-custom-connect')?.addEventListener('click', doCustomConnect)
    // 鏈嶅姟鍟嗛璁炬寜閽?
    el.querySelectorAll('.hermes-preset-btn').forEach(btn => {
      btn.addEventListener('click', () => {
        const baseUrlInput = el.querySelector('#hm-baseurl')
        if (baseUrlInput) baseUrlInput.value = btn.dataset.url
        refreshConfigFormUi()
      })
    })
    for (const selector of ['#hm-baseurl', '#hm-apikey', '#hm-model']) {
      el.querySelector(selector)?.addEventListener('input', () => {
        configSavedForGateway = false
        refreshConfigFormUi()
      })
    }
    // 鑾峰彇妯″瀷鍒楄〃
    el.querySelector('.hermes-fetch-models')?.addEventListener('click', doFetchModels)
    // 妯″瀷涓嬫媺閫夋嫨锛氱偣鍑婚€夐」濉叆 input
    el.querySelector('#hm-model-dropdown')?.addEventListener('click', (e) => {
      const opt = e.target.closest('.hermes-model-option')
      if (!opt) return
      const modelInput = el.querySelector('#hm-model')
      if (modelInput) modelInput.value = opt.dataset.model
      el.querySelector('#hm-model-dropdown').style.display = 'none'
      refreshConfigFormUi()
    })
    // 鐐瑰嚮 input 鏃跺鏋滄湁涓嬫媺灏卞睍寮€
    el.querySelector('#hm-model')?.addEventListener('focus', () => {
      const dd = el.querySelector('#hm-model-dropdown')
      if (dd && dd.children.length > 0) dd.style.display = 'block'
    })
    // 鐐瑰嚮鍏朵粬鍦版柟鍏抽棴涓嬫媺
    document.addEventListener('click', (e) => {
      const dd = el.querySelector('#hm-model-dropdown')
      if (dd && !e.target.closest('.hermes-field')) dd.style.display = 'none'
    })
    // 閰嶇疆淇濆瓨
    el.querySelector('.hermes-config-save')?.addEventListener('click', doSaveConfig)
    el.querySelector('.hermes-config-skip')?.addEventListener('click', () => { phase = 'gateway'; refreshHermes() })
    // Gateway
    el.querySelector('.hermes-gw-start')?.addEventListener('click', doStartGateway)
    el.querySelector('.hermes-gw-next')?.addEventListener('click', () => {
      if (hermesInfo?.gatewayRunning) { phase = 'complete'; draw() }
      else { phase = 'complete'; draw() }
    })
    // 浠〃鐩?
    el.querySelector('.hermes-go-dashboard')?.addEventListener('click', async () => {
      const engine = getActiveEngine()
      if (engine?.detect) await engine.detect()
      window.location.hash = '#/h/dashboard'
    })
    // 鑷姩婊氭棩蹇楀埌搴?
    const logEl = el.querySelector('.hermes-log-content')
    if (logEl) logEl.scrollTop = logEl.scrollHeight
    refreshConfigFormUi()
  }

  // --- 妫€娴嬫祦绋?---
  async function detect() {
    phase = 'detect'
    draw()
    try {
      invalidate('check_hermes', 'check_python')
      const [py, hm, cfg] = await Promise.all([
        api.checkPython(),
        api.checkHermes(),
        api.hermesReadConfig().catch(() => null),
      ])
      pyInfo = py
      hermesInfo = hm
      hydrateSavedConfig(cfg)

      draw()

      // 鑷姩璺宠浆鍒版渶鍚堥€傜殑闃舵锛堜笉鑷姩绂诲紑鍚戝锛岃鐢ㄦ埛鍙互鏌ョ湅鍜屽洖閫€姣忎竴姝ワ級
      await new Promise(r => setTimeout(r, 800))
      if (hm.installed && hm.gatewayRunning) {
        phase = 'complete'
      } else if (hm.installed && hm.configExists) {
        phase = 'gateway'
      } else if (hm.installed) {
        phase = 'configure'
      } else {
        phase = 'install'
      }
      draw()
    } catch (e) {
      configSaving = false
      draw()
      logs.push(`[detect error] ${e?.message || e}`)
      phase = 'install'
      draw()
    }
  }

  // --- 鑷畾涔夎繛鎺ユ祦绋?---
  async function doCustomConnect() {
    const urlInput = el.querySelector('#hm-custom-url')
    const url = urlInput?.value?.trim()
    if (!url) { installError = t('engine.installCustomEmpty'); draw(); return }

    // 鍩虹 URL 鏍煎紡妫€鏌?
    try { new URL(url) } catch { installError = t('engine.installCustomInvalidUrl'); draw(); return }

    installing = true
    installError = null
    draw()

    try {
      // 淇濆瓨 Gateway URL
      await api.hermesSetGatewayUrl(url)

      // 娴嬭瘯杩炴帴
      const health = await api.hermesHealthCheck()
      if (!health) throw new Error(t('engine.installCustomNoResponse'))

      installing = false
      customGatewayUrl = url
      // 杩炴帴鎴愬姛锛岃烦鍒伴厤缃楠?
      phase = 'configure'
      draw()
    } catch (e) {
      installing = false
      installError = t('engine.installCustomFailed', { error: e.message || e })
      draw()
    }
  }

  // --- 瀹夎娴佺▼ ---
  async function doInstall() {
    installing = true
    installError = null
    progress = 0
    logs = []
    draw()

    // 鐩戝惉瀹夎浜嬩欢锛沇eb 妯″紡璺宠繃妗岄潰浜嬩欢鐩戝惉銆?
    try {
      if (!isTauriRuntime()) throw new Error('skip-listen-in-web-mode')
      const { listen } = await import('@tauri-apps/api/event')
      const u1 = await listen('hermes-install-log', (e) => {
        const line = String(e.payload)
        logs.push(line)
        const logEl = el.querySelector('.hermes-log-content')
        if (logEl) {
          const div = document.createElement('div')
          div.textContent = line
          logEl.appendChild(div)
          logEl.scrollTop = logEl.scrollHeight
        }
      })
      const u2 = await listen('hermes-install-progress', (e) => {
        progress = Number(e.payload) || 0
        const bar = el.querySelector('.hermes-progress-bar')
        if (bar) bar.style.width = progress + '%'
        const pctEl = el.querySelector('.hermes-progress-text')
        if (pctEl) pctEl.textContent = progress >= 100 ? t('engine.installSuccess') : t('engine.installingBtn')
        // 鏇存柊鐧惧垎姣旀暟瀛?
        const pctNum = bar?.parentElement?.nextElementSibling?.querySelector('span:last-child')
        if (pctNum) pctNum.textContent = Math.min(progress, 100) + '%'
      })
      unlisten = () => { u1(); u2() }
    } catch (_) {}

    try {
      await api.installHermes('uv-tool', ['web'])
      installing = false
      progress = 100
      logs.push('鉁?' + t('engine.installSuccess'))
      phase = 'configure'
      draw()
    } catch (e) {
      installing = false
      installError = String(e.message || e)
      logs.push(`鉂?${t('engine.installFailed')}: ${e}`)
      draw()
    } finally {
      if (unlisten) { unlisten(); unlisten = null }
    }
  }

  // --- 鑾峰彇妯″瀷鍒楄〃 ---
  async function doFetchModels() {
    const btn = el.querySelector('.hermes-fetch-models')
    const resultEl = el.querySelector('#hm-fetch-result')
    const dropdown = el.querySelector('#hm-model-dropdown')
    syncConfigDraftFromDom()
    const baseUrl = el.querySelector('#hm-baseurl')?.value?.trim()
    const apiKey = el.querySelector('#hm-apikey')?.value?.trim()

    if (!baseUrl) {
      if (resultEl) resultEl.innerHTML = `<span style="color:var(--warning)">${t('engine.configFetchNeedUrl')}</span>`
      return
    }
    if (!apiKey) {
      if (resultEl) resultEl.innerHTML = `<span style="color:var(--warning)">${t('engine.configFetchNeedKey')}</span>`
      return
    }

    if (btn) { btn.disabled = true; btn.textContent = t('engine.configFetching') }
    if (resultEl) resultEl.innerHTML = `<span style="color:var(--text-tertiary)">${t('engine.configFetching')}</span>`

    try {
      const models = await api.hermesFetchModels(baseUrl, apiKey, QTCOOL.api, 'custom')

      if (models.length === 0) {
        if (resultEl) resultEl.innerHTML = `<span style="color:var(--warning)">${t('engine.configFetchNotSupported')}</span>`
        return
      }

      if (resultEl) resultEl.innerHTML = `<span style="color:var(--success)">鉁?${t('engine.configFetchSuccess', { count: models.length })}</span>`
      if (dropdown) {
        dropdown.innerHTML = models.map(m =>
          `<div class="hermes-model-option" data-model="${m}" style="padding:6px 12px;cursor:pointer;font-size:13px;border-bottom:1px solid var(--border-primary)">${m}</div>`
        ).join('')
        dropdown.style.display = 'block'
      }
    } catch (err) {
      // 缃戠粶閿欒鎴栦笉鏀寔
      const msg = err.message || String(err)
      if (resultEl) {
        if (msg.includes('403') || msg.includes('404') || msg.includes('405') || msg.includes('timeout') || msg.includes('Failed to fetch')) {
          resultEl.innerHTML = `<span style="color:var(--warning)">${t('engine.configFetchNotSupported')}</span>`
        } else {
          resultEl.innerHTML = `<span style="color:var(--error)">鉁?${t('engine.configFetchFailed', { error: msg })}</span>`
        }
      }
    } finally {
      if (btn) { btn.disabled = false; btn.textContent = t('engine.configFetchModels') }
    }
  }

  // --- 閰嶇疆淇濆瓨 ---
  async function doSaveConfig() {
    syncConfigDraftFromDom()
    const baseUrl = configDraft.baseUrl
    const apiKey = configDraft.apiKey
    const model = configDraft.model
    // Persist the DeepAi-only preset through Hermes' native openai-api path so
    // runtime auth resolves against OPENAI_API_KEY instead of the custom key.
    // 浠?baseUrl 鎺ㄦ柇 provider id锛涙帹涓嶅嚭鏉ユ椂鐢?'custom'锛岃鍚庣鎸夐€氱敤 OpenAI 鍏煎澶勭悊
    const matched = inferProviderByBaseUrl(hermesProviders, baseUrl)
    const provider = matched?.id === 'deepai' ? 'openai-api' : (matched?.id || 'custom')

    if (!apiKey) {
      toast(t('engine.installCustomEmpty') || '璇疯緭鍏?API Key', 'warning')
      return
    }
    try {
      configSaving = true
      draw()
      await api.configureHermes(provider, apiKey, model, baseUrl)
      const latestConfig = await api.hermesReadConfig().catch(() => null)
      hydrateSavedConfig(latestConfig || {
        base_url: baseUrl,
        api_key: apiKey,
        model,
        config_exists: true,
      })
      configSaving = false
      configSavedForGateway = true
      toast(t('engine.configSaved'), 'success')
      phase = 'gateway'
      await refreshHermes()
    } catch (e) {
      configSaving = false
      draw()
      const msg = String(e?.message || e).replace(/^Error:\s*/, '')
      toast(`${t('engine.configSaveFailed') || '閰嶇疆淇濆瓨澶辫触'}: ${msg}`, 'error')
    }
  }

  // --- Gateway 鍚姩 ---
  let gwStarting = false
  async function doStartGateway() {
    const btn = el.querySelector('.hermes-gw-start')
    if (btn) { btn.disabled = true; btn.textContent = t('engine.gatewayStarting') }
    gwStarting = true
    try {
      await api.hermesGatewayAction('start')
      await refreshHermes()
    } catch (e) {
      const msg = String(e).replace(/^Error:\s*/, '')
      // 鍦?Gateway 闃舵鏄剧ず閿欒淇℃伅
      const errEl = el.querySelector('#hm-gw-error')
      if (errEl) {
        errEl.textContent = msg || t('engine.gatewayStartFailed')
        errEl.style.display = 'block'
      } else {
        toast(msg || t('engine.gatewayStartFailed'), 'error')
      }
    } finally {
      gwStarting = false
      if (btn) { btn.disabled = false; btn.textContent = t('engine.gatewayStartBtn') }
    }
  }

  // --- 鍒锋柊 hermes 鐘舵€?---
  async function refreshHermes() {
    invalidate('check_hermes')
    try { hermesInfo = await api.checkHermes() } catch (_) {}
    // 宸插畨瑁呬笖 Gateway 鍦ㄨ繍琛?鈫?鏇存柊寮曟搸鐘舵€佸苟璺宠浆浠〃鐩?
    if (hermesInfo?.installed && hermesInfo?.gatewayRunning) {
      phase = 'complete'
      const engine = getActiveEngine()
      if (engine?.detect) await engine.detect()
      window.location.hash = '#/h/dashboard'
      return
    }
    draw()
  }

  // 鍚姩妫€娴嬪墠鍏堝姞杞?provider registry锛岀劧鍚庡惎鍔ㄦ娴?
  ;(async () => {
    detect()
  })()

  return el
}

// ============================================================================
// Helper: render the grouped provider buttons shown in renderConfigure()
// ============================================================================

function renderGroupedProviderButtons() {
  return ''

  if (hermesGroups.oauth.length) {
    const oauthItems = hermesGroups.oauth.map(p =>
      `<div style="font-size:11px;color:var(--text-tertiary);margin-right:10px"><code>${p.name}</code>锛?{t('engine.hermesProviderOAuthRunHint') || '闇€杩愯'} <code>${p.cliAuthHint}</code></div>`
    ).join('')
    parts.push(`<div style="${sectionStyle}"><div style="${titleStyle}">OAuth</div><div style="display:flex;flex-wrap:wrap;gap:4px 0">${oauthItems}</div></div>`)
  }

  return parts.join('')
}

