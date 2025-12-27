# ============================================
# 部署 alou-edge 到 Cloudflare Workers (Windows)
# ============================================

Write-Host "=== 部署 alou-edge 到 Cloudflare Workers ===" -ForegroundColor Cyan

# 检查 Node.js
Write-Host "检查 Node.js..." -ForegroundColor Yellow
try {
    $nodeVersion = node --version
    Write-Host "✓ Node.js 已安装: $nodeVersion" -ForegroundColor Green
} catch {
    Write-Host "✗ Node.js 未安装，请先安装 Node.js" -ForegroundColor Red
    Write-Host "下载地址: https://nodejs.org/" -ForegroundColor Yellow
    exit 1
}

# 检查 npm
Write-Host "检查 npm..." -ForegroundColor Yellow
try {
    $npmVersion = npm --version
    Write-Host "✓ npm 已安装: $npmVersion" -ForegroundColor Green
} catch {
    Write-Host "✗ npm 未安装" -ForegroundColor Red
    exit 1
}

# 检查 Rust
Write-Host "检查 Rust..." -ForegroundColor Yellow
try {
    $rustcVersion = rustc --version
    Write-Host "✓ Rust 已安装: $rustcVersion" -ForegroundColor Green
} catch {
    Write-Host "✗ Rust 未安装，请先安装 Rust" -ForegroundColor Red
    Write-Host "下载地址: https://rustup.rs/" -ForegroundColor Yellow
    exit 1
}

# 检查 wasm32 目标
Write-Host "检查 wasm32 目标..." -ForegroundColor Yellow
$rustupTargets = rustup target list --installed
if ($rustupTargets -match "wasm32-unknown-unknown") {
    Write-Host "✓ wasm32-unknown-unknown 目标已安装" -ForegroundColor Green
} else {
    Write-Host "安装 wasm32-unknown-unknown 目标..." -ForegroundColor Yellow
    rustup target add wasm32-unknown-unknown
}

# 检查 worker-build
Write-Host "检查 worker-build..." -ForegroundColor Yellow
try {
    $workerBuildVersion = worker-build --version
    Write-Host "✓ worker-build 已安装: $workerBuildVersion" -ForegroundColor Green
} catch {
    Write-Host "安装 worker-build..." -ForegroundColor Yellow
    cargo install worker-build --force
}

# 检查 wrangler
Write-Host "检查 wrangler..." -ForegroundColor Yellow
try {
    $wranglerVersion = wrangler --version
    Write-Host "✓ wrangler 已安装: $wranglerVersion" -ForegroundColor Green
} catch {
    Write-Host "安装 wrangler..." -ForegroundColor Yellow
    npm install -g wrangler
}

# 检查 Cloudflare 登录状态
Write-Host "检查 Cloudflare 登录状态..." -ForegroundColor Yellow
try {
    $whoami = wrangler whoami
    Write-Host "✓ 已登录 Cloudflare: $whoami" -ForegroundColor Green
} catch {
    Write-Host "✗ 未登录 Cloudflare" -ForegroundColor Red
    Write-Host "请先登录: wrangler login" -ForegroundColor Yellow
    exit 1
}

# 清理之前的构建
Write-Host "清理之前的构建..." -ForegroundColor Yellow
cargo clean

# 构建 WASM
Write-Host "构建 WASM..." -ForegroundColor Yellow
cargo build --target wasm32-unknown-unknown --release

# 检查 WASM 文件
$wasmFile = "target\wasm32-unknown-unknown\release\alou_edge.wasm"
if (Test-Path $wasmFile) {
    $size = (Get-Item $wasmFile).Length / 1KB
    Write-Host "✓ WASM 文件构建成功: $([math]::Round($size, 2)) KB" -ForegroundColor Green
} else {
    Write-Host "✗ WASM 文件构建失败" -ForegroundColor Red
    exit 1
}

# 使用 worker-build 生成 JavaScript 绑定
Write-Host "生成 JavaScript 绑定..." -ForegroundColor Yellow
worker-build --release

# 检查生成的文件
if (Test-Path "build\worker\shim.mjs") {
    Write-Host "✓ JavaScript 绑定生成成功" -ForegroundColor Green
} else {
    Write-Host "✗ JavaScript 绑定生成失败" -ForegroundColor Red
    exit 1
}

# 检查 secrets
Write-Host "检查必要的 secrets..." -ForegroundColor Yellow
$secrets = wrangler secret list 2>&1
if ($secrets -match "AI_API_KEY") {
    Write-Host "✓ AI_API_KEY 已配置" -ForegroundColor Green
} else {
    Write-Host "⚠ AI_API_KEY 未配置，请运行: wrangler secret put AI_API_KEY" -ForegroundColor Yellow
}

if ($secrets -match "JWT_SECRET") {
    Write-Host "✓ JWT_SECRET 已配置" -ForegroundColor Green
} else {
    Write-Host "⚠ JWT_SECRET 未配置，请运行: wrangler secret put JWT_SECRET" -ForegroundColor Yellow
}

# 部署到 Cloudflare Workers
Write-Host "部署到 Cloudflare Workers..." -ForegroundColor Yellow
wrangler deploy

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "=== 部署成功! ===" -ForegroundColor Green
    Write-Host ""
    Write-Host "Worker URL: https://alou-edge.yuanjieliu65.workers.dev" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "测试部署:" -ForegroundColor Yellow
    Write-Host "curl https://alou-edge.yuanjieliu65.workers.dev/api/health" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "查看日志:" -ForegroundColor Yellow
    Write-Host "wrangler tail" -ForegroundColor Cyan
} else {
    Write-Host "✗ 部署失败" -ForegroundColor Red
    exit 1
}

