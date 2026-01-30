/**
 * memoryStore - 本地内存存储
 * 将群聊数据存储在内存中，避免localStorage空间限制
 */

import { useState, useEffect } from 'react'

/**
 * 存储统计信息接口
 */
export interface StorageStats {
  size: number
  maxSize: number
  usage: number
  keys: string[]
  memoryUsage: {
    bytes: number
    formatted: string
  }
}

/**
 * 存储事件类型
 */
export type StorageEventType = 'set' | 'remove' | 'clear'

/**
 * 存储事件
 */
export interface StorageEvent {
  key: string
  type: StorageEventType
  value: any
}

/**
 * 监听器回调类型
 */
export type StorageListener = (event: StorageEvent) => void

/**
 * 清理选项接口
 */
export interface CleanupOptions {
  maxAge?: number
  maxItems?: number
  keepPatterns?: string[]
}

/**
 * DIAP群组数据接口
 */
export interface DiapGroupData {
  groupId: string
  [key: string]: any
}

/**
 * 内存存储类
 * 提供类似localStorage的API，但数据存储在内存中
 */
class MemoryStorage {
  private data: Map<string, any> = new Map()
  private maxSize: number = 1000 // 最大存储项目数
  private listeners: Map<string, Set<StorageListener>> = new Map()

  /**
   * 设置存储项
   */
  setItem(key: string, value: any): void {
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

  /**
   * 获取存储项
   */
  getItem(key: string): any | null {
    const value = this.data.get(key)
    console.log(`[MemoryStorage] 读取数据: ${key}, 存在: ${!!value}`)
    return value || null
  }

  /**
   * 移除存储项
   */
  removeItem(key: string): void {
    const existed = this.data.has(key)
    this.data.delete(key)
    this.notifyListeners(key, 'remove', null)
    
    if (existed) {
      console.log(`[MemoryStorage] 删除数据: ${key}`)
    }
  }

  /**
   * 清空所有数据
   */
  clear(): void {
    const size = this.data.size
    this.data.clear()
    this.notifyListeners('*', 'clear', null)
    console.log(`[MemoryStorage] 清空所有数据，删除了 ${size} 个项目`)
  }

  /**
   * 通过索引获取键名
   */
  key(index: number): string | null {
    const keys = Array.from(this.data.keys())
    return keys[index] || null
  }

  /**
   * 获取存储项数量
   */
  get length(): number {
    return this.data.size
  }

  /**
   * 获取所有键名
   */
  keys(): string[] {
    return Array.from(this.data.keys())
  }

  /**
   * 获取所有值
   */
  values(): any[] {
    return Array.from(this.data.values())
  }

  /**
   * 获取所有键值对
   */
  entries(): [string, any][] {
    return Array.from(this.data.entries())
  }

  /**
   * 添加事件监听器
   */
  addEventListener(type: string, callback: StorageListener): void {
    if (!this.listeners.has(type)) {
      this.listeners.set(type, new Set())
    }
    this.listeners.get(type)!.add(callback)
  }

  /**
   * 移除事件监听器
   */
  removeEventListener(type: string, callback: StorageListener): void {
    if (this.listeners.has(type)) {
      this.listeners.get(type)!.delete(callback)
    }
  }

  /**
   * 通知所有监听器
   */
  private notifyListeners(key: string, type: StorageEventType, value: any): void {
    // 通知特定类型的监听器
    if (this.listeners.has(type)) {
      this.listeners.get(type)!.forEach((callback: StorageListener) => {
        try {
          callback({ key, type, value })
        } catch (error) {
          console.error('[MemoryStorage] 监听器错误:', error)
        }
      })
    }

    // 通知通配符监听器
    if (this.listeners.has('*')) {
      this.listeners.get('*')!.forEach((callback: StorageListener) => {
        try {
          callback({ key, type, value })
        } catch (error) {
          console.error('[MemoryStorage] 通配符监听器错误:', error)
        }
      })
    }
  }

  /**
   * 获取存储统计
   */
  getStats(): StorageStats {
    return {
      size: this.data.size,
      maxSize: this.maxSize,
      usage: Math.round((this.data.size / this.maxSize) * 100),
      keys: Array.from(this.data.keys()),
      memoryUsage: this.estimateMemoryUsage()
    }
  }

  /**
   * 估算内存使用量
   */
  private estimateMemoryUsage(): { bytes: number; formatted: string } {
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

  /**
   * 格式化字节数
   */
  private formatBytes(bytes: number): string {
    if (bytes === 0) return '0 Bytes'
    const k = 1024
    const sizes = ['Bytes', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
  }

  /**
   * 清理数据
   */
  cleanup(options: CleanupOptions = {}): number {
    const {
      maxAge = 30 * 60 * 1000, // 30分钟
      maxItems = 500,
      keepPatterns = []
    } = options

    const now = Date.now()
    const keysToRemove: string[] = []
    
    for (const [key, value] of this.data) {
      let shouldRemove = false
      
      // 检查项目数量限制
      if (this.data.size > maxItems) {
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
      this.data.delete(key)
    })
    
    console.log(`[memoryStore] 清理完成，删除了 ${keysToRemove.length} 个项目`)
    return keysToRemove.length
  }
}

// 创建全局内存存储实例
const memoryStorage = new MemoryStorage()

/**
 * 内存存储适配器
 * 提供与localStorage兼容的API
 */
export interface MemoryStore {
  setItem: (key: string, value: any) => void
  getItem: (key: string) => any | null
  removeItem: (key: string) => void
  clear: () => void
  key: (index: number) => string | null
  length: number
  keys: () => string[]
  values: () => any[]
  entries: () => [string, any][]
  getStats: () => StorageStats
  addEventListener: (type: string, callback: StorageListener) => void
  removeEventListener: (type: string, callback: StorageListener) => void
  setItems: (items: Record<string, any>) => void
  getItems: (keys: string[]) => Record<string, any>
  removeItems: (keys: string[]) => void
  getGroupChats: (channelId: string) => any[]
  setGroupChats: (channelId: string, chats: any[]) => void
  getActiveActionId: (channelId: string) => string | null
  setActiveActionId: (channelId: string, actionId: string | null) => void
  getDiapGroups: () => DiapGroupData[]
  setDiapGroup: (groupId: string, groupData: any) => void
  removeDiapGroup: (groupId: string) => void
  cleanup: (options?: CleanupOptions) => number
}

export const memoryStore: MemoryStore = {
  setItem: (key: string, value: any) => memoryStorage.setItem(key, value),
  getItem: (key: string) => memoryStorage.getItem(key),
  removeItem: (key: string) => memoryStorage.removeItem(key),
  clear: () => memoryStorage.clear(),
  key: (index: number) => memoryStorage.key(index),
  get length() { return memoryStorage.length },
  keys: () => memoryStorage.keys(),
  values: () => memoryStorage.values(),
  entries: () => memoryStorage.entries(),
  
  // 额外方法
  getStats: () => memoryStorage.getStats(),
  addEventListener: (type: string, callback: StorageListener) => memoryStorage.addEventListener(type, callback),
  removeEventListener: (type: string, callback: StorageListener) => memoryStorage.removeEventListener(type, callback),
  
  // 批量操作
  setItems: (items: Record<string, any>) => {
    Object.entries(items).forEach(([key, value]) => {
      memoryStorage.setItem(key, value)
    })
  },
  
  getItems: (keys: string[]) => {
    const result: Record<string, any> = {}
    keys.forEach(key => {
      result[key] = memoryStorage.getItem(key)
    })
    return result
  },
  
  removeItems: (keys: string[]) => {
    keys.forEach(key => {
      memoryStorage.removeItem(key)
    })
  },
  
  // 数据管理
  getGroupChats: (channelId: string) => {
    const key = `cluster_actions_group_chats_${channelId}`
    const data = memoryStorage.getItem(key)
    return data ? JSON.parse(data) : []
  },
  
  setGroupChats: (channelId: string, chats: any[]) => {
    const key = `cluster_actions_group_chats_${channelId}`
    memoryStorage.setItem(key, JSON.stringify(chats))
  },
  
  getActiveActionId: (channelId: string) => {
    const key = `cluster_actions_active_id_${channelId}`
    return memoryStorage.getItem(key)
  },
  
  setActiveActionId: (channelId: string, actionId: string | null) => {
    const key = `cluster_actions_active_id_${channelId}`
    if (actionId) {
      memoryStorage.setItem(key, actionId)
    } else {
      memoryStorage.removeItem(key)
    }
  },
  
  // DIAP群聊专用方法
  getDiapGroups: () => {
    const groups: DiapGroupData[] = []
    for (const [key, value] of memoryStorage.entries()) {
      if (key.startsWith('diap_group_chat_')) {
        const groupId = key.replace('diap_group_chat_', '')
        groups.push({ groupId, ...JSON.parse(value) })
      }
    }
    return groups
  },
  
  setDiapGroup: (groupId: string, groupData: any) => {
    const key = `diap_group_chat_${groupId}`
    memoryStorage.setItem(key, JSON.stringify(groupData))
  },
  
  removeDiapGroup: (groupId: string) => {
    const key = `diap_group_chat_${groupId}`
    memoryStorage.removeItem(key)
  },
  
  // 清理和优化
  cleanup: (options?: CleanupOptions) => memoryStorage.cleanup(options)
}

/**
 * 使用内存存储Hook
 * 提供React Hook接口
 */
export interface UseMemoryStoreReturn {
  stats: StorageStats
  storage: MemoryStore
  clear: () => void
  cleanup: (options?: CleanupOptions) => number
}

export const useMemoryStore = (): UseMemoryStoreReturn => {
  const [stats, setStats] = useState<StorageStats>(memoryStorage.getStats())
  
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
    
    cleanup: (options?: CleanupOptions) => {
      const removed = memoryStorage.cleanup(options)
      setStats(memoryStorage.getStats())
      return removed
    }
  }
}

export default memoryStore
