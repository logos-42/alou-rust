# 测试修复后的 Worker
# 这个脚本会测试异步任务创建、状态查询和任务执行

$WorkerURL = "https://alou-edge.yuanjieliu65.workers.dev"

Write-Host "=== 测试修复后的 Worker ===" -ForegroundColor Green
Write-Host "Worker URL: $WorkerURL"
Write-Host ""

# 1. 健康检查
Write-Host "1. 健康检查..." -ForegroundColor Yellow
try {
    $healthResponse = Invoke-RestMethod -Uri "$WorkerURL/api/health" -Method Get
    Write-Host "   状态: $($healthResponse.status)"
    Write-Host "   版本: $($healthResponse.version)"
} catch {
    Write-Host "   健康检查失败: $($_.Exception.Message)" -ForegroundColor Red
}
Write-Host ""

# 2. 同步请求测试
Write-Host "2. 同步请求测试..." -ForegroundColor Yellow
$syncBody = @{
    prompt = "你好，请用中文回复。这是一个同步测试。"
    model = "deepseek-chat"
    tools = @()
    history = @()
} | ConvertTo-Json

try {
    $syncResponse = Invoke-RestMethod -Uri "$WorkerURL/api/agent/chat" -Method Post -Body $syncBody -ContentType "application/json"
    Write-Host "   同步请求成功: $($syncResponse.success)"
    Write-Host "   响应长度: $($syncResponse.response.Length)"
    if ($syncResponse.response) {
        Write-Host "   响应预览: $($syncResponse.response.Substring(0, [Math]::Min(100, $syncResponse.response.Length)))..."
    }
} catch {
    Write-Host "   同步请求失败: $($_.Exception.Message)" -ForegroundColor Red
}
Write-Host ""

# 3. 异步任务创建（带工具调用，应该触发异步）
Write-Host "3. 创建异步任务（带工具调用）..." -ForegroundColor Yellow
$asyncBody = @{
    prompt = "你好，请用中文回复。这是一个异步任务测试，请告诉我当前时间。"
    model = "deepseek-chat"
    tools = @(
        @{
            name = "get_current_time"
            description = "获取当前时间"
            parameters = @{
                type = "object"
                properties = @{}
                required = @()
            }
        }
    )
    history = @()
} | ConvertTo-Json

try {
    $asyncResponse = Invoke-RestMethod -Uri "$WorkerURL/api/agent/chat" -Method Post -Body $asyncBody -ContentType "application/json"
    $taskId = $asyncResponse.task_id
    $status = $asyncResponse.status
    Write-Host "   任务ID: $taskId"
    Write-Host "   初始状态: $status"
    if ($asyncResponse.estimated_time) {
        Write-Host "   预计时间: $($asyncResponse.estimated_time)秒"
    }
} catch {
    Write-Host "   异步任务创建失败: $($_.Exception.Message)" -ForegroundColor Red
    $taskId = $null
}
Write-Host ""

if ($taskId) {
    # 4. 等待5秒，让alarm触发
    Write-Host "4. 等待5秒，让alarm触发任务执行..." -ForegroundColor Yellow
    Start-Sleep -Seconds 5
    Write-Host ""

    # 5. 查询任务状态（多次查询）
    Write-Host "5. 查询任务状态..." -ForegroundColor Yellow
    for ($i = 1; $i -le 6; $i++) {
        Write-Host "   第$i次查询..." -ForegroundColor Gray
        try {
            $statusResponse = Invoke-RestMethod -Uri "$WorkerURL/api/agent/tasks/$taskId/status" -Method Get -TimeoutSec 10
            Write-Host "     状态: $($statusResponse.status)"
            Write-Host "     进度: $($statusResponse.progress)"
            Write-Host "     当前步骤: $($statusResponse.current_step)"
            
            if ($statusResponse.status -eq "completed") {
                if ($statusResponse.response) {
                    Write-Host "     结果: $($statusResponse.response.Substring(0, [Math]::Min(100, $statusResponse.response.Length)))..." -ForegroundColor Green
                } else {
                    Write-Host "     任务完成，但无响应内容" -ForegroundColor Green
                }
                break
            } elseif ($statusResponse.status -eq "failed") {
                Write-Host "     错误: $($statusResponse.error)" -ForegroundColor Red
                break
            }
        } catch {
            Write-Host "     查询失败: $($_.Exception.Message)" -ForegroundColor Red
        }
        
        if ($i -lt 6) {
            Start-Sleep -Seconds 5
        }
    }
    Write-Host ""

    # 6. 最终状态
    Write-Host "6. 最终状态检查..." -ForegroundColor Yellow
    try {
        $finalStatus = Invoke-RestMethod -Uri "$WorkerURL/api/agent/tasks/$taskId/status" -Method Get -TimeoutSec 10
        Write-Host "   最终状态: $($finalStatus.status)" -ForegroundColor $(if ($finalStatus.status -eq "completed") { "Green" } else { "Red" })
        Write-Host "   最终进度: $($finalStatus.progress)"
        if ($finalStatus.error) {
            Write-Host "   错误信息: $($finalStatus.error)" -ForegroundColor Red
        }
    } catch {
        Write-Host "   最终状态查询失败: $($_.Exception.Message)" -ForegroundColor Red
    }
}

Write-Host ""
Write-Host "=== 测试完成 ===" -ForegroundColor Green
