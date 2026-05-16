import { spawn } from 'node:child_process'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const DEFAULT_ROUTES = ['/dashboard', '/settings', '/services', '/assistant', '/logs', '/about']
const DEFAULT_CHROME = 'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe'

function parseArgs(argv) {
  const args = {
    baseUrl: 'http://127.0.0.1:1421',
    password: process.env.AGENTDOCK_SMOKE_PASSWORD || '123456',
    chrome: process.env.CHROME_PATH || DEFAULT_CHROME,
    outDir: path.join('docs', 'release', 'ui-smoke-2026-05-15'),
    routes: [...DEFAULT_ROUTES],
    help: false,
  }

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i]
    if (arg === '--base-url') {
      args.baseUrl = argv[++i]
    } else if (arg === '--password') {
      args.password = argv[++i]
    } else if (arg === '--chrome') {
      args.chrome = argv[++i]
    } else if (arg === '--out-dir') {
      args.outDir = argv[++i]
    } else if (arg === '--routes') {
      args.routes = argv[++i].split(',').map((route) => route.trim()).filter(Boolean)
    } else if (arg === '--help' || arg === '-h') {
      args.help = true
    } else {
      throw new Error(`Unknown argument: ${arg}`)
    }
  }
  return args
}

function wait(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

async function waitForJson(url, timeoutMs = 10_000) {
  const start = Date.now()
  while (Date.now() - start < timeoutMs) {
    try {
      const response = await fetch(url)
      if (response.ok) return response.json()
    } catch {}
    await wait(200)
  }
  throw new Error(`Timed out waiting for ${url}`)
}

class CdpClient {
  constructor(ws) {
    this.ws = ws
    this.id = 0
    this.pending = new Map()
    this.events = []
    ws.onmessage = (event) => {
      const message = JSON.parse(event.data)
      if (message.id && this.pending.has(message.id)) {
        const { resolve, reject } = this.pending.get(message.id)
        this.pending.delete(message.id)
        if (message.error) reject(new Error(message.error.message || JSON.stringify(message.error)))
        else resolve(message.result || {})
      } else if (message.method) {
        this.events.push(message)
      }
    }
  }

  send(method, params = {}) {
    const id = ++this.id
    this.ws.send(JSON.stringify({ id, method, params }))
    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject })
    })
  }
}

async function connectCdp(port) {
  const targets = await waitForJson(`http://127.0.0.1:${port}/json/list`)
  const pageTarget = targets.find((target) => target.type === 'page' && target.webSocketDebuggerUrl)
  if (!pageTarget) {
    throw new Error('Chrome CDP page target was not found')
  }
  const ws = new WebSocket(pageTarget.webSocketDebuggerUrl)
  await new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error('Timed out opening Chrome CDP websocket')), 10_000)
    ws.onopen = () => {
      clearTimeout(timer)
      resolve()
    }
    ws.onerror = (event) => {
      clearTimeout(timer)
      reject(new Error(`Chrome CDP websocket failed: ${event.message || 'unknown error'}`))
    }
  })
  return new CdpClient(ws)
}

async function evaluate(client, expression, awaitPromise = true) {
  const result = await client.send('Runtime.evaluate', {
    expression,
    awaitPromise,
    returnByValue: true,
  })
  if (result.exceptionDetails) {
    throw new Error(result.exceptionDetails.text || 'Runtime evaluation failed')
  }
  return result.result?.value
}

function routeSlug(route) {
  return route.replace(/^\//, '').replace(/[^a-z0-9_-]+/gi, '-')
}

function analyzeRoute(route, value) {
  const text = String(value?.text || '').trim()
  const hasPage = Boolean(value?.hasPage)
  const hasLogin = Boolean(value?.hasLogin)
  const hasSplash = Boolean(value?.hasSplash)
  const title = String(value?.title || '')
  const errorText = [
    '模块加载失败',
    'Module load failed',
    'Failed to fetch dynamically imported module',
    'Uncaught',
    'ReferenceError',
    'TypeError',
  ].find((needle) => text.includes(needle))

  const ok = hasPage && !hasLogin && !hasSplash && text.length > 80 && !errorText
  return {
    route,
    ok,
    title,
    textLength: text.length,
    hasPage,
    hasLogin,
    hasSplash,
    errorText: errorText || null,
    excerpt: text.slice(0, 500),
  }
}

async function dismissTransientOverlays(client) {
  const result = await evaluate(client, `
    (() => {
      const overlays = [...document.querySelectorAll('.modal-overlay:not(#login-overlay)')]
      const unexpected = []
      let dismissed = 0
      for (const overlay of overlays) {
        const isKnownGatewayConflict = !!overlay.querySelector(
          '#gateway-conflict-open-cleanup, #gateway-conflict-open-settings, #gateway-conflict-refresh'
        )
        if (isKnownGatewayConflict) {
          overlay.remove()
          dismissed += 1
          continue
        }
        unexpected.push((overlay.innerText || overlay.textContent || '').trim().slice(0, 240))
      }
      if (unexpected.length > 0) {
        return { ok: false, unexpected }
      }
      document.scrollingElement?.scrollTo(0, 0)
      window.scrollTo(0, 0)
      document.querySelectorAll('*').forEach((element) => {
        if (element.scrollTop) element.scrollTop = 0
        if (element.scrollLeft) element.scrollLeft = 0
      })
      return { ok: true, dismissed }
    })()
  `)
  if (!result?.ok) {
    throw new Error(`Unexpected modal overlay before screenshot: ${(result?.unexpected || []).join(' | ')}`)
  }
  await wait(250)
}

export async function runUiSmoke({
  baseUrl,
  password,
  chrome,
  outDir,
  routes = DEFAULT_ROUTES,
} = {}) {
  if (!fs.existsSync(chrome)) {
    throw new Error(`Chrome executable does not exist: ${chrome}`)
  }
  fs.mkdirSync(outDir, { recursive: true })

  const port = 9333 + Math.floor(Math.random() * 200)
  const userDataDir = fs.mkdtempSync(path.join(os.tmpdir(), 'agentdock-chrome-smoke-'))
  const chromeProcess = spawn(chrome, [
    '--headless=new',
    '--disable-gpu',
    '--no-sandbox',
    `--remote-debugging-port=${port}`,
    `--user-data-dir=${userDataDir}`,
    `${baseUrl}/#/dashboard`,
  ], { stdio: 'ignore' })

  try {
    const client = await connectCdp(port)
    await client.send('Page.enable')
    await client.send('Runtime.enable')
    await client.send('Emulation.setDeviceMetricsOverride', {
      width: 1440,
      height: 1000,
      deviceScaleFactor: 1,
      mobile: false,
    })
    await wait(1500)

    await evaluate(client, `
      new Promise((resolve, reject) => {
        const done = () => resolve(true)
        const overlay = document.querySelector('#login-overlay')
        if (!overlay) {
          sessionStorage.setItem('clawpanel_authed', '1')
          done()
          return
        }
        const input = overlay.querySelector('#login-pw')
        const form = overlay.querySelector('#login-form')
        if (!input || !form) {
          reject(new Error('login form not found'))
          return
        }
        input.value = ${JSON.stringify(password)}
        form.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true }))
        const started = Date.now()
        const timer = setInterval(() => {
          const errorText = overlay.querySelector('#login-error')?.textContent || ''
          if (!document.querySelector('#login-overlay')) {
            clearInterval(timer)
            done()
          } else if (errorText) {
            clearInterval(timer)
            reject(new Error(errorText))
          } else if (Date.now() - started > 10000) {
            clearInterval(timer)
            reject(new Error('login overlay did not close'))
          }
        }, 150)
      })
    `)

    const results = []
    for (const route of routes) {
      await evaluate(client, `location.hash = ${JSON.stringify(`#${route}`)}`)
      await wait(3500)
      await dismissTransientOverlays(client)
      const value = await evaluate(client, `({
        title: document.title,
        text: document.body.innerText,
        hasPage: !!document.querySelector('.page, .dashboard-overview, .settings-page, .assistant-page, .hermes-logs-page'),
        hasLogin: !!document.querySelector('#login-overlay'),
        hasSplash: !!document.querySelector('#splash:not(.hide)'),
      })`)
      const analyzed = analyzeRoute(route, value)
      const screenshot = await client.send('Page.captureScreenshot', { format: 'png', captureBeyondViewport: false })
      const screenshotPath = path.join(outDir, `${routeSlug(route)}.png`)
      fs.writeFileSync(screenshotPath, Buffer.from(screenshot.data, 'base64'))
      results.push({ ...analyzed, screenshot: screenshotPath })
    }

    const failed = results.filter((result) => !result.ok)
    const summary = {
      baseUrl,
      generatedAt: new Date().toISOString(),
      routes: results,
      pass: failed.length === 0,
    }
    fs.writeFileSync(path.join(outDir, 'summary.json'), `${JSON.stringify(summary, null, 2)}\n`)
    if (failed.length > 0) {
      throw new Error(`UI smoke failed for routes: ${failed.map((result) => result.route).join(', ')}`)
    }
    return summary
  } finally {
    chromeProcess.kill()
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === path.resolve(fileURLToPath(import.meta.url))) {
  try {
    const args = parseArgs(process.argv.slice(2))
    if (args.help) {
      console.log('Usage: node scripts/smoke-ui-routes.mjs [--base-url <url>] [--password <password>] [--routes /dashboard,/settings] [--out-dir <dir>] [--chrome <path>]')
      process.exit(0)
    }

    const result = await runUiSmoke(args)
    console.log(`UI smoke passed for ${result.routes.length} routes`)
    console.log(`Output: ${path.resolve(args.outDir)}`)
    for (const route of result.routes) {
      console.log(`${route.ok ? 'PASS' : 'FAIL'} ${route.route} text=${route.textLength} screenshot=${route.screenshot}`)
    }
  } catch (error) {
    console.error(error?.message || error)
    process.exit(1)
  }
}
