Write-Host "测试任务状态查询..." -ForegroundColor Green

# 使用新创建的任务ID
$taskId = "task_19b7a3d9c95_65c15"
$url = "https://alou-edge.yuanjieliu65.workers.dev/api/tasks/$taskId"

Write-Host "查询URL: $url" -ForegroundColor Gray

try {
    Write-Host "发送请求..." -ForegroundColor Yellow
    $response = Invoke-WebRequest -Uri $url -Method Get -UseBasicParsing -TimeoutSec 30
    
    Write-Host "✅ 响应状态: $($response.StatusCode)" -ForegroundColor Green
    
    $text = [System.Text.Encoding]::UTF8.GetString($response.Content)
    Write-Host "响应内容: $text" -ForegroundColor White
    
} catch {
    Write-Host "❌ 请求失败" -ForegroundColor Red
    Write-Host "错误类型: $($_.Exception.GetType().Name)" -ForegroundColor Red
    Write-Host "错误消息: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "`n测试完成" -ForegroundColor Green
