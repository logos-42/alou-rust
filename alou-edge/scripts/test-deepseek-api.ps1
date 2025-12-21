# 测试 DeepSeek API Key 是否有效
param(
    [Parameter(Mandatory=$false)]
    [string]$ApiKey = ""
)

$ErrorActionPreference = "Stop"

Write-Host "`n=== DeepSeek API Key 测试工具 ===" -ForegroundColor Cyan

if ([string]::IsNullOrEmpty($ApiKey)) {
    Write-Host "`n请提供 DeepSeek API Key 进行测试" -ForegroundColor Yellow
    Write-Host "用法: .\scripts\test-deepseek-api.ps1 -ApiKey 'your-api-key'`n" -ForegroundColor White
    exit 1
}

Write-Host "`n测试 API Key: $($ApiKey.Substring(0, [Math]::Min(8, $ApiKey.Length)))..." -ForegroundColor Cyan

try {
    $body = @{
        model = "deepseek-chat"
        messages = @(
            @{
                role = "user"
                content = "Hello"
            }
        )
        max_tokens = 10
    } | ConvertTo-Json

    $headers = @{
        "Content-Type" = "application/json"
        "Authorization" = "Bearer $ApiKey"
    }

    Write-Host "发送测试请求到 DeepSeek API..." -ForegroundColor Yellow
    
    $response = Invoke-RestMethod -Uri "https://api.deepseek.com/v1/chat/completions" `
        -Method Post `
        -Body $body `
        -Headers $headers `
        -ErrorAction Stop

    Write-Host "`n✅ API Key 有效！" -ForegroundColor Green
    Write-Host "响应模型: $($response.model)" -ForegroundColor White
    Write-Host "响应内容: $($response.choices[0].message.content)" -ForegroundColor White
    
} catch {
    $statusCode = $_.Exception.Response.StatusCode.value__
    $errorMessage = $_.Exception.Message
    
    Write-Host "`n❌ API Key 测试失败" -ForegroundColor Red
    Write-Host "状态码: $statusCode" -ForegroundColor Red
    
    if ($_.Exception.Response) {
        try {
            $reader = New-Object System.IO.StreamReader($_.Exception.Response.GetResponseStream())
            $responseBody = $reader.ReadToEnd()
            Write-Host "错误详情: $responseBody" -ForegroundColor Red
            
            $errorJson = $responseBody | ConvertFrom-Json
            if ($errorJson.error) {
                Write-Host "`n错误类型: $($errorJson.error.type)" -ForegroundColor Yellow
                Write-Host "错误消息: $($errorJson.error.message)" -ForegroundColor Yellow
                
                if ($errorJson.error.message -match "governor" -or $errorJson.error.message -match "Authentication Fails") {
                    Write-Host "`n💡 建议：" -ForegroundColor Cyan
                    Write-Host "  1. 检查 API Key 是否正确" -ForegroundColor White
                    Write-Host "  2. 确认 API Key 是否已过期或被禁用" -ForegroundColor White
                    Write-Host "  3. 前往 DeepSeek 平台检查账户状态和余额" -ForegroundColor White
                    Write-Host "  4. 如果持续出现，可能需要等待一段时间（限流）" -ForegroundColor White
                }
            }
        } catch {
            Write-Host "无法解析错误响应" -ForegroundColor Red
        }
    }
    
    Write-Host "`n错误信息: $errorMessage" -ForegroundColor Red
}

Write-Host "`n"

