# 诊断 alarm 函数问题
# 这个脚本会创建多个任务并监控状态

$WorkerURL = "https://alou-edge.yuanjieliu65.workers.dev"

Write-Host "=== 诊断 Durable Object Alarm 问题 ===" -ForegroundColor Green
Write-Host "Worker URL: $WorkerURL"
Write-Host ""

# 创建3个测试任务
$tasks = @()

for ($i = 1; $i -le 3; $i++) {
    Write-Host "创建测试任务 #$i..." -ForegroundColor Yellow
    
    $body = @{
        prompt = "Test task $i - Please reply with 'Hello from task $i'"
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
        $response = Invoke-RestMethod -Uri "$WorkerURL/api/agent/chat" -Method Post -Body $body -ContentType "application/json" -TimeoutSec 30
        $taskId = $response.taskId
        
        if ($taskId) {
            $tasks += @{
                Id = $taskId
                Created = Get-Date
                Status = $response.status
            }
            Write-Host "  ✅ 任务创建成功: $taskId" -ForegroundColor Green
        } else {
            Write-Host "  ❌ 任务创建失败：无任务ID" -ForegroundColor Red
        }
    } catch {
        Write-Host "  ❌ 任务创建异常: $($_.Exception.Message)" -ForegroundColor Red
    }
    
    Start-Sleep -Milliseconds 500
}

Write-Host ""
Write-Host "总共创建了 $($tasks.Count) 个任务" -ForegroundColor Cyan
Write-Host ""

if ($tasks.Count -gt 0) {
    # 监控任务状态
    Write-Host "开始监控任务状态（持续60秒）..." -ForegroundColor Yellow
    Write-Host ""
    
    $startTime = Get-Date
    $endTime = $startTime.AddSeconds(60)
    
    while ((Get-Date) -lt $endTime) {
        $elapsed = [math]::Round(((Get-Date) - $startTime).TotalSeconds, 1)
        Write-Host "[$elapsed 秒] 查询任务状态..." -ForegroundColor Gray
        
        foreach ($task in $tasks) {
            try {
                $statusResponse = Invoke-RestMethod -Uri "$WorkerURL/api/tasks/$($task.Id)" -Method Get -TimeoutSec 10
                $task.Status = $statusResponse.status
                $task.Progress = $statusResponse.progress
                $task.LastCheck = Get-Date
                
                Write-Host "  $($task.Id): $($task.Status) (进度: $($task.Progress))" -ForegroundColor $(if ($task.Status -eq "completed") { "Green" } elseif ($task.Status -eq "failed") { "Red" } else { "Yellow" })
            } catch {
                Write-Host "  $($task.Id): ❌ 查询失败 ($($_.Exception.Message))" -ForegroundColor Red
            }
        }
        
        Write-Host ""
        
        # 检查是否所有任务都已完成或失败
        $completedTasks = $tasks | Where-Object { $_.Status -in @("completed", "failed") }
        if ($completedTasks.Count -eq $tasks.Count) {
            Write-Host "所有任务都已完成或失败，停止监控" -ForegroundColor Green
            break
        }
        
        # 等待5秒
        Start-Sleep -Seconds 5
    }
    
    # 最终状态汇总
    Write-Host ""
    Write-Host "=== 最终状态汇总 ===" -ForegroundColor Green
    
    foreach ($task in $tasks) {
        $color = if ($task.Status -eq "completed") { "Green" } elseif ($task.Status -eq "failed") { "Red" } else { "Yellow" }
        Write-Host "任务 $($task.Id): $($task.Status)" -ForegroundColor $color
    }
    
    # 统计
    $completed = ($tasks | Where-Object { $_.Status -eq "completed" }).Count
    $failed = ($tasks | Where-Object { $_.Status -eq "failed" }).Count
    $other = $tasks.Count - $completed - $failed
    
    Write-Host ""
    Write-Host "统计:" -ForegroundColor Cyan
    Write-Host "  已完成: $completed" -ForegroundColor Green
    Write-Host "  已失败: $failed" -ForegroundColor Red
    Write-Host "  其他状态: $other" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "=== 诊断完成 ===" -ForegroundColor Green
Write-Host ""
Write-Host "请检查 Cloudflare Workers 实时日志中的以下信息：" -ForegroundColor Cyan
Write-Host "1. alarm 函数是否被触发" -ForegroundColor Cyan
Write-Host "2. alarm 函数执行时的错误信息" -ForegroundColor Cyan
Write-Host "3. Durable Object 实例的状态" -ForegroundColor Cyan
Write-Host "4. 任何 panic 或未捕获的异常" -ForegroundColor Cyan

