# Alou Edge API 测试命令
# 替换 YOUR_SUBDOMAIN 为实际部署的子域名

WORKER_URL="https://alou-edge.yuanjieliu65.workers.dev"

echo "🚀 测试 Alou Edge API"
echo "📍 Worker URL: $WORKER_URL"
echo ""

# 1. 健康检查
echo "1️⃣ 健康检查"
curl -s "$WORKER_URL/api/health" | jq '.'
echo ""

# 2. 创建会话
echo "2️⃣ 创建会话"
SESSION_RESPONSE=$(curl -s -X POST "$WORKER_URL/api/session" \
  -H "Content-Type: application/json" \
  -d '{
    "wallet_address": "0x1234567890123456789012345678901234567890",
    "chain": "ethereum"
  }')

echo "$SESSION_RESPONSE" | jq '.'
SESSION_ID=$(echo "$SESSION_RESPONSE" | jq -r '.session_id')
echo "Session ID: $SESSION_ID"
echo ""

# 3. 创建智能体
echo "3️⃣ 创建智能体"
AGENT_RESPONSE=$(curl -s -X POST "$WORKER_URL/api/agent/create" \
  -H "Content-Type: application/json" \
  -d "{
    \"session_id\": \"$SESSION_ID\",
    \"wallet_address\": \"0x1234567890123456789012345678901234567890\",
    \"chain\": \"ethereum\",
    \"name\": \"测试智能体\",
    \"avatar_cid\": \"QmTest123\",
    \"role_description\": \"这是一个测试智能体\"
  }")

echo "$AGENT_RESPONSE" | jq '.'
echo ""

# 4. 与智能体对话
echo "4️⃣ 与智能体对话"
curl -s -X POST "$WORKER_URL/api/agent/chat" \
  -H "Content-Type: application/json" \
  -d "{
    \"session_id\": \"$SESSION_ID\",
    \"message\": \"你好，请介绍一下你自己\"
  }" | jq '.'
echo ""

# 5. 搜索智能体
echo "5️⃣ 搜索智能体"
curl -s -X POST "$WORKER_URL/api/agent/search" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "测试"
  }' | jq '.'
echo ""

# 6. 解析智能体
echo "6️⃣ 解析智能体"
curl -s -X POST "$WORKER_URL/api/agent/resolve" \
  -H "Content-Type: application/json" \
  -d "{
    \"target\": \"test-agent.eth\",
    \"session_id\": \"$SESSION_ID\"
  }" | jq '.'
echo ""

# 7. 获取DIAP身份
echo "7️⃣ 获取DIAP身份"
curl -s -X GET "$WORKER_URL/api/agent/diap-identity?session_id=$SESSION_ID" | jq '.'
echo ""

# 8. 测试Claude Agent SDK
echo "8️⃣ 测试Claude Agent SDK"
curl -s -X POST "$WORKER_URL/api/agent/create-claude-agent" \
  -H "Content-Type: application/json" \
  -d "{
    \"session_id\": \"$SESSION_ID\",
    \"wallet_address\": \"0x1234567890123456789012345678901234567890\",
    \"chain\": \"ethereum\",
    \"name\": \"Claude测试智能体\",
    \"avatar_cid\": \"QmClaude123\"
  }" | jq '.'
echo ""

# 9. 测试钱包认证
echo "9️⃣ 测试钱包认证"
curl -s -X POST "$WORKER_URL/api/wallet/auth" \
  -H "Content-Type: application/json" \
  -d "{
    \"wallet_address\": \"0x1234567890123456789012345678901234567890\",
    \"signature\": \"0x1234567890abcdef\",
    \"message\": \"测试认证消息\"
  }" | jq '.'
echo ""

echo "✅ API 测试完成！"
echo ""
echo "💡 提示："
echo "- 某些API可能需要有效的钱包签名"
echo "- AI功能需要设置AI_API_KEY密钥"
echo "- DIAP功能需要本地IPFS节点运行"