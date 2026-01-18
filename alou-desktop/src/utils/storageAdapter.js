/**
 * 存储适配器
 * 提供统一的存储接口，可以在 localStorage 和内存存储之间切换
 */

import { 
  setMemoryItem, 
  getMemoryItem, 
  removeMemoryItem, 
  hasMemoryItem,
  getMemoryStats,
  cleanupMemoryStorage 
} from './memoryStorage'

/**
 * 存储类型枚举
 */
export const STORAGE_TYPES = {
  LOCAL: 'local',
  MEMORY: 'memory'
}

/**
 * 当前存储类型
 */
let currentStorageType = STORAGE_TYPES.MEMORY // 默认使用内存存储

/**
 * 设置存储类型
 */
export const setStorageType = (type) => {
  if (Object.values(STORAGE_TYPES).includes(type)) {
    currentStorageType = type
    console.log(`[StorageAdapter] 切换到 ${type} 存储模式`)
    return true
  }
  return false
}

/**
 * 获取当前存储类型
 */
export const getStorageType = () => currentStorageType

/**
 * 统一的存储接口
 */
class StorageAdapter {
  /**
   * 设置数据
   */
  static setItem(key, value, options = {}) {
    if (currentStorageType === STORAGE_TYPES.MEMORY) {
      return setMemoryItem(key, value, options)
    } else {
      try {
        if (typeof window === 'undefined') return false
        
        const serializedValue = JSON.stringify({
          data: value,
          timestamp: Date.now(),
          ...options
        })
        
        localStorage.setItem(key, serializedValue)
        return true
      } catch (error) {
        console.error('[StorageAdapter] localStorage 设置失败:', error)
        // 降级到内存存储
        return setMemoryItem(key, value, options)
      }
    }
  }

  /**
   * 获取数据
   */
  static getItem(key, defaultValue = null) {
    if (currentStorageType === STORAGE_TYPES.MEMORY) {
      return getMemoryItem(key, defaultValue)
    } else {
      try {
        if (typeof window === 'undefined') return defaultValue
        
        const serializedValue = localStorage.getItem(key)
        if (!serializedValue) return defaultValue
        
        const parsed = JSON.parse(serializedValue)
        return parsed.data !== undefined ? parsed.data : parsed
      } catch (error) {
        console.error('[StorageAdapter] localStorage 获取失败:', error)
        // 尝试从内存存储获取
        return getMemoryItem(key, defaultValue)
      }
    }
  }

  /**
   * 删除数据
   */
  static removeItem(key) {
    if (currentStorageType === STORAGE_TYPES.MEMORY) {
      return removeMemoryItem(key)
    } else {
      try {
        if (typeof window === 'undefined') return false
        localStorage.removeItem(key)
        return true
      } catch (error) {
        console.error('[StorageAdapter] localStorage 删除失败:', error)
        return removeMemoryItem(key)
      }
    }
  }

  /**
   * 检查键是否存在
   */
  static hasItem(key) {
    if (currentStorageType === STORAGE_TYPES.MEMORY) {
      return hasMemoryItem(key)
    } else {
      try {
        if (typeof window === 'undefined') return false
        return localStorage.getItem(key) !== null
      } catch (error) {
        console.error('[StorageAdapter] localStorage 检查失败:', error)
        return hasMemoryItem(key)
      }
    }
  }

  /**
   * 获取所有键
   */
  static getKeys() {
    if (currentStorageType === STORAGE_TYPES.MEMORY) {
      // 从内存存储获取
      const memoryStorage = window.AlouMemoryStorage
      return memoryStorage ? memoryStorage.keys() : []
    } else {
      try {
        if (typeof window === 'undefined') return []
        const keys = []
        for (let i = 0; i < localStorage.length; i++) {
          const key = localStorage.key(i)
          if (key) keys.push(key)
        }
        return keys
      } catch (error) {
        console.error('[StorageAdapter] localStorage 获取键失败:', error)
        return []
      }
    }
  }

  /**
   * 清空所有数据
   */
  static clear() {
    if (currentStorageType === STORAGE_TYPES.MEMORY) {
      const memoryStorage = window.AlouMemoryStorage
      if (memoryStorage) {
        memoryStorage.clear()
      }
    } else {
      try {
        if (typeof window !== 'undefined') {
          localStorage.clear()
        }
      } catch (error) {
        console.error('[StorageAdapter] localStorage 清空失败:', error)
      }
    }
  }

  /**
   * 获取存储统计信息
   */
  static getStats() {
    if (currentStorageType === STORAGE_TYPES.MEMORY) {
      return getMemoryStats()
    } else {
      try {
        if (typeof window === 'undefined') return null
        
        let totalSize = 0
        let itemCount = 0
        
        for (let i = 0; i < localStorage.length; i++) {
          const key = localStorage.key(i)
          if (key) {
            const value = localStorage.getItem(key)
            if (value) {
              totalSize += new Blob([key + value]).size
              itemCount++
            }
          }
        }
        
        return {
          totalItems: itemCount,
          memoryUsage: totalSize,
          memoryUsageFormatted: this.formatBytes(totalSize),
          usagePercentage: Math.round((totalSize / (5 * 1024 * 1024)) * 100), // 假设5MB限制
          storageType: currentStorageType
        }
      } catch (error) {
        console.error('[StorageAdapter] 获取统计信息失败:', error)
        return null
      }
    }
  }

  /**
   * 清理过期数据
   */
  static cleanup() {
    if (currentStorageType === STORAGE_TYPES.MEMORY) {
      return cleanupMemoryStorage()
    } else {
      // localStorage 的清理逻辑可以在这里实现
      console.log('[StorageAdapter] localStorage 清理功能暂未实现')
      return 0
    }
  }

  /**
   * 格式化字节数
   */
  static formatBytes(bytes) {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
  }

  /**
   * 迁移数据
   */
  static migrateData(fromType, toType) {
    const originalType = currentStorageType
    let migrated = 0
    let errors = []

    try {
      // 切换到源存储类型
      setStorageType(fromType)
      const keys = this.getKeys()

      // 切换到目标存储类型
      setStorageType(toType)

      // 迁移数据
      keys.forEach(key => {
        try {
          const value = this.getItem(key)
          if (value !== null) {
            this.setItem(key, value, { persist: true })
            migrated++
          }
        } catch (error) {
          errors.push(`迁移键 ${key} 失败: ${error.message}`)
        }
      })

      console.log(`[StorageAdapter] 数据迁移完成: ${migrated} 项成功, ${errors.length} 项失败`)
      
      return {
        migrated,
        errors,
        total: keys.length
      }
    } catch (error) {
      console.error('[StorageAdapter] 数据迁移失败:', error)
      return {
        migrated,
        errors: [...errors, `迁移失败: ${error.message}`],
        total: 0
      }
    } finally {
      // 恢复原始存储类型
      setStorageType(originalType)
    }
  }
}

/**
 * 便捷的导出函数
 */
export const setItem = (key, value, options) => StorageAdapter.setItem(key, value, options)
export const getItem = (key, defaultValue) => StorageAdapter.getItem(key, defaultValue)
export const removeItem = (key) => StorageAdapter.removeItem(key)
export const hasItem = (key) => StorageAdapter.hasItem(key)
export const getKeys = () => StorageAdapter.getKeys()
export const clearStorage = () => StorageAdapter.clear()
export const getStorageStats = () => StorageAdapter.getStats()
export const cleanupStorage = () => StorageAdapter.cleanup()
export const migrateStorage = (from, to) => StorageAdapter.migrateData(from, to)

/**
 * 群聊数据专用函数
 */
export const setGroupChatData = (channelId, groupChats, options = {}) => {
  const key = `cluster_actions_group_chats_${channelId || 'global'}`
  return setItem(key, groupChats, { persist: true, ...options })
}

export const getGroupChatData = (channelId, defaultValue = []) => {
  const key = `cluster_actions_group_chats_${channelId || 'global'}`
  return getItem(key, defaultValue)
}

export const setActiveGroupId = (channelId, actionId, options = {}) => {
  const key = `cluster_actions_active_id_${channelId || 'global'}`
  return setItem(key, actionId, { persist: true, ...options })
}

export const getActiveGroupId = (channelId, defaultValue = null) => {
  const key = `cluster_actions_active_id_${channelId || 'global'}`
  return getItem(key, defaultValue)
}

/**
 * 钱包数据专用函数
 */
export const setWalletData = (address, walletType, chainId) => {
  setItem('wallet_address', address, { persist: true })
  setItem('wallet_type', walletType, { persist: true })
  setItem('wallet_chain_id', chainId, { persist: true })
}

export const getWalletData = () => {
  return {
    address: getItem('wallet_address'),
    type: getItem('wallet_type'),
    chainId: getItem('wallet_chain_id')
  }
}

/**
 * 控制台工具
 */
if (typeof window !== 'undefined') {
  window.AlouStorageAdapter = {
    // 存储类型控制
    setType: setStorageType,
    getType: getStorageType,
    migrate: migrateStorage,
    
    // 基础操作
    set: setItem,
    get: getItem,
    remove: removeItem,
    has: hasItem,
    keys: getKeys,
    clear: clearStorage,
    
    // 统计信息
    stats: getStorageStats,
    cleanup: cleanupStorage,
    
    // 专用函数
    setGroupChat: setGroupChatData,
    getGroupChat: getGroupChatData,
    setActiveGroup: setActiveGroupId,
    getActiveGroup: getActiveGroupId,
    setWallet: setWalletData,
    getWallet: getWalletData,
    
    // 帮助
    help: () => {
      console.group('📚 AlouStorageAdapter 使用帮助')
      console.log('存储类型控制:')
      console.log('  AlouStorageAdapter.setType("memory" | "local") - 设置存储类型')
      console.log('  AlouStorageAdapter.getType() - 获取当前存储类型')
      console.log('  AlouStorageAdapter.migrate(from, to) - 迁移数据')
      console.log('')
      console.log('基础操作:')
      console.log('  AlouStorageAdapter.set(key, value, options) - 设置数据')
      console.log('  AlouStorageAdapter.get(key, defaultValue) - 获取数据')
      console.log('  AlouStorageAdapter.remove(key) - 删除数据')
      console.log('  AlouStorageAdapter.has(key) - 检查键是否存在')
      console.log('  AlouStorageAdapter.keys() - 获取所有键')
      console.log('  AlouStorageAdapter.clear() - 清空所有数据')
      console.log('')
      console.log('统计信息:')
      console.log('  AlouStorageAdapter.stats() - 获取存储统计')
      console.log('  AlouStorageAdapter.cleanup() - 清理过期数据')
      console.groupEnd()
    }
  }
  
  console.log('🎉 AlouStorageAdapter 已加载!')
  console.log(`💡 当前存储模式: ${currentStorageType}`)
  console.log('💡 输入 AlouStorageAdapter.help() 查看使用帮助')
  console.log('📊 输入 AlouStorageAdapter.stats() 查看存储状态')
}

export default StorageAdapter
