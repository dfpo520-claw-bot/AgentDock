import assert from 'node:assert/strict'
import { execFileSync } from 'node:child_process'
import test from 'node:test'

test('ui smoke script can be imported without cli argv', () => {
  const output = execFileSync(
    process.execPath,
    [
      '--input-type=module',
      '-e',
      "import('./scripts/smoke-ui-routes.mjs').then(() => console.log('ok'))",
    ],
    { cwd: process.cwd(), encoding: 'utf8' },
  )

  assert.match(output, /ok/)
})

test('ui smoke script exposes package command', () => {
  const packageJson = JSON.parse(execFileSync(process.execPath, [
    '--input-type=module',
    '-e',
    "import fs from 'node:fs'; process.stdout.write(fs.readFileSync('package.json', 'utf8'))",
  ], { cwd: process.cwd(), encoding: 'utf8' }))

  assert.equal(packageJson.scripts['smoke:ui'], 'node scripts/smoke-ui-routes.mjs')
})
