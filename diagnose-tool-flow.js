#!/usr/bin/env node
/**
 * 工具调用流程诊断脚本
 * 测试从发送消息到工具执行的完整流程
 */

const API_BASE_URL = process.env.API_BASE_URL || 'http://127.0.0.1:8787';

console.log('🔍 工具调用流程诊断');
console.log('====================\n');

// 测试1: 发送带有工具的消息
async function testToolExecution() {
  console.log('📤 测试1: 发送需要工具调用的消息');
  
  const testPayload = {
    apiKey: "test-key",
    prompt: "请帮我查询当前目录下的所有文件",
    systemPrompt: "你是一个文件管理助手，可以使用文件系统工具帮助用户管理文件。",
    model: "deepseek-chat",
    agentInfo: {
      session_id: "test-session-tools",
      name: "FileManager",
      role_description: "文件管理助手"
    },
    tools: [
      {
        name: "filesystem",
        description: "文件系统操作工具，支持读取、写入、列出文件和目录",
        parameters: {
          type: "object",
          properties: {
            operation: { 
              type: "string", 
              enum: ["read", "write", "list", "delete"],
              description: "文件操作类型"
            },
            path: { 
              type: "string", 
              description: "文件或目录路径"
            }
          },
          required: ["operation"]
        }
      },
      {
        name: "search",
        description: "搜索工具，支持文本搜索和文件查找",
        parameters: {
          type: "object",
          properties: {
            operation: { 
              type: "string", 
              enum: ["grep", "glob"],
              description: "搜索操作类型"
            },
            pattern: { 
              type: "string", 
              description: "搜索模式"
            }
          },
          required: ["operation", "pattern"]
        }
      }
    ],
    taskType: "async",
    timeout: 60000
  };

  try {
    const response = await fetch(`${API_BASE_URL}/api/ai-task/init-and-start`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(testPayload)
    });

    const data = await response.json();
    console.log('✅ 任务创建成功:', {
      taskId: data.taskId || data.task_id,
      status: data.status,
      estimatedTime: data.estimatedTime
    });

    return data.taskId || data.task_id;
  } catch (error) {
    console.error('❌ 任务创建失败:', error.message);
    return null;
  }
}

// 测试2: 检查任务状态（包括待处理工具）
async function checkTaskStatus(taskId) {
  console.log(`\n📊 测试2: 检查任务状态 (Task: ${taskId})`);
  
  const maxAttempts = 15;
  let hasToolCalls = false;
  
  for (let i = 0; i < maxAttempts; i++) {
    try {
      // 检查状态
      const statusResponse = await fetch(`${API_BASE_URL}/api/ai-task/${taskId}/status`);
      const status = await statusResponse.json();
      
      console.log(`\n  轮询 ${i + 1}/${maxAttempts}:`);
      console.log(`    - 状态: ${status.status}`);
      console.log(`    - 进度: ${(status.progress || 0) * 100}%`);
      console.log(`    - 步骤: ${status.currentStep || 'N/A'}`);
      
      // 检查待处理工具
      if (status.status === 'processing' || status.status === 'running') {
        const toolsResponse = await fetch(`${API_BASE_URL}/api/ai-task/${taskId}/pending-tools`);
        const tools = await toolsResponse.json();
        
        if (tools.toolCalls && tools.toolCalls.length > 0) {
          console.log(`    - 待处理工具: ${tools.toolCalls.length} 个`);
          tools.toolCalls.forEach((tool, idx) => {
            console.log(`      [${idx + 1}] ${tool.tool || tool.name || 'unknown'}:`, JSON.stringify(tool.arguments || {}));
          });
          hasToolCalls = true;
          return { status, tools: tools.toolCalls };
        }
      }
      
      // 任务完成或失败
      if (status.status === 'completed' || status.status === 'failed') {
        console.log(`\n  ✅ 任务结束: ${status.status}`);
        if (status.result) {
          console.log(`  📄 结果:`, JSON.stringify(status.result, null, 2).substring(0, 500));
        }
        return { status, tools: null };
      }
      
      // 等待2秒
      await new Promise(resolve => setTimeout(resolve, 2000));
    } catch (error) {
      console.error(`  ❌ 轮询失败:`, error.message);
    }
  }
  
  console.log(`\n  ⚠️ 达到最大轮询次数`);
  return { status: null, tools: null, noToolCalls: !hasToolCalls };
}

// 测试3: 模拟提交工具结果
async function submitToolResult(taskId, toolCall) {
  console.log(`\n📤 测试3: 提交工具结果 (Task: ${taskId})`);
  
  // 模拟文件系统工具的结果
  const mockResult = {
    tool: toolCall.name,
    result: {
      files: [
        { name: "file1.txt", type: "file", size: 1024 },
        { name: "file2.txt", type: "file", size: 2048 },
        { name: "folder1", type: "directory" }
      ],
      path: ".",
      operation: "list"
    },
    error: null
  };
  
  try {
    const response = await fetch(`${API_BASE_URL}/api/ai-task/${taskId}/tool-result`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        results: [mockResult],
        timestamp: Date.now()
      })
    });

    const data = await response.json();
    console.log('✅ 工具结果提交成功:', data);
    return true;
  } catch (error) {
    console.error('❌ 工具结果提交失败:', error.message);
    return false;
  }
}

// 主测试流程
async function runDiagnostics() {
  console.log('开始诊断测试...\n');
  
  // 步骤1: 创建任务
  const taskId = await testToolExecution();
  if (!taskId) {
    console.error('\n❌ 诊断失败: 无法创建任务');
    return;
  }
  
  // 步骤2: 检查任务状态（等待工具调用）
  const { status, tools } = await checkTaskStatus(taskId);
  
  if (!tools) {
    console.log('\n📋 诊断结果:');
    console.log('====================');
    console.log('❌ AI 没有返回工具调用请求');
    console.log('可能的原因:');
    console.log('  1. AI 认为不需要使用工具');
    console.log('  2. 工具定义不够清晰');
    console.log('  3. AI API Key 未配置或无效');
    console.log('  4. 提示词没有引导 AI 使用工具');
    console.log('\n建议:');
    console.log('  - 检查 .dev.vars 或 secrets 中的 AI_API_KEY');
    console.log('  - 在 systemPrompt 中明确说明可以使用工具');
    console.log('  - 确保工具定义清晰且符合使用场景');
    return;
  }
  
  console.log('\n✅ AI 请求了工具调用！');
  
  // 步骤3: 提交工具结果
  const submitted = await submitToolResult(taskId, tools[0]);
  if (!submitted) {
    console.error('\n❌ 诊断失败: 无法提交工具结果');
    return;
  }
  
  // 步骤4: 再次检查任务状态（看是否继续对话）
  console.log('\n⏳ 等待 AI 继续处理...');
  await new Promise(resolve => setTimeout(resolve, 3000));
  
  const finalResult = await checkTaskStatus(taskId);
  
  console.log('\n📋 诊断结果:');
  console.log('====================');
  if (finalResult.status?.status === 'completed') {
    console.log('✅ 工具调用流程完整！');
    console.log('前端应该:');
    console.log('  1. 通过 Tauri invoke 调用桌面版后端执行工具');
    console.log('  2. 将工具结果提交回 alou-edge');
    console.log('  3. 继续轮询直到任务完成');
  } else {
    console.log('⚠️ 流程未完成，状态:', finalResult.status?.status);
  }
}

// 运行诊断
runDiagnostics().catch(error => {
  console.error('诊断脚本失败:', error);
  process.exit(1);
});
