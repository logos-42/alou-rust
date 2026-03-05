#!/bin/bash

# Alou Desktop 完整安装脚本
# 解决 Applications 中无法打开的问题

set -e

echo "🔧 Alou Desktop 完整安装工具"
echo ""

# 等待构建完成
BUILD_DIR="/Users/apple/Downloads/alou/alou-desktop/src-tauri/target/release/bundle/macos"
if [ ! -d "$BUILD_DIR/Alou.app" ]; then
    echo "⏳ 等待构建完成..."
    sleep 30
fi

# 检查构建产物
if [ ! -d "$BUILD_DIR/Alou.app" ]; then
    echo "❌ 构建未完成或失败"
    exit 1
fi

echo "✅ 构建完成"
echo ""

# 1. 关闭运行中的应用
echo "1️⃣  关闭运行中的应用..."
pkill -x "Alou" 2>/dev/null || true
pkill -x "alou-desktop" 2>/dev/null || true
sleep 2

# 2. 清理旧的 Applications 版本
echo "2️⃣  清理旧版本..."
sudo rm -rf /Applications/Alou.app 2>/dev/null || true
rm -rf /Applications/Alou.app 2>/dev/null || true

# 3. 复制新版本（直接从构建目录）
echo "3️⃣  复制新版本到 Applications..."
cp -R "$BUILD_DIR/Alou.app" /Applications/

# 4. 移除所有隔离属性
echo "4️⃣  移除隔离属性..."
sudo xattr -rd com.apple.quarantine /Applications/Alou.app 2>/dev/null || true
sudo xattr -rd com.apple.provenance /Applications/Alou.app 2>/dev/null || true
xattr -rd com.apple.quarantine /Applications/Alou.app 2>/dev/null || true
xattr -rd com.apple.provenance /Applications/Alou.app 2>/dev/null || true

# 5. 重新签名（关键！）
echo "5️⃣  重新签名应用..."
sudo codesign --force --deep --sign - /Applications/Alou.app 2>/dev/null || true
codesign --force --deep --sign - /Applications/Alou.app 2>/dev/null || true

# 6. 修复权限
echo "6️⃣  修复权限..."
sudo chmod -R 755 /Applications/Alou.app

# 7. 清除缓存
echo "7️⃣  清除缓存..."
rm -rf ~/Library/Caches/com.alou.desktop 2>/dev/null || true
rm -rf ~/Library/Saved\ Application\ State/com.alou.desktop.savedState 2>/dev/null || true

# 8. 验证
echo ""
echo "🔍 验证安装..."
if [ ! -d "/Applications/Alou.app" ]; then
    echo "❌ 安装失败"
    exit 1
fi

# 检查签名
if codesign -dv --verbose=4 /Applications/Alou.app 2>&1 | grep -q "adhoc"; then
    echo "✅ 应用已签名"
else
    echo "⚠️  签名可能有问题"
fi

# 检查隔离属性
if xattr -p com.apple.quarantine /Applications/Alou.app 2>/dev/null; then
    echo "⚠️  仍有隔离属性"
else
    echo "✅ 隔离属性已移除"
fi

echo ""
echo "✅ 安装完成！"
echo ""
echo "📍 应用位置：/Applications/Alou.app"
echo ""

# 询问是否打开
read -p "是否现在打开 Alou？(y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "🚀 启动应用..."
    
    # 再次确保签名
    codesign --force --deep --sign - /Applications/Alou.app 2>/dev/null || true
    
    # 打开应用
    open /Applications/Alou.app
    
    echo ""
    echo "✅ Alou 已启动！"
    echo ""
    echo "💡 如果仍然闪退，请查看崩溃报告或联系开发者"
fi
