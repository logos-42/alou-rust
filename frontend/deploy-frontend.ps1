# 部署前端到Cloudflare Pages
param(
    [Parameter()]
    [string]$ProjectName = "alou-frontend"
)

Write-Host "=== 部署前端到Cloudflare Pages ===" -ForegroundColor Green
Write-Host "项目名称: $ProjectName" -ForegroundColor Cyan
Write-Host ""

# 检查是否已登录
Write-Host "检查Cloudflare登录状态..." -ForegroundColor Yellow
try {
    $whoami = wrangler whoami 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Host "未登录Cloudflare，请先登录" -ForegroundColor Red
        Write-Host "运行: wrangler login" -ForegroundColor Yellow
        exit 1
    }
    Write-Host "✓ 已登录Cloudflare" -ForegroundColor Green
} catch {
    Write-Host "✗ 检查登录状态失败" -ForegroundColor Red
    exit 1
}

# 检查dist目录
if (!(Test-Path "dist")) {
    Write-Host "dist目录不存在，开始构建..." -ForegroundColor Yellow
    npm run build
    if ($LASTEXITCODE -ne 0) {
        Write-Host "✗ 构建失败" -ForegroundColor Red
        exit 1
    }
    Write-Host "✓ 构建完成" -ForegroundColor Green
} else {
    Write-Host "✓ dist目录已存在" -ForegroundColor Green
}

# 检查dist目录内容
$distFiles = Get-ChildItem dist
if ($distFiles.Count -eq 0) {
    Write-Host "dist目录为空，重新构建..." -ForegroundColor Yellow
    npm run build
    if ($LASTEXITCODE -ne 0) {
        Write-Host "✗ 构建失败" -ForegroundColor Red
        exit 1
    }
}

Write-Host "dist目录内容:" -ForegroundColor Gray
Get-ChildItem dist | ForEach-Object { Write-Host "  - $($_.Name)" -ForegroundColor Gray }

# 部署到Cloudflare Pages
Write-Host "`n部署到Cloudflare Pages..." -ForegroundColor Yellow
wrangler pages deploy dist --project-name=$ProjectName

if ($LASTEXITCODE -eq 0) {
    Write-Host "`n=== 部署成功! ===" -ForegroundColor Green
    Write-Host ""
    Write-Host "前端URL: https://$ProjectName.pages.dev" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "下一步:" -ForegroundColor Yellow
    Write-Host "1. 访问 https://$ProjectName.pages.dev 查看前端" -ForegroundColor White
    Write-Host "2. 在Cloudflare Dashboard中配置环境变量:" -ForegroundColor White
    Write-Host "   - VITE_API_BASE_URL = https://alou-edge.yuanjieliu65.workers.dev" -ForegroundColor Gray
    Write-Host "   - VITE_AGENT_API_URL = https://alou-edge.yuanjieliu65.workers.dev" -ForegroundColor Gray
    Write-Host "3. 重新部署以应用环境变量" -ForegroundColor White
} else {
    Write-Host "`n✗ 部署失败" -ForegroundColor Red
    exit 1
}
