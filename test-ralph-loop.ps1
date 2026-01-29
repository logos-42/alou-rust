# Ralph Loop 测试 - 完整的工具循环执行
$body = @{
    prompt = "Please do the following tasks step by step: 1) Use bash to echo 'Step 1 Complete', 2) Use bash to echo 'Step 2 Complete', 3) Use bash to echo 'Final Step Done'. You MUST use the bash tool for each step."
    session_id = "test_ralph_$(Get-Date -Format 'yyyyMMddHHmmss')"
    agent_id = "test"
    model = "deepseek-chat"
    stream = $false
    tools_enabled = $true
    task_type = "async"
    tools = @(@{
        name = "bash"
        description = "Execute bash commands in the terminal. You MUST use this tool to execute commands."
        parameters = @{
            type = "object"
            properties = @{ 
                command = @{ 
                    type = "string"
                    description = "The bash command to execute"
                } 
            }
            required = @("command")
        }
    })
} | ConvertTo-Json -Depth 10

Write-Host "=== Ralph Loop Test ===" -ForegroundColor Cyan
Write-Host "Creating task with multi-step prompt..."
$result = Invoke-RestMethod -Uri "http://localhost:8787/api/ai-task/init-and-start" -Method POST -Body $body -ContentType "application/json"
$taskId = $result.taskId

Write-Host "Task ID: $taskId" -ForegroundColor Green
Write-Host ""

$iteration = 0
$maxIterations = 10

while ($iteration -lt $maxIterations) {
    $iteration++
    Write-Host "--- Iteration $iteration ---" -ForegroundColor Yellow
    
    # 等待一下让任务执行
    Start-Sleep -Seconds 3
    
    # 检查状态
    $status = Invoke-RestMethod -Uri "http://localhost:8787/api/ai-task/$taskId/status"
    
    Write-Host "Status: $($status.status), Progress: $([math]::Round($status.progress * 100))%, Step: $($status.current_step)"
    
    # 检查是否有待处理的工具
    $tools = Invoke-RestMethod -Uri "http://localhost:8787/api/ai-task/$taskId/pending-tools"
    
    if ($tools.toolCalls.Count -gt 0) {
        Write-Host "  🔧 Found $($tools.toolCalls.Count) tool call(s)" -ForegroundColor Cyan
        
        # 模拟执行工具
        $results = @()
        foreach ($tc in $tools.toolCalls) {
            Write-Host "    Executing: $($tc.tool) with args: $($tc.arguments | ConvertTo-Json -Compress)" -ForegroundColor Gray
            
            # 模拟 bash 执行
            $command = $tc.arguments.command
            $output = "Executed: $command"
            
            $results += @{
                tool = $tc.tool
                success = $true
                result = $output
                arguments = $tc.arguments
                timestamp = [DateTimeOffset]::Now.ToUnixTimeMilliseconds()
                tool_call_id = $tc.id
            }
        }
        
        # 提交工具结果
        $resultBody = @{ results = $results } | ConvertTo-Json -Depth 10
        $submitResp = Invoke-RestMethod -Uri "http://localhost:8787/api/ai-task/$taskId/tool-result" -Method POST -Body $resultBody -ContentType "application/json"
        Write-Host "  ✅ Tool results submitted" -ForegroundColor Green
    } else {
        Write-Host "  ⏳ No pending tools"
    }
    
    # 检查是否完成
    if ($status.status -eq "completed") {
        Write-Host ""
        Write-Host "🎉 Task completed!" -ForegroundColor Green
        Write-Host "Final result: $($status.result.response)" -ForegroundColor White
        break
    } elseif ($status.status -eq "failed") {
        Write-Host ""
        Write-Host "❌ Task failed: $($status.error)" -ForegroundColor Red
        break
    }
    
    Write-Host ""
}

if ($iteration -ge $maxIterations) {
    Write-Host "⚠️ Reached max iterations ($maxIterations)" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "=== Test Complete ===" -ForegroundColor Cyan
