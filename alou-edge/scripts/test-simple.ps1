# 简单测试 - 直接测试状态查询

$WorkerURL = "https://alou-edge.yuanjieliu65.workers.dev"

Write-Host "=== 简单测试状态查询 ===" -ForegroundColor Green
Write-Host ""

# 1. 先创建一个任务
Write-Host "1. 创建异步任务..." -ForegroundColor Yellow
$asyncBody = @{
    prompt = "Simple test"
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
    $asyncResponse = Invoke-RestMethod -Uri "$WorkerURL/api/agent/chat" -Method Post -Body $asyncBody -ContentType "application/json" -TimeoutSec 30
    $taskId = $asyncResponse.taskId
    
    if ($taskId) {
        Write-Host "✅ 任务创建成功: $taskId" -ForegroundColor Green
        
        # 2. 直接查询状态（带详细错误信息）
        Write-Host ""
        Write-Host "2. 查询任务状态 (任务ID: $taskId)..." -ForegroundColor Yellow
        
        for ($i = 1; $i -le 3; $i++) {
            Write-Host "   尝试 #$i..." -ForegroundColor Gray
            try {
                $statusResponse = Invoke-WebRequest -Uri "$WorkerURL/api/tasks/$taskId" -Method Get -TimeoutSec 30
                Write-Host "   HTTP 状态码: $($statusResponse.StatusCode)" -ForegroundColor Cyan
                Write-Host "   响应头:" -ForegroundColor Cyan
                $statusResponse.Headers | Format-Table | Out-String | Write-Host
                Write-Host "   响应体:" -ForegroundColor Cyan
                $statusResponse.Content | Write-Host
                
                # 尝试解析 JSON
                try {
                    $jsonResponse = $statusResponse.Content | ConvertFrom-Json
                    Write-Host "   JSON 解析成功:" -ForegroundColor Green
                    $jsonResponse | Format-List | Out-String | Write-Host
                } catch {
                    Write-Host "   JSON 解析失败: $_" -ForegroundColor Yellow
                }
                
                break
            } catch {
                Write-Host "   ❌ 查询失败: $($_.Exception.Message)" -ForegroundColor Red
                if ($_.ErrorDetails) {
                    Write-Host "   错误详情: $($_.ErrorDetails.Message)" -ForegroundColor Red
                }
                if ($i -lt 3) {
                    Write-Host "   等待2秒后重试..." -ForegroundColor Gray
                    Start-Sleep -Seconds 2
                }
            }
        }
    } else {
        Write-Host "❌ 任务创建失败：无任务ID" -ForegroundColor Red
    }
} catch {
    Write-Host "❌ 任务创建异常: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host ""
Write-Host "=== 测试完成 ===" -ForegroundColor Green
