export function createGatewayBannerController({
  api,
  toast,
  t,
  escapeHtml,
  statusIcon,
  isGatewayRunning,
  isGatewayForeign,
  onGatewayChange,
  onEngineChange,
  getActiveEngineId,
  resetAutoRestart,
  isForeignGatewayError,
  showGatewayConflictGuidance,
  refreshGatewayStatus,
}) {
  async function openGatewayConflict(error = null) {
    const services = await api.getServicesStatus().catch(() => [])
    const gw = services?.find?.(s => s.label === 'ai.openclaw.gateway') || services?.[0] || null
    await showGatewayConflictGuidance({ error, service: gw })
  }

  function setupGatewayBanner() {
    const banner = document.getElementById('gw-banner')
    if (!banner) return

    function update(running, foreign) {
      if (getActiveEngineId() !== 'openclaw') {
        banner.classList.add('gw-banner-hidden')
        return
      }
      if (running || sessionStorage.getItem('gw-banner-dismissed')) {
        banner.classList.add('gw-banner-hidden')
        return
      }
      banner.classList.remove('gw-banner-hidden')

      if (foreign) {
        banner.innerHTML = `
          <div class="gw-banner-content">
            <span class="gw-banner-icon">${statusIcon('warning', 16)}</span>
            <span>${t('dashboard.foreignGatewayBanner')}</span>
            <button class="btn btn-sm btn-secondary" id="btn-gw-claim" style="margin-left:auto">${t('dashboard.claimGateway')}</button>
            <a class="btn btn-sm btn-ghost" href="#/services">${t('sidebar.services')}</a>
            <button class="gw-banner-close" id="btn-gw-dismiss" title="${t('common.close')}">&times;</button>
          </div>
        `
        banner.querySelector('#btn-gw-dismiss')?.addEventListener('click', () => {
          banner.classList.add('gw-banner-hidden')
          sessionStorage.setItem('gw-banner-dismissed', '1')
        })
        banner.querySelector('#btn-gw-claim')?.addEventListener('click', async (e) => {
          const btn = e.target
          btn.disabled = true
          btn.textContent = t('common.processing')
          try {
            await api.claimGateway()
            await refreshGatewayStatus()
          } catch (err) {
            btn.disabled = false
            btn.textContent = t('dashboard.claimGateway')
            console.error('[banner] claim failed:', err)
          }
        })
        return
      }

      banner.innerHTML = `
        <div class="gw-banner-content">
          <span class="gw-banner-icon">${statusIcon('info', 16)}</span>
          <span>${t('dashboard.controlUINotRunning')}</span>
          <button class="btn btn-sm btn-secondary" id="btn-gw-start" style="margin-left:auto">${t('dashboard.startBtn')}</button>
          <a class="btn btn-sm btn-ghost" href="#/services">${t('sidebar.services')}</a>
          <button class="gw-banner-close" id="btn-gw-dismiss" title="${t('common.close')}">&times;</button>
        </div>
      `
      banner.querySelector('#btn-gw-dismiss')?.addEventListener('click', () => {
        banner.classList.add('gw-banner-hidden')
        sessionStorage.setItem('gw-banner-dismissed', '1')
      })
      banner.querySelector('#btn-gw-start')?.addEventListener('click', async (e) => {
        const btn = e.target
        btn.disabled = true
        btn.classList.add('btn-loading')
        btn.textContent = t('dashboard.starting')
        try {
          await api.startService('ai.openclaw.gateway')
        } catch (err) {
          if (isForeignGatewayError(err)) {
            await openGatewayConflict(err)
            update(false)
            return
          }
          const errMsg = (err.message || String(err)).slice(0, 120)
          banner.innerHTML = `
            <div class="gw-banner-content" style="flex-wrap:wrap">
              <span class="gw-banner-icon">${statusIcon('info', 16)}</span>
              <span>${t('dashboard.startFail')}</span>
              <button class="btn btn-sm btn-secondary" id="btn-gw-start" style="margin-left:auto">${t('dashboard.retry')}</button>
              <a class="btn btn-sm btn-ghost" href="#/services">${t('sidebar.services')}</a>
              <a class="btn btn-sm btn-ghost" href="#/logs">${t('sidebar.logs')}</a>
            </div>
            <div style="font-size:11px;opacity:0.7;margin-top:4px;font-family:monospace;word-break:break-all">${escapeHtml(errMsg)}</div>
          `
          update(false)
          return
        }
        const t0 = Date.now()
        while (Date.now() - t0 < 30000) {
          try {
            const s = await api.getServicesStatus()
            const gw = s?.find?.(x => x.label === 'ai.openclaw.gateway') || s?.[0]
            if (gw?.running) {
              update(true)
              return
            }
          } catch {}
          const sec = Math.floor((Date.now() - t0) / 1000)
          btn.textContent = `${t('dashboard.starting')} ${sec}s`
          await new Promise(resolve => setTimeout(resolve, 1500))
        }
        let logHint = ''
        try {
          const logs = await api.readLogTail('gateway', 5)
          if (logs?.trim()) {
            logHint = `<div style="font-size:12px;margin-top:4px;opacity:0.8;font-family:monospace;white-space:pre-wrap">${logs.trim().split('\n').slice(-3).join('\n')}</div>`
          }
        } catch {}
        banner.innerHTML = `
          <div class="gw-banner-content">
            <span class="gw-banner-icon">${statusIcon('info', 16)}</span>
            <span>${t('dashboard.startTimeout')}</span>
            <button class="btn btn-sm btn-secondary" id="btn-gw-start" style="margin-left:auto">${t('dashboard.retry')}</button>
            <a class="btn btn-sm btn-ghost" href="#/logs">${t('sidebar.logs')}</a>
          </div>
          ${logHint}
        `
        update(false)
      })
    }

    update(isGatewayRunning(), isGatewayForeign())
    onGatewayChange(update)
    onEngineChange(() => update(isGatewayRunning(), isGatewayForeign()))
  }

  function showGuardianRecovery() {
    const banner = document.getElementById('gw-banner')
    if (!banner) return
    banner.classList.remove('gw-banner-hidden')
    banner.innerHTML = `
      <div class="gw-banner-content" style="flex-wrap:wrap;gap:8px">
        <span class="gw-banner-icon">${statusIcon('warn', 16)}</span>
        <span>${t('dashboard.guardianFailed')}</span>
        <button class="btn btn-sm btn-primary" id="btn-gw-recover-fix" style="margin-left:auto">${t('dashboard.autoFix')}</button>
        <button class="btn btn-sm btn-secondary" id="btn-gw-recover-restart">${t('dashboard.retryStart')}</button>
        <a class="btn btn-sm btn-ghost" href="#/logs">${t('sidebar.logs')}</a>
      </div>
    `
    banner.querySelector('#btn-gw-recover-fix')?.addEventListener('click', async (e) => {
      const btn = e.target
      btn.disabled = true
      btn.textContent = t('dashboard.fixing')
      const overlay = document.createElement('div')
      overlay.className = 'modal-overlay'
      overlay.innerHTML = `
        <div class="modal" style="max-width:560px">
          <div class="modal-title">${t('dashboard.fixModalTitle')}</div>
          <div style="font-size:var(--font-size-sm);color:var(--text-secondary);margin-bottom:12px">
            ${t('dashboard.fixModalDesc')}
          </div>
          <div id="fix-log" style="font-family:var(--font-mono);font-size:11px;background:var(--bg-tertiary);padding:12px;border-radius:var(--radius-md);max-height:300px;overflow-y:auto;white-space:pre-wrap;line-height:1.6;color:var(--text-secondary)">${t('dashboard.fixRunning')}\n</div>
          <div id="fix-status" style="margin-top:12px;font-size:var(--font-size-sm);font-weight:600"></div>
          <div class="modal-actions" style="margin-top:16px">
            <button class="btn btn-secondary btn-sm" id="fix-close" style="display:none">${t('common.close')}</button>
          </div>
        </div>
      `
      document.body.appendChild(overlay)
      const logEl = overlay.querySelector('#fix-log')
      const statusEl = overlay.querySelector('#fix-status')
      const closeBtn = overlay.querySelector('#fix-close')
      closeBtn.onclick = () => overlay.remove()

      try {
        const result = await api.doctorFix()
        const output = result?.stdout || result?.output || JSON.stringify(result, null, 2)
        logEl.textContent = output || t('dashboard.fixDoneNoOutput')
        logEl.scrollTop = logEl.scrollHeight
        if (result?.errors) {
          statusEl.innerHTML = `<span style="color:var(--warning)">${t('dashboard.fixDoneWarning')}${escapeHtml(String(result.errors).slice(0, 200))}</span>`
        } else {
          statusEl.innerHTML = `<span style="color:var(--success)">${t('dashboard.fixDoneRestarting')}</span>`
          resetAutoRestart()
          try {
            await api.startService('ai.openclaw.gateway')
            statusEl.innerHTML = `<span style="color:var(--success)">${t('dashboard.fixDoneRestarted')}</span>`
          } catch (err) {
            if (isForeignGatewayError(err)) await openGatewayConflict(err)
            statusEl.innerHTML = `<span style="color:var(--warning)">${t('dashboard.fixDoneRestartFail')}</span>`
          }
        }
      } catch (err) {
        logEl.textContent += '\n' + (err.message || String(err))
        statusEl.innerHTML = `<span style="color:var(--error)">${t('dashboard.fixFailed')}${escapeHtml(String(err.message || err).slice(0, 200))}</span>`
      }
      closeBtn.style.display = ''
      btn.textContent = t('dashboard.autoFix')
      btn.disabled = false
    })
    banner.querySelector('#btn-gw-recover-restart')?.addEventListener('click', async (e) => {
      const btn = e.target
      btn.disabled = true
      btn.textContent = t('dashboard.fixing')
      resetAutoRestart()
      try {
        await api.startService('ai.openclaw.gateway')
        btn.textContent = t('dashboard.startSent')
      } catch (err) {
        if (isForeignGatewayError(err)) await openGatewayConflict(err)
        btn.textContent = t('dashboard.retryStart')
        btn.disabled = false
      }
    })
  }

  return {
    openGatewayConflict,
    setupGatewayBanner,
    showGuardianRecovery,
  }
}
