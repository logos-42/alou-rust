# 测试不同的 Prompt 模式

function Test-Prompt {
    param($message, $description)
    
    Write-Host "`n=== $description ===" -ForegroundColor Cyan
    Write-Host "消息: $message" -ForegroundColor Yellow
    
    $sessionResponse = Invoke-RestMethod -Uri "http://127.0.0.1:8787/api/session" -Method POST -ContentType "application/json" -Body '{}'
    $sessionId = $sessionResponse.session_id
    
    $body = @{
        session_id = $sessionId
        message = $message
    } | ConvertTo-Json
    
    try {
        $response = Invoke-RestMethod -Uri "http://127.0.0.1:8787/api/agent/chat" -Method POST -ContentType "application/json" -Body $body
        Write-Host "AI 回复:" -ForegroundColor Green
        Write-Host $response.content
    } catch {
        Write-Host "错误: $_" -ForegroundColor Red
    }
}

# 测试不同场景
Test-Prompt "查询我的以太坊余额" "钱包模式"
Test-Prompt "我想兑换一些代币" "DeFi模式"
Test-Prompt "如何铸造NFT" "NFT模式"
Test-Prompt "我要支付0.1 ETH" "支付模式"
Test-Prompt "如何调用智能合约" "开发者模式"
Test-Prompt "什么是区块链" "通用模式"

Write-Host "`n=== 测试完成 ===" -ForegroundColor Green
