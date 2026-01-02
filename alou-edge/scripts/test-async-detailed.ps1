# 详细测试 Durable Objects 异步任务执行
Write-Host "=== 详细测试 Durable Objects 异步任务执行 ===" -ForegroundColor Green
Write-Host "时间: $(Get-Date)" -ForegroundColor Gray
Write-Host ""

# 构建测试请求
$asyncBody = @{
    prompt = "请简要介绍一下 Cloudflare Durable Objects 的工作原理"
    model = "deepseek-chat"
    system_prompt = "你是一个技术专家，请用简洁的语言回答"
    history = @()
    tools = @()
    taskType = "async"
} | ConvertTo-Json

Write-Host "1. 发送异步请求创建任务..." -ForegroundColor Yellow
Write-Host "请求体长度: $($asyncBody.Length) 字符" -ForegroundColor Gray

# 发送异步请求
try {
    $response = Invoke-WebRequest -Uri "https://alou-edge.yuanjieliu65.workers.dev/api/agent/chat" -Method Post -Body $asyncBody -ContentType "application/json" -UseBasicParsing
    $bytes = $response.Content
    $text = [System.Text.Encoding]::UTF8.GetString($bytes)
    
    Write-Host "响应状态: $($response.StatusCode)" -ForegroundColor Green
    Write-Host "响应内容: $text" -ForegroundColor White
    
    # 解析响应获取任务ID
    $responseObj = $text | ConvertFrom-Json
    $taskId = $responseObj.taskId
    
    if ($taskId) {
        Write-Host "✅ 任务创建成功! 任务ID: $taskId" -ForegroundColor Green
        
        # 等待1秒让状态保存完成
        Write-Host "`n2. 等待1秒让状态保存完成..." -ForegroundColor Yellow
        Start-Sleep -Seconds 2
        
        # 查询任务状态
        Write-Host "3. 查询任务状态..." -ForegroundColor Yellow
        $statusUrl = "https://alou-edge.yuanjieliu65.workers.dev/api/tasks/$taskId"
        $statusResponse = Invoke-WebRequest -Uri $statusUrl -Method Get -UseBasicParsing
        $statusText = [System.Text.Encoding]::UTF8.GetString($statusResponse.Content)
        
        Write-Host "初始状态: $statusText" -ForegroundColor White
        
        # 等待 alarm 触发（10ms + 网络延迟）
        Write-Host "`n4. 等待 alarm 触发（15秒）..." -ForegroundColor Yellow
        Start-Sleep -Seconds 15
        
        # 查询任务状态
        Write-Host "`n5. 查询 alarm 触发后的任务状态..." -ForegroundColor Yellow
        $statusResponse2 = Invoke-WebRequest -Uri $statusUrl -Method Get -UseBasicParsing
        $statusText2 = [System.Text.Encoding]::UTF8.GetString($statusResponse2.Content)
        
        Write-Host "alarm 后状态: $statusText2" -ForegroundColor White
        
        # 解析状态
        $statusObj2 = $statusText2 | ConvertFrom-Json
        $status = $statusObj2.status
        $progress = $statusObj2.progress
        
        Write-Host "状态: $status, 进度: $progress" -ForegroundColor Cyan
        
            if ($status -eq "running") {
                Write-Host "✅ 任务正在执行中！" -ForegroundColor Green
                
                # 等待任务完成
                Write-Host "`n6. 等待任务完成（最多60秒）..." -ForegroundColor Yellow
            $maxWait = 60
            $waitInterval = 5
            $waited = 0
            
            while ($waited -lt $maxWait) {
                Start-Sleep -Seconds $waitInterval
                $waited += $waitInterval
                
                $currentResponse = Invoke-WebRequest -Uri $statusUrl -Method Get -UseBasicParsing
                $currentText = [System.Text.Encoding]::UTF8.GetString($currentResponse.Content)
                $currentObj = $currentText | ConvertFrom-Json
                
                Write-Host "等待 $waited 秒 - 状态: $($currentObj.status), 进度: $($currentObj.progress)" -ForegroundColor Gray
                
                if ($currentObj.status -eq "completed") {
                    Write-Host "✅ 任务完成！结果: $($currentObj.result.response)" -ForegroundColor Green
                    break
                } elseif ($currentObj.status -eq "failed") {
                    Write-Host "❌ 任务失败: $($currentObj.error)" -ForegroundColor Red
                    break
                }
            }
            
            if ($waited -ge $maxWait) {
                Write-Host "⚠️  任务超时（等待超过 $maxWait 秒）" -ForegroundColor Yellow
                Write-Host "最终状态: $currentText" -ForegroundColor Red
            }
            
        } elseif ($status -eq "completed") {
            Write-Host "✅ 任务已经完成！" -ForegroundColor Green
            Write-Host "结果: $($statusObj2.result.response)" -ForegroundColor White
            
        } elseif ($status -eq "failed") {
            Write-Host "❌ 任务失败: $($statusObj2.error)" -ForegroundColor Red
            
        } else {
            Write-Host "⚠️  任务状态异常: $status" -ForegroundColor Yellow
        }
        
    } else {
        Write-Host "❌ 响应中没有找到任务ID" -ForegroundColor Red
        Write-Host "响应内容: $text" -ForegroundColor Red
    }
    
} catch {
    Write-Host "❌ 请求失败: $_" -ForegroundColor Red
    Write-Host "错误详情: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "堆栈跟踪: $($_.Exception.StackTrace)" -ForegroundColor DarkRed
}

Write-Host "`n=== 测试完成 ===" -ForegroundColor Green
Write-Host "时间: $(Get-Date)" -ForegroundColor Gray
