#!/bin/bash

# 钱包管理工具测试脚本
# 用于测试智能体的钱包管理功能

API_URL="${API_URL:-http://localhost:8787}"
SESSION_ID="test_$(date +%s)"

echo "🧪 测试钱包管理工具"
echo "===================="
echo ""

# 创建会话
echo "📝 创建测试会话..."
curl -X POST "$API_URL/api/session" \
  -H "Content-Type: application/json" \
  -d "{\"wallet_address\": \"0x1234567890123456789012345678901234567890\"}" \
  -s | jq '.'

echo ""
echo "---"
echo ""

# 测试 1: 列出支持的网络
echo "✅ 测试 1: 列出支持的网络"
curl -X POST "$API_URL/api/agent/chat" \
  -H "Content-Type: application/json" \
  -d "{
    \"session_id\": \"$SESSION_ID\",
    \"message\": \"请列出所有支持的区块链网络\",
    \"wallet_address\": \"0x1234567890123456789012345678901234567890\"
  }" \
  -s | jq '.content'

echo ""
echo "---"
echo ""

# 测试 2: 切换到 Base Sepolia
echo "✅ 测试 2: 切换到 Base Sepolia"
curl -X POST "$API_URL/api/agent/chat" \
  -H "Content-Type: application/json" \
  -d "{
    \"session_id\": \"$SESSION_ID\",
    \"message\": \"请帮我切换到 Base Sepolia 测试网\",
    \"wallet_address\": \"0x1234567890123456789012345678901234567890\"
  }" \
  -s | jq '.'

echo ""
echo "---"
echo ""

# 测试 3: 查询当前网络
echo "✅ 测试 3: 查询当前网络"
curl -X POST "$API_URL/api/agent/chat" \
  -H "Content-Type: application/json" \
  -d "{
    \"session_id\": \"$SESSION_ID\",
    \"message\": \"我现在在哪个网络？\",
    \"wallet_address\": \"0x1234567890123456789012345678901234567890\"
  }" \
  -s | jq '.content'

echo ""
echo "---"
echo ""

# 测试 4: 切换到以太坊主网
echo "✅ 测试 4: 切换到以太坊主网"
curl -X POST "$API_URL/api/agent/chat" \
  -H "Content-Type: application/json" \
  -d "{
    \"session_id\": \"$SESSION_ID\",
    \"message\": \"切换到以太坊主网\",
    \"wallet_address\": \"0x1234567890123456789012345678901234567890\"
  }" \
  -s | jq '.'

echo ""
echo "---"
echo ""

# 测试 5: 检查余额
echo "✅ 测试 5: 检查钱包余额"
curl -X POST "$API_URL/api/agent/chat" \
  -H "Content-Type: application/json" \
  -d "{
    \"session_id\": \"$SESSION_ID\",
    \"message\": \"查看我的钱包余额\",
    \"wallet_address\": \"0x1234567890123456789012345678901234567890\"
  }" \
  -s | jq '.content'

echo ""
echo "===================="
echo "✅ 测试完成！"
