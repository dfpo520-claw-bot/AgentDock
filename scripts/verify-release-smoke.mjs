import crypto from 'node:crypto'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

import { classifyArtifact } from './generate-release-manifest.mjs'

const DEFAULT_BUNDLE_DIR = path.join('src-tauri', 'target', 'release', 'bundle')
const GENERATED_FILES = new Set(['release-manifest.json', 'checksums.sha256'])
const PLATFORM_KINDS = {
  windows: new Set(['nsis', 'msi']),
  macos: new Set(['dmg', 'macos-app-archive']),
  linux: new Set(['appimage', 'deb', 'rpm']),
}
const PLATFORM_INSTALLER_ERRORS = {
  windows: 'missing windows installer artifact',
  macos: 'missing macos installer artifact',
  linux: 'missing linux installer artifact',
}

function repoRoot() {
  return path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
}

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, 'utf8'))
}

function normalizeRelative(filePath) {
  return filePath.split(path.sep).join('/')
}

function sha256File(filePath) {
  const hash = crypto.createHash('sha256')
  hash.update(fs.readFileSync(filePath))
  return hash.digest('hex')
}

function currentPlatform() {
  if (process.platform === 'win32') return 'windows'
  if (process.platform === 'darwin') return 'macos'
  if (process.platform === 'linux') return 'linux'
  return 'any'
}

function assertCondition(condition, message) {
  if (!condition) {
    throw new Error(message)
  }
}

function artifactPath(bundleDir, relativePath) {
  assertCondition(!path.isAbsolute(relativePath), `artifact path must be relative: ${relativePath}`)
  assertCondition(!relativePath.split('/').includes('..'), `artifact path escapes bundle: ${relativePath}`)
  return path.join(bundleDir, ...relativePath.split('/'))
}

function assertSigningBlock(signing) {
  assertCondition(signing?.requiredForRelease === true, 'signing.requiredForRelease must be true')

  for (const [platform, fields] of Object.entries({
    windows: ['certificateEnv', 'passwordEnv', 'present'],
    macos: ['identityEnv', 'certificateEnv', 'passwordEnv', 'notarizationTeamEnv', 'present'],
    linux: ['detachedSignatureRequired', 'certificateEnv', 'present'],
  })) {
    assertCondition(signing[platform], `signing.${platform} is missing`)
    for (const field of fields) {
      assertCondition(
        Object.hasOwn(signing[platform], field),
        `signing.${platform}.${field} is missing`,
      )
    }
  }

  const serialized = JSON.stringify(signing)
  for (const forbidden of ['PRIVATE KEY-----', 'BEGIN CERTIFICATE', 'passwordValue', 'secretValue']) {
    assertCondition(!serialized.includes(forbidden), `signing block appears to contain secret material`)
  }
}

function parseChecksumFile(checksumsPath) {
  return fs
    .readFileSync(checksumsPath, 'utf8')
    .split(/\r?\n/)
    .filter(Boolean)
}

export function verifyReleaseSmoke({
  rootDir = repoRoot(),
  bundleDir = path.join(rootDir, DEFAULT_BUNDLE_DIR),
  platform = currentPlatform(),
} = {}) {
  const resolvedBundleDir = path.resolve(bundleDir)
  assertCondition(fs.existsSync(resolvedBundleDir), `bundle directory does not exist: ${resolvedBundleDir}`)

  const manifestPath = path.join(resolvedBundleDir, 'release-manifest.json')
  const checksumsPath = path.join(resolvedBundleDir, 'checksums.sha256')
  assertCondition(fs.existsSync(manifestPath), 'release-manifest.json is missing')
  assertCondition(fs.existsSync(checksumsPath), 'checksums.sha256 is missing')

  const packageJson = readJson(path.join(rootDir, 'package.json'))
  const tauriConfig = readJson(path.join(rootDir, 'src-tauri', 'tauri.conf.json'))
  const manifest = readJson(manifestPath)
  const artifacts = manifest.artifacts || []

  assertCondition(manifest.productName === tauriConfig.productName, 'manifest productName mismatch')
  assertCondition(manifest.packageName === packageJson.name, 'manifest packageName mismatch')
  assertCondition(manifest.version === packageJson.version, 'manifest version mismatch')
  assertCondition(manifest.tauriIdentifier === tauriConfig.identifier, 'manifest tauriIdentifier mismatch')
  assertCondition(manifest.artifactCount === artifacts.length, 'manifest artifactCount mismatch')
  assertCondition(artifacts.length > 0, 'release manifest has no artifacts')
  assertSigningBlock(manifest.signing)

  const expectedChecksumLines = []
  const kinds = new Set()
  for (const artifact of artifacts) {
    assertCondition(artifact.path, 'artifact path is missing')
    assertCondition(!GENERATED_FILES.has(path.basename(artifact.path)), `generated file listed as artifact: ${artifact.path}`)

    const fullPath = artifactPath(resolvedBundleDir, artifact.path)
    assertCondition(fs.existsSync(fullPath), `artifact file is missing: ${artifact.path}`)

    const stat = fs.statSync(fullPath)
    const actualSha = sha256File(fullPath)
    const expectedKind = classifyArtifact(artifact.path)

    assertCondition(artifact.bytes === stat.size, `byte size mismatch for ${artifact.path}`)
    assertCondition(artifact.sha256 === actualSha, `sha256 mismatch for ${artifact.path}`)
    assertCondition(artifact.kind === expectedKind, `artifact kind mismatch for ${artifact.path}`)

    kinds.add(artifact.kind)
    expectedChecksumLines.push(`${artifact.sha256}  ${normalizeRelative(artifact.path)}`)
  }

  const checksumLines = parseChecksumFile(checksumsPath)
  assertCondition(
    JSON.stringify(checksumLines) === JSON.stringify(expectedChecksumLines),
    'checksums.sha256 does not match release manifest artifacts',
  )

  const requiredKinds = PLATFORM_KINDS[platform]
  if (requiredKinds) {
    const hasInstaller = [...kinds].some((kind) => requiredKinds.has(kind))
    assertCondition(hasInstaller, PLATFORM_INSTALLER_ERRORS[platform])
  }

  return {
    bundleDir: resolvedBundleDir,
    artifactCount: artifacts.length,
    kinds: [...kinds].sort(),
    platform,
  }
}

function parseArgs(argv) {
  const args = { bundleDir: null, platform: currentPlatform(), help: false }
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i]
    if (arg === '--bundle-dir') {
      args.bundleDir = argv[++i]
    } else if (arg === '--platform') {
      args.platform = argv[++i]
    } else if (arg === '--help' || arg === '-h') {
      args.help = true
    } else {
      throw new Error(`Unknown argument: ${arg}`)
    }
  }
  return args
}

if (process.argv[1] && path.resolve(process.argv[1]) === path.resolve(fileURLToPath(import.meta.url))) {
  try {
    const args = parseArgs(process.argv.slice(2))
    if (args.help) {
      console.log('Usage: node scripts/verify-release-smoke.mjs [--bundle-dir <dir>] [--platform windows|macos|linux|any]')
      process.exit(0)
    }

    const rootDir = repoRoot()
    const bundleDir = args.bundleDir
      ? path.resolve(args.bundleDir)
      : path.join(rootDir, DEFAULT_BUNDLE_DIR)
    const result = verifyReleaseSmoke({ rootDir, bundleDir, platform: args.platform })

    console.log(`Release smoke verification passed for ${result.artifactCount} artifacts`)
    console.log(`Bundle: ${result.bundleDir}`)
    console.log(`Platform: ${result.platform}`)
    console.log(`Kinds: ${result.kinds.join(', ')}`)
  } catch (error) {
    console.error(error?.message || error)
    process.exit(1)
  }
}
