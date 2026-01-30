# 简单工具调用测试
$body = @{
    prompt = "Execute the bash command: pwd"
    session_id = "test_$(Get-Date -Format 'yyyyMMddHHmmss')"
    agent_id = "test"
    model = "deepseek-chat"
    stream = $false
    tools_enabled = $true
    task_type = "async"
    tools = @(@{
        name = "bash"
        description = "Execute bash commands"
        parameters = @{
            type = "object"
            properties = @{ 
                command = @{ type = "string" } 
            }
            required = @("command")
        }
    })
} | ConvertTo-Json -Depth 10

Write-Host "Creating task with tool..."
$response = Invoke-WebRequest -Uri "http://localhost:8787/api/ai-task/init-and-start" -Method POST -Body $body -ContentType "application/json" -UseBasicParsing
$content = [System.Text.Encoding]::UTF8.GetString($response.Content)
$result = $content | ConvertFrom-Json
$taskId = $result.taskId
Write-Host "Task ID: $taskId"
Write-Host ""

# 等待5秒让任务执行
Start-Sleep -Seconds 5

# 检查状态
$statusResp = Invoke-WebRequest -Uri "http://localhost:8787/api/ai-task/$taskId/status" -UseBasicParsing
$statusContent = [System.Text.Encoding]::UTF8.GetString($statusResp.Content)
$status = $statusContent | ConvertFrom-Json
Write-Host "Status: $($status.status)"
Write-Host "Step: $($status.current_step)"
Write-Host ""

# 检查工具调用
$toolsResp = Invoke-WebRequest -Uri "http://localhost:8787/api/ai-task/$taskId/pending-tools" -UseBasicParsing
$toolsContent = [System.Text.Encoding]::UTF8.GetString($toolsResp.Content)
$tools = $toolsContent | ConvertFrom-Json
Write-Host "Tool calls count: $($tools.toolCalls.Count)"
if ($tools.toolCalls.Count -gt 0) {
    Write-Host "Tool calls:"
    $tools.toolCalls | ForEach-Object {
        Write-Host "  - $($_.tool): $($_.arguments | ConvertTo-Json -Compress)"
    }
}
