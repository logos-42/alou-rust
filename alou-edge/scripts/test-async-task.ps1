# 测试异步任务执行
Write-Host "测试 Durable Objects 异步任务执行..." -ForegroundColor Green

# 构建测试请求
$asyncBody = @{
    prompt = "测试异步长序列任务 - 请写一篇关于 Cloudflare Durable Objects 的技术文章"
    model = "deepseek-chat"
    system_prompt = "你是一个技术专家，请详细解释技术概念"
    history = @()
    tools = @()
    taskType = "async"
} | ConvertTo-Json

Write-Host "发送异步请求..." -ForegroundColor Yellow
Write-Host "请求体: $asyncBody" -ForegroundColor Gray

# 发送异步请求
try {
    $response = Invoke-WebRequest -Uri "https://alou-edge.yuanjieliu65.workers.dev/api/agent/chat" -Method Post -Body $asyncBody -ContentType "application/json" -UseBasicParsing
    $bytes = $response.Content
    $text = [System.Text.Encoding]::UTF8.GetString($bytes)
    
    Write-Host "响应状态: $($response.StatusCode)" -ForegroundColor Green
    Write-Host "响应内容: $text" -ForegroundColor White
    
    # 解析响应获取任务ID
    $responseObj = $text | ConvertFrom-Json
    $taskId = $responseObj.task_id
    
    if ($taskId) {
        Write-Host "任务创建成功! 任务ID: $taskId" -ForegroundColor Green
        
        # 等待几秒让任务开始执行
        Write-Host "等待5秒让任务开始执行..." -ForegroundColor Yellow
        Start-Sleep -Seconds 5
        
        # 查询任务状态
        Write-Host "查询任务状态..." -ForegroundColor Yellow
        $statusUrl = "https://alou-edge.yuanjieliu65.workers.dev/api/agent/task-status/$taskId"
        $statusResponse = Invoke-WebRequest -Uri $statusUrl -Method Get -UseBasicParsing
        $statusBytes = $statusResponse.Content
        $statusText = [System.Text.Encoding]::UTF8.GetString($statusBytes)
        
        Write-Host "任务状态响应: $statusText" -ForegroundColor White
        
        # 等待更多时间让任务完成
        Write-Host "等待30秒让任务完成..." -ForegroundColor Yellow
        Start-Sleep -Seconds 30
        
        # 再次查询任务状态
        Write-Host "再次查询任务状态..." -ForegroundColor Yellow
        $finalStatusResponse = Invoke-WebRequest -Uri $statusUrl -Method Get -UseBasicParsing
        $finalStatusBytes = $finalStatusResponse.Content
        $finalStatusText = [System.Text.Encoding]::UTF8.GetString($finalStatusBytes)
        
        Write-Host "最终任务状态: $finalStatusText" -ForegroundColor White
        
    } else {
        Write-Host "响应中没有找到任务ID" -ForegroundColor Red
        Write-Host "响应内容: $text" -ForegroundColor Red
    }
    
} catch {
    Write-Host "请求失败: $_" -ForegroundColor Red
    Write-Host "错误详情: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "测试完成!" -ForegroundColor Green
