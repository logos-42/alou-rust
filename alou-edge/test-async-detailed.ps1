Write-Host "=== 详细测试异步任务 ===" -ForegroundColor Green

# 构建请求体
$body = @{
    prompt = "你好，请介绍一下自己"
    model = "deepseek-chat"
    taskType = "async"
} | ConvertTo-Json

Write-Host "`n1. 创建异步任务..." -ForegroundColor Yellow
Write-Host "请求体: $body" -ForegroundColor Gray

try {
    # 发送异步请求
    $response = Invoke-WebRequest -Uri "https://alou-edge.yuanjieliu65.workers.dev/api/agent/chat" -Method Post -Body $body -ContentType "application/json" -UseBasicParsing
    $text = [System.Text.Encoding]::UTF8.GetString($response.Content)
    
    Write-Host "✓ 响应状态: $($response.StatusCode)" -ForegroundColor Green
    Write-Host "✓ 响应内容: $text" -ForegroundColor White
    
    # 解析响应获取任务ID
    $responseObj = $text | ConvertFrom-Json
    $taskId = if ($responseObj.taskId) { $responseObj.taskId } else { $responseObj.task_id }
    
    if ($taskId) {
        Write-Host "✓ 任务ID: $taskId" -ForegroundColor Cyan
        
        # 等待2秒让 Durable Object 初始化
        Write-Host "`n2. 等待2秒让任务初始化..." -ForegroundColor Yellow
        Start-Sleep -Seconds 2
        
        # 检查状态（多次尝试）
        $maxAttempts = 6
        $attempt = 0
        $completed = $false
        
        while ($attempt -lt $maxAttempts -and -not $completed) {
            $attempt++
            Write-Host "`n3.$attempt 检查任务状态 (尝试 $attempt/$maxAttempts)..." -ForegroundColor Yellow
            
            try {
                $statusUrl = "https://alou-edge.yuanjieliu65.workers.dev/api/tasks/$taskId"
                Write-Host "状态URL: $statusUrl" -ForegroundColor Gray
                
                $statusResponse = Invoke-WebRequest -Uri $statusUrl -Method Get -UseBasicParsing
                $statusText = [System.Text.Encoding]::UTF8.GetString($statusResponse.Content)
                $statusObj = $statusText | ConvertFrom-Json
                
                Write-Host "✓ 状态响应: $statusText" -ForegroundColor White
                
                # 检查状态
                if ($statusObj.status -eq "completed") {
                    Write-Host "✓ 任务已完成！" -ForegroundColor Green
                    if ($statusObj.result -and $statusObj.result.response) {
                        Write-Host "`n任务结果:" -ForegroundColor Cyan
                        Write-Host $statusObj.result.response -ForegroundColor White
                    }
                    $completed = $true
                } elseif ($statusObj.status -eq "failed") {
                    Write-Host "✗ 任务失败: $($statusObj.error)" -ForegroundColor Red
                    $completed = $true
                } elseif ($statusObj.status -eq "running") {
                    Write-Host "⟳ 任务运行中... 进度: $($statusObj.progress * 100)%" -ForegroundColor Yellow
                    Write-Host "  当前步骤: $($statusObj.current_step)" -ForegroundColor Gray
                } else {
                    Write-Host "⏳ 任务状态: $($statusObj.status)" -ForegroundColor Yellow
                }
                
            } catch {
                Write-Host "✗ 状态查询错误: $_" -ForegroundColor Red
                Write-Host "  错误详情: $($_.Exception.Message)" -ForegroundColor Red
                
                # 如果是 500 错误，显示响应内容
                if ($_.Exception.Response) {
                    $reader = New-Object System.IO.StreamReader($_.Exception.Response.GetResponseStream())
                    $errorBody = $reader.ReadToEnd()
                    Write-Host "  错误响应: $errorBody" -ForegroundColor Red
                }
            }
            
            # 如果未完成，等待5秒后重试
            if (-not $completed -and $attempt -lt $maxAttempts) {
                Write-Host "  等待5秒后重试..." -ForegroundColor Gray
                Start-Sleep -Seconds 5
            }
        }
        
        if (-not $completed) {
            Write-Host "`n⚠ 任务在 $($maxAttempts * 5) 秒后仍未完成" -ForegroundColor Yellow
        }
        
    } else {
        Write-Host "✗ 没有获取到任务ID" -ForegroundColor Red
    }
    
} catch {
    Write-Host "✗ 创建任务错误: $_" -ForegroundColor Red
    Write-Host "  错误详情: $($_.Exception.Message)" -ForegroundColor Red
    
    # 显示响应内容
    if ($_.Exception.Response) {
        $reader = New-Object System.IO.StreamReader($_.Exception.Response.GetResponseStream())
        $errorBody = $reader.ReadToEnd()
        Write-Host "  错误响应: $errorBody" -ForegroundColor Red
    }
}

Write-Host "`n=== 测试完成 ===" -ForegroundColor Green
