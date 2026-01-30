# Agent Skills 循环执行测试
# 测试 Agent Skills 与工具的循环调用能力

$body = @{
    prompt = "Please help me analyze some data step by step using Agent Skills:
              1) First, discover what skills are available in the test-skills directory
              2) Then load the data-analyzer skill to understand its capabilities  
              3) Finally, execute the data-analyzer skill to analyze some sample data
              You should use the agent_skills tool for each step."
    session_id = "test_agent_skills_$(Get-Date -Format 'yyyyMMddHHmmss')"
    agent_id = "test"
    model = "deepseek-chat"
    stream = $false
    tools_enabled = $true
    task_type = "async"
    tools = @(
        @{
            name = "agent_skills"
            description = "Manage and execute standard Agent Skills protocol skills. Use this tool to discover, load, and execute skills from the ~/.alou/skills/ directory."
            parameters = @{
                type = "object"
                properties = @{
                    action = @{
                        type = "string"
                        enum = @("discover", "load", "execute", "list", "search")
                        description = "The action to perform with Agent Skills"
                    }
                    skill_name = @{
                        type = "string"
                        description = "Name of the skill to load or execute"
                    }
                    inputs = @{
                        type = "object"
                        description = "Input parameters for skill execution"
                    }
                    query = @{
                        type = "string"
                        description = "Search query for finding skills"
                    }
                }
                required = @("action")
            }
        }
        @{
            name = "bash"
            description = "Execute bash commands. Use this for file operations and system tasks."
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
        }
    )
} | ConvertTo-Json -Depth 10

Write-Host "=== Agent Skills Loop Test ===" -ForegroundColor Cyan
Write-Host "Creating task with Agent Skills workflow..."
$result = Invoke-RestMethod -Uri "http://localhost:8787/api/ai-task/init-and-start" -Method POST -Body $body -ContentType "application/json"
$taskId = $result.taskId

Write-Host "Task ID: $taskId" -ForegroundColor Green
Write-Host ""

$iteration = 0
$maxIterations = 15

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
            
            $output = ""
            $success = $true
            
            try {
                if ($tc.tool -eq "agent_skills") {
                    # 模拟 Agent Skills 工具执行
                    $action = $tc.arguments.action
                    
                    switch ($action) {
                        "discover" {
                            $output = @{
                                skills = @(
                                    @{
                                        name = "web-scraper"
                                        description = "Expert at extracting structured data from web pages using various scraping techniques"
                                        path = "test-skills/web-scraper"
                                    }
                                    @{
                                        name = "data-analyzer"
                                        description = "Expert at analyzing datasets and generating insights from structured data"
                                        path = "test-skills/data-analyzer"
                                    }
                                )
                                count = 2
                            } | ConvertTo-Json -Depth 5
                        }
                        "load" {
                            $skillName = $tc.arguments.skill_name
                            $output = @{
                                skill = @{
                                    name = $skillName
                                    description = "Loaded skill: $skillName"
                                    instructions = "Full instructions for $skillName skill loaded successfully"
                                    loaded = $true
                                }
                            } | ConvertTo-Json -Depth 5
                        }
                        "execute" {
                            $skillName = $tc.arguments.skill_name
                            $inputs = $tc.arguments.inputs
                            $output = @{
                                skill_name = $skillName
                                result = @{
                                    success = $true
                                    output = @{
                                        analysis_type = "summary"
                                        message = "Successfully executed $skillName skill"
                                        processed_inputs = $inputs
                                    }
                                    execution_time_ms = 1500
                                    logs = @("Skill loaded", "Processing inputs", "Execution completed")
                                }
                            } | ConvertTo-Json -Depth 5
                        }
                        default {
                            $output = @{
                                error = "Unknown action: $action"
                            } | ConvertTo-Json
                            $success = $false
                        }
                    }
                } elseif ($tc.tool -eq "bash") {
                    # 模拟 bash 执行
                    $command = $tc.arguments.command
                    $output = "Executed: $command`nCommand completed successfully"
                } else {
                    $output = "Tool $($tc.tool) executed successfully"
                }
            } catch {
                $output = "Error executing tool: $($_.Exception.Message)"
                $success = $false
            }
            
            $results += @{
                tool = $tc.tool
                success = $success
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
Write-Host "=== Agent Skills Test Complete ===" -ForegroundColor Cyan