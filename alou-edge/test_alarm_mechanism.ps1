# 测试 Alarm 机制的 PowerShell 脚本
# 这个脚本会创建一个异步任务并检查 alarm 是否触发

Write-Host "=== 测试 Alarm 机制 ===" -ForegroundColor Cyan

# 1. 首先检查 wrangler 是否已登录
Write-Host "1. 检查 wrangler 登录状态..." -ForegroundColor Yellow
$wranglerLogin = npx wrangler whoami 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ wrangler 未登录，请先运行: npx wrangler login" -ForegroundColor Red
    exit 1
}
Write-Host "✅ wrangler 已登录" -ForegroundColor Green

# 2. 启动本地开发服务器
Write-Host "`n2. 启动本地开发服务器..." -ForegroundColor Yellow
Write-Host "   在另一个终端中运行: npx wrangler dev" -ForegroundColor Yellow
Write-Host "   然后按 Enter 继续..." -ForegroundColor Yellow
Read-Host

# 3. 创建测试任务
Write-Host "`n3. 创建测试任务..." -ForegroundColor Yellow

$testPayload = @{
    prompt = "测试 alarm 机制"
    model = "deepseek-chat"
    task_type = "async"
    system_prompt = "这是一个测试任务，用于验证 alarm 机制是否正常工作。"
} | ConvertTo-Json

Write-Host "请求负载:" -ForegroundColor Gray
Write-Host $testPayload -ForegroundColor Gray

# 发送请求到本地开发服务器
Write-Host "`n发送请求到 http://localhost:8787/api/agent/async..." -ForegroundColor Yellow

try {
    $response = Invoke-RestMethod -Uri "http://localhost:8787/api/agent/async" `
        -Method Post `
        -Body $testPayload `
        -ContentType "application/json" `
        -Headers @{
            "Authorization" = "Bearer test-token"
        }
    
    Write-Host "✅ 任务创建成功!" -ForegroundColor Green
    Write-Host "响应:" -ForegroundColor Gray
    $response | ConvertTo-Json -Depth 10 | Write-Host -ForegroundColor Gray
    
    $taskId = $response.taskId
    if ($taskId) {
        Write-Host "`n任务ID: $taskId" -ForegroundColor Cyan
        
        # 4. 等待1秒，让 alarm 触发
        Write-Host "`n4. 等待1秒，让 alarm 触发..." -ForegroundColor Yellow
        Start-Sleep -Seconds 1
        
        # 5. 检查任务状态
        Write-Host "`n5. 检查任务状态..." -ForegroundColor Yellow
        $statusResponse = Invoke-RestMethod -Uri "http://localhost:8787/api/tasks/$taskId/status" `
            -Method Get `
            -Headers @{
                "Authorization" = "Bearer test-token"
            }
        
        Write-Host "状态响应:" -ForegroundColor Gray
        $statusResponse | ConvertTo-Json -Depth 10 | Write-Host -ForegroundColor Gray
        
        # 6. 检查控制台输出
        Write-Host "`n6. 检查控制台输出..." -ForegroundColor Yellow
        Write-Host "   请查看 wrangler dev 的控制台，应该能看到:" -ForegroundColor Yellow
        Write-Host "   - '!!! ALARM ACTIVE !!!'" -ForegroundColor Green
        Write-Host "   - '🚨 AITaskDO ALARM STARTED for task: ...'" -ForegroundColor Green
        Write-Host "   - 其他 alarm 执行日志" -ForegroundColor Green
        
    } else {
        Write-Host "❌ 未获取到任务ID" -ForegroundColor Red
    }
    
} catch {
    Write-Host "❌ 请求失败: $_" -ForegroundColor Red
    Write-Host "错误详情:" -ForegroundColor Red
    $_.Exception.Response | Format-List -Force | Out-String | Write-Host -ForegroundColor Red
}

Write-Host "`n=== 测试完成 ===" -ForegroundColor Cyan
Write-Host "`n下一步操作:" -ForegroundColor Yellow
Write-Host "1. 查看 wrangler dev 控制台，确认 alarm 是否触发" -ForegroundColor White
Write-Host "2. 如果看到 '!!! ALARM ACTIVE !!!'，说明 alarm 机制正常工作" -ForegroundColor White
Write-Host "3. 如果没有看到，请检查:" -ForegroundColor White
Write-Host "   - Durable Object 是否正确配置" -ForegroundColor White
Write-Host "   - set_alarm 参数是否正确" -ForegroundColor White
Write-Host "   - 时间戳计算是否正确" -ForegroundColor White
