import assert from 'node:assert/strict'
import { execFileSync } from 'node:child_process'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import test from 'node:test'

import { writeReleaseManifest } from '../scripts/generate-release-manifest.mjs'

function createBundleWithArtifacts(files) {
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'agentdock-release-smoke-'))
  const bundleDir = path.join(tmp, 'bundle')

  for (const [relativePath, contents] of files) {
    const fullPath = path.join(bundleDir, relativePath)
    fs.mkdirSync(path.dirname(fullPath), { recursive: true })
    fs.writeFileSync(fullPath, contents)
  }

  writeReleaseManifest({ rootDir: process.cwd(), bundleDir })
  return bundleDir
}

test('release smoke verifier accepts a complete windows installer bundle', () => {
  const bundleDir = createBundleWithArtifacts([
    ['nsis/AgentDock-setup.exe', 'installer-bytes'],
    ['msi/AgentDock.msi', 'msi-bytes'],
  ])

  const output = execFileSync(
    process.execPath,
    ['scripts/verify-release-smoke.mjs', '--bundle-dir', bundleDir, '--platform', 'windows'],
    { cwd: process.cwd(), encoding: 'utf8' },
  )

  assert.match(output, /Release smoke verification passed/)
  assert.match(output, /2 artifacts/)
  assert.match(output, /nsis/)
  assert.match(output, /msi/)
})

test('release smoke verifier rejects checksum drift', () => {
  const bundleDir = createBundleWithArtifacts([
    ['nsis/AgentDock-setup.exe', 'installer-bytes'],
  ])
  fs.writeFileSync(path.join(bundleDir, 'nsis', 'AgentDock-setup.exe'), 'tampered-bytes!')

  assert.throws(
    () => execFileSync(
      process.execPath,
      ['scripts/verify-release-smoke.mjs', '--bundle-dir', bundleDir, '--platform', 'windows'],
      { cwd: process.cwd(), encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] },
    ),
    /sha256 mismatch/,
  )
})

test('release smoke verifier rejects bundles without a platform installer', () => {
  const bundleDir = createBundleWithArtifacts([
    ['archives/AgentDock.zip', 'archive-bytes'],
  ])

  assert.throws(
    () => execFileSync(
      process.execPath,
      ['scripts/verify-release-smoke.mjs', '--bundle-dir', bundleDir, '--platform', 'windows'],
      { cwd: process.cwd(), encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] },
    ),
    /missing windows installer artifact/,
  )
})
