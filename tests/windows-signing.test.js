import assert from 'node:assert/strict'
import { execFileSync } from 'node:child_process'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import test from 'node:test'

import { verifyWindowsSigning } from '../scripts/verify-windows-signing.mjs'

function fakeInstaller() {
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'agentdock-signing-'))
  const file = path.join(tmp, 'AgentDock-setup.exe')
  fs.writeFileSync(file, 'unsigned installer bytes')
  return file
}

test('windows signing module can be imported without cli argv', () => {
  const output = execFileSync(
    process.execPath,
    [
      '--input-type=module',
      '-e',
      "import('./scripts/verify-windows-signing.mjs').then(() => console.log('ok'))",
    ],
    { cwd: process.cwd(), encoding: 'utf8' },
  )

  assert.match(output, /ok/)
})

test('windows signing verifier allows unsigned artifacts only for local smoke', () => {
  const result = verifyWindowsSigning({ file: fakeInstaller(), allowUnsigned: true })

  assert.equal(result.publishable, false)
  if (process.platform === 'win32') {
    assert.match(result.status, /NotSigned|UnknownError|HashMismatch|NotTrusted/)
  } else {
    assert.equal(result.status, 'UnsupportedPlatform')
  }
})

test('windows signing verifier rejects unsigned artifacts for publishing', () => {
  assert.throws(
    () => verifyWindowsSigning({ file: fakeInstaller() }),
    /not publishable|can only be verified on Windows/,
  )
})
