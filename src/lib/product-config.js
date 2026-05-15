import { api } from './tauri-api.js'

export const PRODUCT_CONFIG = Object.freeze({
  productId: 'agentdock',
  panelConfigFile: 'agentdock.json',
  productDataDirName: '.agentdock',
  legacyPanelConfigFile: 'clawpanel.json',
  legacyDataDirName: '.openclaw',
  releaseChannel: 'stable',
})

export function isProductPanelConfigFile(name) {
  return String(name || '').trim() === PRODUCT_CONFIG.panelConfigFile
}

export function isKnownLegacyPanelConfigFile(name) {
  return String(name || '').trim() === PRODUCT_CONFIG.legacyPanelConfigFile
}

export function checkLegacyConfigMigration() {
  return api.detectLegacyConfigMigration()
}

export function applyLegacyConfigDecision(decision) {
  return api.applyLegacyConfigMigration(decision)
}
