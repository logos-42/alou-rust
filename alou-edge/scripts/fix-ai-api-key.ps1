# 修复 AI_API_KEY 配置脚本
param(
    [Parameter(Mandatory=$false)]
    [string]$ApiKey = ""
)

$ErrorActionPreference = "Stop"

Write-Host "`n=== 修复 Workers AI_API_KEY 配置 ===" -ForegroundColor Cyan

if ([string]::IsNullOrEmpty($ApiKey)) {
    Write-Host "`n⚠️  请提供新的 DeepSeek API Key" -ForegroundColor Yellow
    Write-Host "`n使用方法：" -ForegroundColor Cyan
    Write-Host "  .\scripts\fix-ai-api-key.ps1 -ApiKey 'your-new-api-key'`n" -ForegroundColor White
    Write-Host "或者交互式输入（不推荐，可能会在历史记录中留下痕迹）：" -ForegroundColor Yellow
    Write-Host "  `$apiKey = Read-Host '请输入 DeepSeek API Key'; .\scripts\fix-ai-api-key.ps1 -ApiKey `$apiKey`n" -ForegroundColor White
    
    Write-Host "获取 DeepSeek API Key：" -ForegroundColor Cyan
    Write-Host "  1. 访问 https://platform.deepseek.com/" -ForegroundColor White
    Write-Host "  2. 登录账户" -ForegroundColor White
    Write-Host "  3. 进入 API Keys 页面" -ForegroundColor White
    Write-Host "  4. 创建或复制 API Key`n" -ForegroundColor White
    
    exit 1
}

# 验证 API Key 格式
if (-not $ApiKey.StartsWith("sk-")) {
    Write-Host "⚠️  警告：API Key 通常以 'sk-' 开头" -ForegroundColor Yellow
    $confirm = Read-Host "是否继续？(y/n)"
    if ($confirm -ne "y" -and $confirm -ne "Y") {
        Write-Host "已取消" -ForegroundColor Yellow
        exit 0
    }
}

Write-Host "`n步骤 1: 测试 API Key 是否有效..." -ForegroundColor Cyan
$testScript = Join-Path $PSScriptRoot "test-deepseek-api.ps1"
if (Test-Path $testScript) {
    & $testScript -ApiKey $ApiKey
    if ($LASTEXITCODE -ne 0) {
        Write-Host "`n⚠️  API Key 测试失败，但仍然可以继续设置" -ForegroundColor Yellow
        $confirm = Read-Host "是否继续设置？(y/n)"
        if ($confirm -ne "y" -and $confirm -ne "Y") {
            Write-Host "已取消" -ForegroundColor Yellow
            exit 0
        }
    }
} else {
    Write-Host "⚠️  测试脚本不存在，跳过测试" -ForegroundColor Yellow
}

Write-Host "`n步骤 2: 更新 Workers Secret..." -ForegroundColor Cyan
try {
    # 使用 wrangler secret put 命令
    Write-Host "正在更新 AI_API_KEY secret..." -ForegroundColor Yellow
    
    # 创建临时文件来传递 secret（避免在命令行中显示）
    $tempFile = [System.IO.Path]::GetTempFileName()
    $ApiKey | Out-File -FilePath $tempFile -Encoding utf8 -NoNewline
    
    # 使用 wrangler secret put 命令
    $process = Start-Process -FilePath "npx" `
        -ArgumentList "wrangler", "secret", "put", "AI_API_KEY" `
        -NoNewWindow -Wait -PassThru -RedirectStandardInput $tempFile
    
    Remove-Item $tempFile -Force
    
    if ($process.ExitCode -eq 0) {
        Write-Host "`n✅ AI_API_KEY 已成功更新！" -ForegroundColor Green
        Write-Host "`n下一步：" -ForegroundColor Cyan
        Write-Host "  1. 等待几秒钟让 Workers 重新加载配置" -ForegroundColor White
        Write-Host "  2. 重新测试 API 调用" -ForegroundColor White
        Write-Host "  3. 如果仍有问题，查看 Workers 日志：" -ForegroundColor White
        Write-Host "     npx wrangler tail`n" -ForegroundColor White
    } else {
        Write-Host "`n❌ 更新失败，请手动运行以下命令：" -ForegroundColor Red
        Write-Host "  npx wrangler secret put AI_API_KEY`n" -ForegroundColor White
        Write-Host "然后在提示时输入你的 API Key`n" -ForegroundColor Yellow
    }
    
} catch {
    Write-Host "`n❌ 更新过程中出错: $_" -ForegroundColor Red
    Write-Host "`n请手动运行以下命令：" -ForegroundColor Yellow
    Write-Host "  npx wrangler secret put AI_API_KEY`n" -ForegroundColor White
}

Write-Host ""

