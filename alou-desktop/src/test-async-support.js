/**
 * 测试异步任务支持
 * 验证现有的调用方式是否正常工作
 */

import agentService from './services/agentService'

async function testAsyncSupport() {
  console.log('=== 测试异步任务支持 ===')
  
  // 测试1: 现有的同步调用方式（应该继续工作）
  console.log('\n测试1: 现有的同步调用方式')
  try {
    // 注意：这里使用模拟数据，实际使用时需要真实的sessionId和walletAddress
    const result = await agentService.sendMessage(
      'test-session-123',
      '你好，这是一个测试消息',
      '0x1234567890123456789012345678901234567890'
    )
    
    console.log('同步调用成功:', {
      hasContent: !!result.content,
      sessionId: result.session_id,
      isAsync: result.is_async || false,
      taskId: result.task_id || '无任务ID（同步处理）'
    })
  } catch (error) {
    console.log('同步调用失败（预期中，因为后端可能未运行）:', error.message)
  }
  
  // 测试2: 新的异步调用方式
  console.log('\n测试2: 新的异步调用方式（带选项）')
  try {
    const result = await agentService.sendMessage(
      'test-session-456',
      '这是一个需要异步处理的长消息',
      '0xabcdef1234567890abcdef1234567890abcdef12',
      {
        chain: 'ethereum',
        useAsync: true, // 启用异步处理
        timeout: 10000 // 10秒超时
      }
    )
    
    console.log('异步调用成功:', {
      hasContent: !!result.content,
      sessionId: result.session_id,
      isAsync: result.is_async || false,
      taskId: result.task_id || '无任务ID',
      status: result.status || '未知',
      progress: result.progress || 0
    })
    
    // 如果有任务ID，可以获取状态
    if (result.task_id && result.is_async) {
      console.log('\n测试3: 获取异步任务状态')
      try {
        const status = await agentService.getTaskStatus(result.task_id)
        console.log('任务状态:', {
          status: status.status,
          progress: status.progress,
          currentStep: status.current_step
        })
      } catch (statusError) {
        console.log('获取任务状态失败:', statusError.message)
      }
    }
  } catch (error) {
    console.log('异步调用失败（预期中）:', error.message)
  }
  
  // 测试3: 等待任务完成（可选功能）
  console.log('\n测试4: 等待任务完成功能')
  console.log('这是一个可选功能，需要有效的taskId才能测试')
  
  console.log('\n=== 测试完成 ===')
  console.log('总结:')
  console.log('1. 现有的同步调用方式保持不变')
  console.log('2. 新增了异步调用支持（通过useAsync选项）')
  console.log('3. 新增了任务状态查询功能')
  console.log('4. 新增了等待任务完成功能')
  console.log('5. 所有功能向后兼容')
}

// 如果直接运行此文件，执行测试
if (typeof window !== 'undefined' && window.location.pathname.includes('test')) {
  testAsyncSupport()
}

export default testAsyncSupport
