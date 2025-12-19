#!/usr/bin/env node
/**
 * Prepare Kubo binary for build based on current platform
 * This script copies the platform-specific binary to the generic name
 * that Rust code expects (ipfs.exe for Windows, ipfs for macOS/Linux)
 */

import fs from 'fs'
import path from 'path'
import os from 'os'
import { fileURLToPath } from 'url'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)

const KUBO_DIR = path.join(__dirname, '..', 'src-tauri', 'kubo')
const platform = os.platform()

let sourceFile, targetFile

if (platform === 'win32') {
  // Windows: use ipfs.exe (already correct)
  sourceFile = path.join(KUBO_DIR, 'ipfs.exe')
  targetFile = path.join(KUBO_DIR, 'ipfs.exe')
} else if (platform === 'darwin') {
  // macOS: copy ipfs.darwin to ipfs
  sourceFile = path.join(KUBO_DIR, 'ipfs.darwin')
  targetFile = path.join(KUBO_DIR, 'ipfs')
} else {
  // Linux: copy ipfs.linux to ipfs
  sourceFile = path.join(KUBO_DIR, 'ipfs.linux')
  targetFile = path.join(KUBO_DIR, 'ipfs')
}

if (!fs.existsSync(sourceFile)) {
  console.error(`❌ Error: Platform-specific binary not found: ${sourceFile}`)
  console.error(`   Please run: npm run setup:kubo:all`)
  process.exit(1)
}

// Copy the platform-specific binary to the generic name
if (platform !== 'win32') {
  // For macOS/Linux, copy to generic 'ipfs' name
  fs.copyFileSync(sourceFile, targetFile)
  // Set executable permissions
  fs.chmodSync(targetFile, '755')
  console.log(`✓ Prepared ${platform} binary: ${path.basename(sourceFile)} -> ${path.basename(targetFile)}`)
} else {
  // For Windows, ipfs.exe already exists
  console.log(`✓ Windows binary ready: ${path.basename(targetFile)}`)
}

console.log(`✓ Kubo binary prepared for ${platform} build`)

