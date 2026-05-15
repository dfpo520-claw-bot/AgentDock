import { execFileSync } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

function parseArgs(argv) {
  const args = { file: null, allowUnsigned: false, help: false }
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i]
    if (arg === '--file') {
      args.file = argv[++i]
    } else if (arg === '--allow-unsigned') {
      args.allowUnsigned = true
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

function readAuthenticodeSignature(filePath) {
  const command = [
    '$ErrorActionPreference = "Stop"',
    `$signature = Get-AuthenticodeSignature -LiteralPath '${filePath.replaceAll("'", "''")}'`,
    '[pscustomobject]@{ Status = $signature.Status.ToString(); StatusMessage = $signature.StatusMessage; SignerSubject = $signature.SignerCertificate.Subject } | ConvertTo-Json -Compress',
  ].join('; ')

  return JSON.parse(
    execFileSync(
      'powershell.exe',
      ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-Command', command],
      { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] },
    ),
  )
}

export function verifyWindowsSigning({ file, allowUnsigned = false } = {}) {
  assertCondition(file, '--file is required')
  const resolvedFile = path.resolve(file)
  assertCondition(fs.existsSync(resolvedFile), `file does not exist: ${resolvedFile}`)

  if (process.platform !== 'win32') {
    const result = {
      file: resolvedFile,
      platform: process.platform,
      status: 'UnsupportedPlatform',
      publishable: false,
    }
    if (allowUnsigned) return result
    throw new Error(`Windows Authenticode signing can only be verified on Windows, current platform: ${process.platform}`)
  }

  const signature = readAuthenticodeSignature(resolvedFile)
  const publishable = signature.Status === 'Valid'
  const result = {
    file: resolvedFile,
    platform: 'win32',
    status: signature.Status,
    statusMessage: signature.StatusMessage || null,
    signerSubject: signature.SignerSubject || null,
    publishable,
  }

  if (!publishable && !allowUnsigned) {
    throw new Error(`Windows installer is not publishable: signature status is ${result.status}`)
  }

  return result
}

if (process.argv[1] && path.resolve(process.argv[1]) === path.resolve(fileURLToPath(import.meta.url))) {
  try {
    const args = parseArgs(process.argv.slice(2))
    if (args.help) {
      console.log('Usage: node scripts/verify-windows-signing.mjs --file <installer.exe|msi> [--allow-unsigned]')
      process.exit(0)
    }

    const result = verifyWindowsSigning(args)
    console.log(`Windows signing status: ${result.status}`)
    console.log(`File: ${result.file}`)
    console.log(`Publishable: ${result.publishable ? 'yes' : 'no'}`)
    if (result.signerSubject) {
      console.log(`Signer: ${result.signerSubject}`)
    }
    if (!result.publishable && args.allowUnsigned) {
      console.log('Unsigned artifact accepted for local smoke only because --allow-unsigned was provided')
    }
  } catch (error) {
    console.error(error?.message || error)
    process.exit(1)
  }
}
