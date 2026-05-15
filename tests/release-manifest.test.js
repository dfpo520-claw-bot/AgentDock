import assert from 'node:assert/strict'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import test from 'node:test'

import { writeReleaseManifest } from '../scripts/generate-release-manifest.mjs'

test('release manifest records deterministic artifact checksums', () => {
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'agentdock-release-'))
  const bundleDir = path.join(tmp, 'bundle')
  const nestedDir = path.join(bundleDir, 'nsis')
  fs.mkdirSync(nestedDir, { recursive: true })
  fs.writeFileSync(path.join(nestedDir, 'AgentDock-setup.exe'), 'installer-bytes')
  fs.writeFileSync(path.join(bundleDir, 'release-manifest.json'), 'stale')
  fs.writeFileSync(path.join(bundleDir, 'checksums.sha256'), 'stale')

  const { manifest, manifestPath, checksumsPath } = writeReleaseManifest({
    rootDir: process.cwd(),
    bundleDir,
  })

  assert.equal(manifest.productName, 'AgentDock')
  assert.equal(manifest.artifactCount, 1)
  assert.equal(manifest.artifacts[0].path, 'nsis/AgentDock-setup.exe')
  assert.match(manifest.artifacts[0].sha256, /^[a-f0-9]{64}$/)
  assert.equal(fs.existsSync(manifestPath), true)
  assert.equal(fs.existsSync(checksumsPath), true)
  assert.match(fs.readFileSync(checksumsPath, 'utf8'), /nsis\/AgentDock-setup\.exe/)
})
