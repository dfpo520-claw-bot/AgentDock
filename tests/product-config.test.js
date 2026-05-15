import test from 'node:test'
import assert from 'node:assert/strict'
import {
  PRODUCT_CONFIG,
  isKnownLegacyPanelConfigFile,
  isProductPanelConfigFile,
} from '../src/lib/product-config.js'

test('PRODUCT_CONFIG exposes product-owned config defaults', () => {
  assert.equal(PRODUCT_CONFIG.productId, 'agentdock')
  assert.equal(PRODUCT_CONFIG.panelConfigFile, 'agentdock.json')
  assert.equal(PRODUCT_CONFIG.productDataDirName, '.agentdock')
  assert.equal(PRODUCT_CONFIG.legacyPanelConfigFile, 'clawpanel.json')
  assert.equal(PRODUCT_CONFIG.legacyDataDirName, '.openclaw')
  assert.equal(PRODUCT_CONFIG.releaseChannel, 'stable')
})

test('config filename helpers distinguish product and legacy files', () => {
  assert.equal(isProductPanelConfigFile('agentdock.json'), true)
  assert.equal(isProductPanelConfigFile('clawpanel.json'), false)
  assert.equal(isKnownLegacyPanelConfigFile('clawpanel.json'), true)
  assert.equal(isKnownLegacyPanelConfigFile('agentdock.json'), false)
})
