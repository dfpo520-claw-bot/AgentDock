import test from 'node:test'
import assert from 'node:assert/strict'
import fs from 'node:fs'

test('app-shell gateway banner module exports a stable controller factory', async () => {
  const mod = await import('../src/app-shell/gateway-banner.js')

  assert.equal(typeof mod.createGatewayBannerController, 'function')
})

test('main delegates gateway shell handling to app-shell gateway banner module', () => {
  const text = fs.readFileSync('src/main.js', 'utf8')

  assert.match(text, /from '\.\/app-shell\/gateway-banner\.js'/)
  assert.match(text, /createGatewayBannerController\(/)
  assert.match(text, /setupGatewayBanner\(/)
  assert.match(text, /showGuardianRecovery\(/)
})
