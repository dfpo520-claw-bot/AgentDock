export const PRODUCT_IDENTITY = Object.freeze({
  id: 'agentdock',
  name: 'AgentDock',
  displayName: 'AgentDock',
  assistantNameZh: 'DeepAi助手',
  assistantNameEn: 'DeepAi Assistant',
  tagline: 'Multi-engine AI agent operations console',
  description: 'AgentDock - production desktop console for multi-engine AI agent operations',
  tauriIdentifier: 'com.agentdock.desktop',
  homepage: 'https://github.com/dfpo520-claw-bot/AgentDock',
  homepageHost: 'github.com/dfpo520-claw-bot/AgentDock',
  supportUrl: 'https://github.com/dfpo520-claw-bot/AgentDock/issues',
  repositoryUrl: 'https://github.com/dfpo520-claw-bot/AgentDock',
  releaseUrl: 'https://github.com/dfpo520-claw-bot/AgentDock/releases',
  updateManifestUrl: 'https://raw.githubusercontent.com/dfpo520-claw-bot/AgentDock/master/update/latest.json',
  legacyProductName: 'AgentDock',
})

export function productTitle(suffix = '') {
  const clean = String(suffix || '').trim()
  return clean ? `${PRODUCT_IDENTITY.name} - ${clean}` : PRODUCT_IDENTITY.name
}
