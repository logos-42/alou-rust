# 诊断 API 错误脚本
Write-Host "`n=== DeepSeek API 错误诊断 ===" -ForegroundColor Cyan

Write-Host "`n错误信息：" -ForegroundColor Yellow
Write-Host "  - 状态码: 401 (Unauthorized)" -ForegroundColor White
Write-Host "  - 错误消息: Authentication Fails (governor)" -ForegroundColor White

Write-Host "`n可能的原因：" -ForegroundColor Yellow
Write-Host "  1. ✅ API Key 已设置但可能无效或过期" -ForegroundColor White
Write-Host "  2. ✅ API Key 格式不正确" -ForegroundColor White
Write-Host "  3. ✅ DeepSeek 账户余额不足" -ForegroundColor White
Write-Host "  4. ✅ 触发了限流保护（governor）" -ForegroundColor White

Write-Host "`n解决方案：" -ForegroundColor Green
Write-Host "`n方案 1: 更新 AI_API_KEY Secret" -ForegroundColor Cyan
Write-Host "  运行以下命令更新 API Key：" -ForegroundColor White
Write-Host "    npx wrangler secret put AI_API_KEY" -ForegroundColor Yellow
Write-Host "  （然后输入你的新 DeepSeek API Key）`n" -ForegroundColor Gray

Write-Host "方案 2: 测试 API Key 是否有效" -ForegroundColor Cyan
Write-Host "  如果你有 API Key，可以先用测试脚本验证：" -ForegroundColor White
Write-Host "    .\scripts\test-deepseek-api.ps1 -ApiKey 'your-api-key'`n" -ForegroundColor Yellow

Write-Host "方案 3: 检查 DeepSeek 账户状态" -ForegroundColor Cyan
Write-Host "  1. 访问 https://platform.deepseek.com/" -ForegroundColor White
Write-Host "  2. 登录并检查：" -ForegroundColor White
Write-Host "     - API Key 是否有效" -ForegroundColor Gray
Write-Host "     - 账户余额是否充足" -ForegroundColor Gray
Write-Host "     - 是否有任何限制或警告`n" -ForegroundColor Gray

Write-Host "方案 4: 查看 Workers 实时日志" -ForegroundColor Cyan
Write-Host "  运行以下命令查看实时日志：" -ForegroundColor White
Write-Host "    npx wrangler tail`n" -ForegroundColor Yellow

Write-Host "当前配置状态：" -ForegroundColor Cyan
Write-Host "  ✓ AI_API_KEY secret 已存在于 Workers" -ForegroundColor Green
Write-Host "  ⚠️  但可能值不正确或已过期`n" -ForegroundColor Yellow

Write-Host "快速修复命令：" -ForegroundColor Cyan
Write-Host "  cd alou-edge" -ForegroundColor White
Write-Host "  npx wrangler secret put AI_API_KEY" -ForegroundColor Yellow
Write-Host "  (输入你的 DeepSeek API Key)`n" -ForegroundColor Gray

