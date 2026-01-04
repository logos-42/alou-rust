#!/usr/bin/env node
/**
 * 测试 Claude Agent SDK 格式要求
 * 验证 SDK 如何发送请求和使用 ANTHROPIC_BASE_URL
 */

import { query } from '@anthropic-ai/claude-agent-sdk';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

async function testSdkFormat() {
  console.log('=== 测试 Claude Agent SDK 格式要求 ===\n');
  
  // 测试 1: 检查 SDK 的 query 函数签名
  console.log('1. 检查 SDK query 函数签名...');
  console.log('   query 函数类型:', typeof query);
  console.log('   query 是函数:', typeof query === 'function');
  
  // 测试 2: 模拟 SDK 请求格式
  console.log('\n2. 模拟 SDK 请求格式...');
  
  const testOptions = {
    prompt: "Hello, world!",
    systemPrompt: "You are a helpful assistant.",
    model: "claude-3-5-sonnet-20241022",
    maxTokens: 100,
    temperature: 0.7,
  };
  
  console.log('   请求选项结构:');
  console.log('   - prompt:', testOptions.prompt);
  console.log('   - systemPrompt:', testOptions.systemPrompt);
  console.log('   - model:', testOptions.model);
  console.log('   - maxTokens:', testOptions.maxTokens);
  console.log('   - temperature:', testOptions.temperature);
  
  // 测试 3: 检查环境变量使用
  console.log('\n3. 检查环境变量使用...');
  
  // 设置不同的 ANTHROPIC_BASE_URL 进行测试
  const testUrls = [
    'http://127.0.0.1:8787/api/claude-agent/query',
    'http://127.0.0.1:8787/api/agent/chat',
    'https://api.anthropic.com/v1/',
  ];
  
  console.log('   可能的 ANTHROPIC_BASE_URL 值:');
  testUrls.forEach((url, index) => {
    console.log(`   ${index + 1}. ${url}`);
  });
  
  // 测试 4: 检查 SDK 实际行为
  console.log('\n4. 分析 SDK 实际行为...');
  
  // 根据 Claude Agent SDK 文档，它可能：
  // 1. 使用 ANTHROPIC_BASE_URL 作为 API 端点
  // 2. 发送特定格式的 HTTP 请求
  // 3. 期望特定格式的响应
  
  console.log('   SDK 可能的行为:');
  console.log('   - 使用 POST 方法');
  console.log('   - 发送 JSON 请求体');
  console.log('   - 包含 Authorization 头');
  console.log('   - 期望 JSON 响应');
  
  // 测试 5: 检查后端端点格式要求
  console.log('\n5. 检查后端端点格式要求...');
  
  // 检查 /api/claude-agent/query 端点
  console.log('   /api/claude-agent/query 端点:');
  console.log('   - 期望 ClaudeSdkRequest 格式');
  console.log('   - 返回 ClaudeSdkResponse 格式');
  
  // 检查 /api/agent/chat 端点
  console.log('\n   /api/agent/chat 端点:');
  console.log('   - 期望兼容性请求格式');
  console.log('   - 返回兼容性响应格式');
  
  // 测试 6: 格式兼容性分析
  console.log('\n6. 格式兼容性分析...');
  
  const claudeSdkFormat = {
    apiKey: "string (optional)",
    prompt: "string (required)",
    systemPrompt: "string (optional)",
    history: "array (optional)",
    tools: "array (optional)",
    model: "string (required)",
    maxTokens: "number (required)",
    temperature: "number (optional)",
  };
  
  const chatEndpointFormat = {
    // 需要检查实际格式
    prompt: "string (required)",
    system_prompt: "string (optional)",
    model: "string (optional)",
    max_tokens: "number (optional)",
    temperature: "number (optional)",
  };
  
  console.log('   Claude SDK 格式:', JSON.stringify(claudeSdkFormat, null, 2));
  console.log('\n   Chat 端点格式:', JSON.stringify(chatEndpointFormat, null, 2));
  
  // 测试 7: 建议
  console.log('\n7. 建议:');
  console.log('   - 如果 SDK 强制要求特定格式，需要保持 /api/claude-agent/query 端点');
  console.log('   - 可以在后端内部将请求转发到 /api/agent/chat');
  console.log('   - 需要确保格式转换正确');
  console.log('   - 建议先测试当前配置是否工作');
  
  console.log('\n=== 测试完成 ===');
}

// 运行测试
testSdkFormat().catch(error => {
  console.error('测试失败:', error);
  process.exit(1);
});
