# Durable Objects部署脚本
# 用于部署和验证Durable Objects架构

param(
    [string]$Environment = "development",
    [switch]$TestOnly = $false,
    [switch]$DeployOnly = $false
)

Write-Host "=== Durable Objects部署脚本 ===" -ForegroundColor Cyan
Write-Host "环境: $Environment" -ForegroundColor Yellow

# 设置错误处理
$ErrorActionPreference = "Stop"

# 检查wrangler是否安装
function Test-Wrangler {
    try {
        $null = wrangler --version
        return $true
    }
    catch {
        return $false
    }
}

# 运行测试
function Run-Tests {
    Write-Host "`n=== 运行测试 ===" -ForegroundColor Green
    
    # 运行Rust测试
    Write-Host "运行Rust测试..." -ForegroundColor Yellow
    cargo test --tests
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Rust测试失败!" -ForegroundColor Red
        exit 1
    }
    
    # 运行兼容性测试
    Write-Host "运行兼容性测试..." -ForegroundColor Yellow
    cargo test compatibility_test
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "兼容性测试失败!" -ForegroundColor Red
        exit 1
    }
    
    # 运行Durable Objects测试
    Write-Host "运行Durable Objects测试..." -ForegroundColor Yellow
    cargo test durable_objects_test
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Durable Objects测试失败!" -ForegroundColor Red
        exit 1
    }
    
    Write-Host "所有测试通过!" -ForegroundColor Green
}

# 构建项目
function Build-Project {
    Write-Host "`n=== 构建项目 ===" -ForegroundColor Green
    
    # 清理之前的构建
    Write-Host "清理构建..." -ForegroundColor Yellow
    Remove-Item -Path "build" -Recurse -Force -ErrorAction SilentlyContinue
    
    # 构建Rust WASM
    Write-Host "构建Rust WASM..." -ForegroundColor Yellow
    cargo build --release --target wasm32-unknown-unknown
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Rust构建失败!" -ForegroundColor Red
        exit 1
    }
    
    # 构建TypeScript
    Write-Host "构建TypeScript..." -ForegroundColor Yellow
    npm run build
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "TypeScript构建失败!" -ForegroundColor Red
        exit 1
    }
    
    Write-Host "构建完成!" -ForegroundColor Green
}

# 部署到Cloudflare
function Deploy-To-Cloudflare {
    param(
        [string]$Env
    )
    
    Write-Host "`n=== 部署到Cloudflare ===" -ForegroundColor Green
    Write-Host "部署环境: $Env" -ForegroundColor Yellow
    
    # 检查wrangler配置
    if (-not (Test-Path "wrangler.toml")) {
        Write-Host "找不到wrangler.toml文件!" -ForegroundColor Red
        exit 1
    }
    
    # 部署到指定环境
    if ($Env -eq "production") {
        Write-Host "部署到生产环境..." -ForegroundColor Yellow
        wrangler deploy --env production
    }
    elseif ($Env -eq "staging") {
        Write-Host "部署到预发布环境..." -ForegroundColor Yellow
        wrangler deploy --env staging
    }
    else {
        Write-Host "部署到开发环境..." -ForegroundColor Yellow
        wrangler deploy
    }
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "部署失败!" -ForegroundColor Red
        exit 1
    }
    
    Write-Host "部署成功!" -ForegroundColor Green
}

# 验证部署
function Verify-Deployment {
    param(
        [string]$Env
    )
    
    Write-Host "`n=== 验证部署 ===" -ForegroundColor Green
    
    # 获取部署URL
    $deploymentUrl = if ($Env -eq "production") {
        "https://alou-edge-prod.YOUR_WORKER_SUBDOMAIN.workers.dev"
    }
    elseif ($Env -eq "staging") {
        "https://alou-edge-staging.YOUR_WORKER_SUBDOMAIN.workers.dev"
    }
    else {
        "https://alou-edge.YOUR_WORKER_SUBDOMAIN.workers.dev"
    }
    
    Write-Host "部署URL: $deploymentUrl" -ForegroundColor Yellow
    
    # 测试健康检查
    Write-Host "测试健康检查..." -ForegroundColor Yellow
    try {
        $healthResponse = Invoke-RestMethod -Uri "$deploymentUrl/api/health" -Method Get
        if ($healthResponse.status -eq "healthy") {
            Write-Host "健康检查通过!" -ForegroundColor Green
        }
        else {
            Write-Host "健康检查失败: $($healthResponse | ConvertTo-Json)" -ForegroundColor Red
            exit 1
        }
    }
    catch {
        Write-Host "健康检查请求失败: $_" -ForegroundColor Red
        exit 1
    }
    
    # 测试状态检查
    Write-Host "测试状态检查..." -ForegroundColor Yellow
    try {
        $statusResponse = Invoke-RestMethod -Uri "$deploymentUrl/api/status" -Method Get
        if ($statusResponse.status -eq "operational") {
            Write-Host "状态检查通过!" -ForegroundColor Green
        }
        else {
            Write-Host "状态检查失败: $($statusResponse | ConvertTo-Json)" -ForegroundColor Red
            exit 1
        }
    }
    catch {
        Write-Host "状态检查请求失败: $_" -ForegroundColor Red
        exit 1
    }
    
    # 测试兼容性API
    Write-Host "测试兼容性API..." -ForegroundColor Yellow
    try {
        $testRequest = @{
            prompt = "Hello, this is a test"
            model = "deepseek-chat"
            taskType = "sync"
        } | ConvertTo-Json
        
        $compatResponse = Invoke-RestMethod -Uri "$deploymentUrl/api/agent/chat" -Method Post -Body $testRequest -ContentType "application/json"
        
        if ($compatResponse.success) {
            Write-Host "兼容性API测试通过!" -ForegroundColor Green
        }
        else {
            Write-Host "兼容性API测试失败: $($compatResponse | ConvertTo-Json)" -ForegroundColor Red
            exit 1
        }
    }
    catch {
        Write-Host "兼容性API测试请求失败: $_" -ForegroundColor Red
        exit 1
    }
    
    Write-Host "所有验证通过!" -ForegroundColor Green
}

# 主执行流程
try {
    # 检查wrangler
    if (-not (Test-Wrangler)) {
        Write-Host "wrangler未安装，请先安装: npm install -g wrangler" -ForegroundColor Red
        exit 1
    }
    
    # 切换到项目目录
    $originalDir = Get-Location
    Set-Location $PSScriptRoot/..
    
    # 运行测试（除非指定了只部署）
    if (-not $DeployOnly) {
        Run-Tests
    }
    
    # 构建项目（除非指定了只测试）
    if (-not $TestOnly) {
        Build-Project
        
        # 部署到Cloudflare
        Deploy-To-Cloudflare -Env $Environment
        
        # 验证部署
        Verify-Deployment -Env $Environment
    }
    
    Write-Host "`n=== 部署流程完成 ===" -ForegroundColor Cyan
    
    # 返回原始目录
    Set-Location $originalDir
}
catch {
    Write-Host "`n部署过程中发生错误: $_" -ForegroundColor Red
    Write-Host "堆栈跟踪: $($_.ScriptStackTrace)" -ForegroundColor Red
    exit 1
}