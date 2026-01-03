Write-Host "=== 深度调试 Alarm 问题 ===" -ForegroundColor Green

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
Write-Host "`n2. 创建异步任务（带调试信息）..." -ForegroundColor Yellow
$taskBody = @{
    prompt = "这是一个调试请求，请回复'调试成功'"
    model = "deepseek-chat"
    system_prompt = "你是一个调试助手"
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
    
    # 3. 立即检查状态（应该能看到调试日志）
    Write-Host "`n3. 立即检查任务状态（查看调试日志）..." -ForegroundColor Yellow
    Start-Sleep -Seconds 2
    
    $statusUrl = "https://alou-edge.yuanjieliu65.workers.dev/api/tasks/$taskId"
    
    # 多次检查状态，查看调试信息
    for ($i = 1; $i -le 5; $i++) {
        Write-Host "`n  第 $i 次检查状态..." -ForegroundColor Gray
        try {
            $statusResponse = Invoke-WebRequest -Uri $statusUrl -Method Get -UseBasicParsing
            $statusText = [System.Text.Encoding]::UTF8.GetString($statusResponse.Content)
            $statusObj = $statusText | ConvertFrom-Json
            
            Write-Host "  状态: $($statusObj.status)" -ForegroundColor White
            Write-Host "  进度: $($statusObj.progress * 100)%" -ForegroundColor White
            Write-Host "  当前步骤: $($statusObj.current_step)" -ForegroundColor White
            
            # 检查是否有调试信息
            if ($statusText -match "DEBUG") {
                Write-Host "  🔍 发现调试信息！" -ForegroundColor Cyan
                Write-Host "  $statusText" -ForegroundColor Gray
            }
            
            if ($statusObj.status -eq "completed") {
                Write-Host "  ✅ 任务完成！" -ForegroundColor Green
                if ($statusObj.result -and $statusObj.result.response) {
                    Write-Host "  AI回复: $($statusObj.result.response)" -ForegroundColor White
                }
                break
            } elseif ($statusObj.status -eq "running") {
                Write-Host "  🔄 任务正在运行中" -ForegroundColor Yellow
            } elseif ($statusObj.status -eq "failed") {
                Write-Host "  ❌ 任务失败: $($statusObj.error)" -ForegroundColor Red
                break
            }
            
        } catch {
            Write-Host "  ✗ 状态查询错误: $_" -ForegroundColor Red
        }
        
        if ($i -lt 5) {
            Write-Host "  等待3秒后重试..." -ForegroundColor Gray
            Start-Sleep -Seconds 3
        }
    }
    
    # 4. 如果还是 queued，尝试手动触发
    Write-Host "`n4. 如果任务还是 queued，尝试手动触发..." -ForegroundColor Yellow
    $finalStatusResponse = Invoke-WebRequest -Uri $statusUrl -Method Get -UseBasicParsing
    $finalStatusText = [System.Text.Encoding]::UTF8.GetString($finalStatusResponse.Content)
    $finalStatusObj = $finalStatusText | ConvertFrom-Json
    
    if ($finalStatusObj.status -eq "queued") {
        Write-Host "  ⚠ 任务仍然在 queued 状态" -ForegroundColor Yellow
        Write-Host "  🛠 尝试手动设置 alarm..." -ForegroundColor Cyan
        
        # 尝试通过状态查询触发调试代码中的 alarm 设置
        for ($j = 1; $j -le 3; $j++) {
            Write-Host "  第 $j 次手动触发..." -ForegroundColor Gray
            $triggerResponse = Invoke-WebRequest -Uri $statusUrl -Method Get -UseBasicParsing
            $triggerText = [System.Text.Encoding]::UTF8.GetString($triggerResponse.Content)
            Write-Host "  触发响应: $triggerText" -ForegroundColor Gray
            Start-Sleep -Seconds 2
        }
        
        # 等待 alarm 触发
        Write-Host "  等待10秒让手动设置的 alarm 触发..." -ForegroundColor Yellow
        Start-Sleep -Seconds 10
        
        # 再次检查状态
        $finalCheckResponse = Invoke-WebRequest -Uri $statusUrl -Method Get -UseBasicParsing
        $finalCheckText = [System.Text.Encoding]::UTF8.GetString($finalCheckResponse.Content)
        $finalCheckObj = $finalCheckText | ConvertFrom-Json
        
        Write-Host "  手动触发后状态: $($finalCheckObj.status)" -ForegroundColor White
    }
    
    # 5. 分析结果
    Write-Host "`n5. 问题分析:" -ForegroundColor Yellow
    
    if ($finalStatusObj.status -eq "queued") {
        Write-Host "  ❌ 核心问题: Alarm 没有触发" -ForegroundColor Red
        Write-Host "  🔍 可能原因:" -ForegroundColor Yellow
        Write-Host "    1. Durable Object 的 alarm 方法签名错误" -ForegroundColor Yellow
        Write-Host "    2. set_alarm 调用失败（权限/配置问题）" -ForegroundColor Yellow
        Write-Host "    3. Cloudflare 的 Durable Object 迁移问题" -ForegroundColor Yellow
        Write-Host "    4. worker-rs 版本兼容性问题" -ForegroundColor Yellow
        Write-Host "  💡 建议操作:" -ForegroundColor Cyan
        Write-Host "    1. 运行: npx wrangler tail --format pretty" -ForegroundColor White
        Write-Host "    2. 查看是否有 '!!! ALARM ACTIVE !!!' 日志" -ForegroundColor White
        Write-Host "    3. 检查 Cloudflare Dashboard 的 Durable Objects 状态" -ForegroundColor White
    } elseif ($finalStatusObj.status -eq "running") {
        Write-Host "  ✅ Alarm 已触发，任务正在运行" -ForegroundColor Green
        Write-Host "  ⏳ 请等待任务完成..." -ForegroundColor Yellow
    } elseif ($finalStatusObj.status -eq "completed") {
        Write-Host "  🎉 恭喜！AI调用成功运行！" -ForegroundColor Green
    } elseif ($finalStatusObj.status -eq "failed") {
        Write-Host "  ⚠ Alarm 已触发，但AI调用失败" -ForegroundColor Yellow
        Write-Host "  🔍 错误信息: $($finalStatusObj.error)" -ForegroundColor Red
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

Write-Host "`n=== 调试完成 ===" -ForegroundColor Green
Write-Host "`n重要提示:" -ForegroundColor Cyan
Write-Host "请在新终端运行以下命令查看实时日志:" -ForegroundColor White
Write-Host "npx wrangler tail --format pretty" -ForegroundColor Green
Write-Host "`n如果看不到 '!!! ALARM ACTIVE !!!' 日志，说明:" -ForegroundColor Yellow
Write-Host "1. alarm 函数根本没有被调用" -ForegroundColor White
Write-Host "2. 需要检查 Durable Object 配置和迁移" -ForegroundColor White
