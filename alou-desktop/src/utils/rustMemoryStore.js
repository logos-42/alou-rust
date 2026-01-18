/**
 * rustMemoryStore - Rust内存存储接口
 * 通过Tauri调用Rust内存管理器，提供高性能的内存存储
 */

/**
 * Rust内存存储类
 * 提供与localStorage兼容的API，但数据存储在Rust管理的内存中
 */
class RustMemoryStorage {
  constructor() {
    this.isReady = false
    this.init()
  }

  async init() {
    if (this.isReady) return
    
    try {
      // 检查Tauri API是否可用
      if (typeof window !== 'undefined' && window.__TAURI__) {
        // 测试连接
        await window.__TAURI__.invoke('get_memory_stats')
        this.isReady = true
        console.log('[RustMemoryStore] Rust内存存储已初始化')
      } else {
        console.warn('[RustMemoryStore] Tauri API不可用，降级到JavaScript内存存储')
        this.isReady = false
      }
    } catch (error) {
      console.error('[RustMemoryStore] 初始化失败:', error)
      this.isReady = false
    }
  }

  // 等待Rust初始化
  async waitForReady() {
    let attempts = 0
    while (!this.isReady && attempts < 50) {
      await new Promise(resolve => setTimeout(resolve, 100))
      await this.init()
      attempts++
    }
    
    if (!this.isReady) {
      throw new Error('Rust内存存储初始化失败')
    }
  }

  // 基础存储API
  async setItem(key, value) {
    await this.waitForReady()
    try {
      await window.__TAURI__.invoke('set_memory_item', { key, value })
      console.log(`[RustMemoryStore] 存储到Rust内存: ${key}`)
    } catch (error) {
      console.error('[RustMemoryStore] 存储失败:', error)
      throw error
    }
  }

  async getItem(key) {
    await this.waitForReady()
    try {
      const result = await window.__TAURI__.invoke('get_memory_item', { key })
      console.log(`[RustMemoryStore] 从Rust内存读取: ${key}, 存在: ${!!result}`)
      return result
    } catch (error) {
      console.error('[RustMemoryStore] 读取失败:', error)
      return null
    }
  }

  async removeItem(key) {
    await this.waitForReady()
    try {
      const result = await window.__TAURI__.invoke('remove_memory_item', { key })
      console.log(`[RustMemoryStore] 从Rust内存删除: ${key}, 成功: ${result}`)
      return result
    } catch (error) {
      console.error('[RustMemoryStore] 删除失败:', error)
      return false
    }
  }

  async clear() {
    await this.waitForReady()
    try {
      await window.__TAURI__.invoke('clear_memory')
      console.log('[RustMemoryStore] 清空Rust内存')
    } catch (error) {
      console.error('[RustMemoryStore] 清空失败:', error)
      throw error
    }
  }

  async getKeys() {
    await this.waitForReady()
    try {
      const result = await window.__TAURI__.invoke('get_memory_keys')
      console.log(`[RustMemoryStore] 获取Rust内存键列表: ${result.length} 个`)
      return result
    } catch (error) {
      console.error('[RustMemoryStore] 获取键列表失败:', error)
      return []
    }
  }

  async getStats() {
    await this.waitForReady()
    try {
      const result = await window.__TAURI__.invoke('get_memory_stats')
      console.log('[RustMemoryStore] 获取统计信息:', result)
      return result
    } catch (error) {
      console.error('[RustMemoryStore] 获取统计失败:', error)
      return null
    }
  }

  // 高级功能
  async cleanupExpired() {
    await this.waitForReady()
    try {
      const result = await window.__TAURI__.invoke('cleanup_expired_memory')
      console.log(`[RustMemoryStore] 清理过期数据: ${result} 个项目`)
      return result
    } catch (error) {
      console.error('[RustMemoryStore] 清理过期数据失败:', error)
      return 0
    }
  }

  async cleanupLRU(keepCount = 100) {
    await this.waitForReady()
    try {
      const result = await window.__TAURI__.invoke('cleanup_lru_memory', { keepCount })
      console.log(`[RustMemoryStore] LRU清理: 保留 ${keepCount} 个，删除 ${result} 个项目`)
      return result
    } catch (error) {
      console.error('[RustMemoryStore] LRU清理失败:', error)
      return 0
    }
  }

  async setExpiration(key, expiresInSeconds) {
    await this.waitForReady()
    try {
      await window.__TAURI__.invoke('set_memory_expiration', { key, expiresInSeconds })
      console.log(`[RustMemoryStore] 设置过期时间: ${key} (${expiresInSeconds} 秒后)`)
    } catch (error) {
      console.error('[RustMemoryStore] 设置过期时间失败:', error)
    }
  }

  // 批量操作
  async setItems(items) {
    await this.waitForReady()
    const promises = Object.entries(items).map(([key, value]) => this.setItem(key, value))
    await Promise.all(promises)
  }

  async getItems(keys) {
    await this.waitForReady()
    const promises = keys.map(key => this.getItem(key))
    const results = await Promise.all(promises)
    
    const result = {}
    keys.forEach((key, index) => {
      result[key] = results[index]
    })
    return result
  }

  async removeItems(keys) {
    await this.waitForReady()
    const promises = keys.map(key => this.removeItem(key))
    await Promise.all(promises)
  }
}

// 创建全局实例
const rustMemoryStoreInstance = new RustMemoryStorage()

/**
 * 兼容localStorage的适配器
 * 可以在Rust内存和JavaScript内存之间切换
 */
export const rustMemoryStore = {
  // 基础API
  setItem: (key, value) => rustMemoryStoreInstance.setItem(key, value),
  getItem: (key) => rustMemoryStoreInstance.getItem(key),
  removeItem: (key) => rustMemoryStoreInstance.removeItem(key),
  clear: () => rustMemoryStoreInstance.clear(),
  key: (index) => rustMemoryStoreInstance.getKeys().then(keys => keys[index] || null),
  get length() { return rustMemoryStoreInstance.getKeys().then(keys => keys.length) },
  keys: () => rustMemoryStoreInstance.getKeys(),
  values: () => rustMemoryStoreInstance.getKeys().then(keys => 
    Promise.all(keys.map(key => rustMemoryStoreInstance.getItem(key)))
  ),
  entries: () => rustMemoryStoreInstance.getKeys().then(keys => 
    Promise.all(keys.map(key => 
      rustMemoryStoreInstance.getItem(key).then(value => [key, value])
    ))
  ),

  // 群聊专用方法
  getGroupChats: async (channelId) => {
    const key = `cluster_actions_group_chats_${channelId}`
    const data = await rustMemoryStoreInstance.getItem(key)
    return data ? JSON.parse(data) : []
  },

  setGroupChats: async (channelId, chats) => {
    const key = `cluster_actions_group_chats_${channelId}`
    await rustMemoryStoreInstance.setItem(key, JSON.stringify(chats))
  },

  getActiveActionId: async (channelId) => {
    const key = `cluster_actions_active_id_${channelId}`
    return await rustMemoryStoreInstance.getItem(key)
  },

  setActiveActionId: async (channelId, actionId) => {
    const key = `cluster_actions_active_id_${channelId}`
    if (actionId) {
      await rustMemoryStoreInstance.setItem(key, actionId)
    } else {
      await rustMemoryStoreInstance.removeItem(key)
    }
  },

  // DIAP群聊方法
  getDiapGroups: async () => {
    const keys = await rustMemoryStoreInstance.getKeys()
    const groupKeys = keys.filter(key => key.startsWith('diap_group_chat_'))
    
    const groups = []
    for (const key of groupKeys) {
      const value = await rustMemoryStoreInstance.getItem(key)
      if (value) {
        const groupId = key.replace('diap_group_chat_', '')
        groups.push({ groupId, ...JSON.parse(value) })
      }
    }
    return groups
  },

  setDiapGroup: async (groupId, groupData) => {
    const key = `diap_group_chat_${groupId}`
    await rustMemoryStoreInstance.setItem(key, JSON.stringify(groupData))
  },

  removeDiapGroup: async (groupId) => {
    const key = `diap_group_chat_${groupId}`
    await rustMemoryStoreInstance.removeItem(key)
  },

  // 统计信息
  getStats: () => rustMemoryStoreInstance.getStats(),
  
  // 清理方法
  cleanup: (options = {}) => rustMemoryStoreInstance.cleanupExpired(),
  cleanupLRU: (keepCount) => rustMemoryStoreInstance.cleanupLRU(keepCount),
  
  // 等待就绪
  ready: () => rustMemoryStoreInstance.waitForReady(),
  
  // 批量操作
  setItems: (items) => rustMemoryStoreInstance.setItems(items),
  getItems: (keys) => rustMemoryStoreInstance.getItems(keys),
  removeItems: (keys) => rustMemoryStoreInstance.removeItems(keys)
}

export default rustMemoryStore
