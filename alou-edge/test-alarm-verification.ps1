Write-Host "=== 验证 Alarm 是否触发 ===" -ForegroundColor Green

# 1. 首先检查 Worker 是否在线
Write-Host "`n1. 检查 Worker 健康状态..." -ForegroundColor Yellow
try {
    $healthResponse = Invoke-WebRequest -Uri "https://alou-edge.yuanjieliu65.workers.dev/api/health" -Method Get -UseBasicParsing
    Write-Host "✓ Worker 在线: $($healthResponse.StatusCode)" -ForegroundColor Green
    $healthText = [System.Text.Encoding]::UTF8.GetString($healthResponse.Content)
    Write-Host "  响应: $healthText" -ForegroundColor Gray
} catch {
    Write-Host "✗ Worker 不可用: $_" -ForegroundColor Red
    exit 1
}

# 2. 创建异步任务
Write-Host "`n2. 创建异步任务..." -ForegroundColor Yellow
$taskBody = @{
    prompt = "这是一个测试请求，请回复'测试成功'"
    model = "deepseek-chat"
    system_prompt = "你是一个测试助手"
    history = @()
    tools = @()
    taskType = "async"
} | ConvertTo-Json

Write-Host "请求体: $taskBody" -ForegroundColor Gray

try {
    $createResponse = Invoke-WebRequest -Uri "https://alou-edge.yuanjieliu65.workers.dev/api/agent/chat" -Method Post -Body $taskBody -ContentType "application/json" -UseBasicParsing
    $createText = [System.Text.Encoding]::UTF8.GetString($createResponse.Content)
    Write-Host "✓ 任务创建响应: $($createResponse.StatusCode)" -ForegroundColor Green
    Write-Host "  响应内容: $createText" -ForegroundColor White
    
    $responseObj = $createText | ConvertFrom-Json
    $taskId = if ($responseObj.taskId) { $responseObj.taskId } else { $responseObj.task_id }
    
    if (-not $taskId) {
        Write-Host "✗ 没有获取到任务ID" -ForegroundColor Red
        Write-Host "  完整响应: $createText" -ForegroundColor Red
        exit 1
    }
    
    Write-Host "✓ 任务ID: $taskId" -ForegroundColor Cyan
    
    # 3. 立即检查状态（应该返回 queued）
    Write-Host "`n3. 立即检查任务状态（应该返回 queued）..." -ForegroundColor Yellow
    Start-Sleep -Seconds 1
    
    $statusUrl = "https://alou-edge.yuanjieliu65.workers.dev/api/tasks/$taskId"
    $statusResponse = Invoke-WebRequest -Uri $statusUrl -Method Get -UseBasicParsing
    $statusText = [System.Text.Encoding]::UTF8.GetString($statusResponse.Content)
    $statusObj = $statusText | ConvertFrom-Json
    
    Write-Host "✓ 初始状态: $($statusObj.status)" -ForegroundColor Green
    Write-Host "  完整状态: $statusText" -ForegroundColor Gray
    
    # 4. 等待 Alarm 触发（等待10秒）
    Write-Host "`n4. 等待10秒让 Alarm 触发..." -ForegroundColor Yellow
    Write-Host "  注意：请同时运行 'npx wrangler tail' 查看日志" -ForegroundColor Cyan
    Write-Host "  你应该能看到 '!!! ALARM ACTIVE !!!' 和 '🚨 AITaskDO ALARM STARTED' 日志" -ForegroundColor Cyan
    
    for ($i = 1; $i -le 10; $i++) {
        Write-Host "  等待 $i/10 秒..." -ForegroundColor Gray
        Start-Sleep -Seconds 1
    }
    
    # 5. 再次检查状态
    Write-Host "`n5. 检查 Alarm 触发后的状态..." -ForegroundColor Yellow
    
    $statusResponse2 = Invoke-WebRequest -Uri $statusUrl -Method Get -UseBasicParsing
    $statusText2 = [System.Text.Encoding]::UTF8.GetString($statusResponse2.Content)
    $statusObj2 = $statusText2 | ConvertFrom-Json
    
    Write-Host "✓ 10秒后状态: $($statusObj2.status)" -ForegroundColor Green
    Write-Host "  完整状态: $statusText2" -ForegroundColor Gray
    
    # 6. 分析结果
    Write-Host "`n6. 结果分析:" -ForegroundColor Yellow
    
    if ($statusObj2.status -eq "completed") {
        Write-Host "✅ 恭喜！AI调用成功运行！" -ForegroundColor Green
        if ($statusObj2.result -and $statusObj2.result.response) {
            Write-Host "   AI回复: $($statusObj2.result.response)" -ForegroundColor White
        }
    } elseif ($statusObj2.status -eq "running") {
        Write-Host "🔄 Alarm 已触发，任务正在运行中" -ForegroundColor Yellow
        Write-Host "   进度: $($statusObj2.progress * 100)%" -ForegroundColor Cyan
        Write-Host "   当前步骤: $($statusObj2.current_step)" -ForegroundColor Cyan
        
        # 继续等待完成
        Write-Host "`n7. 继续等待任务完成..." -ForegroundColor Yellow
        $maxWait = 30
        $waited = 0
        
        while ($waited -lt $maxWait) {
            Start-Sleep -Seconds 5
            $waited += 5
            
            $statusResponse3 = Invoke-WebRequest -Uri $statusUrl -Method Get -UseBasicParsing
            $statusText3 = [System.Text.Encoding]::UTF8.GetString($statusResponse3.Content)
            $statusObj3 = $statusText3 | ConvertFrom-Json
            
            Write-Host "   $waited 秒后状态: $($statusObj3.status)" -ForegroundColor Gray
            
            if ($statusObj3.status -eq "completed") {
                Write-Host "✅ 任务完成！" -ForegroundColor Green
                if ($statusObj3.result -and $statusObj3.result.response) {
                    Write-Host "   AI回复: $($statusObj3.result.response)" -ForegroundColor White
                }
                break
            } elseif ($statusObj3.status -eq "failed") {
                Write-Host "❌ 任务失败: $($statusObj3.error)" -ForegroundColor Red
                break
            }
        }
        
        if ($waited -ge $maxWait) {
            Write-Host "⚠ 任务在 $maxWait 秒后仍未完成" -ForegroundColor Yellow
        }
        
    } elseif ($statusObj2.status -eq "queued") {
        Write-Host "❌ Alarm 可能没有触发！" -ForegroundColor Red
        Write-Host "   任务仍然在 queued 状态" -ForegroundColor Red
        Write-Host "   可能原因:" -ForegroundColor Yellow
        Write-Host "   1. set_alarm 时间单位错误" -ForegroundColor Yellow
        Write-Host "   2. Alarm 调度失败" -ForegroundColor Yellow
        Write-Host "   3. Durable Object 没有正确初始化" -ForegroundColor Yellow
        Write-Host "   请检查 wrangler tail 日志" -ForegroundColor Cyan
    } elseif ($statusObj2.status -eq "failed") {
        Write-Host "❌ 任务失败: $($statusObj2.error)" -ForegroundColor Red
        Write-Host "   Alarm 已触发，但AI调用失败" -ForegroundColor Red
        Write-Host "   可能原因:" -ForegroundColor Yellow
        Write-Host "   1. AI_API_KEY 未配置" -ForegroundColor Yellow
        Write-Host "   2. AI服务不可用" -ForegroundColor Yellow
        Write-Host "   3. 网络连接问题" -ForegroundColor Yellow
    } else {
        Write-Host "❓ 未知状态: $($statusObj2.status)" -ForegroundColor Yellow
    }
    
} catch {
    Write-Host "✗ 测试过程中出错: $_" -ForegroundColor Red
    Write-Host "  错误详情: $($_.Exception.Message)" -ForegroundColor Red
    
    if ($_.Exception.Response) {
        $reader = New-Object System.IO.StreamReader($_.Exception.Response.GetResponseStream())
        $errorBody = $reader.ReadToEnd()
        Write-Host "  错误响应: $errorBody" -ForegroundColor Red
    }
}

Write-Host "`n=== 测试完成 ===" -ForegroundColor Green
Write-Host "`n重要提示:" -ForegroundColor Cyan
Write-Host "1. 请在新终端运行: npx wrangler tail" -ForegroundColor White
Write-Host "2. 查看是否有 '!!! ALARM ACTIVE !!!' 日志" -ForegroundColor White
Write-Host "3. 如果没有看到日志，说明 Alarm 没有触发" -ForegroundColor White
