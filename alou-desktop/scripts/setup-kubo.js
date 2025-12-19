#!/usr/bin/env node
/**
 * Cross-platform script to download and setup Kubo (IPFS) binary
 */

import https from 'https'
import fs from 'fs'
import path from 'path'
import { execSync } from 'child_process'
import os from 'os'
import { fileURLToPath } from 'url'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)

const KUBO_VERSION = 'v0.39.0'
const KUBO_DIR = path.join(__dirname, '..', 'src-tauri', 'kubo')

// Parse command line arguments for target platform
const args = process.argv.slice(2)
let targetPlatform = null
if (args.includes('--platform') || args.includes('-p')) {
  const index = args.includes('--platform') ? args.indexOf('--platform') : args.indexOf('-p')
  targetPlatform = args[index + 1]
  if (!targetPlatform || targetPlatform.startsWith('-')) {
    console.error('Error: --platform requires a value (darwin, win32, or linux)')
    process.exit(1)
  }
}

// Detect platform (use target platform if specified, otherwise use current platform)
const platform = targetPlatform || os.platform()
const arch = os.arch()

// Validate platform
const validPlatforms = ['win32', 'darwin', 'linux']
if (!validPlatforms.includes(platform)) {
  console.error(`Error: Invalid platform "${platform}". Valid options: ${validPlatforms.join(', ')}`)
  process.exit(1)
}

let kuboFile, kuboUrl, extractCommand

if (platform === 'win32') {
  kuboFile = `kubo_${KUBO_VERSION}_windows-amd64.zip`
  kuboUrl = `https://dist.ipfs.tech/kubo/${KUBO_VERSION}/${kuboFile}`
  // Use tar or 7z for cross-platform extraction on Windows
  extractCommand = (file) => {
    // Try to use tar (available in Windows 10+)
    try {
      execSync('tar --version', { stdio: 'ignore' })
      return `tar -xf "${file}" -C "${path.dirname(file)}"`
    } catch {
      // Fallback to PowerShell
      return `powershell -Command "Expand-Archive -Path '${file}' -DestinationPath '${path.dirname(file)}' -Force"`
    }
  }
} else if (platform === 'darwin') {
  // Support both amd64 and arm64 for macOS
  const macArch = arch === 'arm64' ? 'arm64' : 'amd64'
  kuboFile = `kubo_${KUBO_VERSION}_darwin-${macArch}.tar.gz`
  kuboUrl = `https://dist.ipfs.tech/kubo/${KUBO_VERSION}/${kuboFile}`
  extractCommand = (file) => {
    // On Windows, use tar (available in Windows 10+)
    if (os.platform() === 'win32') {
      return `tar -xzf "${file}" -C "${path.dirname(file)}"`
    } else {
      return `tar -xzf "${file}" -C "${path.dirname(file)}"`
    }
  }
} else {
  kuboFile = `kubo_${KUBO_VERSION}_linux-amd64.tar.gz`
  kuboUrl = `https://dist.ipfs.tech/kubo/${KUBO_VERSION}/${kuboFile}`
  extractCommand = (file) => {
    if (os.platform() === 'win32') {
      return `tar -xzf "${file}" -C "${path.dirname(file)}"`
    } else {
      return `tar -xzf "${file}" -C "${path.dirname(file)}"`
    }
  }
}

// 为了支持多平台打包，使用平台特定的文件名
// macOS: ipfs.darwin, Linux: ipfs.linux, Windows: ipfs.exe
let binaryName, targetPath
if (platform === 'win32') {
  binaryName = 'ipfs.exe'
  targetPath = path.join(KUBO_DIR, binaryName)
} else if (platform === 'darwin') {
  binaryName = 'ipfs.darwin'
  targetPath = path.join(KUBO_DIR, binaryName)
} else {
  binaryName = 'ipfs.linux'
  targetPath = path.join(KUBO_DIR, binaryName)
}

// Check if binary already exists
if (fs.existsSync(targetPath)) {
  const stats = fs.statSync(targetPath)
  const fileSizeMB = (stats.size / (1024 * 1024)).toFixed(2)
  console.log(`✓ Kubo binary already exists at: ${targetPath}`)
  console.log(`  File size: ${fileSizeMB} MB`)
  console.log(`  Skipping download...`)
  process.exit(0)
}

// Create directory
if (!fs.existsSync(KUBO_DIR)) {
  fs.mkdirSync(KUBO_DIR, { recursive: true })
}

const tempFile = path.join(os.tmpdir(), kuboFile)

// Clean up any existing temp files
if (fs.existsSync(tempFile)) {
  try {
    fs.unlinkSync(tempFile)
    console.log('Cleaned up previous temp file')
  } catch (err) {
    console.warn(`Could not remove temp file: ${err.message}`)
  }
}

console.log(`Downloading Kubo ${KUBO_VERSION} for ${platform}...`)
console.log(`URL: ${kuboUrl}`)

// Download file with progress
const downloadFile = (url, dest) => {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest)
    let downloadedBytes = 0
    let totalBytes = 0
    
    https.get(url, (response) => {
      if (response.statusCode !== 200) {
        file.close()
        fs.unlinkSync(dest)
        reject(new Error(`Failed to download: HTTP ${response.statusCode}`))
        return
      }
      
      totalBytes = parseInt(response.headers['content-length'] || '0', 10)
      
      response.on('data', (chunk) => {
        downloadedBytes += chunk.length
        if (totalBytes > 0) {
          const percent = ((downloadedBytes / totalBytes) * 100).toFixed(1)
          process.stdout.write(`\rDownloading: ${percent}% (${(downloadedBytes / (1024 * 1024)).toFixed(2)} MB / ${(totalBytes / (1024 * 1024)).toFixed(2)} MB)`)
        }
      })
      
      response.pipe(file)
      
      file.on('finish', () => {
        file.close()
        if (totalBytes > 0) {
          console.log() // New line after progress
        }
        console.log('Download completed successfully')
        resolve()
      })
      
      file.on('error', (err) => {
        fs.unlinkSync(dest)
        reject(err)
      })
    }).on('error', (err) => {
      if (fs.existsSync(dest)) {
        fs.unlinkSync(dest)
      }
      reject(err)
    })
  })
}

// Main execution
(async () => {
  try {
    // Download
    await downloadFile(kuboUrl, tempFile)
    
    // Extract
    console.log('Extracting...')
    execSync(extractCommand(tempFile), { stdio: 'inherit' })
    
    // Copy binary - try multiple possible paths
    // 注意：解压后的文件名是 'ipfs' 或 'ipfs.exe'（不是平台特定的名称）
    const extractedBinaryName = platform === 'win32' ? 'ipfs.exe' : 'ipfs'
    const possiblePaths = [
      path.join(path.dirname(tempFile), 'kubo', 'kubo', extractedBinaryName),
      path.join(path.dirname(tempFile), 'kubo', extractedBinaryName),
      path.join(path.dirname(tempFile), extractedBinaryName),
    ]
    
    let binaryFound = false
    for (const extractedPath of possiblePaths) {
      if (fs.existsSync(extractedPath)) {
        fs.copyFileSync(extractedPath, targetPath)
        binaryFound = true
        console.log(`Binary copied from: ${extractedPath} to ${targetPath}`)
        break
      }
    }
    
    if (!binaryFound) {
      console.error(`Binary not found. Searched paths:`)
      possiblePaths.forEach(p => console.error(`  - ${p}`))
      process.exit(1)
    }
    
    // Set executable permission (Unix)
    if (platform !== 'win32') {
      fs.chmodSync(targetPath, '755')
      // 为了兼容性，如果是 macOS 或 Linux，同时创建一个通用的 'ipfs' 文件（如果不存在）
      // 这样旧代码或打包时可以使用通用文件名
      const genericPath = path.join(KUBO_DIR, 'ipfs')
      if (!fs.existsSync(genericPath)) {
        fs.copyFileSync(targetPath, genericPath)
        fs.chmodSync(genericPath, '755')
        console.log(`Also created generic 'ipfs' file for compatibility: ${genericPath}`)
      }
    }
    
    // Verify installation
    const stats = fs.statSync(targetPath)
    const fileSizeMB = (stats.size / (1024 * 1024)).toFixed(2)
    console.log(`✓ Kubo binary installed: ${targetPath}`)
    console.log(`  File size: ${fileSizeMB} MB`)
    
    // Cleanup
    if (fs.existsSync(tempFile)) {
      fs.unlinkSync(tempFile)
    }
    const extractedDir = path.join(path.dirname(tempFile), 'kubo')
    if (fs.existsSync(extractedDir)) {
      fs.rmSync(extractedDir, { recursive: true, force: true })
    }
    
    console.log('Setup completed successfully!')
  } catch (error) {
    console.error('Error:', error.message)
    if (fs.existsSync(tempFile)) {
      fs.unlinkSync(tempFile)
    }
    process.exit(1)
  }
})()

