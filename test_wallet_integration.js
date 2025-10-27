// 钱包地址集成测试脚本
// 在浏览器控制台运行此脚本

console.log('=== 钱包地址集成测试 ===\n');

// 1. 检查 localStorage 中的钱包地址
const walletAddress = localStorage.getItem('wallet_address');
console.log('1. 本地存储的钱包地址:', walletAddress || '未设置');

// 2. 测试 session 创建
async function testSessionCreation() {
  console.log('\n2. 测试创建 session...');
  
  const response = await fetch('https://alou-edge.yuanjieliu65.workers.dev/api/session', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      wallet_address: walletAddress || undefined
    })
  });
  
  if (response.ok) {
    const data = await response.json();
    console.log('✅ Session 创建成功:', data.session_id);
    return data.session_id;
  } else {
    console.error('❌ Session 创建失败:', await response.text());
    return null;
  }
}

// 3. 测试聊天请求
async function testChatRequest(sessionId) {
  console.log('\n3. 测试聊天请求...');
  
  if (!sessionId) {
    console.log('⚠️ 跳过: 没有有效的 session_id');
    return;
  }
  
  const response = await fetch('https://alou-edge.yuanjieliu65.workers.dev/api/agent/chat', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      session_id: sessionId,
      message: 'test',
      wallet_address: walletAddress || undefined
    })
  });
  
  if (response.ok) {
    const data = await response.json();
    console.log('✅ 聊天请求成功');
  } else {
    console.error('❌ 聊天请求失败:', await response.text());
  }
}

// 4. 检查 session 信息
async function checkSessionInfo(sessionId) {
  console.log('\n4. 检查 session 信息...');
  
  if (!sessionId) return;
  
  const response = await fetch(`https://alou-edge.yuanjieliu65.workers.dev/api/session/${sessionId}`);
  
  if (response.ok) {
    const session = await response.json();
    console.log('Session 详情:', JSON.stringify(session, null, 2));
    console.log('钱包地址绑定:', session.wallet_address || '未绑定');
  }
}

// 运行测试
async function runTests() {
  const sessionId = await testSessionCreation();
  await testChatRequest(sessionId);
  await checkSessionInfo(sessionId);
}

runTests();

