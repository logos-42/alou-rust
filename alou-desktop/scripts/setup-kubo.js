#!/usr/bin/env node
/**
 * Cross-platform script to download and setup Kubo (IPFS) binary
 */

const https = require('https')
const fs = require('fs')
const path = require('path')
const { execSync } = require('child_process')
const os = require('os')

const KUBO_VERSION = 'v0.24.0'
const KUBO_DIR = path.join(__dirname, '..', 'src-tauri', 'kubo')

// Detect platform
const platform = os.platform()
const arch = os.arch()

let kuboFile, kuboUrl, extractCommand

if (platform === 'win32') {
  kuboFile = `kubo_${KUBO_VERSION}_windows-amd64.zip`
  kuboUrl = `https://dist.ipfs.tech/kubo/${KUBO_VERSION}/${kuboFile}`
  extractCommand = (file) => `powershell -Command "Expand-Archive -Path '${file}' -DestinationPath '${path.dirname(file)}' -Force"`
} else if (platform === 'darwin') {
  kuboFile = `kubo_${KUBO_VERSION}_darwin-amd64.tar.gz`
  kuboUrl = `https://dist.ipfs.tech/kubo/${KUBO_VERSION}/${kuboFile}`
  extractCommand = (file) => `tar -xzf "${file}" -C "${path.dirname(file)}"`
} else {
  kuboFile = `kubo_${KUBO_VERSION}_linux-amd64.tar.gz`
  kuboUrl = `https://dist.ipfs.tech/kubo/${KUBO_VERSION}/${kuboFile}`
  extractCommand = (file) => `tar -xzf "${file}" -C "${path.dirname(file)}"`
}

const binaryName = platform === 'win32' ? 'ipfs.exe' : 'ipfs'
const tempFile = path.join(os.tmpdir(), kuboFile)

// Create directory
if (!fs.existsSync(KUBO_DIR)) {
  fs.mkdirSync(KUBO_DIR, { recursive: true })
}

console.log(`Downloading Kubo ${KUBO_VERSION} for ${platform}...`)
console.log(`URL: ${kuboUrl}`)

// Download file
const file = fs.createWriteStream(tempFile)
https.get(kuboUrl, (response) => {
  if (response.statusCode !== 200) {
    console.error(`Failed to download: ${response.statusCode}`)
    process.exit(1)
  }
  response.pipe(file)
  file.on('finish', () => {
    file.close()
    console.log('Extracting...')
    
    // Extract
    try {
      execSync(extractCommand(tempFile), { stdio: 'inherit' })
      
      // Copy binary
      const extractedPath = path.join(
        path.dirname(tempFile),
        'kubo',
        binaryName
      )
      const targetPath = path.join(KUBO_DIR, binaryName)
      
      if (fs.existsSync(extractedPath)) {
        fs.copyFileSync(extractedPath, targetPath)
        
        // Set executable permission (Unix)
        if (platform !== 'win32') {
          fs.chmodSync(targetPath, '755')
        }
        
        console.log(`✓ Kubo binary installed: ${targetPath}`)
      } else {
        console.error(`Binary not found at: ${extractedPath}`)
        process.exit(1)
      }
      
      // Cleanup
      fs.unlinkSync(tempFile)
      const extractedDir = path.join(path.dirname(tempFile), 'kubo')
      if (fs.existsSync(extractedDir)) {
        fs.rmSync(extractedDir, { recursive: true, force: true })
      }
    } catch (error) {
      console.error('Extraction failed:', error)
      process.exit(1)
    }
  })
}).on('error', (err) => {
  console.error('Download failed:', err)
  process.exit(1)
})

