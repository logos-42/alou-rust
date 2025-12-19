# PowerShell script to download and setup Kubo (IPFS) binary for Windows

$KUBO_VERSION = "v0.39.0"
$KUBO_DIR = "src-tauri\kubo"
$KUBO_FILE = "kubo_${KUBO_VERSION}_windows-amd64.zip"
$KUBO_URL = "https://dist.ipfs.tech/kubo/${KUBO_VERSION}/${KUBO_FILE}"

# Create directory
New-Item -ItemType Directory -Force -Path $KUBO_DIR | Out-Null

# Clean up any existing temp files from previous runs
Write-Host "Cleaning up previous download attempts..."
$tempFile = "$env:TEMP\$KUBO_FILE"
$tempExtractDir = "$env:TEMP\kubo"
if (Test-Path $tempFile) {
    try {
        Remove-Item $tempFile -Force -ErrorAction SilentlyContinue
    } catch {
        Write-Warning "Could not remove existing temp file, will use unique name"
        $tempFile = "$env:TEMP\kubo_${KUBO_VERSION}_windows-amd64_$([System.Guid]::NewGuid().ToString('N').Substring(0,8)).zip"
    }
}
if (Test-Path $tempExtractDir) {
    Remove-Item $tempExtractDir -Recurse -Force -ErrorAction SilentlyContinue
}

# Download Kubo
Write-Host "Downloading Kubo ${KUBO_VERSION} for Windows..."
Write-Host "URL: $KUBO_URL"
try {
    Invoke-WebRequest -Uri $KUBO_URL -OutFile $tempFile -ErrorAction Stop
    Write-Host "Download completed successfully"
} catch {
    Write-Error "Failed to download Kubo: $_"
    exit 1
}

# Extract
Write-Host "Extracting Kubo..."
try {
    Expand-Archive -Path $tempFile -DestinationPath "$env:TEMP\kubo" -Force -ErrorAction Stop
    Write-Host "Extraction completed successfully"
} catch {
    Write-Error "Failed to extract archive: $_"
    Remove-Item $tempFile -Force -ErrorAction SilentlyContinue
    exit 1
}

# Copy binary - try multiple possible paths
Write-Host "Copying binary to ${KUBO_DIR}..."
$possiblePaths = @(
    "$env:TEMP\kubo\kubo\ipfs.exe",
    "$env:TEMP\kubo\ipfs.exe",
    "$env:TEMP\ipfs.exe"
)

$binaryFound = $false
foreach ($possiblePath in $possiblePaths) {
    if (Test-Path $possiblePath) {
        Copy-Item $possiblePath -Destination "$KUBO_DIR\ipfs.exe" -Force -ErrorAction Stop
        Write-Host "Binary copied from: $possiblePath"
        $binaryFound = $true
        break
    }
}

if (-not $binaryFound) {
    Write-Error "Binary not found in extracted archive. Searched paths:"
    foreach ($path in $possiblePaths) {
        Write-Host "  - $path"
    }
    Remove-Item $tempFile -Force -ErrorAction SilentlyContinue
    Remove-Item "$env:TEMP\kubo" -Recurse -Force -ErrorAction SilentlyContinue
    exit 1
}

# Cleanup
Write-Host "Cleaning up temporary files..."
Remove-Item $tempFile -Force -ErrorAction SilentlyContinue
Remove-Item "$env:TEMP\kubo" -Recurse -Force -ErrorAction SilentlyContinue

# Verify installation
if (Test-Path "$KUBO_DIR\ipfs.exe") {
    $fileSize = (Get-Item "$KUBO_DIR\ipfs.exe").Length / 1MB
    Write-Host "✓ Kubo setup complete!"
    Write-Host "Binary location: ${KUBO_DIR}\ipfs.exe"
    Write-Host "File size: $([math]::Round($fileSize, 2)) MB"
} else {
    Write-Error "Installation verification failed - binary not found at destination"
    exit 1
}

