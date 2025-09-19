#!/bin/bash

# Alou Pay 更新脚本
# 用于更新已部署的应用

set -e

APP_NAME="alou-pay"
APP_DIR="/opt/${APP_NAME}"

echo "🔄 开始更新 ${APP_NAME}..."

# 检查是否在项目根目录
if [ ! -f "Cargo.toml" ] || [ ! -d "frontend" ]; then
    echo "❌ 请在项目根目录运行此脚本"
    exit 1
fi

# 拉取最新代码
echo "📥 拉取最新代码..."
git pull origin main || echo "⚠️ Git pull 失败，继续使用本地代码"

# 重新构建 Rust 后端
echo "🦀 重新构建 Rust 后端..."
source $HOME/.cargo/env
cargo build --release --bin agent-http-server

# 停止后端服务
echo "🛑 停止后端服务..."
pm2 stop alou-backend

# 更新后端
echo "📦 更新 Rust 后端..."
cp target/release/agent-http-server ${APP_DIR}/backend/
cp agent_config.json ${APP_DIR}/backend/ 2>/dev/null || true
cp mcp.json ${APP_DIR}/backend/ 2>/dev/null || true

# 重新构建前端
echo "🎨 重新构建 Vue 前端..."
cd frontend
npm ci
npm run build
cd ..

# 更新前端
echo "📦 更新 Vue 前端..."
sudo rm -rf ${APP_DIR}/frontend/*
sudo cp -r frontend/dist/* ${APP_DIR}/frontend/

# 重启服务
echo "🚀 重启服务..."
pm2 restart alou-backend
sudo systemctl reload nginx

# 显示状态
echo "📊 服务状态："
pm2 status

echo "✅ 更新完成！"
