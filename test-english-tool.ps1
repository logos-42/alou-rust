$body = @{
    prompt = "Use the bash tool to execute this command: echo 'Hello World'. You MUST call the bash tool, do not just explain it."
    session_id = "test_english_$(Get-Date -Format 'yyyyMMddHHmmss')"
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

Write-Host "Creating task with English prompt..."
$response = Invoke-WebRequest -Uri "http://localhost:8787/api/ai-task/init-and-start" -Method POST -Body $body -ContentType "application/json" -UseBasicParsing
$content = [System.Text.Encoding]::UTF8.GetString($response.Content)
Write-Host "Response: $content"
$result = $content | ConvertFrom-Json
$taskId = $result.taskId

Write-Host "`nTask ID: $taskId"
Write-Host "`nWaiting 5 seconds for AI to process..."
Start-Sleep -Seconds 5

# Check pending tools
Write-Host "`nChecking pending tools..."
$toolsResp = Invoke-WebRequest -Uri "http://localhost:8787/api/ai-task/$taskId/pending-tools" -UseBasicParsing
$toolsContent = [System.Text.Encoding]::UTF8.GetString($toolsResp.Content)
Write-Host "Pending tools response: $toolsContent"
$tools = $toolsContent | ConvertFrom-Json

if ($tools.toolCalls.Count -gt 0) {
    Write-Host "`n✅ SUCCESS! Got $($tools.toolCalls.Count) tool calls!"
    foreach ($tc in $tools.toolCalls) {
        Write-Host "  Tool: $($tc.tool), ID: $($tc.id)"
        Write-Host "  Arguments: $($tc.arguments | ConvertTo-Json -Compress)"
    }
} else {
    Write-Host "`n❌ No tool calls received"
}
