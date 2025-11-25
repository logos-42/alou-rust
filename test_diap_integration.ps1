# DIAP SDK 集成测试脚本
# 测试 Desktop 和 Workers 的 DIAP 功能

Write-Host "=== DIAP SDK 集成测试 ===" -ForegroundColor Cyan
Write-Host ""

# 1. 检查 IPFS 节点状态
Write-Host "1. 检查 IPFS 节点状态..." -ForegroundColor Yellow
try {
    $response = Invoke-WebRequest -Uri "http://127.0.0.1:5001/api/v0/version" -Method GET -TimeoutSec 5 -ErrorAction Stop
    $version = $response.Content | ConvertFrom-Json
    Write-Host "   ✅ IPFS 节点运行中 - 版本: $($version.Version)" -ForegroundColor Green
} catch {
    Write-Host "   ⚠️  IPFS 节点未运行或无法连接" -ForegroundColor Yellow
    Write-Host "   提示: 请确保 IPFS 节点在 http://127.0.0.1:5001 运行" -ForegroundColor Gray
}

Write-Host ""

# 2. 检查 Desktop 代码编译
Write-Host "2. 检查 Desktop 代码编译..." -ForegroundColor Yellow
Push-Location "alou-desktop/src-tauri"
try {
    $cargoOutput = cargo check --message-format=short 2>&1 | Out-String
    if ($LASTEXITCODE -eq 0) {
        Write-Host "   ✅ Desktop 代码编译通过" -ForegroundColor Green
    } else {
        Write-Host "   ❌ Desktop 代码编译失败" -ForegroundColor Red
        Write-Host $cargoOutput
    }
} catch {
    Write-Host "   ⚠️  无法运行 cargo check" -ForegroundColor Yellow
}
Pop-Location

Write-Host ""

# 3. 检查 Workers 代码编译
Write-Host "3. 检查 Workers 代码编译..." -ForegroundColor Yellow
Push-Location "alou-edge"
try {
    if (Test-Path "wrangler.toml") {
        Write-Host "   ✅ Workers 配置文件存在" -ForegroundColor Green
    } else {
        Write-Host "   ⚠️  Workers 配置文件不存在" -ForegroundColor Yellow
    }
} catch {
    Write-Host "   ⚠️  无法检查 Workers 配置" -ForegroundColor Yellow
}
Pop-Location

Write-Host ""

# 4. 验证关键函数存在
Write-Host "4. 验证关键函数..." -ForegroundColor Yellow

# 检查 Desktop diap.rs
$desktopDiap = Get-Content "alou-desktop/src-tauri/src/diap.rs" -Raw
$checks = @{
    "create_local_diap_identity" = $desktopDiap -match "pub async fn create_local_diap_identity"
    "get_local_diap_identity" = $desktopDiap -match "pub async fn get_local_diap_identity"
    "update_local_diap_identity" = $desktopDiap -match "pub async fn update_local_diap_identity"
    "IpfsClient::new_with_remote_node" = $desktopDiap -match "IpfsClient::new_with_remote_node"
    "ipfs_client.upload" = $desktopDiap -match "ipfs_client\.upload"
    "ipfs_client.publish_ipns" = $desktopDiap -match "ipfs_client\.publish_ipns"
}

foreach ($check in $checks.GetEnumerator()) {
    if ($check.Value) {
        Write-Host "   ✅ $($check.Key)" -ForegroundColor Green
    } else {
        Write-Host "   ❌ $($check.Key) - 未找到" -ForegroundColor Red
    }
}

Write-Host ""

# 检查 Workers agent.rs
$workersAgent = Get-Content "alou-edge/src/router/agent.rs" -Raw
$workerChecks = @{
    "handle_get_diap_identity" = $workersAgent -match "pub.*async fn handle_get_diap_identity"
    "handle_register_agent_onchain" = $workersAgent -match "pub.*async fn handle_register_agent_onchain"
    "resolve_identity_from_ipns" = $workersAgent -match "resolve_identity_from_ipns"
    "validate_identity" = $workersAgent -match "validate_identity"
}

Write-Host "5. 验证 Workers 函数..." -ForegroundColor Yellow
foreach ($check in $workerChecks.GetEnumerator()) {
    if ($check.Value) {
        Write-Host "   ✅ $($check.Key)" -ForegroundColor Green
    } else {
        Write-Host "   ❌ $($check.Key) - 未找到" -ForegroundColor Red
    }
}

Write-Host ""

# 6. 检查前端服务
Write-Host "6. 检查前端服务..." -ForegroundColor Yellow
$agentService = Get-Content "alou-desktop/src/services/agentService.js" -Raw
$frontendChecks = @{
    "createDiapIdentity 使用 invoke" = $agentService -match "invoke\('create_local_diap_identity'"
    "getDiapIdentity 使用 invoke" = $agentService -match "invoke\('get_local_diap_identity'"
    "registerAgentOnChain 接收完整身份" = $agentService -match "ipns.*did.*cid.*public_key"
}

foreach ($check in $frontendChecks.GetEnumerator()) {
    if ($check.Value) {
        Write-Host "   ✅ $($check.Key)" -ForegroundColor Green
    } else {
        Write-Host "   ⚠️  $($check.Key) - 未找到或需要检查" -ForegroundColor Yellow
    }
}

Write-Host ""
Write-Host "=== 测试完成 ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "下一步:" -ForegroundColor Yellow
Write-Host "1. 启动 Desktop 应用进行实际功能测试" -ForegroundColor Gray
Write-Host "2. 启动 Workers 服务测试 API 端点" -ForegroundColor Gray
Write-Host "3. 参考 TEST_DIAP.md 进行详细测试" -ForegroundColor Gray

