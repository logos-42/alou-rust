#!/usr/bin/env node
/**
 * 测试模型转换逻辑
 * 模拟 convert_claude_model_to_deepseek 函数的行为
 */

// 模拟 Rust 中的 convert_claude_model_to_deepseek 函数
function convertClaudeModelToDeepseek(claudeModel) {
  // 这是根据 alou-edge/src/router/claude.rs 中的实现模拟的
  switch (claudeModel) {
    // Claude 模型映射到 deepseek-chat
    case 'claude-3-5-sonnet-20241022':
      return 'deepseek-chat';
    case 'claude-3-haiku-20240307':
      return 'deepseek-chat';
    case 'claude-3-opus-20240229':
      return 'deepseek-chat';
    
    // DeepSeek 模型保持不变
    case 'deepseek-chat':
      return 'deepseek-chat';
    case 'deepseek-reasoner':
      return 'deepseek-reasoner';
    
    // 其他模型默认使用 deepseek-chat
    default:
      return 'deepseek-chat';
  }
}

// 测试用例
const testCases = [
  // Claude 模型
  { input: 'claude-3-5-sonnet-20241022', expected: 'deepseek-chat' },
  { input: 'claude-3-haiku-20240307', expected: 'deepseek-chat' },
  { input: 'claude-3-opus-20240229', expected: 'deepseek-chat' },
  
  // DeepSeek 模型
  { input: 'deepseek-chat', expected: 'deepseek-chat' },
  { input: 'deepseek-reasoner', expected: 'deepseek-reasoner' },
  
  // 其他模型
  { input: 'gpt-4o', expected: 'deepseek-chat' },
  { input: 'unknown-model', expected: 'deepseek-chat' },
];

// 运行测试
console.log('=== 测试模型转换逻辑 ===\n');

let passed = 0;
let failed = 0;

testCases.forEach((testCase, index) => {
  const result = convertClaudeModelToDeepseek(testCase.input);
  const isPass = result === testCase.expected;
  
  console.log(`测试 ${index + 1}: ${testCase.input} -> ${result}`);
  console.log(`   期望: ${testCase.expected}`);
  console.log(`   结果: ${isPass ? '✅ 通过' : '❌ 失败'}\n`);
  
  if (isPass) {
    passed++;
  } else {
    failed++;
  }
});

console.log(`=== 测试总结 ===`);
console.log(`通过: ${passed}`);
console.log(`失败: ${failed}`);
console.log(`总计: ${testCases.length}`);

// 测试 claude-agent.js 中的路由配置
console.log('\n=== 测试 claude-agent.js 路由配置 ===\n');

const routingConfig = {
  'claude-3-5-sonnet-20241022': 'deepseek',
  'claude-3-haiku-20240307': 'deepseek',
  'claude-3-opus-20240229': 'deepseek',
  'deepseek-chat': 'deepseek',
  'deepseek-reasoner': 'deepseek',
  'gpt-4o': 'openai',
  'gpt-4-turbo': 'openai',
  'gpt-3.5-turbo': 'openai',
  'kimi-k2': 'kimi',
  'qwen-max': 'qwen'
};

console.log('路由配置验证:');
Object.entries(routingConfig).forEach(([model, provider]) => {
  console.log(`  ${model} -> ${provider}`);
});

console.log('\n=== 完整工作流程测试 ===\n');

// 模拟一个完整的请求流程
const simulateRequest = (model) => {
  const provider = routingConfig[model] || 'deepseek';
  const finalModel = convertClaudeModelToDeepseek(model);
  
  console.log(`请求模型: ${model}`);
  console.log(`路由到提供商: ${provider}`);
  console.log(`转换后模型: ${finalModel}`);
  console.log(`最终发送到 AI 服务的模型: ${finalModel}\n`);
};

// 测试几个关键模型
simulateRequest('claude-3-5-sonnet-20241022');
simulateRequest('deepseek-chat');
simulateRequest('gpt-4o');

console.log('=== 测试完成 ===');
