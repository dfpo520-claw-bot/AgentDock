import assert from 'node:assert/strict'
import { execFileSync } from 'node:child_process'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import test from 'node:test'

import { planWindowsArtifactSigning } from '../scripts/sign-windows-artifact.mjs'

function fakeInstaller() {
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'agentdock-sign-plan-'))
  const file = path.join(tmp, 'AgentDock-setup.exe')
  fs.writeFileSync(file, 'installer bytes')
  return file
}

test('windows artifact signing module can be imported without cli argv', () => {
  const output = execFileSync(
    process.execPath,
    [
      '--input-type=module',
      '-e',
      "import('./scripts/sign-windows-artifact.mjs').then(() => console.log('ok'))",
    ],
    { cwd: process.cwd(), encoding: 'utf8' },
  )

  assert.match(output, /ok/)
})

test('windows artifact signing requires a certificate thumbprint', () => {
  assert.throws(
    () => planWindowsArtifactSigning({ file: fakeInstaller(), signtool: process.execPath }),
    /thumbprint is required/,
  )
})

test('windows artifact signing rejects malformed thumbprints before invoking signtool', () => {
  assert.throws(
    () => planWindowsArtifactSigning({
      file: fakeInstaller(),
      thumbprint: 'not-a-thumbprint',
      signtool: process.execPath,
    }),
    /40-character SHA1 hex string/,
  )
})
