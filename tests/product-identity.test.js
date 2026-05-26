import test from 'node:test'
import assert from 'node:assert/strict'

import {
  PRODUCT_IDENTITY,
  productTitle,
} from '../src/lib/product-identity.js'

const OLD_QING = ['qing', 'chen'].join('')
const OLD_QING_CLOUD = `${OLD_QING}cloud`
const OLD_PANEL = ['Claw', 'Panel'].join('')
const OLD_PANEL_DOMAIN = ['claw', 'qt', 'cool'].join('.')

function literalPattern(value, flags = '') {
  return new RegExp(value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), flags)
}

test('PRODUCT_IDENTITY exposes the production fork identity', () => {
  assert.equal(PRODUCT_IDENTITY.id, 'agentdock')
  assert.equal(PRODUCT_IDENTITY.name, 'AgentDock')
  assert.equal(PRODUCT_IDENTITY.displayName, 'AgentDock')
  assert.equal(PRODUCT_IDENTITY.assistantNameZh, 'DeepAi助手')
  assert.equal(PRODUCT_IDENTITY.assistantNameEn, 'DeepAi Assistant')
  assert.equal(PRODUCT_IDENTITY.tagline, 'Multi-engine AI agent operations console')
  assert.equal(PRODUCT_IDENTITY.description, 'AgentDock - production desktop console for multi-engine AI agent operations')
  assert.equal(PRODUCT_IDENTITY.tauriIdentifier, 'com.agentdock.desktop')
  assert.equal(PRODUCT_IDENTITY.homepage, 'https://github.com/dfpo520-claw-bot/AgentDock')
  assert.equal(PRODUCT_IDENTITY.homepageHost, 'github.com/dfpo520-claw-bot/AgentDock')
  assert.equal(PRODUCT_IDENTITY.supportUrl, 'https://github.com/dfpo520-claw-bot/AgentDock/issues')
  assert.equal(PRODUCT_IDENTITY.repositoryUrl, 'https://github.com/dfpo520-claw-bot/AgentDock')
  assert.equal(PRODUCT_IDENTITY.releaseUrl, 'https://github.com/dfpo520-claw-bot/AgentDock/releases')
  assert.equal(PRODUCT_IDENTITY.updateManifestUrl, 'https://raw.githubusercontent.com/dfpo520-claw-bot/AgentDock/main/update/latest.json')
  assert.equal(PRODUCT_IDENTITY.legacyProductName, 'AgentDock')
})

test('visible identity fields no longer use the fork app name', () => {
  const visible = [
    PRODUCT_IDENTITY.name,
    PRODUCT_IDENTITY.displayName,
    PRODUCT_IDENTITY.assistantNameZh,
    PRODUCT_IDENTITY.assistantNameEn,
    PRODUCT_IDENTITY.tagline,
    PRODUCT_IDENTITY.description,
    PRODUCT_IDENTITY.homepage,
    PRODUCT_IDENTITY.homepageHost,
    PRODUCT_IDENTITY.supportUrl,
    PRODUCT_IDENTITY.repositoryUrl,
    PRODUCT_IDENTITY.releaseUrl,
    PRODUCT_IDENTITY.updateManifestUrl,
  ].join('\n')

  assert.doesNotMatch(visible, literalPattern(OLD_PANEL))
  assert.doesNotMatch(visible, literalPattern(OLD_PANEL_DOMAIN))
  assert.doesNotMatch(visible, literalPattern(`${OLD_QING_CLOUD}/${OLD_PANEL.toLowerCase()}`, 'i'))
  assert.doesNotMatch(visible, literalPattern(`${OLD_QING_CLOUD}/agentdock`, 'i'))
})

test('productTitle formats document and window titles', () => {
  assert.equal(productTitle(), 'AgentDock')
  assert.equal(productTitle('Settings'), 'AgentDock - Settings')
})
