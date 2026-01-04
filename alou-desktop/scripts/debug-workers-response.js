#!/usr/bin/env node
/**
 * 调试 Workers 响应问题
 * 检查为什么返回硬编码响应
 */

import fetch from 'node-fetch';

async function debugWorkersResponse() {
  console.log('=== 调试 Workers 响应问题 ===\n');
  
  const workersUrl = 'https://alou-edge.yuanjieliu65.workers.dev';
  
  // 测试 1: 直接测试 /api/claude-agent/query 端点
  console.log('1. 测试 /api/claude-agent/query 端点...');
  
  const testRequest = {
    apiKey: "alou-backend-default-token",
    prompt: "Hello, world! Please respond with a simple greeting.",
    systemPrompt: "You are a helpful assistant.",
    model: "claude-3-5-sonnet-20241022",
    maxTokens: 100,
    temperature: 0.7,
  };
  
  try {
    const response = await fetch(`${workersUrl}/api/claude-agent/query`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(testRequest),
    });
    
    console.log('   响应状态:', response.status, response.statusText);
    
    const responseText = await response.text();
    console.log('   响应长度:', responseText.length);
    
    try {
      const responseData = JSON.parse(responseText);
      console.log('   响应解析成功');
      
      // 分析响应内容
      console.log('\n   响应分析:');
      console.log('   - success:', responseData.success);
      console.log('   - response length:', responseData.response?.length || 0);
      console.log('   - response preview:', responseData.response?.substring(0, 100) || '无');
      
      if (responseData.metadata) {
        console.log('   - metadata.routed_to:', responseData.metadata.routed_to);
        console.log('   - metadata.model_used:', responseData.metadata.model_used);
        console.log('   - metadata.model_provider:', responseData.metadata.model_provider);
      }
      
      // 检查是否是硬编码响应
      if (responseData.response && responseData.response.includes('模拟响应')) {
        console.log('\n   🔍 检测到硬编码响应关键词: "模拟响应"');
        console.log('   可能原因: AI 服务调用失败，回退到模拟响应');
      }
      
      if (responseData.response && responseData.response.includes('这是来自')) {
        console.log('\n   🔍 检测到硬编码响应模式: "这是来自"');
        console.log('   这来自 create_mock_response 函数');
      }
      
    } catch (parseError) {
      console.log('   响应解析失败:', parseError.message);
      console.log('   响应内容前200字符:', responseText.substring(0, 200));
    }
    
  } catch (error) {
    console.log('   请求错误:', error.message);
  }
  
  // 测试 2: 检查环境变量配置
  console.log('\n2. 可能的根本原因:');
  console.log('   a) Workers 环境变量 AI_API_KEY 未设置或无效');
  console.log('   b) DeepSeek API 密钥无效或配额用完');
  console.log('   c) 网络连接问题');
  console.log('   d) AI 客户端配置错误');
  
  // 测试 3: 检查其他端点
  console.log('\n3. 检查其他端点状态...');
  
  const endpoints = [
    '/api/health',
    '/api/status',
  ];
  
  for (const endpoint of endpoints) {
    try {
      const healthResponse = await fetch(`${workersUrl}${endpoint}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({}),
      });
      
      console.log(`   ${endpoint}: ${healthResponse.status} ${healthResponse.statusText}`);
    } catch (error) {
      console.log(`   ${endpoint}: 错误 - ${error.message}`);
    }
  }
  
  console.log('\n=== 调试建议 ===\n');
  console.log('1. 检查 Workers 日志:');
  console.log('   wrangler tail');
  console.log('\n2. 验证环境变量:');
  console.log('   wrangler secret list');
  console.log('   wrangler secret get AI_API_KEY');
  console.log('\n3. 测试 DeepSeek API 密钥:');
  console.log('   curl -X POST https://api.deepseek.com/chat/completions \\');
  console.log('     -H "Content-Type: application/json" \\');
  console.log('     -H "Authorization: Bearer YOUR_API_KEY" \\');
  console.log('     -d \'{"model":"deepseek-chat","messages":[{"role":"user","content":"Hello"}]}\'');
  console.log('\n4. 检查后端代码:');
  console.log('   - 确保 AiClient::new 正确创建');
  console.log('   - 确保没有意外使用模拟响应');
  console.log('   - 检查错误处理逻辑');
}

// 使用动态导入
import('node-fetch').then(({ default: fetch }) => {
  debugWorkersResponse();
}).catch(error => {
  console.error('无法加载 node-fetch:', error);
  process.exit(1);
});
