# Windows部署测试脚本
Write-Host "Testing deployment configuration..." -ForegroundColor Green

# 检查项目结构
Write-Host "Checking project structure..."
if (-not (Test-Path "Cargo.toml")) {
    Write-Host "ERROR: Cargo.toml not found" -ForegroundColor Red
    exit 1
}

if (-not (Test-Path "frontend")) {
    Write-Host "ERROR: frontend directory not found" -ForegroundColor Red
    exit 1
}

Write-Host "OK: Project structure is correct" -ForegroundColor Green

# 测试后端构建
Write-Host "Testing backend build..."
$buildResult = cargo build --release --bin agent-http-server 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "OK: Backend build successful" -ForegroundColor Green
} else {
    Write-Host "ERROR: Backend build failed" -ForegroundColor Red
    Write-Host $buildResult
    exit 1
}

# 检查构建产物
Write-Host "Checking build artifacts..."
$backendBinary = "target/release/agent-http-server.exe"
if (Test-Path $backendBinary) {
    Write-Host "OK: Backend binary found" -ForegroundColor Green
} else {
    Write-Host "ERROR: Backend binary not found" -ForegroundColor Red
}

$frontendDist = "frontend/dist"
if (Test-Path $frontendDist) {
    Write-Host "OK: Frontend build directory found" -ForegroundColor Green
} else {
    Write-Host "ERROR: Frontend build directory not found" -ForegroundColor Red
}

Write-Host ""
Write-Host "=== DEPLOYMENT SUMMARY ===" -ForegroundColor Cyan
Write-Host "Target Server: 140.179.186.11"
Write-Host "Backend Port: 3001"
Write-Host "Frontend Port: 80 (via Nginx)"
Write-Host ""
Write-Host "Ready for cloud deployment!" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:"
Write-Host "1. Upload project to server"
Write-Host "2. Run: chmod +x deploy/cloud_deploy.sh"
Write-Host "3. Set: export DEEPSEEK_API_KEY='your-key'"
Write-Host "4. Deploy: ./deploy/cloud_deploy.sh"
