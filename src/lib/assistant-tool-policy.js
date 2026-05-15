const FILE_READ_TOOLS = ['read_file', 'list_directory']
const SYSTEM_TOOLS = ['get_system_info', 'list_processes', 'check_port']
const KNOWLEDGE_TOOLS = ['skills_list', 'skills_info', 'skills_check', 'skillhub_search']

const INTERACTIVE_TOOLS = ['ask_user']
const NETWORK_TOOLS = ['web_search', 'fetch_url']
const COMMAND_TOOLS = ['run_command']
const FILE_WRITE_TOOLS = ['write_file']
const INSTALL_TOOLS = ['skills_install_dep', 'skillhub_install']

export const ASSISTANT_TOOL_POLICY = Object.freeze({
  readOnlyTools: Object.freeze([...FILE_READ_TOOLS, ...SYSTEM_TOOLS, ...KNOWLEDGE_TOOLS]),
  interactiveTools: Object.freeze([...INTERACTIVE_TOOLS]),
  networkTools: Object.freeze([...NETWORK_TOOLS]),
  commandTools: Object.freeze([...COMMAND_TOOLS]),
  fileWriteTools: Object.freeze([...FILE_WRITE_TOOLS]),
  installTools: Object.freeze([...INSTALL_TOOLS]),
  readOnlyBlockedTools: Object.freeze([
    ...COMMAND_TOOLS,
    ...FILE_WRITE_TOOLS,
    ...INSTALL_TOOLS,
  ]),
  confirmationRequiredTools: Object.freeze([
    ...NETWORK_TOOLS,
    ...COMMAND_TOOLS,
    ...FILE_WRITE_TOOLS,
    ...INSTALL_TOOLS,
  ]),
})

const TOOL_EFFECTS = new Map([
  ...FILE_READ_TOOLS.map(name => [name, 'file-read']),
  ...SYSTEM_TOOLS.map(name => [name, 'system-read']),
  ...KNOWLEDGE_TOOLS.map(name => [name, 'knowledge-read']),
  ...INTERACTIVE_TOOLS.map(name => [name, 'interactive']),
  ...NETWORK_TOOLS.map(name => [name, 'network']),
  ...COMMAND_TOOLS.map(name => [name, 'command']),
  ...FILE_WRITE_TOOLS.map(name => [name, 'file-write']),
  ...INSTALL_TOOLS.map(name => [name, 'install']),
])

export function getAssistantToolPolicy(toolName) {
  const effect = TOOL_EFFECTS.get(toolName) || 'unknown'
  return {
    effect,
    blockedInReadOnly: ASSISTANT_TOOL_POLICY.readOnlyBlockedTools.includes(toolName),
    requiresConfirmation: ASSISTANT_TOOL_POLICY.confirmationRequiredTools.includes(toolName),
    interactive: ASSISTANT_TOOL_POLICY.interactiveTools.includes(toolName),
  }
}

export function evaluateAssistantToolRequest(toolName, args = {}, mode = {}, helpers = {}) {
  const policy = getAssistantToolPolicy(toolName)
  const critical = toolName === 'run_command'
    && typeof helpers.isCriticalCommand === 'function'
    && helpers.isCriticalCommand(args.command || '')

  if (policy.effect === 'unknown') {
    return {
      ...policy,
      allowed: false,
      critical,
      requiresConfirmation: false,
      rejectionKey: 'assistant.toolUnknown',
    }
  }

  if (mode.readOnly && policy.blockedInReadOnly) {
    return {
      ...policy,
      allowed: false,
      critical,
      requiresConfirmation: false,
      rejectionKey: 'assistant.toolRejectedReadOnly',
    }
  }

  const requiresConfirmation = critical
    || Boolean(mode.confirmDanger && policy.requiresConfirmation)

  return {
    ...policy,
    allowed: true,
    critical,
    requiresConfirmation,
    rejectionKey: critical ? 'assistant.toolRejectedDanger' : 'assistant.toolRejected',
  }
}
