# 最终 UTF-8 测试
$baseUrl = "https://alou-edge.yuanjieliu65.workers.dev"

Write-Host "=== 测试 UTF-8 编码修复 ===" -ForegroundColor Cyan

# 测试根路由
Write-Host "`n1. 测试根路由..." -ForegroundColor Yellow
$rootResponse = Invoke-WebRequest -Uri $baseUrl -Method GET
Write-Host "Content-Type: $($rootResponse.Headers['Content-Type'])" -ForegroundColor Green
Write-Host "内容: $($rootResponse.Content)" -ForegroundColor White

# 测试健康检查
Write-Host "`n2. 测试健康检查..." -ForegroundColor Yellow
$healthResponse = Invoke-WebRequest -Uri "$baseUrl/api/health" -Method GET
Write-Host "Content-Type: $($healthResponse.Headers['Content-Type'])" -ForegroundColor Green

# 创建会话并测试中文
Write-Host "`n3. 测试中文 AI 响应..." -ForegroundColor Yellow
$sessionResponse = Invoke-RestMethod -Uri "$baseUrl/api/session" -Method POST -ContentType "application/json" -Body '{}'
$sessionId = $sessionResponse.session_id

$body = @{
    session_id = $sessionId
    message = "你好"
} | ConvertTo-Json

$chatResponse = Invoke-WebRequest -Uri "$baseUrl/api/agent/chat" -Method POST -ContentType "application/json; charset=utf-8" -Body $body
Write-Host "Content-Type: $($chatResponse.Headers['Content-Type'])" -ForegroundColor Green
Write-Host "`nAI 回复:" -ForegroundColor Yellow
$content = ($chatResponse.Content | ConvertFrom-Json).content
Write-Host $content -ForegroundColor White

Write-Host "`n=== 测试完成 ===" -ForegroundColor Cyan
