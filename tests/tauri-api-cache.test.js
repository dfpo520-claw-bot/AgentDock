import test from 'node:test'
import assert from 'node:assert/strict'

function deferred() {
  let resolve
  const promise = new Promise(r => { resolve = r })
  return { promise, resolve }
}

function jsonResponse(data) {
  return {
    ok: true,
    status: 200,
    headers: { get: () => 'application/json' },
    json: async () => data,
  }
}

test('invalidated in-flight cached calls cannot overwrite newer cache values', async () => {
  const first = deferred()
  const second = deferred()
  const requests = [first, second]
  const urls = []

  globalThis.window = { location: { hostname: 'localhost' } }
  globalThis.fetch = async (url) => {
    urls.push(url)
    const next = requests.shift()
    assert.ok(next, 'unexpected extra fetch')
    return next.promise
  }

  const { api, invalidate } = await import(`../src/lib/tauri-api.js?cache-race=${Date.now()}`)
  invalidate()

  const stale = api.listAgents()
  invalidate('list_agents')
  const fresh = api.listAgents()

  second.resolve(jsonResponse([{ id: 'fresh' }]))
  assert.deepEqual(await fresh, [{ id: 'fresh' }])

  first.resolve(jsonResponse([{ id: 'stale' }]))
  assert.deepEqual(await stale, [{ id: 'stale' }])

  assert.deepEqual(await api.listAgents(), [{ id: 'fresh' }])
  assert.deepEqual(urls, ['/__api/list_agents', '/__api/list_agents'])
})
