import assert from 'node:assert/strict'
import { execFileSync } from 'node:child_process'
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
  fs.mkdirSync(path.join(bundleDir, 'msi'), { recursive: true })
  fs.writeFileSync(path.join(bundleDir, 'msi', 'AgentDock.msi'), 'msi-bytes')
  fs.writeFileSync(path.join(bundleDir, 'release-manifest.json'), 'stale')
  fs.writeFileSync(path.join(bundleDir, 'checksums.sha256'), 'stale')

  const { manifest, manifestPath, checksumsPath } = writeReleaseManifest({
    rootDir: process.cwd(),
    bundleDir,
  })

  assert.equal(manifest.productName, 'AgentDock')
  assert.equal(manifest.artifactCount, 2)
  assert.equal(manifest.bundle.active, true)
  assert.deepEqual(manifest.bundle.targets, ['all'])
  assert.deepEqual(manifest.bundle.icons, [
    'icons/32x32.png',
    'icons/128x128.png',
    'icons/128x128@2x.png',
    'icons/icon.icns',
    'icons/icon.ico',
  ])
  assert.equal(manifest.releaseChannel, 'production')
  assert.equal(manifest.signing.requiredForRelease, true)
  assert.equal(manifest.signing.windows.certificateEnv, 'TAURI_SIGNING_PRIVATE_KEY')
  assert.equal(manifest.signing.windows.present, false)
  assert.equal(manifest.signing.macos.identityEnv, 'APPLE_SIGNING_IDENTITY')
  assert.equal(manifest.signing.linux.detachedSignatureRequired, false)
  const artifactsByPath = Object.fromEntries(
    manifest.artifacts.map((artifact) => [artifact.path, artifact]),
  )
  assert.equal(artifactsByPath['nsis/AgentDock-setup.exe'].kind, 'nsis')
  assert.match(artifactsByPath['nsis/AgentDock-setup.exe'].sha256, /^[a-f0-9]{64}$/)
  assert.equal(artifactsByPath['msi/AgentDock.msi'].kind, 'msi')
  assert.equal(fs.existsSync(manifestPath), true)
  assert.equal(fs.existsSync(checksumsPath), true)
  assert.match(fs.readFileSync(checksumsPath, 'utf8'), /nsis\/AgentDock-setup\.exe/)
})

test('release manifest classifies installer and archive artifacts', () => {
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'agentdock-release-kinds-'))
  const bundleDir = path.join(tmp, 'bundle')
  const files = [
    ['dmg/AgentDock.dmg', 'dmg'],
    ['macos/AgentDock.app.tar.gz', 'macos-app-archive'],
    ['appimage/AgentDock.AppImage', 'appimage'],
    ['deb/agentdock.deb', 'deb'],
    ['rpm/agentdock.rpm', 'rpm'],
    ['updater/AgentDock.tar.gz.sig', 'signature'],
    ['archives/agentdock.zip', 'archive'],
    ['notes/readme.txt', 'other'],
  ]

  for (const [relativePath] of files) {
    const fullPath = path.join(bundleDir, relativePath)
    fs.mkdirSync(path.dirname(fullPath), { recursive: true })
    fs.writeFileSync(fullPath, relativePath)
  }

  const { manifest } = writeReleaseManifest({
    rootDir: process.cwd(),
    bundleDir,
  })

  const kindsByPath = Object.fromEntries(
    manifest.artifacts.map((artifact) => [artifact.path, artifact.kind]),
  )
  for (const [relativePath, kind] of files) {
    assert.equal(kindsByPath[relativePath], kind)
  }
})

test('release manifest module can be imported without cli argv', () => {
  const output = execFileSync(
    process.execPath,
    [
      '--input-type=module',
      '-e',
      "import('./scripts/generate-release-manifest.mjs').then(() => console.log('ok'))",
    ],
    { cwd: process.cwd(), encoding: 'utf8' },
  )

  assert.match(output, /ok/)
})
