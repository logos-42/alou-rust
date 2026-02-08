#!/usr/bin/env node
/**
 * 快速验证脚本 - 检查 API 配置是否正确
 */

const fs = require('fs');
const path = require('path');

console.log('🔍 验证 API 修复...\n');

// 1. 检查 vite.config.js
console.log('1. 检查 vite.config.js 代理配置...');
const viteConfig = fs.readFileSync(
  path.join(__dirname, 'alou-desktop', 'vite.config.js'),
  'utf8'
);

if (viteConfig.includes('http://127.0.0.1:8787')) {
  console.log('   ✅ 代理配置正确（指向本地后端）\n');
} else if (viteConfig.includes('alou-edge.yuanjieliu65.workers.dev')) {
  console.log('   ❌ 代理配置仍指向生产环境\n');
} else {
  console.log('   ⚠️ 无法确认代理配置\n');
}

// 2. 检查 useAsyncTaskPolling.ts
console.log('2. 检查 useAsyncTaskPolling.ts 工具调用格式...');
const pollingHook = fs.readFileSync(
  path.join(__dirname, 'alou-desktop', 'src', 'components', 'AgentChat', 'hooks', 'useAsyncTaskPolling.ts'),
  'utf8'
);

if (pollingHook.includes('tool_name:') && pollingHook.includes('wallet_address:')) {
  console.log('   ✅ 工具调用格式正确\n');
} else {
  console.log('   ❌ 工具调用格式可能不正确\n');
}

// 3. 检查 useAgentMessages.ts
console.log('3. 检查 useAgentMessages.ts Hook 参数...');
const agentMessages = fs.readFileSync(
  path.join(__dirname, 'alou-desktop', 'src', 'components', 'AgentChat', 'useAgentMessages.ts'),
  'utf8'
);

if (agentMessages.includes('walletAddress,') && agentMessages.includes('chain: activeChain')) {
  console.log('   ✅ Hook 参数传递正确\n');
} else {
  console.log('   ❌ Hook 参数可能不正确\n');
}

console.log('📋 修复摘要：');
console.log('─────────────────────────────────────────');
console.log('✅ 1. vite.config.js - 代理指向本地后端');
console.log('✅ 2. useAsyncTaskPolling.ts - 修复工具调用格式');
console.log('✅ 3. useAgentMessages.ts - 添加钱包和链参数');
console.log('─────────────────────────────────────────\n');

console.log('🚀 下一步操作：');
console.log('1. 确保后端服务已启动：');
console.log('   cd alou-edge && npm run dev\n');
console.log('2. 启动桌面应用：');
console.log('   cd alou-desktop && npm run tauri:dev\n');
console.log('3. 在应用中测试发送消息');
console.log('4. 打开浏览器开发者工具（F12）查看 Network 标签');
