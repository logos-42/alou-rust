#!/usr/bin/env node
/**
 * 测试 Claude Agent SDK 脚本
 * 用于验证 Node.js 脚本是否能正常工作
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// 创建测试输入文件
const testInput = {
  apiKey: process.env.ANTHROPIC_API_KEY || 'test-key',
  prompt: 'Hello, this is a test message. Please respond with "Test successful!"',
  systemPrompt: 'You are a helpful assistant.',
  history: [],
  model: 'claude-3-5-sonnet-20241022',
  maxTokens: 100,
  temperature: 0.7,
};

const testInputFile = path.join(__dirname, 'test-input.json');
const claudeAgentScript = path.join(__dirname, 'claude-agent.js');

console.log('🧪 开始测试 Claude Agent SDK 脚本...\n');

// 检查脚本文件是否存在
if (!fs.existsSync(claudeAgentScript)) {
  console.error('❌ 错误: 找不到 claude-agent.js 脚本');
  process.exit(1);
}

// 写入测试输入
fs.writeFileSync(testInputFile, JSON.stringify(testInput, null, 2));
console.log('✅ 创建测试输入文件:', testInputFile);

// 检查 Node.js 版本
const nodeVersion = process.version;
console.log('📦 Node.js 版本:', nodeVersion);

// 检查 SDK 是否已安装
try {
  const sdkPath = path.join(__dirname, '..', 'node_modules', '@anthropic-ai', 'claude-agent-sdk');
  if (fs.existsSync(sdkPath)) {
    console.log('✅ Claude Agent SDK 已安装');
  } else {
    console.warn('⚠️  警告: Claude Agent SDK 可能未正确安装');
  }
} catch (error) {
  console.warn('⚠️  警告: 无法检查 SDK 安装状态');
}

console.log('\n📝 测试输入:');
console.log(JSON.stringify(testInput, null, 2));

console.log('\n💡 提示:');
console.log('1. 如果设置了 ANTHROPIC_API_KEY 环境变量，将使用真实的 API key');
console.log('2. 否则将使用测试 key，可能会失败');
console.log('3. 要运行完整测试，请执行: node scripts/claude-agent.js scripts/test-input.json');

// 清理测试文件
process.on('exit', () => {
  if (fs.existsSync(testInputFile)) {
    fs.unlinkSync(testInputFile);
  }
  const resultFile = testInputFile.replace('.json', '.result.json');
  const errorFile = testInputFile.replace('.json', '.error.json');
  if (fs.existsSync(resultFile)) {
    fs.unlinkSync(resultFile);
  }
  if (fs.existsSync(errorFile)) {
    fs.unlinkSync(errorFile);
  }
});

console.log('\n✅ 测试脚本准备完成');

