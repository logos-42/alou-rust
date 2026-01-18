/**
 * memoryStore - 本地内存存储
 * 将群聊数据存储在内存中，避免localStorage空间限制
 */

/**
 * 内存存储类
 * 提供类似localStorage的API，但数据存储在内存中
 */
class MemoryStorage {
  constructor() {
    this.data = new Map()
    this.maxSize = 1000 // 最大存储项目数
    this.listeners = new Map()
  }

  setItem(key, value) {
    // 检查容量限制
    if (this.data.size >= this.maxSize) {
      // 移除最旧的项目（LRU策略）
      const firstKey = this.data.keys().next().value
      if (firstKey) {
        this.data.delete(firstKey)
        console.log(`[MemoryStorage] 容量已满，移除最旧项目: ${firstKey}`)
      }
    }

    this.data.set(key, value)
    this.notifyListeners(key, 'set', value)
    
    console.log(`[MemoryStorage] 存储数据: ${key}`)
  }

  getItem(key) {
    const value = this.data.get(key)
    console.log(`[MemoryStorage] 读取数据: ${key}, 存在: ${!!value}`)
    return value || null
  }

  removeItem(key) {
    const existed = this.data.has(key)
    this.data.delete(key)
    this.notifyListeners(key, 'remove', null)
    
    if (existed) {
      console.log(`[MemoryStorage] 删除数据: ${key}`)
    }
  }

  clear() {
    const size = this.data.size
    this.data.clear()
    this.notifyListeners('*', 'clear', null)
    console.log(`[MemoryStorage] 清空所有数据，删除了 ${size} 个项目`)
  }

  key(index) {
    const keys = Array.from(this.data.keys())
    return keys[index] || null
  }

  get length() {
    return this.data.size
  }

  keys() {
    return Array.from(this.data.keys())
  }

  values() {
    return Array.from(this.data.values())
  }

  entries() {
    return Array.from(this.data.entries())
  }

  // 事件监听器
  addEventListener(type, callback) {
    if (!this.listeners.has(type)) {
      this.listeners.set(type, new Set())
    }
    this.listeners.get(type).add(callback)
  }

  removeEventListener(type, callback) {
    if (this.listeners.has(type)) {
      this.listeners.get(type).delete(callback)
    }
  }

  notifyListeners(key, type, value) {
    // 通知特定类型的监听器
    if (this.listeners.has(type)) {
      this.listeners.get(type).forEach(callback => {
        try {
          callback({ key, type, value })
        } catch (error) {
          console.error('[MemoryStorage] 监听器错误:', error)
        }
      })
    }

    // 通知通配符监听器
    if (this.listeners.has('*')) {
      this.listeners.get('*').forEach(callback => {
        try {
          callback({ key, type, value })
        } catch (error) {
          console.error('[MemoryStorage] 通配符监听器错误:', error)
        }
      })
    }
  }

  // 获取存储统计
  getStats() {
    return {
      size: this.data.size,
      maxSize: this.maxSize,
      usage: Math.round((this.data.size / this.maxSize) * 100),
      keys: Array.from(this.data.keys()),
      memoryUsage: this.estimateMemoryUsage()
    }
  }

  // 估算内存使用量
  estimateMemoryUsage() {
    let totalSize = 0
    for (const [key, value] of this.data) {
      try {
        const size = (key + JSON.stringify(value)).length * 2 // UTF-16编码
        totalSize += size
      } catch (error) {
        // 忽略序列化错误
      }
    }
    return {
      bytes: totalSize,
      formatted: this.formatBytes(totalSize)
    }
  }

  // 格式化字节数
  formatBytes(bytes) {
    if (bytes === 0) return '0 Bytes'
    const k = 1024
    const sizes = ['Bytes', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
  }
}

// 创建全局内存存储实例
const memoryStorage = new MemoryStorage()

/**
 * 内存存储适配器
 * 提供与localStorage兼容的API
 */
export const memoryStore = {
  setItem: (key, value) => memoryStorage.setItem(key, value),
  getItem: (key) => memoryStorage.getItem(key),
  removeItem: (key) => memoryStorage.removeItem(key),
  clear: () => memoryStorage.clear(),
  key: (index) => memoryStorage.key(index),
  get length() { return memoryStorage.length },
  keys: () => memoryStorage.keys(),
  values: () => memoryStorage.values(),
  entries: () => memoryStorage.entries(),
  
  // 额外方法
  getStats: () => memoryStorage.getStats(),
  addEventListener: (type, callback) => memoryStorage.addEventListener(type, callback),
  removeEventListener: (type, callback) => memoryStorage.removeEventListener(type, callback),
  
  // 批量操作
  setItems: (items) => {
    Object.entries(items).forEach(([key, value]) => {
      memoryStorage.setItem(key, value)
    })
  },
  
  getItems: (keys) => {
    const result = {}
    keys.forEach(key => {
      result[key] = memoryStorage.getItem(key)
    })
    return result
  },
  
  removeItems: (keys) => {
    keys.forEach(key => {
      memoryStorage.removeItem(key)
    })
  },
  
  // 数据管理
  getGroupChats: (channelId) => {
    const key = `cluster_actions_group_chats_${channelId}`
    const data = memoryStorage.getItem(key)
    return data ? JSON.parse(data) : []
  },
  
  setGroupChats: (channelId, chats) => {
    const key = `cluster_actions_group_chats_${channelId}`
    memoryStorage.setItem(key, JSON.stringify(chats))
  },
  
  getActiveActionId: (channelId) => {
    const key = `cluster_actions_active_id_${channelId}`
    return memoryStorage.getItem(key)
  },
  
  setActiveActionId: (channelId, actionId) => {
    const key = `cluster_actions_active_id_${channelId}`
    if (actionId) {
      memoryStorage.setItem(key, actionId)
    } else {
      memoryStorage.removeItem(key)
    }
  },
  
  // DIAP群聊专用方法
  getDiapGroups: () => {
    const groups = []
    for (const [key, value] of memoryStorage.entries()) {
      if (key.startsWith('diap_group_chat_')) {
        const groupId = key.replace('diap_group_chat_', '')
        groups.push({ groupId, ...JSON.parse(value) })
      }
    }
    return groups
  },
  
  setDiapGroup: (groupId, groupData) => {
    const key = `diap_group_chat_${groupId}`
    memoryStorage.setItem(key, JSON.stringify(groupData))
  },
  
  removeDiapGroup: (groupId) => {
    const key = `diap_group_chat_${groupId}`
    memoryStorage.removeItem(key)
  },
  
  // 清理和优化
  cleanup: (options = {}) => {
    const {
      maxAge = 30 * 60 * 1000, // 30分钟
      maxItems = 500,
      keepPatterns = [] // 保留匹配这些模式的键
    } = options

    const now = Date.now()
    const keysToRemove = []
    
    for (const [key, value] of memoryStorage.entries()) {
      let shouldRemove = false
      
      // 检查项目数量限制
      if (memoryStorage.data.size > maxItems) {
        shouldRemove = true
      }
      
      // 检查时间限制（如果数据包含时间戳）
      try {
        const parsed = JSON.parse(value)
        if (parsed.timestamp || parsed.created_at) {
          const age = now - (parsed.timestamp || parsed.created_at)
          if (age > maxAge) {
            shouldRemove = true
          }
        }
      } catch (error) {
        // 忽略解析错误
      }
      
      // 检查保留模式
      const shouldKeep = keepPatterns.some(pattern => key.includes(pattern))
      if (shouldKeep) {
        shouldRemove = false
      }
      
      if (shouldRemove) {
        keysToRemove.push(key)
      }
    }
    
    // 执行删除
    keysToRemove.forEach(key => {
      memoryStorage.removeItem(key)
    })
    
    console.log(`[memoryStore] 清理完成，删除了 ${keysToRemove.length} 个项目`)
    return keysToRemove.length
  }
}

/**
 * 内存存储Hook
 * 提供React Hook接口
 */
import { useState, useEffect } from 'react'

export const useMemoryStore = () => {
  const [stats, setStats] = useState(memoryStorage.getStats())
  
  useEffect(() => {
    const handleChange = () => {
      setStats(memoryStorage.getStats())
    }
    
    memoryStorage.addEventListener('*', handleChange)
    
    return () => {
      memoryStorage.removeEventListener('*', handleChange)
    }
  }, [])
  
  return {
    stats,
    storage: memoryStore,
    
    // 便捷方法
    clear: () => {
      memoryStorage.clear()
      setStats(memoryStorage.getStats())
    },
    
    cleanup: (options) => {
      const removed = memoryStorage.cleanup(options)
      setStats(memoryStorage.getStats())
      return removed
    }
  }
}

export default memoryStore
