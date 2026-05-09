# 图标文件说明

Tauri 应用需要图标文件。当前配置使用 `image.png` 作为主图标。

## 推荐的图标文件结构

为了更好的跨平台支持，建议提供以下图标文件：

### 必需文件（最小配置）
- `image.png` - 主图标文件（至少 512x512 像素）

### 完整配置（可选，用于更好的显示效果）
- `32x32.png` - 32x32 像素图标（Windows/Linux）
- `128x128.png` - 128x128 像素图标（Windows/Linux）
- `128x128@2x.png` - 256x256 像素图标（macOS Retina）
- `icon.icns` - macOS 图标集
- `icon.ico` - Windows 图标文件

## 从单个图标生成所有图标

如果你只有一个 `image.png` 文件，Tauri 会在构建时自动生成所需的图标格式。

确保 `image.png` 至少是 512x512 像素，以获得最佳效果。

