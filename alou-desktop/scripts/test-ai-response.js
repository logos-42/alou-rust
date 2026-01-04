#!/usr/bin/env node
/**
 * 测试 AI 响应是否真实
 */

async function testAIResponse() {
  console.log('=== 测试 AI 响应真实性 ===\n');
  
  const localUrl = 'http://127.0.0.1:8787';
  
  // 测试 1: 发送一个需要推理的请求
  console.log('1. 发送需要推理的请求...');
  
  const testRequest = {
    apiKey: "alou-backend-default-token",
    prompt: "What is 12345 * 67890? Please calculate and show your work.",
    systemPrompt: "You are a helpful math assistant.",
    model: "claude-3-5-sonnet-20241022",
    maxTokens: 200,
    temperature: 0.1,
  };
  
  try {
    const response = await fetch(`${localUrl}/api/claude-agent/query`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(testRequest),
    });
    
    const responseText = await response.text();
    
    try {
      const responseData = JSON.parse(responseText);
      
      console.log('   响应状态:', response.status, response.statusText);
      console.log('   响应内容:', responseData.response?.substring(0, 200) || '无');
      
      // 分析响应
      if (responseData.response) {
        const resp = responseData.response.toLowerCase();
        
        console.log('\n   响应分析:');
        
        // 检查是否是简单的问候
        if (resp.includes('hello') || resp.includes('hi') || resp.includes('how are you')) {
          console.log('   ❌ 检测到简单问候 - 可能是硬编码响应');
        }
        
        // 检查是否包含数学计算
        if (resp.includes('12345') && resp.includes('67890')) {
          console.log('   ✅ 响应包含原始数字 - 可能是真实计算');
        }
        
        if (resp.includes('838,102,050') || resp.includes('838102050')) {
          console.log('   ✅ 响应包含正确计算结果 (12345*67890=838,102,050)');
        } else if (resp.includes('calculate') || resp.includes('multiplication')) {
          console.log('   ⚠️  响应提到计算，但没有具体结果');
        } else {
          console.log('   ❌ 响应没有进行数学计算');
        }
        
        // 检查响应长度
        if (responseData.response.length < 50) {
          console.log('   ⚠️  响应过短 (小于50字符)，可能是硬编码');
        } else {
          console.log(`   ✅ 响应长度正常 (${responseData.response.length} 字符)`);
        }
      }
      
    } catch (parseError) {
      console.log('   解析失败:', parseError.message);
    }
    
  } catch (error) {
    console.log('   请求错误:', error.message);
  }
  
  // 测试 2: 检查后端日志
  console.log('\n2. 根本原因诊断:');
  console.log('   如果响应是硬编码的，可能原因:');
  console.log('   a) 本地 .dev.vars 中没有设置 AI_API_KEY');
  console.log('   b) AI_API_KEY 无效或过期');
  console.log('   c) DeepSeek API 服务不可用');
  console.log('   d) 代码中有默认的模拟响应逻辑');
  
  console.log('\n3. 检查步骤:');
  console.log('   a) 查看运行 wrangler dev 的终端输出');
  console.log('   b) 检查是否有 "AI service error" 或 "Failed to create AI client" 日志');
  console.log('   c) 检查 .dev.vars 文件:');
  console.log('      cat alou-edge/.dev.vars');
  console.log('   d) 直接测试 DeepSeek API:');
  console.log('      curl -X POST https://api.deepseek.com/chat/completions \\');
  console.log('        -H "Content-Type: application/json" \\');
  console.log('        -H "Authorization: Bearer YOUR_API_KEY" \\');
  console.log('        -d \'{"model":"deepseek-chat","messages":[{"role":"user","content":"What is 12345 * 67890?"}]}\'');
}

// 使用动态导入
import('node-fetch').then(({ default: fetch }) => {
  testAIResponse();
}).catch(error => {
  console.error('无法加载 node-fetch:', error);
  process.exit(1);
});
