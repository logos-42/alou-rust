# 测试区块链工具调用 - 更明确的请求

# 1. 创建会话
Write-Host "=== 创建会话 ===" -ForegroundColor Green
$sessionResponse = Invoke-RestMethod -Uri "http://127.0.0.1:8787/api/session" -Method POST -ContentType "application/json" -Body '{}'
$sessionId = $sessionResponse.session_id
Write-Host "Session ID: $sessionId"

# 2. 测试查询以太坊余额 - 更明确的请求
Write-Host "`n=== 测试查询以太坊余额 (明确请求) ===" -ForegroundColor Green
$message1 = @{
    session_id = $sessionId
    message = "使用query_blockchain工具查询地址0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb的ETH余额"
} | ConvertTo-Json

$response1 = Invoke-RestMethod -Uri "http://127.0.0.1:8787/api/agent/chat" -Method POST -ContentType "application/json" -Body $message1
Write-Host "AI Response:"
Write-Host $response1.content
if ($response1.tool_calls) {
    Write-Host "`nTool Calls:" -ForegroundColor Yellow
    $response1.tool_calls | ForEach-Object {
        Write-Host "  - $($_.name): $($_.id)"
    }
}

Write-Host "`n=== 测试完成 ===" -ForegroundColor Green
