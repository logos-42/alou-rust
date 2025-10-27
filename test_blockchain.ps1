# 测试区块链工具调用

# 1. 创建会话
Write-Host "=== 创建会话 ===" -ForegroundColor Green
$sessionResponse = Invoke-RestMethod -Uri "http://127.0.0.1:8787/api/session" -Method POST -ContentType "application/json" -Body '{}'
$sessionId = $sessionResponse.session_id
Write-Host "Session ID: $sessionId"

# 2. 测试查询以太坊余额
Write-Host "`n=== 测试查询以太坊余额 ===" -ForegroundColor Green
$message1 = @{
    session_id = $sessionId
    message = "请帮我查询这个以太坊地址的余额：0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"
} | ConvertTo-Json

$response1 = Invoke-RestMethod -Uri "http://127.0.0.1:8787/api/agent/chat" -Method POST -ContentType "application/json" -Body $message1
Write-Host "AI Response:"
Write-Host $response1.content

# 3. 测试查询Solana余额
Write-Host "`n=== 测试查询Solana余额 ===" -ForegroundColor Green
$message2 = @{
    session_id = $sessionId
    message = "请查询这个Solana地址的余额：7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU"
} | ConvertTo-Json

$response2 = Invoke-RestMethod -Uri "http://127.0.0.1:8787/api/agent/chat" -Method POST -ContentType "application/json" -Body $message2
Write-Host "AI Response:"
Write-Host $response2.content

Write-Host "`n=== 测试完成 ===" -ForegroundColor Green
