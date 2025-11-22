#!/usr/bin/env node
/**
 * Cross-platform build script for Tauri release
 * Automatically selects the correct bundle format based on platform
 */

import { execSync } from 'child_process'
import os from 'os'
import { fileURLToPath } from 'url'
import path from 'path'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)

const platform = os.platform()

// Determine bundle format based on platform
let bundles
if (platform === 'win32') {
  // Only build NSIS for faster compilation (MSI can be built separately if needed)
  bundles = 'nsis'
  console.log('Building for Windows (NSIS installer)...')
  console.log('Note: Rust compilation may take several minutes on first build. This is normal!')
} else if (platform === 'darwin') {
  bundles = 'dmg'
  console.log('Building for macOS (DMG)...')
} else {
  bundles = 'appimage'
  console.log('Building for Linux (AppImage)...')
}

console.log(`Using bundles: ${bundles}`)

try {
  // Run setup:kubo
  console.log('\n1. Setting up Kubo binary...')
  execSync('npm run setup:kubo', { stdio: 'inherit', cwd: path.join(__dirname, '..') })
  
  // Build frontend
  console.log('\n2. Building frontend...')
  execSync('npm run build', { stdio: 'inherit', cwd: path.join(__dirname, '..') })
  
  // Build Tauri with appropriate bundles
  console.log(`\n3. Building Tauri bundles (${bundles})...`)
  console.log('   ⏳ This step may take 5-15 minutes depending on your system...')
  console.log('   ⏳ Rust is compiling dependencies - please be patient...\n')
  
  const startTime = Date.now()
  execSync(`tauri build --bundles ${bundles}`, { stdio: 'inherit', cwd: path.join(__dirname, '..') })
  const elapsedTime = ((Date.now() - startTime) / 1000 / 60).toFixed(1)
  console.log(`\n   ✓ Rust compilation completed in ${elapsedTime} minutes`)
  
  console.log('\n✓ Build completed successfully!')
  console.log(`\nOutput location: ${path.join(__dirname, '..', 'src-tauri', 'target', 'release', 'bundle')}`)
} catch (error) {
  console.error('\n✗ Build failed:', error.message)
  process.exit(1)
}
