/**
 * rustMemoryExample - Rust内存存储使用示例
 * 展示如何在应用中使用Rust内存存储
 */

import { rustMemoryStore } from './rustMemoryStore'

/**
 * 群聊数据接口
 */
export interface GroupChatData {
  action_id: string
  description: string
  agents: Array<{
    id: string
    name: string
    mode: string
  }>
}

/**
 * 替换为Rust内存存储
 */
const replaceWithRustMemory = async (): Promise<void> => {
  console.log('=== 开始使用Rust内存存储 ===')
  
  try {
    // 等待Rust内存存储就绪
    await rustMemoryStore.ready()
    
    // 测试基本操作
    await rustMemoryStore.setItem('test_key', 'test_value')
    const value = await rustMemoryStore.getItem('test_key')
    console.log('读取测试:', value) // 应该输出 'test_value'
    
    // 测试群聊数据
    const testData: GroupChatData = {
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

/**
 * Rust内存存储Hook返回接口
 */
export interface UseRustMemoryStoreReturn {
  setItem: typeof rustMemoryStore.setItem
  getItem: typeof rustMemoryStore.getItem
  removeItem: typeof rustMemoryStore.removeItem
  clear: typeof rustMemoryStore.clear
  getGroupChats: typeof rustMemoryStore.getGroupChats
  setGroupChats: typeof rustMemoryStore.setGroupChats
  getActiveActionId: typeof rustMemoryStore.getActiveActionId
  setActiveActionId: typeof rustMemoryStore.setActiveActionId
  getDiapGroups: typeof rustMemoryStore.getDiapGroups
  setDiapGroup: typeof rustMemoryStore.setDiapGroup
  removeDiapGroup: typeof rustMemoryStore.removeDiapGroup
  getStats: typeof rustMemoryStore.getStats
  ready: typeof rustMemoryStore.ready
  setItems: typeof rustMemoryStore.setItems
  getItems: typeof rustMemoryStore.getItems
  removeItems: typeof rustMemoryStore.removeItems
}

/**
 * 使用Rust内存存储
 */
export const useRustMemoryStore = (): UseRustMemoryStoreReturn => {
  // 在你的应用启动时调用
  replaceWithRustMemory()

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
