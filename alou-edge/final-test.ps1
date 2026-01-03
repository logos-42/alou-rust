Write-Host "=== 最终测试：验证所有修复 ===" -ForegroundColor Green

# 1. 创建异步任务
Write-Host "`n1. 创建异步任务..." -ForegroundColor Yellow
$taskBody = @{
    prompt = "请回复'AI调用成功'来证明系统正常工作"
    model = "deepseek-chat"
    system_prompt = "你是一个测试助手"
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
    
    # 2. 立即检查状态（应该显示真实状态）
    Write-Host "`n2. 立即检查任务状态（应该显示真实状态）..." -ForegroundColor Yellow
    Start-Sleep -Seconds 2
    
    $statusUrl = "https://alou-edge.yuanjieliu65.workers.dev/api/tasks/$taskId"
    
    # 多次检查状态变化
    $maxChecks = 10
    $completed = $false
    
    for ($i = 1; $i -le $maxChecks; $i++) {
        Write-Host "`n  第 $i 次检查状态..." -ForegroundColor Gray
        try {
            $statusResponse = Invoke-WebRequest -Uri $statusUrl -Method Get -UseBasicParsing
            $statusText = [System.Text.Encoding]::UTF8.GetString($statusResponse.Content)
            $statusObj = $statusText | ConvertFrom-Json
            
            Write-Host "  状态: $($statusObj.status)" -ForegroundColor White
            Write-Host "  进度: $($statusObj.progress * 100)%" -ForegroundColor White
            Write-Host "  当前步骤: $($statusObj.current_step)" -ForegroundColor White
            
            if ($statusObj.status -eq "completed") {
                Write-Host "  ✅ 任务完成！" -ForegroundColor Green
                if ($statusObj.result -and $statusObj.result.response) {
                    Write-Host "  AI回复: $($statusObj.result.response)" -ForegroundColor White
                }
                $completed = $true
                break
            } elseif ($statusObj.status -eq "running") {
                Write-Host "  🔄 任务正在运行中" -ForegroundColor Yellow
            } elseif ($statusObj.status -eq "failed") {
                Write-Host "  ❌ 任务失败: $($statusObj.error)" -ForegroundColor Red
                $completed = $true
                break
            } elseif ($statusObj.status -eq "queued") {
                Write-Host "  ⏳ 任务排队中" -ForegroundColor Yellow
            }
            
        } catch {
            Write-Host "  ✗ 状态查询错误: $_" -ForegroundColor Red
        }
        
        if (-not $completed -and $i -lt $maxChecks) {
            Write-Host "  等待5秒后重试..." -ForegroundColor Gray
            Start-Sleep -Seconds 5
        }
    }
    
    # 3. 结果分析
    Write-Host "`n3. 最终结果分析:" -ForegroundColor Yellow
    
    if ($completed) {
        if ($statusObj.status -eq "completed") {
            Write-Host "  🎉 恭喜！AI调用成功运行！" -ForegroundColor Green
            Write-Host "  ✅ 所有修复生效：" -ForegroundColor Cyan
            Write-Host "    1. handle_get_status 返回真实状态 ✓" -ForegroundColor White
            Write-Host "    2. alarm 函数正确触发 ✓" -ForegroundColor White
            Write-Host "    3. AI服务调用成功 ✓" -ForegroundColor White
        } elseif ($statusObj.status -eq "failed") {
            Write-Host "  ⚠ Alarm 已触发，但AI调用失败" -ForegroundColor Yellow
            Write-Host "  🔍 错误信息: $($statusObj.error)" -ForegroundColor Red
        }
    } else {
        Write-Host "  ❌ 任务在 $($maxChecks * 5) 秒后仍未完成" -ForegroundColor Red
        Write-Host "  🔍 可能问题：" -ForegroundColor Yellow
        Write-Host "    1. alarm 仍然没有触发" -ForegroundColor White
        Write-Host "    2. AI服务调用超时" -ForegroundColor White
        Write-Host "    3. 其他未知问题" -ForegroundColor White
    }
    
} catch {
    Write-Host "✗ 测试过程中出错: $_" -ForegroundColor Red
}

Write-Host "`n=== 测试完成 ===" -ForegroundColor Green
Write-Host "`n重要提示:" -ForegroundColor Cyan
Write-Host "请在新终端运行以下命令查看实时日志:" -ForegroundColor White
Write-Host "npx wrangler tail --format pretty" -ForegroundColor Green
