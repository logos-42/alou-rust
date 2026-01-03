Write-Host "=== 测试同步化修复 ===" -ForegroundColor Green

Write-Host "`n重要提示：请在新终端运行以下命令查看实时日志：" -ForegroundColor Cyan
Write-Host "npx wrangler tail --format pretty" -ForegroundColor Green
Write-Host "你应该能看到：" -ForegroundColor White
Write-Host "1. '!!! ALARM SET SUCCESS !!!'" -ForegroundColor Green
Write-Host "2. '🚨 AITaskDO ALARM STARTED'" -ForegroundColor Green
Write-Host "3. AI调用相关的日志" -ForegroundColor Green

# 1. 创建异步任务
Write-Host "`n1. 创建异步任务..." -ForegroundColor Yellow
$taskBody = @{
    prompt = "测试同步化修复，请回复'同步修复成功'"
    model = "deepseek-chat"
    system_prompt = "测试助手"
    history = @()
    tools = @()
    taskType = "async"
} | ConvertTo-Json

try {
    $createResponse = Invoke-WebRequest -Uri "https://alou-edge.yuanjieliu65.workers.dev/api/agent/chat" -Method Post -Body $taskBody -ContentType "application/json" -UseBasicParsing
    $createText = [System.Text.Encoding]::UTF8.GetString($createResponse.Content)
    Write-Host "✓ 任务创建响应: $($createResponse.StatusCode)" -ForegroundColor Green
    Write-Host "  响应内容: $createText" -ForegroundColor White
    
    $responseObj = $createText | ConvertFrom-Json
    $taskId = if ($responseObj.taskId) { $responseObj.taskId } else { $responseObj.task_id }
    
    if (-not $taskId) {
        Write-Host "✗ 没有获取到任务ID" -ForegroundColor Red
        exit 1
    }
    
    Write-Host "✓ 任务ID: $taskId" -ForegroundColor Cyan
    
    # 2. 等待并检查状态
    Write-Host "`n2. 等待10秒让Alarm触发..." -ForegroundColor Yellow
    Start-Sleep -Seconds 10
    
    $statusUrl = "https://alou-edge.yuanjieliu65.workers.dev/api/tasks/$taskId"
    
    Write-Host "`n3. 检查任务状态..." -ForegroundColor Yellow
    try {
        $statusResponse = Invoke-WebRequest -Uri $statusUrl -Method Get -UseBasicParsing
        $statusText = [System.Text.Encoding]::UTF8.GetString($statusResponse.Content)
        $statusObj = $statusText | ConvertFrom-Json
        
        Write-Host "  状态: $($statusObj.status)" -ForegroundColor White
        Write-Host "  进度: $($statusObj.progress * 100)%" -ForegroundColor White
        Write-Host "  当前步骤: $($statusObj.current_step)" -ForegroundColor White
        
        if ($statusObj.status -eq "completed") {
            Write-Host "  🎉 恭喜！同步化修复成功！" -ForegroundColor Green
            Write-Host "  ✅ Alarm 正确触发，AI调用成功" -ForegroundColor Cyan
        } elseif ($statusObj.status -eq "running") {
            Write-Host "  🔄 Alarm 已触发，任务正在运行中" -ForegroundColor Yellow
            Write-Host "  ⏳ 请等待任务完成..." -ForegroundColor Gray
        } elseif ($statusObj.status -eq "queued") {
            Write-Host "  ❌ 问题依然存在！任务仍然在 queued 状态" -ForegroundColor Red
            Write-Host "  🔍 可能原因：" -ForegroundColor Yellow
            Write-Host "    1. 查看日志是否有 '!!! ALARM SET SUCCESS !!!'" -ForegroundColor White
            Write-Host "    2. 如果没有，说明 set_alarm 仍然失败" -ForegroundColor White
            Write-Host "    3. 如果有，但看不到 '🚨 AITaskDO ALARM STARTED'，说明 alarm 函数没被调用" -ForegroundColor White
        } elseif ($statusObj.status -eq "failed") {
            Write-Host "  ⚠ Alarm 已触发，但AI调用失败" -ForegroundColor Yellow
            Write-Host "  🔍 错误信息: $($statusObj.error)" -ForegroundColor Red
        }
        
    } catch {
        Write-Host "  ✗ 状态查询错误: $_" -ForegroundColor Red
    }
    
} catch {
    Write-Host "✗ 测试过程中出错: $_" -ForegroundColor Red
}

Write-Host "`n=== 测试完成 ===" -ForegroundColor Green
Write-Host "`n关键验证步骤：" -ForegroundColor Cyan
Write-Host "1. 运行: npx wrangler tail --format pretty" -ForegroundColor White
Write-Host "2. 查看是否有 '!!! ALARM SET SUCCESS !!!' 日志" -ForegroundColor White
Write-Host "3. 查看是否有 '🚨 AITaskDO ALARM STARTED' 日志" -ForegroundColor White
Write-Host "4. 根据日志判断问题所在" -ForegroundColor White
