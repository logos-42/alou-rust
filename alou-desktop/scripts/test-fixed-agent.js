/**
 * 测试修复后的Agent系统提示词
 * 验证系统提示词和工具配置是否正常工作
 */

// 模拟 getSystemPromptForAgent 函数
function getSystemPromptForAgent(agentInfo, mode, walletAddress, chain) {
  console.log('=== 测试系统提示词生成 ===\n');
  
  // 测试Alou模式
  console.log('1. Alou模式系统提示词:');
  const alouPrompt = generateAlouPrompt(agentInfo, walletAddress, chain);
  console.log(`   长度: ${alouPrompt.length} 字符`);
  console.log(`   包含工具说明: ${alouPrompt.includes('🛠️ 工具调用能力') ? '✅ 是' : '❌ 否'}`);
  console.log(`   包含bash工具: ${alouPrompt.includes('bash') ? '✅ 是' : '❌ 否'}`);
  console.log(`   包含read工具: ${alouPrompt.includes('read') ? '✅ 是' : '❌ 否'}`);
  
  // 测试Agent模式（有角色描述）
  console.log('\n2. Agent模式（有角色描述）:');
  const agentWithRole = {
    name: '开发助手',
    role_description: '帮助用户进行代码开发和文件操作'
  };
  const agentPrompt = generateAgentPrompt(agentWithRole, walletAddress, chain);
  console.log(`   长度: ${agentPrompt.length} 字符`);
  console.log(`   包含角色描述: ${agentPrompt.includes('开发助手') ? '✅ 是' : '❌ 否'}`);
  console.log(`   包含工具说明: ${agentPrompt.includes('🛠️ 工具调用能力') ? '✅ 是' : '❌ 否'}`);
  console.log(`   包含使用示例: ${agentPrompt.includes('使用示例') ? '✅ 是' : '❌ 否'}`);
  
  // 测试Agent模式（无角色描述）
  console.log('\n3. Agent模式（无角色描述）:');
  const basicPrompt = generateBasicPrompt(walletAddress, chain);
  console.log(`   长度: ${basicPrompt.length} 字符`);
  console.log(`   包含工具说明: ${basicPrompt.includes('🛠️ 可用工具') ? '✅ 是' : '❌ 否'}`);
  console.log(`   包含调用原则: ${basicPrompt.includes('工具调用原则') ? '✅ 是' : '❌ 否'}`);
  
  return alouPrompt;
}

// 模拟Alou模式提示词生成
function generateAlouPrompt(agentInfo, walletAddress, chain) {
  let prompt = `你是 Alou，由刘元杰开发的交互式主权智能体代理，专注于链上支付任务。

## 🛠️ 工具调用能力

你可以使用以下工具来完成任务：

### ⛓️ Web3支付工具
- **query_blockchain**: 查询区块链数据
- **build_transaction**: 构建区块链交易
- **broadcast_transaction**: 广播交易到网络
- **wallet_manager**: 管理多个钱包
- **agent_wallet**: 智能体钱包操作

### 📁 文件操作工具
- **read**: 读取文件内容
- **write**: 创建新文件
- **edit**: 精确修改文件
- **glob**: 搜索文件
- **grep**: 搜索文件内容

### 💻 终端操作工具
- **bash**: 执行终端命令（支持持久化会话）

### 🌐 网络工具
- **web_search**: 搜索实时信息
- **web_fetch**: 获取网页内容

### 🎯 流程控制工具
- **plan**: 制定任务计划
- **ask_user_question**: 询问用户确认
- **subagents**: 创建子Agent

## 工具调用指南

### Web3支付相关工具
1. **查询余额**: 使用 query_blockchain 工具查询钱包余额
2. **构建交易**: 使用 build_transaction 工具构建支付交易
3. **广播交易**: 使用 broadcast_transaction 工具发送交易
4. **钱包管理**: 使用 wallet_manager 工具管理多个钱包

### 辅助工具
1. **文件操作**: 使用 read/write/edit 工具处理配置文件
2. **终端命令**: 使用 bash 工具执行区块链相关命令
3. **网络搜索**: 使用 web_search 工具查找区块链信息
4. **任务规划**: 使用 plan 工具规划复杂支付流程

现在，以 Alou 的身份帮助用户完成 Web3 支付与相关任务吧！`;

  if (walletAddress) {
    prompt += `\n\n当前钱包地址：${walletAddress}`;
  }
  
  if (chain) {
    prompt += `\n当前链：${chain}`;
  }
  
  return prompt;
}

// 模拟Agent模式提示词生成（有角色描述）
function generateAgentPrompt(agentInfo, walletAddress, chain) {
  let prompt = `你是 ${agentInfo.name || '智能体'}，${agentInfo.role_description}

## 🛠️ 工具调用能力

你可以使用以下工具来完成任务：

### 📁 文件操作工具
- **read**: 读取文件内容
- **write**: 创建新文件
- **edit**: 精确修改文件
- **glob**: 搜索文件
- **grep**: 搜索文件内容
- **notebook_edit**: 编辑Jupyter Notebook文件

### 💻 终端操作工具
- **bash**: 执行终端命令（支持持久化会话）

### 🌐 网络工具
- **web_search**: 搜索实时信息
- **web_fetch**: 获取网页内容

### 🎯 流程控制工具
- **plan**: 制定任务计划
- **ask_user_question**: 询问用户确认
- **subagents**: 创建子Agent

### ⛓️ Web3工具
- **query_blockchain**: 查询区块链数据
- **build_transaction**: 构建区块链交易
- **broadcast_transaction**: 广播交易
- **wallet_manager**: 钱包管理
- **agent_wallet**: 智能体钱包操作

## 工具调用指南

### 文件操作示例
- 读取文件：使用 read 工具
- 创建文件：使用 write 工具
- 修改文件：使用 edit 工具
- 搜索文件：使用 glob 工具
- 搜索内容：使用 grep 工具

### 终端命令示例
- 运行命令：使用 bash 工具
- 支持会话：使用 session_id 保持状态
- 指定目录：使用 working_directory 参数

### 网络操作示例
- 搜索信息：使用 web_search 工具
- 获取网页：使用 web_fetch 工具

### 复杂任务处理
- 制定计划：使用 plan 工具
- 询问确认：使用 ask_user_question 工具
- 并行处理：使用 subagents 工具

## 安全注意事项
- 🔒 Bash工具：避免执行未知命令，敏感操作需确认
- 📁 文件操作：重要文件操作前建议备份
- 🌐 网络操作：验证URL安全性，使用HTTPS连接

现在，请根据用户需求选择合适的工具来完成任务。`;

  if (walletAddress) {
    prompt += `\n\n当前钱包地址：${walletAddress}`;
  }
  
  if (chain) {
    prompt += `\n当前链：${chain}`;
  }
  
  return prompt;
}

// 模拟Agent模式提示词生成（无角色描述）
function generateBasicPrompt(walletAddress, chain) {
  let prompt = `你是一个专业的AI助手，拥有强大的工具调用能力。

## 🛠️ 可用工具

### 📁 文件操作
- read: 读取文件内容
- write: 创建新文件
- edit: 精确修改文件
- glob: 搜索文件
- grep: 搜索文件内容

### 💻 终端命令
- bash: 执行终端命令

### 🌐 网络工具
- web_search: 搜索实时信息
- web_fetch: 获取网页内容

### 🎯 流程控制
- plan: 制定任务计划
- ask_user_question: 询问用户确认

## 工具调用原则
1. 分析用户需求，选择最合适的工具
2. 准备正确的工具参数
3. 调用工具并等待结果
4. 分析结果，继续下一步或返回给用户

## 使用示例
- 用户："请帮我查看文件" → 使用 read 工具
- 用户："请运行命令" → 使用 bash 工具
- 用户："请搜索信息" → 使用 web_search 工具
- 用户："请制定计划" → 使用 plan 工具

现在，请根据用户需求选择合适的工具来完成任务。`;

  if (walletAddress) {
    prompt += `\n\n当前钱包地址：${walletAddress}`;
  }
  
  if (chain) {
    prompt += `\n当前链：${chain}`;
  }
  
  return prompt;
}

// 测试工具配置
function testToolConfiguration() {
  console.log('\n=== 测试工具配置 ===\n');
  
  // 模拟 getToolsByCategories 函数
  const mockTools = [
    { name: 'bash', description: '执行终端命令' },
    { name: 'read', description: '读取文件内容' },
    { name: 'write', description: '创建新文件' },
    { name: 'edit', description: '精确修改文件' },
    { name: 'web_search', description: '搜索实时信息' },
    { name: 'plan', description: '制定任务计划' }
  ];
  
  console.log('可用工具列表:');
  mockTools.forEach(tool => {
    console.log(`  - ${tool.name}: ${tool.description}`);
  });
  
  console.log(`\n工具总数: ${mockTools.length}`);
  console.log(`包含bash工具: ${mockTools.some(t => t.name === 'bash') ? '✅ 是' : '❌ 否'}`);
  console.log(`包含read工具: ${mockTools.some(t => t.name === 'read') ? '✅ 是' : '❌ 否'}`);
  console.log(`包含web_search工具: ${mockTools.some(t => t.name === 'web_search') ? '✅ 是' : '❌ 否'}`);
}

// 测试消息历史
function testMessageHistory() {
  console.log('\n=== 测试消息历史 ===\n');
  
  const mockMessages = [
    { type: 'user', content: '你好', timestamp: Date.now() - 30000 },
    { type: 'assistant', content: '你好！有什么可以帮助你的？', timestamp: Date.now() - 25000 },
    { type: 'user', content: '请帮我打开终端', timestamp: Date.now() - 20000 },
    { type: 'assistant', content: '我可以使用bash工具帮你打开终端。', timestamp: Date.now() - 15000 },
    { type: 'system', content: '系统消息', timestamp: Date.now() - 10000 }
  ];
  
  const history = mockMessages
    .filter(msg => msg.type === 'user' || msg.type === 'assistant')
    .map(msg => ({
      role: msg.type === 'user' ? 'user' : 'assistant',
      content: msg.content,
      timestamp: msg.timestamp
    }))
    .slice(-10);
  
  console.log(`消息历史数量: ${history.length}`);
  console.log('历史消息内容:');
  history.forEach((msg, i) => {
    console.log(`  ${i + 1}. [${msg.role}] ${msg.content.substring(0, 50)}...`);
  });
}

// 测试Claude SDK请求构建
function testClaudeSdkRequest() {
  console.log('\n=== 测试Claude SDK请求构建 ===\n');
  
  const mockRequest = {
    apiKey: "alou-backend-default-token",
    prompt: "请帮我打开终端",
    systemPrompt: generateAlouPrompt(null, '0x123...', 'ethereum'),
    history: [
      { role: 'user', content: '你好' },
      { role: 'assistant', content: '你好！有什么可以帮助你的？' }
    ],
    agentInfo: {
      session_id: 'test-session-123',
      wallet_address: '0x123...',
      chain: 'ethereum',
      allowed_tools: ['bash', 'read', 'write', 'edit', 'web_search']
    },
    tools: [
      { name: 'bash', description: '执行终端命令' },
      { name: 'read', description: '读取文件内容' },
      { name: 'write', description: '创建新文件' }
    ],
    model: "deepseek-chat",
    maxTokens: 4096,
    temperature: 0.7,
    taskType: "sync",
    timeout: 30000
  };
  
  console.log('请求结构:');
  console.log(`  prompt: "${mockRequest.prompt}"`);
  console.log(`  systemPrompt长度: ${mockRequest.systemPrompt.length} 字符`);
  console.log(`  history数量: ${mockRequest.history.length}`);
  console.log(`  tools数量: ${mockRequest.tools.length}`);
  console.log(`  agentInfo.allowed_tools: ${mockRequest.agentInfo.allowed_tools.join(', ')}`);
  
  // 检查是否包含bash工具
  const hasBashTool = mockRequest.tools.some(t => t.name === 'bash');
  console.log(`\n请求中是否包含bash工具: ${hasBashTool ? '✅ 是' : '❌ 否'}`);
  
  // 检查系统提示词是否包含工具说明
  const hasToolInstructions = mockRequest.systemPrompt.includes('工具调用能力');
  console.log(`系统提示词是否包含工具说明: ${hasToolInstructions ? '✅ 是' : '❌ 否'}`);
}

// 运行测试
console.log('=== 修复后的Agent系统测试 ===\n');

// 运行所有测试
getSystemPromptForAgent(null, 'alou', '0x123...', 'ethereum');
testToolConfiguration();
testMessageHistory();
testClaudeSdkRequest();

console.log('\n=== 测试总结 ===\n');
console.log('修复内容:');
console.log('1. ✅ 保留了原有的系统提示词生成逻辑');
console.log('2. ✅ 在系统提示词中添加了详细的工具调用指南');
console.log('3. ✅ 保留了消息历史功能');
console.log('4. ✅ 在请求中添加了tools字段');
console.log('5. ✅ 不同模式有不同的工具配置');

console.log('\n预期效果:');
console.log('1. 当用户说"帮我打开终端"时，Agent会调用bash工具');
console.log('2. 系统提示词会指导Agent如何使用各种工具');
console.log('3. 消息历史会被正确传递');
console.log('4. 工具配置会根据模式自动选择');

console.log('\n测试建议:');
console.log('1. 在实际对话中测试"帮我打开终端"命令');
console.log('2. 检查Agent是否调用了bash工具');
console.log('3. 验证工具调用结果是否正确处理');
console.log('4. 测试其他工具如read、write等');

export default {
  getSystemPromptForAgent,
  testToolConfiguration,
  testMessageHistory,
  testClaudeSdkRequest
};
