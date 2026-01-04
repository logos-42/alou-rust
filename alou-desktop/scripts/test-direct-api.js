#!/usr/bin/env node
/**
 * 直接测试后端 API
 */

import fetch from 'node-fetch';

async function testDirectApi() {
  console.log('=== 直接测试后端 API ===\n');
  
  const testRequest = {
    apiKey: "alou-backend-default-token",
    prompt: "Hello, world! Please respond with a simple greeting.",
    systemPrompt: "You are a helpful assistant.",
    model: "claude-3-5-sonnet-20241022",
    maxTokens: 100,
    temperature: 0.7,
    history: [],
    tools: [],
    agentInfo: {
      name: "test-agent",
      version: "1.0.0"
    }
  };
  
  try {
    console.log('1. 发送请求到 /api/claude-agent/query...');
    const response = await fetch('http://127.0.0.1:8787/api/claude-agent/query', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(testRequest),
    });
    
    console.log('   响应状态:', response.status, response.statusText);
    
    if (response.ok) {
      const data = await response.json();
      console.log('   响应数据:', JSON.stringify(data, null, 2));
      
      if (data.success) {
        console.log('\n   ✅ API 调用成功');
        console.log('   响应内容:', data.response?.substring(0, 200) || '空');
        console.log('   响应长度:', data.response?.length || 0);
      } else {
        console.log('\n   ❌ API 调用失败');
        console.log('   错误信息:', data.error || '未知错误');
      }
    } else {
      const errorText = await response.text();
      console.log('   错误响应:', errorText);
      
      console.log('\n   🔍 可能的问题：');
      console.log('   - 后端服务未运行');
      console.log('   - 端口不正确');
      console.log('   - 端点路径错误');
    }
  } catch (error) {
    console.error('   ❌ 请求失败:', error.message);
    console.error('   堆栈:', error.stack);
    
    console.log('\n   🔍 连接问题：');
    console.log('   - 确保后端服务正在运行 (http://127.0.0.1:8787)');
    console.log('   - 检查防火墙设置');
    console.log('   - 尝试使用 curl 测试: curl -X POST http://127.0.0.1:8787/api/claude-agent/query -H "Content-Type: application/json" -d \'{"prompt":"test"}\'');
  }
  
  console.log('\n=== 测试完成 ===');
}

// 运行测试
testDirectApi().catch(console.error);
