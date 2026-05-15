export const PRODUCT_IDENTITY = Object.freeze({
  id: 'agentdock',
  name: 'AgentDock',
  displayName: 'AgentDock',
  assistantNameZh: 'DeepAi助手',
  assistantNameEn: 'DeepAi Assistant',
  tagline: 'Multi-engine AI agent operations console',
  description: 'AgentDock - production desktop console for multi-engine AI agent operations',
  tauriIdentifier: 'com.agentdock.desktop',
  homepage: 'https://github.com/agentdock/agentdock',
  homepageHost: 'github.com/agentdock/agentdock',
  supportUrl: 'https://github.com/agentdock/agentdock/issues',
  repositoryUrl: 'https://github.com/agentdock/agentdock',
  releaseUrl: 'https://github.com/agentdock/agentdock/releases',
  updateManifestUrl: 'https://raw.githubusercontent.com/agentdock/agentdock/main/update/latest.json',
  legacyProductName: 'ClawPanel',
})

export function productTitle(suffix = '') {
  const clean = String(suffix || '').trim()
  return clean ? `${PRODUCT_IDENTITY.name} - ${clean}` : PRODUCT_IDENTITY.name
}
