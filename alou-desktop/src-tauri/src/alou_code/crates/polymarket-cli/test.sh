#!/bin/bash

# Polymarket CLI 测试脚本
echo "=== Polymarket CLI 测试 ==="

# 创建测试环境文件
cat > test.env << EOF
RPC_URL=https://polygon-rpc.com
PRIVATE_KEY=
MNEMONIC=
POLYMARKET_API_BASE=https://gamma-api.polymarket.com
EOF

echo "1. 测试帮助命令"
cargo run -- --help

echo -e "\n2. 测试市场列表命令"
cargo run -- list --active --limit 5

echo -e "\n3. 测试钱包命令"
cargo run -- wallet

echo -e "\n4. 测试分析命令（使用示例市场ID）"
cargo run -- analyze cli-test-market --timeframe 24h

echo -e "\n5. 测试交易命令（模拟）"
cargo run -- trade test-market-id YES 100.0 --price 0.50 --side buy

echo -e "\n=== 测试完成 ==="
echo "注意：这是一个模拟测试，实际交易需要配置钱包和API密钥"