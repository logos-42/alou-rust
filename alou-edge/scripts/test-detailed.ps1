# 详细测试修复后的 Worker

$WorkerURL = "https://alou-edge.yuanjieliu65.workers.dev"

Write-Host "=== 详细测试修复后的 Worker ===" -ForegroundColor Green
Write-Host "Worker URL: $WorkerURL"
Write-Host ""

# 1. 测试同步请求（不带工具）
Write-Host "1. 测试同步请求（不带工具）..." -ForegroundColor Yellow
$syncBody = @{
    prompt = "Hello, please reply in English. This is a sync test."
    model = "deepseek-chat"
    tools = @()
    history = @()
} | ConvertTo-Json -Depth 10

Write-Host "请求体: $syncBody"
Write-Host ""

try {
    $syncResponse = Invoke-RestMethod -Uri "$WorkerURL/api/agent/chat" -Method Post -Body $syncBody -ContentType "application/json"
    Write-Host "响应: " -NoNewline
    $syncResponse | ConvertTo-Json -Depth 10 | Write-Host
    Write-Host ""
    
    if ($syncResponse.success -eq $true) {
        Write-Host "✅ 同步请求成功" -ForegroundColor Green
        Write-Host "   响应长度: $($syncResponse.response.Length)"
        if ($syncResponse.response) {
            Write-Host "   响应: $($syncResponse.response)"
        }
    } else {
        Write-Host "❌ 同步请求失败" -ForegroundColor Red
        if ($syncResponse.error) {
            Write-Host "   错误: $($syncResponse.error)"
        }
    }
} catch {
    Write-Host "❌ 同步请求异常: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "   响应: $($_.ErrorDetails.Message)" -ForegroundColor Red
}
Write-Host ""

# 2. 测试异步请求（带工具）
Write-Host "2. 测试异步请求（带工具）..." -ForegroundColor Yellow
$asyncBody = @{
    prompt = "What is the current time?"
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

Write-Host "请求体: $asyncBody"
Write-Host ""

try {
    $asyncResponse = Invoke-RestMethod -Uri "$WorkerURL/api/agent/chat" -Method Post -Body $asyncBody -ContentType "application/json"
    Write-Host "响应: " -NoNewline
    $asyncResponse | ConvertTo-Json -Depth 10 | Write-Host
    Write-Host ""
    
    $taskId = $asyncResponse.task_id
    $status = $asyncResponse.status
    
    if ($taskId) {
        Write-Host "✅ 异步任务创建成功" -ForegroundColor Green
        Write-Host "   任务ID: $taskId"
        Write-Host "   初始状态: $status"
        
        if ($asyncResponse.estimated_time) {
            Write-Host "   预计时间: $($asyncResponse.estimated_time)秒"
        }
        
        # 等待并查询状态
        Write-Host ""
        Write-Host "3. 等待并查询任务状态..." -ForegroundColor Yellow
        
        for ($i = 1; $i -le 10; $i++) {
            Write-Host "   等待 $i 秒后查询..." -ForegroundColor Gray
            Start-Sleep -Seconds 1
            
            try {
                $statusResponse = Invoke-RestMethod -Uri "$WorkerURL/api/agent/tasks/$taskId/status" -Method Get -TimeoutSec 10
                Write-Host "     状态: $($statusResponse.status)"
                Write-Host "     进度: $($statusResponse.progress)"
                
                if ($statusResponse.current_step) {
                    Write-Host "     当前步骤: $($statusResponse.current_step)"
                }
                
                if ($statusResponse.status -eq "completed") {
                    Write-Host "     ✅ 任务完成" -ForegroundColor Green
                    if ($statusResponse.response) {
                        Write-Host "     响应: $($statusResponse.response)"
                    }
                    break
                } elseif ($statusResponse.status -eq "failed") {
                    Write-Host "     ❌ 任务失败" -ForegroundColor Red
                    if ($statusResponse.error) {
                        Write-Host "     错误: $($statusResponse.error)"
                    }
                    break
                } elseif ($statusResponse.status -eq "running") {
                    Write-Host "     ⏳ 任务执行中..." -ForegroundColor Yellow
                }
            } catch {
                Write-Host "     ❌ 状态查询失败: $($_.Exception.Message)" -ForegroundColor Red
            }
            
            Write-Host ""
        }
    } else {
        Write-Host "❌ 异步任务创建失败：无任务ID" -ForegroundColor Red
    }
} catch {
    Write-Host "❌ 异步请求异常: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "   响应: $($_.ErrorDetails.Message)" -ForegroundColor Red
}

Write-Host ""
Write-Host "=== 测试完成 ===" -ForegroundColor Green
