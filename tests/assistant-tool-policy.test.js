import test from 'node:test'
import assert from 'node:assert/strict'
import fs from 'node:fs'

import {
  ASSISTANT_TOOL_POLICY,
  evaluateAssistantToolRequest,
  getAssistantToolPolicy,
} from '../src/lib/assistant-tool-policy.js'

const MODES = {
  plan: { readOnly: true, confirmDanger: true },
  execute: { readOnly: false, confirmDanger: true },
  unlimited: { readOnly: false, confirmDanger: true },
}

test('assistant tool policy classifies read, write, network, and command tools', () => {
  assert.equal(getAssistantToolPolicy('read_file').effect, 'file-read')
  assert.equal(getAssistantToolPolicy('write_file').effect, 'file-write')
  assert.equal(getAssistantToolPolicy('fetch_url').effect, 'network')
  assert.equal(getAssistantToolPolicy('run_command').effect, 'command')
  assert.equal(getAssistantToolPolicy('skills_install_dep').effect, 'install')
})

test('assistant read-only mode blocks mutating tools before execution', () => {
  for (const toolName of ASSISTANT_TOOL_POLICY.readOnlyBlockedTools) {
    const decision = evaluateAssistantToolRequest(toolName, {}, MODES.plan)
    assert.equal(decision.allowed, false, `${toolName} should be blocked`)
    assert.equal(decision.rejectionKey, 'assistant.toolRejectedReadOnly')
  }

  assert.equal(evaluateAssistantToolRequest('read_file', {}, MODES.plan).allowed, true)
  assert.equal(evaluateAssistantToolRequest('list_directory', {}, MODES.plan).allowed, true)
})

test('assistant network, write, install, and command tools require explicit confirmation', () => {
  for (const toolName of ['web_search', 'fetch_url', 'write_file', 'skills_install_dep', 'skillhub_install', 'run_command']) {
    const decision = evaluateAssistantToolRequest(toolName, { command: 'node -v' }, MODES.execute)
    assert.equal(decision.allowed, true)
    assert.equal(decision.requiresConfirmation, true, `${toolName} should require confirmation`)
  }
})

test('assistant critical commands are elevated above normal confirmation', () => {
  const decision = evaluateAssistantToolRequest(
    'run_command',
    { command: 'rm -rf /' },
    MODES.unlimited,
    { isCriticalCommand: () => true },
  )

  assert.equal(decision.allowed, true)
  assert.equal(decision.requiresConfirmation, true)
  assert.equal(decision.critical, true)
  assert.equal(decision.rejectionKey, 'assistant.toolRejectedDanger')
})

test('assistant unknown tools are rejected by the policy gate', () => {
  const decision = evaluateAssistantToolRequest('unknown_tool', {}, MODES.unlimited)

  assert.equal(decision.allowed, false)
  assert.equal(decision.effect, 'unknown')
  assert.equal(decision.rejectionKey, 'assistant.toolUnknown')
})

test('assistant policy covers every tool exposed to model providers', () => {
  const assistantJs = fs.readFileSync('src/pages/assistant.js', 'utf8')
  const toolDefsStart = assistantJs.indexOf('const TOOL_DEFS = {')
  const toolDefsEnd = assistantJs.indexOf('const CRITICAL_PATTERNS = [')
  assert.notEqual(toolDefsStart, -1)
  assert.notEqual(toolDefsEnd, -1)

  const toolDefsSource = assistantJs.slice(toolDefsStart, toolDefsEnd)
  const exposedTools = [...toolDefsSource.matchAll(/name: '([^']+)'/g)]
    .map(match => match[1])

  assert.ok(exposedTools.length > 0)
  for (const toolName of exposedTools) {
    assert.notEqual(getAssistantToolPolicy(toolName).effect, 'unknown', `${toolName} must be registered in assistant-tool-policy.js`)
  }
})
