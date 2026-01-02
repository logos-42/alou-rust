# 简单测试状态查询端点
Write-Host "=== 测试状态查询端点 ===" -ForegroundColor Green
Write-Host "时间: $(Get-Date)" -ForegroundColor Gray
Write-Host ""

# 使用一个已知的任务ID进行测试
$taskId = "test_task_123"

Write-Host "1. 查询任务状态 (任务ID: $taskId)..." -ForegroundColor Yellow
$statusUrl = "https://alou-edge.yuanjieliu65.workers.dev/api/tasks/$taskId"

try {
    # 尝试获取状态
    $response = Invoke-WebRequest -Uri $statusUrl -Method Get -UseBasicParsing
    Write-Host "响应状态: $($response.StatusCode)" -ForegroundColor Green
    
    $bytes = $response.Content
    $text = [System.Text.Encoding]::UTF8.GetString($bytes)
    Write-Host "响应内容: $text" -ForegroundColor White
    
} catch {
    Write-Host "❌ 请求失败: $_" -ForegroundColor Red
    Write-Host "错误详情: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "状态码: $($_.Exception.Response.StatusCode)" -ForegroundColor Red
    Write-Host "状态描述: $($_.Exception.Response.StatusDescription)" -ForegroundColor Red
    
    # 尝试获取响应体
    if ($_.Exception.Response) {
        $stream = $_.Exception.Response.GetResponseStream()
        $reader = New-Object System.IO.StreamReader($stream)
        $errorBody = $reader.ReadToEnd()
        Write-Host "错误响应体: $errorBody" -ForegroundColor DarkRed
    }
}

Write-Host "`n=== 测试完成 ===" -ForegroundColor Green
Write-Host "时间: $(Get-Date)" -ForegroundColor Gray
