#!/bin/bash
set -e

echo "=== 快速设置WSL环境用于alou-edge部署 ==="

# 检查是否在WSL中
if [[ ! -f /proc/version ]] || ! grep -q Microsoft /proc/version; then
    echo "警告: 这似乎不在WSL环境中运行"
    echo "建议在WSL中运行此脚本"
fi

# 快速安装必要工具
echo "安装必要工具..."
sudo apt-get update -qq
sudo apt-get install -y build-essential pkg-config libssl-dev curl

# 安装Rust（如果未安装）
if ! command -v rustc &> /dev/null; then
    echo "安装Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# 添加wasm32目标
rustup target add wasm32-unknown-unknown

# 安装worker-build
echo "安装worker-build..."
cargo install worker-build --force

# 安装wrangler（如果未安装）
if ! command -v wrangler &> /dev/null; then
    echo "安装wrangler..."
    npm install -g wrangler
fi

echo ""
echo "=== 设置完成! ==="
echo ""
echo "下一步:"
echo "1. 登录Cloudflare: wrangler login"
echo "2. 进入项目目录: cd /mnt/d/AI/alou-pay/aloupay/alou-edge"
echo "3. 运行部署脚本: ./deploy-wsl.sh"
echo ""
