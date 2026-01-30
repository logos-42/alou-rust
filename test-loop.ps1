# 创建任务
$body = @{
    prompt = "请使用bash工具执行命令: echo 'Test'。你必须调用bash工具，不要只是解释命令。"
    session_id = "test_$(Get-Date -Format 'yyyyMMddHHmmss')"
    agent_id = "test"
    model = "deepseek-chat"
    stream = $false
    tools_enabled = $true
    task_type = "async"
    tools = @(@{
        name = "bash"
        description = "Execute bash commands in the terminal. You MUST use this tool to execute commands, do not just explain them."
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

Write-Host "创建任务..."
Write-Host ""
try {
    $response = Invoke-WebRequest -Uri "http://localhost:8787/api/ai-task/init-and-start" -Method POST -Body $body -ContentType "application/json" -UseBasicParsing
    Write-Host "响应状态: $($response.StatusCode)"
    $content = [System.Text.Encoding]::UTF8.GetString($response.Content)
    Write-Host "响应内容: $content"
    $result = $content | ConvertFrom-Json
    $taskId = $result.taskId
    Write-Host "任务ID: $taskId"
    
    if ([string]::IsNullOrEmpty($taskId)) {
        Write-Host "错误: 任务ID为空！"
        exit 1
    }
} catch {
    Write-Host "错误: $_"
    exit 1
}
Write-Host ""

# 轮询
for ($i = 1; $i -le 15; $i++) {
    Start-Sleep -Seconds 2
    
    # 查询状态
    $statusResp = Invoke-WebRequest -Uri "http://localhost:8787/api/ai-task/$taskId/status" -UseBasicParsing
    $statusContent = [System.Text.Encoding]::UTF8.GetString($statusResp.Content)
    $status = $statusContent | ConvertFrom-Json
    Write-Host "[轮询 $i] 状态: $($status.status), 进度: $([math]::Round($status.progress * 100))%"
    
    # 检查工具
    $toolsResp = Invoke-WebRequest -Uri "http://localhost:8787/api/ai-task/$taskId/pending-tools" -UseBasicParsing
    $toolsContent = [System.Text.Encoding]::UTF8.GetString($toolsResp.Content)
    $tools = $toolsContent | ConvertFrom-Json
    
    if ($tools.toolCalls.Count -gt 0) {
        Write-Host "  🔧 发现 $($tools.toolCalls.Count) 个工具"
        
        # 构造工具结果
        $results = @()
        foreach ($tc in $tools.toolCalls) {
            $results += @{
                tool = $tc.tool
                success = $true
                result = "executed"
                arguments = $tc.arguments
                timestamp = [DateTimeOffset]::Now.ToUnixTimeMilliseconds()
                tool_call_id = $tc.id
            }
        }
        
        $resultBody = @{ results = $results } | ConvertTo-Json -Depth 10
        Invoke-WebRequest -Uri "http://localhost:8787/api/ai-task/$taskId/tool-result" -Method POST -Body $resultBody -ContentType "application/json" -UseBasicParsing | Out-Null
        Write-Host "  ✅ 工具结果已提交"
    }
    
    if ($status.status -eq "completed" -or $status.status -eq "failed") {
        Write-Host ""
        Write-Host "🎉 任务结束: $($status.status)"
        break
    }
}
