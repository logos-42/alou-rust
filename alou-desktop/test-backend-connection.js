/**
 * 测试桌面版与后端连接
 * 运行: node test-backend-connection.js
 */

const API_BASE_URL = process.env.VITE_API_BASE_URL || 
  (process.env.NODE_ENV === 'development' 
    ? 'http://127.0.0.1:8787' 
    : 'https://alou-edge.yuanjieliu65.workers.dev')

async function testConnection() {
  console.log('🔍 测试后端连接...')
  console.log(`📍 后端 URL: ${API_BASE_URL}\n`)

  try {
    // 1. 健康检查
    console.log('1️⃣ 健康检查...')
    const healthRes = await fetch(`${API_BASE_URL}/api/health`)
    if (healthRes.ok) {
      console.log('   ✅ 后端健康检查通过\n')
    } else {
      console.log(`   ❌ 健康检查失败: ${healthRes.status}\n`)
      return
    }

    // 2. 获取工具列表
    console.log('2️⃣ 获取工具列表...')
    const toolsRes = await fetch(`${API_BASE_URL}/api/mcp/tools`)
    if (toolsRes.ok) {
      const toolsData = await toolsRes.json()
      console.log(`   ✅ 获取到 ${toolsData.tools?.length || 0} 个工具\n`)
    } else {
      console.log(`   ❌ 获取工具列表失败: ${toolsRes.status}\n`)
    }

    // 3. 创建测试会话
    console.log('3️⃣ 创建测试会话...')
    const sessionRes = await fetch(`${API_BASE_URL}/api/session`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        wallet_address: '0x123',
        chain: 'ethereum'
      })
    })
    if (sessionRes.ok) {
      const sessionData = await sessionRes.json()
      console.log(`   ✅ 会话创建成功: ${sessionData.session_id}\n`)
      
      // 4. 创建 Agent（设置 agent_type）
      console.log('4️⃣ 创建 Agent（agent_type: claude_agent_sdk）...')
      const agentRes = await fetch(`${API_BASE_URL}/api/agent/create-claude`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          session_id: sessionData.session_id,
          name: 'Test Agent',
          role_description: 'A test agent for connection verification',
          agent_type: 'claude_agent_sdk'
        })
      })
      if (agentRes.ok) {
        const agentData = await agentRes.json()
        console.log(`   ✅ Agent 创建成功`)
        console.log(`   📋 Agent Type: ${agentData.agent_type || agentData.agent_metadata?.agent_type}\n`)
        
        // 5. 测试聊天（应该使用 TypeScript SDK）
        console.log('5️⃣ 测试聊天（应该使用 TypeScript SDK）...')
        const chatRes = await fetch(`${API_BASE_URL}/api/agent/chat`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            session_id: sessionData.session_id,
            message: '你好，请简单介绍一下你自己',
            wallet_address: '0x123',
            chain: 'ethereum'
          })
        })
        if (chatRes.ok) {
          const chatData = await chatRes.json()
          console.log(`   ✅ 聊天成功`)
          console.log(`   💬 响应长度: ${chatData.content?.length || 0} 字符`)
          console.log(`   🔧 工具调用: ${chatData.tool_calls?.length || 0} 个\n`)
        } else {
          const errorText = await chatRes.text()
          console.log(`   ❌ 聊天失败: ${chatRes.status}`)
          console.log(`   📄 错误信息: ${errorText}\n`)
        }
      } else {
        const errorText = await agentRes.text()
        console.log(`   ❌ Agent 创建失败: ${agentRes.status}`)
        console.log(`   📄 错误信息: ${errorText}\n`)
      }
    } else {
      console.log(`   ❌ 会话创建失败: ${sessionRes.status}\n`)
    }

    console.log('✅ 所有测试完成！')
  } catch (error) {
    console.error('❌ 连接测试失败:', error.message)
    if (error.code === 'ECONNREFUSED') {
      console.error('   后端服务器未运行，请先启动后端或检查 URL')
    }
  }
}

testConnection()

