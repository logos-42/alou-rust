# 测试异步任务执行并显示AI回复结果
Write-Host "=== 测试异步任务执行并显示AI回复结果 ===" -ForegroundColor Green
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
Write-Host "请求内容: $asyncBody" -ForegroundColor Gray

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
        
        # 等待 alarm 触发（10ms + 网络延迟）
        Write-Host "`n2. 等待 alarm 触发（5秒）..." -ForegroundColor Yellow
        Start-Sleep -Seconds 5
        
        # 查询任务状态
        Write-Host "`n3. 查询任务状态..." -ForegroundColor Yellow
        $statusUrl = "https://alou-edge.yuanjieliu65.workers.dev/api/tasks/$taskId"
        
        $maxWait = 120  # 最大等待120秒
        $waitInterval = 5  # 每5秒查询一次
        $waited = 0
        $completed = $false
        
        while ($waited -lt $maxWait -and -not $completed) {
            try {
                $statusResponse = Invoke-WebRequest -Uri $statusUrl -Method Get -UseBasicParsing
                $statusText = [System.Text.Encoding]::UTF8.GetString($statusResponse.Content)
                $statusObj = $statusText | ConvertFrom-Json
                
                Write-Host "等待 $waited 秒 - 状态: $($statusObj.status), 进度: $($statusObj.progress)" -ForegroundColor Gray
                
                if ($statusObj.status -eq "completed") {
                    Write-Host "✅ 任务完成！" -ForegroundColor Green
                    
                    # 显示AI的回复结果
                    if ($statusObj.result -and $statusObj.result.response) {
                        Write-Host "`n=== AI 回复内容 ===" -ForegroundColor Cyan
                        Write-Host $statusObj.result.response -ForegroundColor White
                        Write-Host "=== 回复结束 ===" -ForegroundColor Cyan
                        
                        # 显示完整的响应结构
                        Write-Host "`n=== 完整响应结构 ===" -ForegroundColor Yellow
                        Write-Host $statusText -ForegroundColor Gray
                    } else {
                        Write-Host "⚠️  任务完成但没有回复内容" -ForegroundColor Yellow
                        Write-Host "完整响应: $statusText" -ForegroundColor Gray
                    }
                    
                    $completed = $true
                    break
                    
                } elseif ($statusObj.status -eq "failed") {
                    Write-Host "❌ 任务失败: $($statusObj.error)" -ForegroundColor Red
                    $completed = $true
                    break
                    
                } elseif ($statusObj.status -eq "running") {
                    Write-Host "  当前步骤: $($statusObj.currentStep)" -ForegroundColor Cyan
                }
                
            } catch {
                Write-Host "❌ 状态查询失败: $_" -ForegroundColor Red
            }
            
            Start-Sleep -Seconds $waitInterval
            $waited += $waitInterval
        }
        
        if (-not $completed) {
            Write-Host "⚠️  任务超时（等待超过 $maxWait 秒）" -ForegroundColor Yellow
            Write-Host "最终状态查询URL: $statusUrl" -ForegroundColor Gray
        }
        
    } else {
        Write-Host "❌ 响应中没有找到任务ID" -ForegroundColor Red
        Write-Host "响应内容: $text" -ForegroundColor Red
    }
    
} catch {
    Write-Host "❌ 请求失败: $_" -ForegroundColor Red
    Write-Host "错误详情: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "`n=== 测试完成 ===" -ForegroundColor Green
Write-Host "时间: $(Get-Date)" -ForegroundColor Gray
