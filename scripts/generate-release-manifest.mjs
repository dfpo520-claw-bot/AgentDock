import crypto from 'node:crypto'
import fs from 'node:fs'
import path from 'node:path'
import { execFileSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'

const DEFAULT_BUNDLE_DIR = path.join('src-tauri', 'target', 'release', 'bundle')
const GENERATED_FILES = new Set(['release-manifest.json', 'checksums.sha256'])

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, 'utf8'))
}

function repoRoot() {
  return path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
}

function normalizeRelative(filePath) {
  return filePath.split(path.sep).join('/')
}

function sha256File(filePath) {
  const hash = crypto.createHash('sha256')
  hash.update(fs.readFileSync(filePath))
  return hash.digest('hex')
}

function gitValue(args) {
  try {
    return (
      execFileSync('git', args, { encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] }).trim() ||
      null
    )
  } catch {
    return null
  }
}

function normalizeTargets(targets) {
  if (Array.isArray(targets)) return targets
  if (typeof targets === 'string') return [targets]
  return []
}

export function classifyArtifact(relativePath) {
  const normalized = relativePath.toLowerCase()
  if (normalized.endsWith('.tar.gz.sig') || normalized.endsWith('.sig')) return 'signature'
  if (normalized.endsWith('.app.tar.gz')) return 'macos-app-archive'
  if (normalized.endsWith('-setup.exe') || normalized.includes('/nsis/')) return 'nsis'
  if (normalized.endsWith('.msi') || normalized.includes('/msi/')) return 'msi'
  if (normalized.endsWith('.dmg') || normalized.includes('/dmg/')) return 'dmg'
  if (normalized.endsWith('.appimage') || normalized.includes('/appimage/')) return 'appimage'
  if (normalized.endsWith('.deb') || normalized.includes('/deb/')) return 'deb'
  if (normalized.endsWith('.rpm') || normalized.includes('/rpm/')) return 'rpm'
  if (
    normalized.endsWith('.zip') ||
    normalized.endsWith('.tar.gz') ||
    normalized.endsWith('.tgz') ||
    normalized.endsWith('.tar.xz')
  ) {
    return 'archive'
  }
  return 'other'
}

function signingStatus() {
  return {
    requiredForRelease: true,
    windows: {
      certificateEnv: 'TAURI_SIGNING_PRIVATE_KEY',
      passwordEnv: 'TAURI_SIGNING_PRIVATE_KEY_PASSWORD',
      present: Boolean(process.env.TAURI_SIGNING_PRIVATE_KEY),
    },
    macos: {
      identityEnv: 'APPLE_SIGNING_IDENTITY',
      certificateEnv: 'APPLE_CERTIFICATE',
      passwordEnv: 'APPLE_CERTIFICATE_PASSWORD',
      notarizationTeamEnv: 'APPLE_TEAM_ID',
      present: Boolean(process.env.APPLE_SIGNING_IDENTITY || process.env.APPLE_CERTIFICATE),
    },
    linux: {
      detachedSignatureRequired: false,
      certificateEnv: 'LINUX_SIGNING_KEY',
      present: Boolean(process.env.LINUX_SIGNING_KEY),
    },
  }
}

function collectArtifactFiles(bundleDir) {
  const files = []

  function walk(current) {
    for (const entry of fs.readdirSync(current, { withFileTypes: true })) {
      const fullPath = path.join(current, entry.name)
      if (entry.isDirectory()) {
        walk(fullPath)
      } else if (entry.isFile() && !GENERATED_FILES.has(entry.name)) {
        files.push(fullPath)
      }
    }
  }

  walk(bundleDir)
  return files.sort((a, b) => {
    const left = normalizeRelative(path.relative(bundleDir, a))
    const right = normalizeRelative(path.relative(bundleDir, b))
    return left.localeCompare(right)
  })
}

export function buildReleaseManifest({
  rootDir = repoRoot(),
  bundleDir = path.join(rootDir, DEFAULT_BUNDLE_DIR),
} = {}) {
  const packageJson = readJson(path.join(rootDir, 'package.json'))
  const tauriConfig = readJson(path.join(rootDir, 'src-tauri', 'tauri.conf.json'))
  const resolvedBundleDir = path.resolve(bundleDir)

  if (!fs.existsSync(resolvedBundleDir)) {
    throw new Error(`Bundle directory does not exist: ${resolvedBundleDir}`)
  }

  const artifacts = collectArtifactFiles(resolvedBundleDir).map((filePath) => {
    const stat = fs.statSync(filePath)
    const relativePath = normalizeRelative(path.relative(resolvedBundleDir, filePath))
    return {
      path: relativePath,
      kind: classifyArtifact(relativePath),
      bytes: stat.size,
      sha256: sha256File(filePath),
    }
  })

  return {
    schemaVersion: 1,
    productName: tauriConfig.productName,
    packageName: packageJson.name,
    version: packageJson.version,
    releaseChannel: packageJson.releaseChannel || 'production',
    tauriIdentifier: tauriConfig.identifier,
    bundle: {
      active: Boolean(tauriConfig.bundle?.active),
      targets: normalizeTargets(tauriConfig.bundle?.targets),
      icons: tauriConfig.bundle?.icon || [],
      publisher: tauriConfig.bundle?.publisher || null,
      shortDescription: tauriConfig.bundle?.shortDescription || null,
      windows: {
        webviewInstallMode: tauriConfig.bundle?.windows?.webviewInstallMode || null,
        nsis: tauriConfig.bundle?.windows?.nsis || null,
      },
    },
    signing: signingStatus(),
    gitCommit: gitValue(['rev-parse', 'HEAD']),
    gitTag: gitValue(['describe', '--tags', '--exact-match']),
    sourceDateEpoch: process.env.SOURCE_DATE_EPOCH || null,
    artifactCount: artifacts.length,
    artifacts,
  }
}

export function writeReleaseManifest({
  rootDir = repoRoot(),
  bundleDir = path.join(rootDir, DEFAULT_BUNDLE_DIR),
  manifestName = 'release-manifest.json',
  checksumsName = 'checksums.sha256',
} = {}) {
  const resolvedBundleDir = path.resolve(bundleDir)
  const manifest = buildReleaseManifest({ rootDir, bundleDir: resolvedBundleDir })
  const manifestPath = path.join(resolvedBundleDir, manifestName)
  const checksumsPath = path.join(resolvedBundleDir, checksumsName)

  fs.writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`)
  fs.writeFileSync(
    checksumsPath,
    `${manifest.artifacts.map((artifact) => `${artifact.sha256}  ${artifact.path}`).join('\n')}\n`,
  )

  return { manifest, manifestPath, checksumsPath }
}

function parseArgs(argv) {
  const args = { bundleDir: null }
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i]
    if (arg === '--bundle-dir') {
      args.bundleDir = argv[++i]
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
      console.log('Usage: node scripts/generate-release-manifest.mjs [--bundle-dir <dir>]')
      process.exit(0)
    }
    const rootDir = repoRoot()
    const bundleDir = args.bundleDir
      ? path.resolve(args.bundleDir)
      : path.join(rootDir, DEFAULT_BUNDLE_DIR)
    const { manifest, manifestPath, checksumsPath } = writeReleaseManifest({ rootDir, bundleDir })
    console.log(`Wrote ${manifest.artifactCount} artifact checksums`)
    console.log(`Manifest: ${manifestPath}`)
    console.log(`Checksums: ${checksumsPath}`)
  } catch (error) {
    console.error(error?.message || error)
    process.exit(1)
  }
}
