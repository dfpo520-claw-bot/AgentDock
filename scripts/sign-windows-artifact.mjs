import { execFileSync, spawnSync } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const DEFAULT_TIMESTAMP_URL = 'http://timestamp.digicert.com'
const DEFAULT_SIGNTOOL = 'C:\\Program Files (x86)\\Windows Kits\\10\\bin\\10.0.26100.0\\x64\\signtool.exe'

function parseArgs(argv) {
  const args = {
    file: null,
    thumbprint: process.env.WINDOWS_CODESIGN_CERT_THUMBPRINT || null,
    timestampUrl: process.env.WINDOWS_CODESIGN_TIMESTAMP_URL || DEFAULT_TIMESTAMP_URL,
    signtool: process.env.WINDOWS_SIGNTOOL_PATH || DEFAULT_SIGNTOOL,
    dryRun: false,
    help: false,
  }

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i]
    if (arg === '--file') {
      args.file = argv[++i]
    } else if (arg === '--thumbprint') {
      args.thumbprint = argv[++i]
    } else if (arg === '--timestamp-url') {
      args.timestampUrl = argv[++i]
    } else if (arg === '--signtool') {
      args.signtool = argv[++i]
    } else if (arg === '--dry-run') {
      args.dryRun = true
    } else if (arg === '--help' || arg === '-h') {
      args.help = true
    } else {
      throw new Error(`Unknown argument: ${arg}`)
    }
  }
  return args
}

function assertCondition(condition, message) {
  if (!condition) {
    throw new Error(message)
  }
}

function normalizeThumbprint(thumbprint) {
  return String(thumbprint || '').replace(/\s+/g, '').toUpperCase()
}

function readCodeSigningCertificate(thumbprint) {
  const escapedThumbprint = thumbprint.replaceAll("'", "''")
  const command = [
    '$ErrorActionPreference = "Stop"',
    `$thumbprint = '${escapedThumbprint}'`,
    '$cert = Get-ChildItem -Path Cert:\\CurrentUser\\My,Cert:\\LocalMachine\\My -CodeSigningCert -ErrorAction SilentlyContinue | Where-Object { $_.Thumbprint -eq $thumbprint } | Select-Object -First 1',
    '$result = if ($null -eq $cert) { [pscustomobject]@{ Found = $false } } else { [pscustomobject]@{ Found = $true; Subject = $cert.Subject; Thumbprint = $cert.Thumbprint; NotAfter = $cert.NotAfter.ToString("o"); HasPrivateKey = $cert.HasPrivateKey } }',
    '$result | ConvertTo-Json -Compress',
  ].join('; ')

  return JSON.parse(
    execFileSync(
      'powershell.exe',
      ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-Command', command],
      { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] },
    ),
  )
}

function buildSigntoolArgs({ file, thumbprint, timestampUrl }) {
  return [
    'sign',
    '/fd',
    'sha256',
    '/sha1',
    thumbprint,
    '/tr',
    timestampUrl,
    '/td',
    'sha256',
    file,
  ]
}

export function planWindowsArtifactSigning({
  file,
  thumbprint,
  timestampUrl = DEFAULT_TIMESTAMP_URL,
  signtool = DEFAULT_SIGNTOOL,
} = {}) {
  assertCondition(file, '--file is required')
  const resolvedFile = path.resolve(file)
  const normalizedThumbprint = normalizeThumbprint(thumbprint)

  assertCondition(fs.existsSync(resolvedFile), `file does not exist: ${resolvedFile}`)
  assertCondition(normalizedThumbprint, 'WINDOWS_CODESIGN_CERT_THUMBPRINT or --thumbprint is required')
  assertCondition(/^[A-F0-9]{40}$/.test(normalizedThumbprint), 'code signing certificate thumbprint must be a 40-character SHA1 hex string')
  assertCondition(fs.existsSync(signtool), `signtool.exe does not exist: ${signtool}`)

  const certificate = readCodeSigningCertificate(normalizedThumbprint)
  assertCondition(certificate.Found === true, `code signing certificate not found in CurrentUser/My or LocalMachine/My: ${normalizedThumbprint}`)
  assertCondition(certificate.HasPrivateKey === true, `code signing certificate is missing a private key: ${normalizedThumbprint}`)

  return {
    file: resolvedFile,
    thumbprint: normalizedThumbprint,
    timestampUrl,
    signtool,
    certificate,
    args: buildSigntoolArgs({ file: resolvedFile, thumbprint: normalizedThumbprint, timestampUrl }),
  }
}

export function signWindowsArtifact(options = {}) {
  const plan = planWindowsArtifactSigning(options)
  if (options.dryRun) {
    return { ...plan, signed: false }
  }

  const result = spawnSync(plan.signtool, plan.args, {
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  })

  if (result.status !== 0) {
    throw new Error(`signtool failed with exit code ${result.status}\n${result.stdout || ''}${result.stderr || ''}`)
  }

  return { ...plan, signed: true, output: result.stdout || result.stderr || '' }
}

if (process.argv[1] && path.resolve(process.argv[1]) === path.resolve(fileURLToPath(import.meta.url))) {
  try {
    const args = parseArgs(process.argv.slice(2))
    if (args.help) {
      console.log('Usage: node scripts/sign-windows-artifact.mjs --file <installer.exe|msi> [--thumbprint <sha1>] [--timestamp-url <url>] [--signtool <path>] [--dry-run]')
      process.exit(0)
    }

    const result = signWindowsArtifact(args)
    console.log(`Windows signing ${result.signed ? 'completed' : 'dry run passed'}`)
    console.log(`File: ${result.file}`)
    console.log(`Signer: ${result.certificate.Subject}`)
    console.log(`Thumbprint: ${result.thumbprint}`)
    console.log(`Timestamp URL: ${result.timestampUrl}`)
  } catch (error) {
    console.error(error?.message || error)
    process.exit(1)
  }
}
