# 测试后端错误
$body = @{
    session_id = "test_$(Get-Date -Format 'yyyyMMddHHmmss')"
    message = "你好"
} | ConvertTo-Json

Write-Host "发送请求..." -ForegroundColor Yellow
Write-Host "Body: $body" -ForegroundColor Cyan

try {
    $response = Invoke-WebRequest -Uri "https://alou-edge.yuanjieliu65.workers.dev/api/agent/chat" -Method POST -ContentType "application/json" -Body $body
    Write-Host "成功: $($response.StatusCode)" -ForegroundColor Green
    Write-Host $response.Content
} catch {
    Write-Host "错误: $($_.Exception.Message)" -ForegroundColor Red
    if ($_.Exception.Response) {
        $reader = New-Object System.IO.StreamReader($_.Exception.Response.GetResponseStream())
        $responseBody = $reader.ReadToEnd()
        Write-Host "响应内容: $responseBody" -ForegroundColor Yellow
    }
}
