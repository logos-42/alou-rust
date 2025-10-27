# 检查 Cloudflare Workers 日志
Write-Host "=== 查看 Cloudflare Workers 日志 ===" -ForegroundColor Cyan
Write-Host "请在浏览器中打开以下链接查看实时日志：" -ForegroundColor Yellow
Write-Host "https://dash.cloudflare.com/" -ForegroundColor Green
Write-Host ""
Write-Host "导航路径：" -ForegroundColor Yellow
Write-Host "1. 登录 Cloudflare Dashboard" -ForegroundColor White
Write-Host "2. 选择 Workers & Pages" -ForegroundColor White
Write-Host "3. 点击 'alou-edge'" -ForegroundColor White
Write-Host "4. 点击 'Logs' 标签" -ForegroundColor White
Write-Host "5. 点击 'Begin log stream' 开始查看实时日志" -ForegroundColor White
Write-Host ""
Write-Host "然后运行测试脚本：" -ForegroundColor Yellow
Write-Host ".\test_detailed_error.ps1" -ForegroundColor Green
