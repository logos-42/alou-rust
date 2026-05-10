#!/bin/bash

# Polymarket CLI 安装脚本

set -e

echo "=== Polymarket CLI 安装 ==="

# 检查 Rust 是否安装
if ! command -v cargo &> /dev/null; then
    echo "错误: Rust 未安装"
    echo "请先安装 Rust: https://rustup.rs/"
    exit 1
fi

echo "1. 克隆仓库（如果尚未克隆）"
if [ ! -d "polymarket-cli" ]; then
    echo "请先克隆仓库: git clone <repository-url>"
    echo "然后进入目录: cd polymarket-cli"
    exit 1
fi

echo "2. 构建项目"
cargo build --release

echo "3. 创建配置文件"
if [ ! -f ".env" ]; then
    if [ -f ".env.example" ]; then
        cp .env.example .env
        echo "已创建 .env 文件，请编辑配置"
    else
        echo "警告: .env.example 文件不存在"
        echo "请手动创建 .env 文件"
    fi
fi

echo "4. 安装到系统路径（可选）"
read -p "是否安装到 /usr/local/bin? (y/n): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    sudo cp target/release/polymarket-cli /usr/local/bin/
    echo "已安装到 /usr/local/bin/polymarket-cli"
fi

echo "5. 验证安装"
if command -v polymarket-cli &> /dev/null || [ -f "target/release/polymarket-cli" ]; then
    echo "安装成功!"
    echo ""
    echo "使用方法:"
    echo "  polymarket-cli --help"
    echo ""
    echo "下一步:"
    echo "  1. 编辑 .env 文件配置钱包和API"
    echo "  2. 运行测试: ./test.sh"
    echo "  3. 查看示例: cat EXAMPLES.md"
else
    echo "安装可能失败，请检查错误信息"
fi

echo ""
echo "=== 安装完成 ==="