#!/bin/bash
set -e

echo "=== 在WSL中部署alou-edge到Cloudflare Workers ==="

# 检查是否在WSL中
if [[ ! -f /proc/version ]] || ! grep -q Microsoft /proc/version; then
    echo "警告: 这似乎不在WSL环境中运行"
    echo "建议在WSL中运行此脚本"
fi

# 更新包列表
echo "更新包列表..."
sudo apt-get update

# 安装构建工具
echo "安装构建工具..."
sudo apt-get install -y build-essential pkg-config libssl-dev curl

# 检查并安装Rust
if ! command -v rustc &> /dev/null; then
    echo "安装Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "Rust已安装: $(rustc --version)"
fi

# 添加wasm32目标
echo "添加wasm32-unknown-unknown目标..."
rustup target add wasm32-unknown-unknown

# 安装worker-build
echo "安装worker-build..."
cargo install worker-build --force

# 检查并安装Node.js
if ! command -v node &> /dev/null; then
    echo "安装Node.js..."
    curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
    sudo apt-get install -y nodejs
else
    echo "Node.js已安装: $(node --version)"
fi

# 安装wrangler
if ! command -v wrangler &> /dev/null; then
    echo "安装wrangler..."
    npm install -g wrangler
else
    echo "Wrangler已安装: $(wrangler --version)"
fi

# 检查Cloudflare登录状态
echo "检查Cloudflare登录状态..."
if ! wrangler whoami &> /dev/null; then
    echo "请先登录Cloudflare:"
    echo "wrangler login"
    exit 1
fi

echo "✓ 已登录Cloudflare"

# 进入项目目录
PROJECT_DIR="/mnt/d/AI/alou-pay/aloupay/alou-edge"
if [[ -d "$PROJECT_DIR" ]]; then
    cd "$PROJECT_DIR"
    echo "进入项目目录: $PROJECT_DIR"
else
    echo "错误: 找不到项目目录 $PROJECT_DIR"
    echo "请确保项目路径正确"
    exit 1
fi

# 清理之前的构建
echo "清理之前的构建..."
cargo clean

# 构建WASM
echo "构建WASM..."
cargo build --target wasm32-unknown-unknown --release

# 检查WASM文件
WASM_FILE="target/wasm32-unknown-unknown/release/alou_edge.wasm"
if [[ -f "$WASM_FILE" ]]; then
    SIZE=$(du -h "$WASM_FILE" | cut -f1)
    echo "✓ WASM文件构建成功: $SIZE"
else
    echo "✗ WASM文件构建失败"
    exit 1
fi

# 使用worker-build生成JavaScript绑定
echo "使用worker-build生成JavaScript绑定..."
worker-build --release

# 检查生成的文件
if [[ -f "build/worker/shim.mjs" ]]; then
    echo "✓ JavaScript绑定生成成功"
else
    echo "✗ JavaScript绑定生成失败"
    exit 1
fi

# 检查secrets
echo "检查必要的secrets..."
SECRETS=$(wrangler secret list 2>&1)
if echo "$SECRETS" | grep -q "AI_API_KEY"; then
    echo "✓ AI_API_KEY已配置"
else
    echo "⚠ AI_API_KEY未配置，请运行: wrangler secret put AI_API_KEY"
fi

if echo "$SECRETS" | grep -q "JWT_SECRET"; then
    echo "✓ JWT_SECRET已配置"
else
    echo "⚠ JWT_SECRET未配置，请运行: wrangler secret put JWT_SECRET"
fi

# 部署到Cloudflare Workers
echo "部署到Cloudflare Workers..."
wrangler deploy

if [[ $? -eq 0 ]]; then
    echo ""
    echo "=== 部署成功! ==="
    echo ""
    echo "Worker URL: https://alou-edge.yuanjieliu65.workers.dev"
    echo ""
    echo "测试部署:"
    echo "curl https://alou-edge.yuanjieliu65.workers.dev/api/health"
    echo ""
    echo "查看日志:"
    echo "wrangler tail"
else
    echo "✗ 部署失败"
    exit 1
fi
