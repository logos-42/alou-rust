# 详细测试后端错误
Write-Host "=== 测试后端详细错误 ===" -ForegroundColor Cyan

# 1. 测试健康检查
Write-Host "`n1. 测试健康检查..." -ForegroundColor Yellow
try {
    $health = Invoke-RestMethod -Uri "https://alou-edge.yuanjieliu65.workers.dev/api/health"
    Write-Host "健康检查成功: $($health.status)" -ForegroundColor Green
} catch {
    Write-Host "健康检查失败: $_" -ForegroundColor Red
}

# 2. 创建会话
Write-Host "`n2. 创建会话..." -ForegroundColor Yellow
try {
    $session = Invoke-RestMethod -Uri "https://alou-edge.yuanjieliu65.workers.dev/api/session" -Method POST -ContentType "application/json" -Body '{}'
    Write-Host "会话创建成功: $($session.session_id)" -ForegroundColor Green
    $sessionId = $session.session_id
} catch {
    Write-Host "会话创建失败: $_" -ForegroundColor Red
    exit 1
}

# 3. 发送消息
Write-Host "`n3. 发送消息..." -ForegroundColor Yellow
$body = @{
    session_id = $sessionId
    message = "你好"
} | ConvertTo-Json

Write-Host "请求体: $body" -ForegroundColor Cyan

try {
    $response = Invoke-WebRequest -Uri "https://alou-edge.yuanjieliu65.workers.dev/api/agent/chat" -Method POST -ContentType "application/json" -Body $body -UseBasicParsing
    Write-Host "成功: $($response.StatusCode)" -ForegroundColor Green
    $content = $response.Content | ConvertFrom-Json
    Write-Host "AI 回复: $($content.content)" -ForegroundColor White
} catch {
    Write-Host "失败: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "状态码: $($_.Exception.Response.StatusCode.value__)" -ForegroundColor Yellow
    
    if ($_.Exception.Response) {
        $stream = $_.Exception.Response.GetResponseStream()
        $reader = New-Object System.IO.StreamReader($stream)
        $errorBody = $reader.ReadToEnd()
        Write-Host "错误详情: $errorBody" -ForegroundColor Yellow
    }
}

Write-Host "`n=== 测试完成 ===" -ForegroundColor Cyan
