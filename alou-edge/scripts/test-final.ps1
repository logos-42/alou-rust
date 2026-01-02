# 最终测试修复后的 Worker

$WorkerURL = "https://alou-edge.yuanjieliu65.workers.dev"

Write-Host "=== 最终测试修复后的 Worker ===" -ForegroundColor Green
Write-Host "Worker URL: $WorkerURL"
Write-Host ""

# 1. 测试异步请求（带工具）
Write-Host "1. 创建异步任务（带工具调用）..." -ForegroundColor Yellow
$asyncBody = @{
    prompt = "What is the current time? Please use the get_current_time tool."
    model = "deepseek-chat"
    tools = @(
        @{
            name = "get_current_time"
            description = "Get current time"
            parameters = @{
                type = "object"
                properties = @{}
                required = @()
            }
        }
    )
    history = @()
} | ConvertTo-Json -Depth 10

try {
    $asyncResponse = Invoke-RestMethod -Uri "$WorkerURL/api/agent/chat" -Method Post -Body $asyncBody -ContentType "application/json"
    
    Write-Host "响应详情:" -ForegroundColor Cyan
    $asyncResponse | Format-List | Out-String | Write-Host
    
    $taskId = $asyncResponse.taskId
    $status = $asyncResponse.status
    
    if ($taskId) {
        Write-Host "✅ 异步任务创建成功" -ForegroundColor Green
        Write-Host "   任务ID: $taskId"
        Write-Host "   初始状态: $status"
        Write-Host "   预计时间: $($asyncResponse.estimatedTime)秒"
        
        # 2. 立即查询状态
        Write-Host ""
        Write-Host "2. 立即查询任务状态..." -ForegroundColor Yellow
        try {
            $statusResponse = Invoke-RestMethod -Uri "$WorkerURL/api/tasks/$taskId" -Method Get -TimeoutSec 10
            Write-Host "   立即状态: $($statusResponse.status)"
            Write-Host "   进度: $($statusResponse.progress)"
            if ($statusResponse.current_step) {
                Write-Host "   当前步骤: $($statusResponse.current_step)"
            }
        } catch {
            Write-Host "   ❌ 立即状态查询失败: $($_.Exception.Message)" -ForegroundColor Red
        }
        
        # 3. 等待10秒后查询
        Write-Host ""
        Write-Host "3. 等待10秒后查询（让alarm触发）..." -ForegroundColor Yellow
        Start-Sleep -Seconds 10
        
        try {
            $statusResponse = Invoke-RestMethod -Uri "$WorkerURL/api/tasks/$taskId" -Method Get -TimeoutSec 10
            Write-Host "   10秒后状态: $($statusResponse.status)" -ForegroundColor $(if ($statusResponse.status -eq "completed") { "Green" } elseif ($statusResponse.status -eq "failed") { "Red" } else { "Yellow" })
            Write-Host "   进度: $($statusResponse.progress)"
            if ($statusResponse.current_step) {
                Write-Host "   当前步骤: $($statusResponse.current_step)"
            }
            
            if ($statusResponse.status -eq "completed") {
                Write-Host "   ✅ 任务完成！" -ForegroundColor Green
                if ($statusResponse.response) {
                    Write-Host "   响应: $($statusResponse.response)"
                }
            } elseif ($statusResponse.status -eq "failed") {
                Write-Host "   ❌ 任务失败" -ForegroundColor Red
                if ($statusResponse.error) {
                    Write-Host "   错误: $($statusResponse.error)"
                }
            } elseif ($statusResponse.status -eq "running") {
                Write-Host "   ⏳ 任务仍在执行中，继续等待..." -ForegroundColor Yellow
                
                # 4. 再等待20秒
                Start-Sleep -Seconds 20
                
                try {
                    $finalStatus = Invoke-RestMethod -Uri "$WorkerURL/api/tasks/$taskId" -Method Get -TimeoutSec 10
                    Write-Host "   最终状态: $($finalStatus.status)" -ForegroundColor $(if ($finalStatus.status -eq "completed") { "Green" } elseif ($finalStatus.status -eq "failed") { "Red" } else { "Yellow" })
                    Write-Host "   最终进度: $($finalStatus.progress)"
                    
                    if ($finalStatus.status -eq "completed" -and $finalStatus.response) {
                        Write-Host "   最终响应: $($finalStatus.response)"
                    } elseif ($finalStatus.error) {
                        Write-Host "   最终错误: $($finalStatus.error)"
                    }
                } catch {
                    Write-Host "   ❌ 最终状态查询失败: $($_.Exception.Message)" -ForegroundColor Red
                }
            }
        } catch {
            Write-Host "   ❌ 10秒后状态查询失败: $($_.Exception.Message)" -ForegroundColor Red
        }
    } else {
        Write-Host "❌ 异步任务创建失败：无任务ID" -ForegroundColor Red
    }
} catch {
    Write-Host "❌ 异步请求异常: $($_.Exception.Message)" -ForegroundColor Red
    if ($_.ErrorDetails) {
        Write-Host "   错误详情: $($_.ErrorDetails.Message)" -ForegroundColor Red
    }
}

Write-Host ""
Write-Host "=== 测试完成 ===" -ForegroundColor Green
