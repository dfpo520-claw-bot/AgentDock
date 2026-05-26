import test from 'node:test'
import assert from 'node:assert/strict'
import { api } from '../src/lib/tauri-api.js'
import {
  PRODUCT_CONFIG,
  applyLegacyConfigDecision,
  checkLegacyConfigMigration,
  describeLegacyConfigDetection,
  isKnownLegacyPanelConfigFile,
  isProductPanelConfigFile,
} from '../src/lib/product-config.js'

test('PRODUCT_CONFIG exposes product-owned config defaults', () => {
  assert.equal(PRODUCT_CONFIG.productId, 'agentdock')
  assert.equal(PRODUCT_CONFIG.panelConfigFile, 'agentdock.json')
  assert.equal(PRODUCT_CONFIG.productDataDirName, '.agentdock')
  assert.equal(PRODUCT_CONFIG.legacyPanelConfigFile, 'agentdock.json')
  assert.equal(PRODUCT_CONFIG.legacyDataDirName, '.openclaw')
  assert.equal(PRODUCT_CONFIG.releaseChannel, 'stable')
})

test('config filename helpers recognize AgentDock-owned files', () => {
  assert.equal(isProductPanelConfigFile('agentdock.json'), true)
  assert.equal(isProductPanelConfigFile('openclaw.json'), false)
  assert.equal(isKnownLegacyPanelConfigFile('agentdock.json'), true)
  assert.equal(isKnownLegacyPanelConfigFile('openclaw.json'), false)
})

test('product config exposes migration API wrappers', () => {
  assert.equal(typeof checkLegacyConfigMigration, 'function')
  assert.equal(typeof applyLegacyConfigDecision, 'function')
})

test('tauri api exposes legacy config migration commands', () => {
  assert.equal(typeof api.detectLegacyConfigMigration, 'function')
  assert.equal(typeof api.applyLegacyConfigMigration, 'function')
})

test('migration wrappers delegate to tauri api commands', async () => {
  const originalDetect = api.detectLegacyConfigMigration
  const originalApply = api.applyLegacyConfigMigration

  api.detectLegacyConfigMigration = async () => ({ hasLegacyConfig: true })
  api.applyLegacyConfigMigration = async (action) => ({ action })

  try {
    assert.deepEqual(
      await checkLegacyConfigMigration(),
      { hasLegacyConfig: true },
    )
    assert.deepEqual(
      await applyLegacyConfigDecision('import'),
      { action: 'import' },
    )
  } finally {
    api.detectLegacyConfigMigration = originalDetect
    api.applyLegacyConfigMigration = originalApply
  }
})

test('legacy detection summary normalizes migration payloads', () => {
  assert.deepEqual(
    describeLegacyConfigDetection({
      needed: true,
      detectedItems: ['legacyPanelConfig'],
      legacyConfigPath: 'C:/Users/demo/.openclaw/agentdock.json',
      recommendedAction: 'import',
    }),
    {
      needed: true,
      items: ['legacyPanelConfig'],
      legacyPath: 'C:/Users/demo/.openclaw/agentdock.json',
      recommendedAction: 'import',
    },
  )

  assert.deepEqual(
    describeLegacyConfigDetection({
      needed: false,
      legacyDataDir: 'C:/Users/demo/.openclaw',
    }),
    {
      needed: false,
      items: [],
      legacyPath: 'C:/Users/demo/.openclaw',
      recommendedAction: 'ignore',
    },
  )
})
