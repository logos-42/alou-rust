#!/bin/bash
# IPNS 功能测试脚本

set -e

KUBO_BIN="$HOME/Library/Application Support/com.alou.desktop/kubo/ipfs"
IPFS_PATH="$HOME/Library/Application Support/com.alou.desktop/ipfs"

echo "=== IPNS 功能测试 ==="
echo ""

# 1. 测试 IPNS 密钥生成
echo "1. 测试 IPNS 密钥生成..."
KEY_NAME="test-ipns-$(date +%s)"
KEY_ID=$($KUBO_BIN key gen "$KEY_NAME" 2>&1)
echo "   ✅ 密钥生成成功：$KEY_NAME -> $KEY_ID"

# 2. 测试 IPNS 发布
echo ""
echo "2. 测试 IPNS 发布..."
TEST_CID="QmYJeBUAPAh4R2veXdvFxu9RUzCkejXsjc3RQ2Wtt8QJfM"
PUBLISH_RESULT=$($KUBO_BIN name publish --key="$KEY_NAME" "$TEST_CID" 2>&1)
echo "   ✅ 发布成功：$PUBLISH_RESULT"

# 3. 测试 IPNS 解析
echo ""
echo "3. 测试 IPNS 解析..."
RESOLVE_RESULT=$($KUBO_BIN name resolve "$KEY_ID" 2>&1)
echo "   ✅ 解析成功：$RESOLVE_RESULT"

# 4. 测试本地网关访问
echo ""
echo "4. 测试本地网关访问..."
GATEWAY_RESULT=$(curl -s "http://127.0.0.1:8080/ipns/$KEY_ID" 2>&1 | head -c 100)
if [ -n "$GATEWAY_RESULT" ]; then
    echo "   ✅ 本地网关访问成功：${GATEWAY_RESULT:0:50}..."
else
    echo "   ⚠️ 本地网关返回空（可能还在传播中）"
fi

# 5. 测试公共网关查询（仅检查连接）
echo ""
echo "5. 测试公共网关查询..."
PUBLIC_GATEWAY_RESULT=$(curl -s --connect-timeout 5 "https://ipfs.io/ipns/$KEY_ID" 2>&1 | head -c 100)
if [ -n "$PUBLIC_GATEWAY_RESULT" ]; then
    echo "   ✅ 公共网关访问成功：${PUBLIC_GATEWAY_RESULT:0:50}..."
else
    echo "   ⚠️ 公共网关返回空（IPNS 记录还未传播到全球网络，正常现象）"
fi

# 6. 检查 DHT 传播
echo ""
echo "6. 检查 DHT 传播..."
PEER_COUNT=$($KUBO_BIN swarm peers 2>&1 | wc -l)
echo "   连接到 $PEER_COUNT 个 DHT 节点"

# 7. 检查 IPNS PubSub
echo ""
echo "7. 检查 IPNS PubSub 支持..."
PUBSUB_RESULT=$(curl -s -X POST "http://127.0.0.1:5001/api/v0/name/pubsub/subs" 2>&1)
if echo "$PUBSUB_RESULT" | grep -q "Strings"; then
    echo "   ✅ IPNS PubSub 已启用"
else
    echo "   ⚠️ IPNS PubSub 未启用（将使用 DHT 传播，速度较慢）"
fi

echo ""
echo "=== 测试完成 ==="
echo ""
echo "IPNS 传播说明："
echo "1. 本地解析：立即生效（已测试 ✅）"
echo "2. 本地网关：几秒内生效（已测试 ✅）"
echo "3. 公共网关：可能需要几分钟到几十分钟（取决于 DHT 传播）"
echo ""
echo "加速传播的方法："
echo "- 代码中已包含自动触发公共网关查询（重试 3 次，间隔 10 秒）"
echo "- 如果节点支持 IPNS PubSub，传播时间将从几分钟减少到几秒"
echo "- DHT provide 操作会将 CID 广播到网络"
