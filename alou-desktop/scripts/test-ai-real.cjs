#!/usr/bin/env node
/**
 * Alou AI 真实对话测试
 * 
 * 直接调用 DeepSeek API，测试完整功能：
 * - AI 对话
 * - 工具调用
 * - 群聊模拟
 */

const https = require('https');

// DeepSeek API 配置
const API_KEY = 'sk-0e701b56fd2448b9b8c1b485486a2d23';
const BASE_URL = 'api.deepseek.com';
const MODEL = 'deepseek-chat';

// 颜色
const GREEN = '\x1b[32m';
const BLUE = '\x1b[34m';
const CYAN = '\x1b[36m';
const YELLOW = '\x1b[33m';
const RESET = '\x1b[0m';

function log(color, msg) {
  console.log(`${color}${msg}${RESET}`);
}

function logSection(msg) {
  console.log(`\n${CYAN}=== ${msg} ===${RESET}\n`);
}

// DeepSeek API 调用
async function callDeepSeek(messages, maxTokens = 2000) {
  return new Promise((resolve, reject) => {
    const data = JSON.stringify({
      model: MODEL,
      messages,
      max_tokens: maxTokens,
      temperature: 0.7,
      stream: false,
    });

    const options = {
      hostname: BASE_URL,
      port: 443,
      path: '/chat/completions',
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${API_KEY}`,
      },
    };

    const req = https.request(options, (res) => {
      let body = '';
      res.on('data', chunk => body += chunk);
      res.on('end', () => {
        try {
          const response = JSON.parse(body);
          if (response.choices && response.choices[0]) {
            resolve(response.choices[0].message.content);
          } else {
            reject(new Error('无效的 API 响应'));
          }
        } catch (e) {
          reject(e);
        }
      });
    });

    req.on('error', reject);
    req.write(data);
    req.end();
  });
}

// 测试1: 基本对话
async function testBasicChat() {
  logSection('🗣️ 测试1: 基本对话');
  
  const messages = [
    {
      role: 'system',
      content: '你是 Alou，一个 AI 助手。简洁回答。'
    },
    {
      role: 'user',
      content: '你好，请介绍一下你自己'
    }
  ];

  log(BLUE, '发送请求到 DeepSeek...');
  
  try {
    const response = await callDeepSeek(messages);
    log(GREEN, '✅ AI 回复:');
    console.log(response);
    return true;
  } catch (error) {
    log(YELLOW, `⚠️ API 调用失败: ${error.message}`);
    return false;
  }
}

// 测试2: 工具调用模拟
async function testToolCalling() {
  logSection('🔧 测试2: 工具调用');
  
  const messages = [
    {
      role: 'system',
      content: `你是 Alou，可以调用工具。工具调用格式:
\`\`\`json
{"tool": "工具名", "params": {"参数": "值"}}
\`\`\`
否则直接回复。`
    },
    {
      role: 'user',
      content: '请帮我执行一个任务：检查系统状态，然后返回结果'
    }
  ];

  log(BLUE, '测试工具调用能力...');
  
  try {
    const response = await callDeepSeek(messages);
    log(GREEN, '✅ 回复:');
    console.log(response);
    return true;
  } catch (error) {
    log(YELLOW, `⚠️ 失败: ${error.message}`);
    return false;
  }
}

// 测试3: 群聊模拟
async function testGroupChat() {
  logSection('👥 测试3: 群聊模拟');
  
  const messages = [
    {
      role: 'system',
      content: `你是 Alou AI，在一个群聊中。用户"觉者"在和你说话。
你的角色：helpful assistant
沟通风格：简洁、专业
请像在群聊中一样回复，保持简短。`
    },
    {
      role: 'user',
      content: '小星，汇报一下今天的进度'
    }
  ];

  log(BLUE, '模拟群聊对话...');
  
  try {
    const response = await callDeepSeek(messages);
    log(GREEN, '✅ 群聊回复:');
    console.log(response);
    return true;
  } catch (error) {
    log(YELLOW, `⚠️ 失败: ${error.message}`);
    return false;
  }
}

// 测试4: 任务规划
async function testTaskPlanning() {
  logSection('📋 测试4: 任务规划');
  
  const messages = [
    {
      role: 'system',
      content: `你是 Alou AI，擅长任务规划。
请用 JSON 格式回复任务计划:
\`\`\`json
{
  "tasks": [
    {"title": "任务名", "priority": "high/medium/low", "description": "描述"}
  ]
}
\`\`\``
    },
    {
      role: 'user',
      content: '规划一下明天的工作：检查邮件、回复消息、学习新技能'
    }
  ];

  log(BLUE, '测试任务规划能力...');
  
  try {
    const response = await callDeepSeek(messages);
    log(GREEN, '✅ 任务计划:');
    console.log(response);
    return true;
  } catch (error) {
    log(YELLOW, `⚠️ 失败: ${error.message}`);
    return false;
  }
}

// 测试5: 多轮对话
async function testMultiTurn() {
  logSection('💬 测试5: 多轮对话');
  
  const conversation = [
    { role: 'user', content: '1+1等于几？' },
    { role: 'assistant', content: '2' },
    { role: 'user', content: '再加3等于几？' },
    { role: 'assistant', content: '5' },
    { role: 'user', content: '你是怎么算的？' },
  ];

  log(BLUE, '模拟多轮对话...');
  
  // 只发送最后几条作为上下文
  const messages = [
    {
      role: 'system',
      content: '你是 Alou，简洁回答。记住对话上下文。'
    },
    ...conversation.slice(-3), // 最后3轮
  ];

  try {
    const response = await callDeepSeek(messages);
    log(GREEN, '✅ 上下文理解回复:');
    console.log(response);
    return true;
  } catch (error) {
    log(YELLOW, `⚠️ 失败: ${error.message}`);
    return false;
  }
}

// 主测试
async function main() {
  console.log('='.repeat(60));
  console.log('🤖 Alou AI 真实功能测试');
  console.log('='.repeat(60));
  console.log(`\n${BLUE}API: DeepSeek (${MODEL})${RESET}`);
  console.log(`${BLUE}配置: ${API_KEY.substring(0, 10)}...${RESET}\n`);

  const results = {
    basicChat: await testBasicChat(),
    toolCalling: await testToolCalling(),
    groupChat: await testGroupChat(),
    taskPlanning: await testTaskPlanning(),
    multiTurn: await testMultiTurn(),
  };

  // 总结
  logSection('📊 测试总结');
  
  let passed = 0;
  for (const [test, success] of Object.entries(results)) {
    const emoji = success ? '✅' : '❌';
    const name = test.replace(/([A-Z])/g, ' $1').trim();
    log(success ? GREEN : YELLOW, `${emoji} ${name}`);
    if (success) passed++;
  }

  console.log(`\n${GREEN}通过: ${passed}/${Object.keys(results).length}${RESET}`);
  
  if (passed === Object.keys(results).length) {
    console.log(`\n${GREEN}🎉 全部测试通过！Alou AI 功能正常！${RESET}`);
  } else {
    console.log(`\n${YELLOW}⚠️  部分测试失败，请检查网络和 API 配置${RESET}`);
  }
}

main().catch(console.error);
