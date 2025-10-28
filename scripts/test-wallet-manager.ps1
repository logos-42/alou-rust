# 钱包管理工具测试脚本 (PowerShell)
# 用于测试智能体的钱包管理功能

$API_URL = if ($env:API_URL) { $env:API_URL } else { "http://localhost:8787" }
$SESSION_ID = "test_$(Get-Date -Format 'yyyyMMddHHmmss')"
$WALLET_ADDRESS = "0x1234567890123456789012345678901234567890"

Write-Host "🧪 测试钱包管理工具" -ForegroundColor Cyan
Write-Host "====================" -ForegroundColor Cyan
Write-Host ""

# 创建会话
Write-Host "📝 创建测试会话..." -ForegroundColor Yellow
$sessionBody = @{
    wallet_address = $WALLET_ADDRESS
} | ConvertTo-Json

try {
    $response = Invoke-RestMethod -Uri "$API_URL/api/session" -Method Post -Body $sessionBody -ContentType "application/json"
    Write-Host "✅ 会话创建成功" -ForegroundColor Green
    Write-Host ""
} catch {
    Write-Host "❌ 会话创建失败: $_" -ForegroundColor Red
    exit 1
}

Write-Host "---" -ForegroundColor Gray
Write-Host ""

# 测试 1: 列出支持的网络
Write-Host "✅ 测试 1: 列出支持的网络" -ForegroundColor Green
$body1 = @{
    session_id = $SESSION_ID
    message = "请列出所有支持的区块链网络"
    wallet_address = $WALLET_ADDRESS
} | ConvertTo-Json

try {
    $response1 = Invoke-RestMethod -Uri "$API_URL/api/agent/chat" -Method Post -Body $body1 -ContentType "application/json"
    Write-Host $response1.content -ForegroundColor White
} catch {
    Write-Host "❌ 测试失败: $_" -ForegroundColor Red
}

Write-Host ""
Write-Host "---" -ForegroundColor Gray
Write-Host ""

# 测试 2: 切换到 Base Sepolia
Write-Host "✅ 测试 2: 切换到 Base Sepolia" -ForegroundColor Green
$body2 = @{
    session_id = $SESSION_ID
    message = "请帮我切换到 Base Sepolia 测试网"
    wallet_address = $WALLET_ADDRESS
} | ConvertTo-Json

try {
    $response2 = Invoke-RestMethod -Uri "$API_URL/api/agent/chat" -Method Post -Body $body2 -ContentType "application/json"
    Write-Host "响应: $($response2.content)" -ForegroundColor White
    if ($response2.tool_calls) {
        Write-Host "工具调用: $($response2.tool_calls | ConvertTo-Json -Depth 5)" -ForegroundColor Cyan
    }
} catch {
    Write-Host "❌ 测试失败: $_" -ForegroundColor Red
}

Write-Host ""
Write-Host "---" -ForegroundColor Gray
Write-Host ""

# 测试 3: 查询当前网络
Write-Host "✅ 测试 3: 查询当前网络" -ForegroundColor Green
$body3 = @{
    session_id = $SESSION_ID
    message = "我现在在哪个网络？"
    wallet_address = $WALLET_ADDRESS
} | ConvertTo-Json

try {
    $response3 = Invoke-RestMethod -Uri "$API_URL/api/agent/chat" -Method Post -Body $body3 -ContentType "application/json"
    Write-Host $response3.content -ForegroundColor White
} catch {
    Write-Host "❌ 测试失败: $_" -ForegroundColor Red
}

Write-Host ""
Write-Host "---" -ForegroundColor Gray
Write-Host ""

# 测试 4: 切换到以太坊主网
Write-Host "✅ 测试 4: 切换到以太坊主网" -ForegroundColor Green
$body4 = @{
    session_id = $SESSION_ID
    message = "切换到以太坊主网"
    wallet_address = $WALLET_ADDRESS
} | ConvertTo-Json

try {
    $response4 = Invoke-RestMethod -Uri "$API_URL/api/agent/chat" -Method Post -Body $body4 -ContentType "application/json"
    Write-Host "响应: $($response4.content)" -ForegroundColor White
    if ($response4.tool_calls) {
        Write-Host "工具调用: $($response4.tool_calls | ConvertTo-Json -Depth 5)" -ForegroundColor Cyan
    }
} catch {
    Write-Host "❌ 测试失败: $_" -ForegroundColor Red
}

Write-Host ""
Write-Host "---" -ForegroundColor Gray
Write-Host ""

# 测试 5: 检查余额
Write-Host "✅ 测试 5: 检查钱包余额" -ForegroundColor Green
$body5 = @{
    session_id = $SESSION_ID
    message = "查看我的钱包余额"
    wallet_address = $WALLET_ADDRESS
} | ConvertTo-Json

try {
    $response5 = Invoke-RestMethod -Uri "$API_URL/api/agent/chat" -Method Post -Body $body5 -ContentType "application/json"
    Write-Host $response5.content -ForegroundColor White
} catch {
    Write-Host "❌ 测试失败: $_" -ForegroundColor Red
}

Write-Host ""
Write-Host "====================" -ForegroundColor Cyan
Write-Host "✅ 测试完成！" -ForegroundColor Green
