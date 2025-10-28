// 工作流工具测试脚本
// 在浏览器控制台或Node.js中运行

const API_BASE_URL = 'https://alou-edge.yuanjieliu65.workers.dev';

console.log('=== 工作流工具测试 ===\n');

// 1. 创建测试session
async function createSession() {
  console.log('1. 创建测试session...');
  const response = await fetch(`${API_BASE_URL}/api/session`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({})
  });
  
  if (!response.ok) {
    throw new Error(`Session创建失败: ${response.status}`);
  }
  
  const data = await response.json();
  console.log('✅ Session创建成功:', data.session_id);
  return data.session_id;
}

// 2. 通过AI测试工作流工具 - 创建
async function testWorkflowCreate(sessionId) {
  console.log('\n2. 通过AI测试创建工作流...');
  
  const response = await fetch(`${API_BASE_URL}/api/agent/chat`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      session_id: sessionId,
      message: '请帮我创建一个名为"测试工作流"的工作流，包含以下步骤：1) 使用echo工具输出"步骤1" 2) 使用echo工具输出"步骤2"，依赖步骤1',
      wallet_address: undefined
    })
  });
  
  if (!response.ok) {
    throw new Error(`聊天请求失败: ${response.status}`);
  }
  
  const data = await response.json();
  console.log('AI回复:', data.content);
  return data;
}

// 3. 直接调用工作流工具 - 列出所有工作流
async function testWorkflowList(sessionId) {
  console.log('\n3. 直接测试列出所有工作流...');
  
  // 注意：这里需要直接调用工具API，暂时用AI来测试
  const response = await fetch(`${API_BASE_URL}/api/agent/chat`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      session_id: sessionId,
      message: '请帮我列出所有的工作流',
      wallet_address: undefined
    })
  });
  
  if (!response.ok) {
    throw new Error(`聊天请求失败: ${response.status}`);
  }
  
  const data = await response.json();
  console.log('AI回复:', data.content);
  return data;
}

// 4. 测试创建工作流（模拟工具调用）
async function testWorkflowCreateWithTool(sessionId) {
  console.log('\n4. 模拟工具调用创建工作流...');
  
  // 模拟直接通过AI工具调用
  const response = await fetch(`${API_BASE_URL}/api/agent/chat`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      session_id: sessionId,
      message: '请使用workflow工具，action为create，创建一个名为"钱包查询工作流"的工作流，包含一个步骤：使用query_blockchain工具查询地址0xb4eDCf79055a8232670eBb1c8c664dFf4e70066的余额',
      wallet_address: '0xb4eDCf79055a8232670eBb1c8c664dFf4e70066'
    })
  });
  
  if (!response.ok) {
    throw new Error(`聊天请求失败: ${response.status}`);
  }
  
  const data = await response.json();
  console.log('AI回复:', data.content);
  console.log('工具调用:', JSON.stringify(data.tool_calls, null, 2));
  return data;
}

// 运行测试
async function runTests() {
  try {
    console.log('开始测试工作流工具...\n');
    
    // 创建session
    const sessionId = await createSession();
    
    // 测试1: 通过AI创建工作流
    await testWorkflowCreate(sessionId);
    
    // 测试2: 列出工作流
    await testWorkflowList(sessionId);
    
    // 测试3: 创建实际可执行的工作流
    await testWorkflowCreateWithTool(sessionId);
    
    console.log('\n✅ 所有测试完成！');
  } catch (error) {
    console.error('❌ 测试失败:', error.message);
  }
}

// 导出函数供在浏览器或Node.js中使用
if (typeof module !== 'undefined' && module.exports) {
  module.exports = { runTests };
} else {
  // 浏览器环境，直接运行
  runTests();
}

