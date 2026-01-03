# 使用正确格式测试异步任务
# 使用 CompatibleRequest 格式

$WorkerURL = "https://alou-edge.yuanjieliu65.workers.dev"

Write-Host "=== 使用正确格式测试异步任务 ===" -ForegroundColor Green
Write-Host ""

# 创建正确的 CompatibleRequest 格式
$body = @{
    prompt = "测试异步任务"
    model = "deepseek-chat"
    tools = @()
    history = @()
    taskType = "async"
} | ConvertTo-Json -Depth 10

Write-Host "1. 发送兼容性格式请求..." -ForegroundColor Yellow
Write-Host "   请求体:" -ForegroundColor Gray
Write-Host $body
Write-Host ""

try {
    $response = Invoke-RestMethod -Uri "$WorkerURL/api/agent/chat" `
        -Method POST `
        -ContentType "application/json" `
        -Body $body `
        -TimeoutSec 30
    
    Write-Host "   ✅ 请求成功" -ForegroundColor Green
    Write-Host "   响应:" -ForegroundColor Cyan
    $response | Format-List | Out-String | Write-Host
    
    # 检查响应
    if ($response.success -eq $true -and $response.task_id) {
        $taskId = $response.task_id
        Write-Host "   ✅ 获取到任务ID: $taskId" -ForegroundColor Green
        
        # 查询任务状态
        Write-Host ""
        Write-Host "2. 查询任务状态..." -ForegroundColor Yellow
        $statusUrl = "$WorkerURL/api/tasks/$taskId"
        
        try {
            $statusResponse = Invoke-RestMethod -Uri $statusUrl -Method Get -TimeoutSec 30
            Write-Host "   ✅ 任务状态查询成功" -ForegroundColor Green
            Write-Host "   状态响应:" -ForegroundColor Cyan
            $statusResponse | Format-List | Out-String | Write-Host
            
            Write-Host "   🎉 Durable Objects 工作正常！" -ForegroundColor Green
            Write-Host "   🎉 错误代码 1101 已修复！" -ForegroundColor Green
            
        } catch {
            Write-Host "   ❌ 状态查询失败: $($_.Exception.Message)" -ForegroundColor Red
        }
        
    } else {
        Write-Host "   ⚠️  响应指示失败或没有任务ID" -ForegroundColor Yellow
        if ($response.error) {
            Write-Host "   错误: $($response.error)" -ForegroundColor Red
        }
    }
    
} catch {
    Write-Host "   ❌ 请求失败: $($_.Exception.Message)" -ForegroundColor Red
    if ($_.ErrorDetails) {
        Write-Host "   错误详情: $($_.ErrorDetails.Message)" -ForegroundColor Red
    }
}

Write-Host ""
Write-Host "=== 测试完成 ===" -ForegroundColor Green
