# 生成 Tauri 图标脚本
# 从 image.png 生成所有需要的图标格式

$iconsDir = Join-Path $PSScriptRoot "..\src-tauri\icons"
$sourceImage = Join-Path $iconsDir "image.png"

if (-not (Test-Path $sourceImage)) {
    Write-Host "错误: 找不到 $sourceImage" -ForegroundColor Red
    exit 1
}

Write-Host "正在生成图标..." -ForegroundColor Green

# 使用 Tauri CLI 生成图标
# 注意: 需要先确保图片是正方形的
Set-Location (Join-Path $PSScriptRoot "..\src-tauri")

# 检查 Tauri CLI
if (-not (Get-Command tauri -ErrorAction SilentlyContinue)) {
    Write-Host "错误: 未找到 Tauri CLI。请运行: npm install -g @tauri-apps/cli" -ForegroundColor Red
    exit 1
}

# 尝试生成图标
try {
    tauri icon icons/image.png
    Write-Host "图标生成成功！" -ForegroundColor Green
} catch {
    Write-Host "警告: 自动生成图标失败。请手动创建 icon.ico 文件。" -ForegroundColor Yellow
    Write-Host "提示: 可以使用在线工具将 image.png 转换为 icon.ico" -ForegroundColor Yellow
}

Set-Location $PSScriptRoot

