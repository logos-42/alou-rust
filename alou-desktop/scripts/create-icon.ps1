# 创建最小的 ICO 文件作为占位符
# 这将创建一个 16x16 的简单图标

$iconsDir = Join-Path $PSScriptRoot "..\src-tauri\icons"
$icoPath = Join-Path $iconsDir "icon.ico"
$pngPath = Join-Path $iconsDir "image.png"

# 尝试使用 .NET 从 PNG 创建 ICO
try {
    Add-Type -AssemblyName System.Drawing
    
    if (Test-Path $pngPath) {
        $img = [System.Drawing.Image]::FromFile($pngPath)
        
        # 创建正方形版本（取最小边）
        $size = [Math]::Min($img.Width, $img.Height)
        $bitmap = New-Object System.Drawing.Bitmap $size, $size
        $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
        $graphics.DrawImage($img, 0, 0, $size, $size)
        
        # 保存为临时 PNG
        $tempPng = Join-Path $iconsDir "temp_square.png"
        $bitmap.Save($tempPng, [System.Drawing.Imaging.ImageFormat]::Png)
        
        $graphics.Dispose()
        $bitmap.Dispose()
        $img.Dispose()
        
        Write-Host "已创建正方形图片: $tempPng" -ForegroundColor Green
        Write-Host "现在请运行: npm run tauri icon $tempPng" -ForegroundColor Yellow
    } else {
        Write-Host "错误: 找不到 $pngPath" -ForegroundColor Red
    }
} catch {
    Write-Host "无法使用 .NET 处理图片: $_" -ForegroundColor Red
    Write-Host "请手动将 image.png 转换为正方形的 icon.ico 文件" -ForegroundColor Yellow
}

