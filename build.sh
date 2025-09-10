#!/bin/bash

# Alou3 Rust 构建脚本

set -e

echo "🚀 开始构建 Alou3 Rust..."

# 检查Rust是否安装
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust未安装，请先安装Rust: https://rustup.rs/"
    exit 1
fi

# 检查Rust版本
RUST_VERSION=$(rustc --version | cut -d' ' -f2)
echo "📦 Rust版本: $RUST_VERSION"

# 检查环境变量
if [ -z "$DEEPSEEK_API_KEY" ]; then
    echo "⚠️  警告: 未设置 DEEPSEEK_API_KEY 环境变量"
    echo "   请设置您的DeepSeek API密钥:"
    echo "   export DEEPSEEK_API_KEY=your_api_key_here"
    echo ""
fi

# 构建项目
echo "🔨 构建项目..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ 构建成功！"
    echo ""
    echo "🎯 使用方法:"
    echo "  ./target/release/alou3-rust --help"
    echo "  ./target/release/alou3-rust chat"
    echo "  ./target/release/alou3-rust exec \"读取文件 /path/to/file.txt\""
    echo ""
    echo "📖 更多信息请查看 README.md"
else
    echo "❌ 构建失败"
    exit 1
fi
