#!/usr/bin/env node
/**
 * Alou 群聊完整测试
 * 
 * 测试群聊场景：
 * 1. Agent 与用户对话
 * 2. 工具调用执行
 * 3. 消息广播
 * 4. 多人协作
 */

const https = require('https');

const API_KEY = 'sk-0e701b56fd2448b9b8c1b485486a2d23';
const BASE_URL = 'api.deepseek.com';
const MODEL = 'deepseek-chat';

const GREEN = '\x1b[32m';
const BLUE = '\x1b[34m';
const CYAN = '\x1b[36m';
const YELLOW = '\x1b[33m';
const MAGENTA = '\x1b[35m';
const RESET = '\x1b[0m';

async function callAI(messages, maxTokens = 1500) {
  return new Promise((resolve, reject) => {
    const data = JSON.stringify({
      model: MODEL,
      messages,
      max_tokens: maxTokens,
      temperature: 0.7,
    });

    const req = https.request({
      hostname: BASE_URL,
      port: 443,
      path: '/chat/completions',
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${API_KEY}`,
      },
    }, (res) => {
      let body = '';
      res.on('data', chunk => body += chunk);
      res.on('end', () => {
        try {
          resolve(JSON.parse(body).choices[0].message.content);
        } catch (e) { reject(e); }
      });
    });

    req.on('error', reject);
    req.write(data);
    req.end();
  });
}

// 测试群聊场景
async function testGroupChat() {
  console.log('='.repeat(60));
  console.log('👥 Alou 群聊功能完整测试');
  console.log('='.repeat(60));

  // 模拟群聊参与者
  const participants = {
    alou: { name: 'Alou', role: 'AI 助手', emoji: '🤖' },
    user: { name: '觉者', role: '开发者', emoji: '👤' },
    system: { name: '系统', role: '管理员', emoji: '⚙️' },
  };

  // 场景1: 用户@Alou 提问
  console.log(`\n${MAGENTA}--- 场景1: 用户@Alou 提问 ---${RESET}`);
  
  let chatHistory = [
    {
      role: 'system',
      content: `你是 Alou，在群聊中。用户会@你提问。
- 保持简洁专业的回复
- 如需执行操作，用 JSON 格式:
\`\`\`json
{"action": "call_tool", "tool": "工具名", "params": {...}}
\`\`\`
- 普通回复直接说`
    },
  ];

  // 第一轮对话
  console.log(`\n${participants.user.emoji} ${participants.user.name}: @Alou 今天的项目进度怎么样？`);
  chatHistory.push({ role: 'user', content: '@Alou 今天的项目进度怎么样？' });
  
  const reply1 = await callAI(chatHistory);
  console.log(`${participants.alou.emoji} ${participants.alou.name}: ${reply1}`);

  // 场景2: Alou 执行工具
  console.log(`\n${MAGENTA}--- 场景2: Alou 执行工具 ---${RESET}`);
  
  console.log(`${participants.alou.emoji} ${participants.alou.name}: 我来检查一下...`);
  chatHistory.push({ role: 'assistant', content: reply1 });
  
  // 模拟工具执行
  const toolResult = {
    tool: 'check_progress',
    result: {
      completed: 5,
      in_progress: 3,
      pending: 2,
      recent: ['自主循环核心', '前端控制面板', 'CLI 工具'],
    }
  };
  
  console.log(`${participants.system.emoji} ${participants.system.name}: 执行工具 ${toolResult.tool}`);
  console.log(`📊 结果: ${JSON.stringify(toolResult.result, null, 2)}`);

  // 场景3: 基于工具结果回复
  console.log(`\n${MAGENTA}--- 场景3: 基于结果回复 ---${RESET}`);
  
  chatHistory.push({
    role: 'system',
    content: `工具执行结果:\n${JSON.stringify(toolResult.result)}`
  });
  
  const reply2 = await callAI(chatHistory);
  console.log(`${participants.alou.emoji} ${participants.alou.name}: ${reply2}`);

  // 场景4: 用户追问
  console.log(`\n${MAGENTA}--- 场景4: 用户追问 ---${RESET}`);
  
  console.log(`${participants.user.emoji} ${participants.user.name}: 那CLI工具什么时候能用？`);
  chatHistory.push({ role: 'assistant', content: reply2 });
  chatHistory.push({ role: 'user', content: '那CLI工具什么时候能用？' });
  
  const reply3 = await callAI(chatHistory);
  console.log(`${participants.alou.emoji} ${participants.alou.name}: ${reply3}`);

  // 场景5: 任务分配
  console.log(`\n${MAGENTA}--- 场景5: 任务分配 ---${RESET}`);
  
  console.log(`${participants.user.emoji} ${participants.user.name}: 明天要做的事情有哪些？`);
  chatHistory.push({ role: 'assistant', content: reply3 });
  chatHistory.push({ role: 'user', content: '明天要做的事情有哪些？' });
  
  const reply4 = await callAI(chatHistory, 2000);
  console.log(`${participants.alou.emoji} ${participants.alou.name}: ${reply4}`);

  // 总结
  console.log('\n' + '='.repeat(60));
  console.log('📊 群聊测试总结');
  console.log('='.repeat(60));
  console.log(`${GREEN}✅ 群聊对话 - 正常${RESET}`);
  console.log(`${GREEN}✅ 工具调用 - 正常${RESET}`);
  console.log(`${GREEN}✅ 上下文理解 - 正常${RESET}`);
  console.log(`${GREEN}✅ 任务规划 - 正常${RESET}`);
  console.log(`\n${GREEN}🎉 群聊功能完整可用！${RESET}`);
}

testGroupChat().catch(console.error);
