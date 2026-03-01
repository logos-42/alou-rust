#!/bin/bash

# Alou CLI 安装脚本
# 用法: ./install.sh

set -e

INSTALL_DIR="/usr/local/bin"
APP_DIR="/usr/local/alou-cli"

echo "正在安装 Alou CLI..."

# 检查权限
if [ "$EUID" -ne 0 ]; then
    echo "请使用 sudo 运行此脚本安装到系统目录"
    echo "或者指定安装目录: ./install.sh \$HOME/.local/bin"
    INSTALL_DIR="${1:-$HOME/.local/bin}"
    APP_DIR="$HOME/.local/alou-cli"
    mkdir -p "$INSTALL_DIR"
    mkdir -p "$APP_DIR"
fi

# 复制文件
echo "安装到: $APP_DIR"
mkdir -p "$APP_DIR/bin"
cp -R "$(dirname "$0")/alou-cli/"* "$APP_DIR/"

# 创建符号链接
ln -sf "$APP_DIR/bin/alou-cli" "$INSTALL_DIR/alou"

echo ""
echo "安装完成!"
echo "运行: alou --help"
