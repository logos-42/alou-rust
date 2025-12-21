# 检查 API 状态和配置的脚本
param(
    [Parameter(Mandatory=$false)]
    [switch]$TestChat = $false
)

$baseUrl = "https://alou-edge.yuanjieliu65.workers.dev"
$ErrorActionPreference = "Continue"

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "  Workers API 状态检查工具" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

# 1. 检查基础端点
Write-Host "1. 检查基础 API 端点..." -ForegroundColor Yellow
try {
    $health = Invoke-RestMethod -Uri "$baseUrl/api/health" -Method Get -ErrorAction Stop
    Write-Host "   ✓ /api/health: $($health.status)" -ForegroundColor Green
} catch {
    Write-Host "   ✗ /api/health: 失败 - $_" -ForegroundColor Red
}

try {
    $status = Invoke-RestMethod -Uri "$baseUrl/api/status" -Method Get -ErrorAction Stop
    Write-Host "   ✓ /api/status: $($status.status)" -ForegroundColor Green
    Write-Host "     版本: $($status.version)" -ForegroundColor Gray
    Write-Host "     Agent Core: $($status.services.agent_core)" -ForegroundColor Gray
} catch {
    Write-Host "   ✗ /api/status: 失败 - $_" -ForegroundColor Red
}

# 2. 测试会话创建
Write-Host "`n2. 测试会话创建..." -ForegroundColor Yellow
try {
    $session = Invoke-RestMethod -Uri "$baseUrl/api/session" -Method Post `
        -Body (@{} | ConvertTo-Json) `
        -ContentType "application/json" `
        -ErrorAction Stop
    $sessionId = $session.session_id
    Write-Host "   ✓ 会话创建成功: $sessionId" -ForegroundColor Green
} catch {
    Write-Host "   ✗ 会话创建失败: $_" -ForegroundColor Red
    $sessionId = $null
}

# 3. 测试 Agent Chat（如果启用）
if ($TestChat -and $sessionId) {
    Write-Host "`n3. 测试 Agent Chat API..." -ForegroundColor Yellow
    try {
        $chatBody = @{
            session_id = $sessionId
            message = "Hello, test message"
        } | ConvertTo-Json
        
        $chatResponse = Invoke-WebRequest -Uri "$baseUrl/api/agent/chat" `
            -Method Post `
            -Body $chatBody `
            -ContentType "application/json" `
            -UseBasicParsing `
            -ErrorAction Stop
        
        Write-Host "   ✓ Agent Chat: 成功 (状态码: $($chatResponse.StatusCode))" -ForegroundColor Green
        Write-Host "   响应预览: $($chatResponse.Content.Substring(0, [Math]::Min(200, $chatResponse.Content.Length)))..." -ForegroundColor Gray
    } catch {
        Write-Host "   ✗ Agent Chat: 失败" -ForegroundColor Red
        Write-Host "     状态码: $($_.Exception.Response.StatusCode.value__)" -ForegroundColor Yellow
        
        # 尝试读取错误响应
        if ($_.Exception.Response) {
            try {
                $stream = $_.Exception.Response.GetResponseStream()
                $reader = New-Object System.IO.StreamReader($stream)
                $responseBody = $reader.ReadToEnd()
                if ($responseBody) {
                    Write-Host "`n     错误响应:" -ForegroundColor Yellow
                    # 尝试解析 JSON
                    try {
                        $errorJson = $responseBody | ConvertFrom-Json
                        $errorJson | ConvertTo-Json -Depth 5 | ForEach-Object {
                            Write-Host "     $_" -ForegroundColor Red
                        }
                    } catch {
                        Write-Host "     $responseBody" -ForegroundColor Red
                    }
                }
            } catch {
                Write-Host "     （无法读取错误响应）" -ForegroundColor Gray
            }
        }
    }
}

# 4. 配置检查
Write-Host "`n4. 配置检查..." -ForegroundColor Yellow
Write-Host "   - API URL: $baseUrl" -ForegroundColor White
Write-Host "   - 建议检查事项:" -ForegroundColor Cyan
Write-Host "     • AI_API_KEY secret 是否已更新" -ForegroundColor Gray
Write-Host "     • Workers 是否已重新部署" -ForegroundColor Gray
Write-Host "     • 是否已等待 1-2 分钟让配置生效" -ForegroundColor Gray

# 5. 诊断建议
Write-Host "`n5. 如果仍有问题，建议：" -ForegroundColor Yellow
Write-Host "   a) 查看 Cloudflare Dashboard 日志:" -ForegroundColor White
Write-Host "      https://dash.cloudflare.com/ → Workers → alou-edge → Logs" -ForegroundColor Gray
Write-Host "   b) 检查浏览器开发者工具 (F12) Network 标签页" -ForegroundColor White
Write-Host "   c) 使用测试脚本验证 API Key:" -ForegroundColor White
Write-Host "      .\scripts\test-deepseek-api.ps1 -ApiKey 'your-api-key'" -ForegroundColor Gray
Write-Host "   d) 重新部署 Workers:" -ForegroundColor White
Write-Host "      npm run deploy" -ForegroundColor Gray

Write-Host "`n========================================`n" -ForegroundColor Cyan

