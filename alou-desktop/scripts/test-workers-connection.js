#!/usr/bin/env node
/**
 * 测试 Workers 生产环境连接
 */

import fetch from 'node-fetch';

async function testWorkersConnection() {
  console.log('=== 测试 Workers 生产环境连接 ===\n');
  
  const workersUrl = 'https://alou-edge.yuanjieliu65.workers.dev';
  
  // 测试 1: 健康检查
  console.log('1. 测试健康检查端点...');
  try {
    const healthResponse = await fetch(`${workersUrl}/api/health`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({}),
    });
    
    if (healthResponse.ok) {
      const healthData = await healthResponse.json();
      console.log('   ✅ 健康检查成功');
      console.log('   状态:', healthResponse.status);
      console.log('   响应:', JSON.stringify(healthData, null, 2));
    } else {
      console.log('   ❌ 健康检查失败');
      console.log('   状态:', healthResponse.status);
      console.log('   状态文本:', healthResponse.statusText);
    }
  } catch (error) {
    console.log('   ❌ 健康检查错误:', error.message);
  }
  
  // 测试 2: Claude Agent SDK 端点
  console.log('\n2. 测试 Claude Agent SDK 端点...');
  const testRequest = {
    apiKey: "alou-backend-default-token",
    prompt: "Hello, world! Please respond with a simple greeting.",
    systemPrompt: "You are a helpful assistant.",
    model: "claude-3-5-sonnet-20241022",
    maxTokens: 100,
    temperature: 0.7,
  };
  
  try {
    const claudeResponse = await fetch(`${workersUrl}/api/claude-agent/query`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(testRequest),
    });
    
    console.log('   请求已发送到:', `${workersUrl}/api/claude-agent/query`);
    console.log('   请求模型:', testRequest.model);
    
    if (claudeResponse.ok) {
      const claudeData = await claudeResponse.json();
      console.log('   ✅ Claude SDK 端点响应成功');
      console.log('   状态:', claudeResponse.status);
      console.log('   成功状态:', claudeData.success);
      console.log('   响应长度:', claudeData.response?.length || 0);
      
      if (claudeData.metadata) {
        console.log('   元数据:');
        console.log('     - 路由目标:', claudeData.metadata.routed_to);
        console.log('     - 使用模型:', claudeData.metadata.model_used);
        console.log('     - 模型提供商:', claudeData.metadata.model_provider);
        console.log('     - 支持工具:', claudeData.metadata.supports_tools);
        console.log('     - 支持系统提示词:', claudeData.metadata.supports_system_prompt);
      }
    } else {
      console.log('   ❌ Claude SDK 端点响应失败');
      console.log('   状态:', claudeResponse.status);
      console.log('   状态文本:', claudeResponse.statusText);
      
      try {
        const errorData = await claudeResponse.text();
        console.log('   错误响应:', errorData.substring(0, 500));
      } catch (e) {
        console.log('   无法读取错误响应:', e.message);
      }
    }
  } catch (error) {
    console.log('   ❌ Claude SDK 端点错误:', error.message);
    console.log('   错误堆栈:', error.stack);
  }
  
  // 测试 3: 检查其他端点
  console.log('\n3. 检查其他可用端点...');
  const endpoints = [
    '/api/agent/chat',
    '/api/status',
    '/v1/models',
  ];
  
  for (const endpoint of endpoints) {
    try {
      const response = await fetch(`${workersUrl}${endpoint}`, {
        method: 'GET',
        headers: {
          'Accept': 'application/json',
        },
      });
      
      console.log(`   ${endpoint}: ${response.status} ${response.statusText}`);
    } catch (error) {
      console.log(`   ${endpoint}: 错误 - ${error.message}`);
    }
  }
  
  console.log('\n=== 测试完成 ===');
  console.log('\n建议:');
  console.log('1. 如果 Workers 连接成功，可以更新 claude-agent.js 中的 ANTHROPIC_BASE_URL');
  console.log('2. 确保 Workers 环境变量已正确配置（特别是 AI_API_KEY）');
  console.log('3. 检查 CORS 设置，确保桌面应用可以访问 Workers');
  console.log('4. 如果遇到模型名称错误，确保 convert_claude_model_to_deepseek 函数已正确部署');
}

// 运行测试
testWorkersConnection().catch(error => {
  console.error('测试失败:', error);
  process.exit(1);
});
