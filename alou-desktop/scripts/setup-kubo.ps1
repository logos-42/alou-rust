# PowerShell script to download and setup Kubo (IPFS) binary for Windows

$KUBO_VERSION = "v0.24.0"
$KUBO_DIR = "src-tauri\kubo"
$KUBO_FILE = "kubo_${KUBO_VERSION}_windows-amd64.zip"
$KUBO_URL = "https://dist.ipfs.tech/kubo/${KUBO_VERSION}/${KUBO_FILE}"

# Create directory
New-Item -ItemType Directory -Force -Path $KUBO_DIR | Out-Null

# Download Kubo
Write-Host "Downloading Kubo ${KUBO_VERSION} for Windows..."
$tempFile = "$env:TEMP\$KUBO_FILE"
Invoke-WebRequest -Uri $KUBO_URL -OutFile $tempFile

# Extract
Write-Host "Extracting Kubo..."
Expand-Archive -Path $tempFile -DestinationPath "$env:TEMP\kubo" -Force

# Copy binary
Write-Host "Copying binary to ${KUBO_DIR}..."
Copy-Item "$env:TEMP\kubo\kubo\ipfs.exe" -Destination "$KUBO_DIR\ipfs.exe" -Force

# Cleanup
Remove-Item $tempFile -Force
Remove-Item "$env:TEMP\kubo" -Recurse -Force

Write-Host "Kubo setup complete!"
Write-Host "Binary location: ${KUBO_DIR}\ipfs.exe"

