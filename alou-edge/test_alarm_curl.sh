#!/bin/bash

# 测试 Alarm 机制的简单脚本
# 使用 curl 发送请求并检查响应

echo "=== 测试 Alarm 机制 ==="

# 创建测试任务
echo "创建测试任务..."
curl -X POST "http://localhost:8787/api/agent/async" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer test-token" \
  -d '{
    "prompt": "测试 alarm 机制",
    "model": "deepseek-chat",
    "task_type": "async",
    "system_prompt": "这是一个测试任务，用于验证 alarm 机制是否正常工作。"
  }'

echo ""
echo ""
echo "等待1秒，让 alarm 触发..."
sleep 1

echo ""
echo "请查看 wrangler dev 控制台，应该能看到:"
echo "- '!!! ALARM ACTIVE !!!'"
echo "- '🚨 AITaskDO ALARM STARTED for task: ...'"
echo "- 其他 alarm 执行日志"

echo ""
echo "=== 测试完成 ==="
echo ""
echo "如果看到 '!!! ALARM ACTIVE !!!'，说明 alarm 机制正常工作"
echo "如果没有看到，请检查 Durable Object 配置和 set_alarm 参数"
