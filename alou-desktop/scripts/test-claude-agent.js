#!/usr/bin/env node
/**
 * 测试 Claude Agent SDK 工作流程
 */

import { exec } from 'child_process';
import { promisify } from 'util';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const execAsync = promisify(exec);

// 测试请求数据
const testRequest = {
  apiKey: "alou-backend-default-token", // 使用 claude-agent.js 中配置的默认密钥
  prompt: "Hello, world! Please respond with a simple greeting.",
  systemPrompt: "You are a helpful assistant.",
  model: "claude-3-5-sonnet-20241022", // 这个会被路由到 deepseek
  maxTokens: 100,
  temperature: 0.7,
  history: [
    {
      role: "user",
      content: "Previous message",
      timestamp: Date.now()
    }
  ],
  tools: [],
  agentInfo: {
    name: "test-agent",
    version: "1.0.0"
  }
};

// 测试文件路径
const testInputFile = path.join(__dirname, 'test-claude-request.json');
const expectedResultFile = testInputFile.replace(/\.json$/, '') + '.result.json';
const expectedErrorFile = testInputFile.replace(/\.json$/, '') + '.error.json';

async function runTest() {
  console.log('=== 测试 Claude Agent SDK 工作流程 ===\n');
  
  // 1. 创建测试请求文件
  console.log('1. 创建测试请求文件...');
  fs.writeFileSync(testInputFile, JSON.stringify(testRequest, null, 2));
  console.log(`   文件已创建: ${testInputFile}`);
  
  // 2. 清理可能存在的旧结果文件
  if (fs.existsSync(expectedResultFile)) {
    fs.unlinkSync(expectedResultFile);
  }
  if (fs.existsSync(expectedErrorFile)) {
    fs.unlinkSync(expectedErrorFile);
  }
  
  // 3. 运行 claude-agent.js
  console.log('\n2. 运行 claude-agent.js...');
  try {
    const { stdout, stderr } = await execAsync(`node "${path.join(__dirname, 'claude-agent.js')}" "${testInputFile}"`);
    
    console.log('   stdout:', stdout.trim());
    console.log('   stderr:', stderr.substring(0, 500) + (stderr.length > 500 ? '...' : ''));
    
    // 4. 检查结果
    console.log('\n3. 检查结果...');
    
    if (fs.existsSync(expectedResultFile)) {
      const resultContent = fs.readFileSync(expectedResultFile, 'utf-8');
      const result = JSON.parse(resultContent);
      
      console.log('   ✅ 成功生成结果文件');
      console.log('   结果文件:', expectedResultFile);
      console.log('   成功状态:', result.success);
      console.log('   响应长度:', result.response?.length || 0);
      console.log('   工具调用数量:', result.toolCalls?.length || 0);
      console.log('   元数据:', JSON.stringify(result.metadata, null, 2));
      
      if (result.success) {
        console.log('\n   🎉 测试成功！Claude Agent SDK 工作正常。');
        console.log('   响应内容:', result.response.substring(0, 200) + (result.response.length > 200 ? '...' : ''));
      } else {
        console.log('\n   ❌ 测试失败：响应不成功');
      }
    } else if (fs.existsSync(expectedErrorFile)) {
      const errorContent = fs.readFileSync(expectedErrorFile, 'utf-8');
      const error = JSON.parse(errorContent);
      
      console.log('   ❌ 生成错误文件');
      console.log('   错误文件:', expectedErrorFile);
      console.log('   错误信息:', error.error);
      console.log('   堆栈:', error.stack?.substring(0, 200) || '无');
      console.log('   元数据:', JSON.stringify(error.metadata, null, 2));
      
      console.log('\n   🔍 错误分析：');
      console.log('   - 检查后端服务是否运行 (http://127.0.0.1:8787)');
      console.log('   - 检查 /api/claude-agent/query 端点是否可用');
      console.log('   - 检查 API 密钥配置');
      console.log('   - 检查 DeepSeek API 密钥是否有效');
    } else {
      console.log('   ❌ 未生成结果文件或错误文件');
      console.log('   可能原因：');
      console.log('   - claude-agent.js 脚本执行失败');
      console.log('   - 脚本提前退出');
      console.log('   - 权限问题');
    }
    
  } catch (error) {
    console.error('   ❌ 执行 claude-agent.js 失败:', error.message);
    console.error('   堆栈:', error.stack);
  }
  
  // 5. 清理
  console.log('\n4. 清理测试文件...');
  try {
    if (fs.existsSync(testInputFile)) fs.unlinkSync(testInputFile);
    if (fs.existsSync(expectedResultFile)) fs.unlinkSync(expectedResultFile);
    if (fs.existsSync(expectedErrorFile)) fs.unlinkSync(expectedErrorFile);
    console.log('   清理完成');
  } catch (cleanupError) {
    console.error('   清理失败:', cleanupError.message);
  }
  
  console.log('\n=== 测试完成 ===');
}

// 运行测试
runTest().catch(console.error);