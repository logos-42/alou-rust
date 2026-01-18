/**
 * rustMemoryExample - Rust内存存储使用示例
 * 展示如何在应用中使用Rust内存存储
 */

import { rustMemoryStore } from './rustMemoryStore'

// 示例：替换localStorage为Rust内存存储
const replaceWithRustMemory = async () => {
  console.log('=== 开始使用Rust内存存储 ===')
  
  try {
    // 等待Rust内存存储就绪
    await rustMemoryStore.ready()
    
    // 测试基本操作
    await rustMemoryStore.setItem('test_key', 'test_value')
    const value = await rustMemoryStore.getItem('test_key')
    console.log('读取测试:', value) // 应该输出 'test_value'
    
    // 测试群聊数据
    const testData = {
      action_id: 'test_action_123',
      description: '测试群聊',
      agents: [
        { id: 'agent1', name: 'Claude', mode: 'agent' },
        { id: 'agent2', name: 'GPT-4', mode: 'agent' }
      ]
    }
    
    await rustMemoryStore.setGroupChats('test_channel', [testData])
    const chats = await rustMemoryStore.getGroupChats('test_channel')
    console.log('群聊数据:', chats)
    
    // 获取统计信息
    const stats = await rustMemoryStore.getStats()
    console.log('内存统计:', stats)
    
    console.log('=== Rust内存存储测试完成 ===')
    
  } catch (error) {
    console.error('Rust内存存储测试失败:', error)
  }
}

// 在现有代码中使用
export const useRustMemoryStore = () => {
  // 在你的应用启动时调用
  replaceWithRustMemory()
  
  // 然后像正常使用localStorage，但现在会自动使用Rust内存
  const exampleUsage = async () => {
    // 设置数据
    await rustMemoryStore.setItem('user_preference', 'dark_mode')
    
    // 读取数据
    const preference = await rustMemoryStore.getItem('user_preference')
    console.log('用户偏好:', preference)
    
    // 清理过期数据
    const cleaned = await rustMemoryStore.cleanupExpired()
    console.log(`清理了 ${cleaned} 个过期项目`)
    
    // 获取最新统计
    const newStats = await rustMemoryStore.getStats()
    console.log('清理后统计:', newStats)
  }
  
  return {
    setItem: rustMemoryStore.setItem,
    getItem: rustMemoryStore.getItem,
    removeItem: rustMemoryStore.removeItem,
    clear: rustMemoryStore.clear,
    getGroupChats: rustMemoryStore.getGroupChats,
    setGroupChats: rustMemoryStore.setGroupChats,
    getActiveActionId: rustMemoryStore.getActiveActionId,
    setActiveActionId: rustMemoryStore.setActiveActionId,
    getDiapGroups: rustMemoryStore.getDiapGroups,
    setDiapGroup: rustMemoryStore.setDiapGroup,
    removeDiapGroup: rustMemoryStore.removeDiapGroup,
    getStats: rustMemoryStore.getStats,
    cleanup: rustMemoryStore.cleanupExpired,
    cleanupLRU: rustMemoryStore.cleanupLRU,
    ready: rustMemoryStore.ready,
    setItems: rustMemoryStore.setItems,
    getItems: rustMemoryStore.getItems,
    removeItems: rustMemoryStore.removeItems
  }
}

export default {
  replaceWithRustMemory,
  useRustMemoryStore
}
