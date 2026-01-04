#!/usr/bin/env node
/**
 * 测试本地后端服务
 */

async function testLocalBackend() {
  console.log('=== 测试本地后端服务 ===\n');
  
  const localUrl = 'http://127.0.0.1:8787';
  
  // 测试 1: 健康检查
  console.log('1. 测试健康检查...');
  try {
    const healthResponse = await fetch(`${localUrl}/api/health`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({}),
    });
    
    console.log('   状态:', healthResponse.status, healthResponse.statusText);
    
    if (healthResponse.ok) {
      const healthData = await healthResponse.json();
      console.log('   响应:', JSON.stringify(healthData, null, 2));
    } else {
      const errorText = await healthResponse.text();
      console.log('   错误响应:', errorText.substring(0, 200));
    }
  } catch (error) {
    console.log('   错误:', error.message);
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
    const response = await fetch(`${localUrl}/api/claude-agent/query`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(testRequest),
    });
    
    console.log('   状态:', response.status, response.statusText);
    
    const responseText = await response.text();
    console.log('   响应长度:', responseText.length);
    
    try {
      const responseData = JSON.parse(responseText);
      console.log('   解析成功');
      
      // 分析响应
      console.log('\n   响应分析:');
      console.log('   - success:', responseData.success);
      console.log('   - response:', responseData.response?.substring(0, 100) || '无');
      
      if (responseData.metadata) {
        console.log('   - routed_to:', responseData.metadata.routed_to);
        console.log('   - model_used:', responseData.metadata.model_used);
        console.log('   - model_provider:', responseData.metadata.model_provider);
      }
      
      // 检查是否是硬编码响应
      if (responseData.response && (
        responseData.response.includes('模拟响应') ||
        responseData.response.includes('这是来自') ||
        responseData.response === '你好'
      )) {
        console.log('\n   🔍 检测到硬编码响应！');
        console.log('   可能原因:');
        console.log('   1. 本地环境变量 AI_API_KEY 未设置');
        console.log('   2. DeepSeek API 调用失败');
        console.log('   3. 代码中使用模拟响应');
      }
      
    } catch (parseError) {
      console.log('   解析失败:', parseError.message);
      console.log('   原始响应:', responseText.substring(0, 200));
    }
    
  } catch (error) {
    console.log('   请求错误:', error.message);
  }
  
  // 测试 3: 检查环境变量
  console.log('\n3. 诊断建议:');
  console.log('   a) 检查本地 .dev.vars 文件:');
  console.log('      cat alou-edge/.dev.vars');
  console.log('\n   b) 检查后端日志中的错误:');
  console.log('      查看运行 wrangler dev 的终端输出');
  console.log('\n   c) 验证 DeepSeek API 密钥:');
  console.log('      curl -X POST https://api.deepseek.com/chat/completions \\');
  console.log('        -H "Content-Type: application/json" \\');
  console.log('        -H "Authorization: Bearer YOUR_API_KEY" \\');
  console.log('        -d \'{"model":"deepseek-chat","messages":[{"role":"user","content":"Hello"}]}\'');
  console.log('\n   d) 检查模型转换是否工作:');
  console.log('      查看后端日志中是否有 "模型转换" 相关日志');
}

// 使用动态导入
import('node-fetch').then(({ default: fetch }) => {
  testLocalBackend();
}).catch(error => {
  console.error('无法加载 node-fetch:', error);
  process.exit(1);
});
