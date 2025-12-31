# Windows PowerShell 测试脚本
# Alou Edge API 测试

$WORKER_URL = "https://alou-edge.yuanjieliu65.workers.dev"

Write-Host "🚀 测试 Alou Edge API" -ForegroundColor Green
Write-Host "📍 Worker URL: $WORKER_URL" -ForegroundColor Cyan
Write-Host ""

# 1. 健康检查
Write-Host "1️⃣ 健康检查" -ForegroundColor Yellow
try {
    $response = Invoke-RestMethod -Uri "$WORKER_URL/api/health" -Method GET
    $response.Content | ConvertFrom-Json | ConvertTo-Json
} catch {
    Write-Host "健康检查失败: $($_.Exception.Message)" -ForegroundColor Red
}
Write-Host ""

# 2. 创建会话
Write-Host "2️⃣ 创建会话" -ForegroundColor Yellow
try {
    $sessionBody = @{
        wallet_address = "0x1234567890123456789012345678901234567890"
        chain = "ethereum"
    } | ConvertTo-Json -Depth 10
    
    $sessionResponse = Invoke-RestMethod -Uri "$WORKER_URL/api/session" `
        -Method POST -ContentType "application/json" -Body $sessionBody
    $sessionData = $sessionResponse.Content | ConvertFrom-Json
    
    Write-Host "会话创建成功:" -ForegroundColor Green
    $sessionData | ConvertTo-Json -Depth 5
    $SESSION_ID = $sessionData.session_id
    Write-Host "Session ID: $SESSION_ID" -ForegroundColor Cyan
} catch {
    Write-Host "创建会话失败: $($_.Exception.Message)" -ForegroundColor Red
}
Write-Host ""

# 3. 创建智能体
if ($SESSION_ID) {
    Write-Host "3️⃣ 创建智能体" -ForegroundColor Yellow
    try {
        $agentBody = @{
            session_id = $SESSION_ID
            wallet_address = "0x1234567890123456789012345678901234567890"
            chain = "ethereum"
            name = "测试智能体"
            avatar_cid = "QmTest123"
            role_description = "这是一个测试智能体，用于验证API功能"
        } | ConvertTo-Json -Depth 10
        
        $agentResponse = Invoke-RestMethod -Uri "$WORKER_URL/api/agent/create" `
            -Method POST -ContentType "application/json" -Body $agentBody
        $agentData = $agentResponse.Content | ConvertFrom-Json
        
        Write-Host "智能体创建成功:" -ForegroundColor Green
        $agentData | ConvertTo-Json -Depth 5
    } catch {
        Write-Host "创建智能体失败: $($_.Exception.Message)" -ForegroundColor Red
    }
}
Write-Host ""

# 4. 搜索智能体
Write-Host "4️⃣ 搜索智能体" -ForegroundColor Yellow
try {
    $searchBody = @{
        query = "测试"
    } | ConvertTo-Json -Depth 10
    
    $searchResponse = Invoke-RestMethod -Uri "$WORKER_URL/api/agent/search" `
        -Method POST -ContentType "application/json" -Body $searchBody
    $searchData = $searchResponse.Content | ConvertFrom-Json
    
    Write-Host "搜索结果:" -ForegroundColor Green
    $searchData | ConvertTo-Json -Depth 5
} catch {
    Write-Host "搜索智能体失败: $($_.Exception.Message)" -ForegroundColor Red
}
Write-Host ""

# 5. 与智能体对话
if ($SESSION_ID) {
    Write-Host "5️⃣ 与智能体对话" -ForegroundColor Yellow
    try {
        $chatBody = @{
            session_id = $SESSION_ID
            message = "你好，请介绍一下你自己和你的功能"
        } | ConvertTo-Json -Depth 10
        
        $chatResponse = Invoke-RestMethod -Uri "$WORKER_URL/api/agent/chat" `
            -Method POST -ContentType "application/json" -Body $chatBody
        $chatData = $chatResponse.Content | ConvertFrom-Json
        
        Write-Host "对话结果:" -ForegroundColor Green
        $chatData | ConvertTo-Json -Depth 5
    } catch {
        Write-Host "对话失败: $($_.Exception.Message)" -ForegroundColor Red
    }
}
Write-Host ""

# 6. 获取DIAP身份
if ($SESSION_ID) {
    Write-Host "6️⃣ 获取DIAP身份" -ForegroundColor Yellow
    try {
        $diapResponse = Invoke-RestMethod -Uri "$WORKER_URL/api/agent/diap-identity?session_id=$SESSION_ID" -Method GET
        $diapData = $diapResponse.Content | ConvertFrom-Json
        
        Write-Host "DIAP身份信息:" -ForegroundColor Green
        $diapData | ConvertTo-Json -Depth 5
    } catch {
        Write-Host "获取DIAP身份失败: $($_.Exception.Message)" -ForegroundColor Red
    }
}
Write-Host ""

Write-Host "✅ PowerShell API 测试完成！" -ForegroundColor Green
Write-Host ""
Write-Host "💡 提示：" -ForegroundColor Cyan
Write-Host "- 某些API可能需要有效的钱包签名" -ForegroundColor White
Write-Host "- AI功能需要设置AI_API_KEY密钥" -ForegroundColor White
Write-Host "- DIAP功能需要本地IPFS节点运行" -ForegroundColor White
Write-Host "- 如果遇到连接问题，请检查Worker状态和网络连接" -ForegroundColor White