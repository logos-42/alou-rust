# 快速测试异步任务
Write-Host "=== 快速测试异步任务 ===" -ForegroundColor Green
Write-Host "时间: $(Get-Date)" -ForegroundColor Gray
Write-Host ""

# 构建测试请求
$asyncBody = @{
    prompt = "请简要介绍一下 Cloudflare Workers 的工作原理"
    model = "deepseek-chat"
    system_prompt = "你是一个技术专家，请用简洁的语言回答，不超过200字"
    history = @()
    tools = @()
    taskType = "async"
} | ConvertTo-Json

Write-Host "1. 创建新异步任务..." -ForegroundColor Yellow

# 发送异步请求
try {
    $response = Invoke-WebRequest -Uri "https://alou-edge.yuanjieliu65.workers.dev/api/agent/chat" -Method Post -Body $asyncBody -ContentType "application/json" -UseBasicParsing
    $bytes = $response.Content
    $text = [System.Text.Encoding]::UTF8.GetString($bytes)
    
    Write-Host "✅ 响应状态: $($response.StatusCode)" -ForegroundColor Green
    Write-Host "响应内容: $text" -ForegroundColor White
    
    # 解析响应获取任务ID
    $responseObj = $text | ConvertFrom-Json
    $taskId = $responseObj.taskId
    
    if ($taskId) {
        Write-Host "✅ 新任务ID: $taskId" -ForegroundColor Green
        
        # 等待3秒让alarm触发
        Write-Host "`n2. 等待3秒让alarm触发..." -ForegroundColor Yellow
        Start-Sleep -Seconds 3
        
        # 查询任务状态（使用正确的端点）
        Write-Host "`n3. 查询任务状态..." -ForegroundColor Yellow
        $statusUrl = "https://alou-edge.yuanjieliu65.workers.dev/api/tasks/$taskId"
        
        try {
            $statusResponse = Invoke-WebRequest -Uri $statusUrl -Method Get -UseBasicParsing
            $statusText = [System.Text.Encoding]::UTF8.GetString($statusResponse.Content)
            $statusObj = $statusText | ConvertFrom-Json
            
            Write-Host "任务状态: $($statusObj.status)" -ForegroundColor Cyan
            Write-Host "进度: $($statusObj.progress)" -ForegroundColor Cyan
            
            if ($statusObj.current_step) {
                Write-Host "当前步骤: $($statusObj.current_step)" -ForegroundColor Cyan
            }
            
            if ($statusObj.error) {
                Write-Host "错误信息: $($statusObj.error)" -ForegroundColor Red
            }
            
            # 显示完整响应
            Write-Host "`n完整状态响应:" -ForegroundColor Yellow
            Write-Host $statusText -ForegroundColor Gray
            
        } catch {
            Write-Host "❌ 状态查询失败: $_" -ForegroundColor Red
        }
        
    } else {
        Write-Host "❌ 响应中没有找到任务ID" -ForegroundColor Red
    }
    
} catch {
    Write-Host "❌ 请求失败: $_" -ForegroundColor Red
}

Write-Host "`n=== 测试完成 ===" -ForegroundColor Green
Write-Host "时间: $(Get-Date)" -ForegroundColor Gray
