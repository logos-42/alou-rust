/**
 * 内存存储工具
 * 将数据存储在内存中，避免 localStorage 空间限制
 * 支持会话持久化和数据恢复
 */

// ==================== 类型定义 ====================

/**
 * 存储选项接口
 */
export interface MemoryStorageOptions {
  persist?: boolean
  ttl?: number
  [key: string]: any
}

/**
 * 存储数据包装接口
 */
export interface StoredDataWrapper<T = any> {
  data: T
  timestamp: number
  ttl: number
  persist: boolean
}

/**
 * 存储统计信息接口
 */
export interface MemoryStorageStats {
  totalItems: number
  expiredItems: number
  maxItems: number
  memoryUsage: number
  memoryUsageFormatted: string
  usagePercentage: number
}

/**
 * DIAP身份接口
 */
export interface DiapIdentity {
  did: string
  cid: string
  ipns: string
  stored_at?: number
  session_id?: string
  [key: string]: any
}

/**
 * 钱包数据接口
 */
export interface WalletData {
  address: string | null
  type: string | null
  chainId: string | null
}

// ==================== 内存存储类 ====================

/**
 * 内存存储类
 */
class MemoryStorage {
  private data: Map<string, string> = new Map()
  private timestamps: Map<string, number> = new Map()
  private maxItems: number = 1000 // 最大存储项目数
  private maxAge: number = 24 * 60 * 60 * 1000 // 24小时过期时间
  private cleanupInterval: number = 5 * 60 * 1000 // 5分钟清理一次过期数据
  private cleanupTimer: NodeJS.Timeout | null = null

  constructor() {
    // 启动定期清理
    this.startCleanup()
    
    // 尝试从 sessionStorage 恢复关键数据
    this.recoverFromSession()
  }

  /**
   * 设置数据
   */
  setItem(key: string, value: any, options: MemoryStorageOptions = {}): boolean {
    const { persist = false, ttl = this.maxAge } = options
    
    try {
      // 序列化数据
      const serializedValue = JSON.stringify({
        data: value,
        timestamp: Date.now(),
        ttl,
        persist
      })
      
      // 检查存储限制
      if (this.data.size >= this.maxItems && !this.data.has(key)) {
        this.evictOldest()
      }
      
      // 存储到内存
      this.data.set(key, serializedValue)
      this.timestamps.set(key, Date.now())
      
      // 如果需要持久化，存储到 sessionStorage
      if (persist && typeof window !== 'undefined' && window.sessionStorage) {
        try {
          window.sessionStorage.setItem(`memory_${key}`, serializedValue)
        } catch (error: any) {
          console.warn('[MemoryStorage] sessionStorage 存储失败:', error)
        }
      }
      
      return true
    } catch (error: any) {
      console.error('[MemoryStorage] 设置数据失败:', error)
      return false
    }
  }

  /**
   * 获取数据
   */
  getItem<T = any>(key: string, defaultValue: T | null = null): T | null {
    try {
      // 首先从内存获取
      let serializedValue = this.data.get(key)
      
      // 如果内存中没有，尝试从 sessionStorage 恢复
      if (!serializedValue && typeof window !== 'undefined' && window.sessionStorage) {
        const sessionValue = window.sessionStorage.getItem(`memory_${key}`)
        if (sessionValue) {
          serializedValue = sessionValue
          this.data.set(key, sessionValue)
          this.timestamps.set(key, Date.now())
        }
      }
      
      if (!serializedValue) {
        return defaultValue
      }
      
      // 解析数据
      const parsed: StoredDataWrapper<T> = JSON.parse(serializedValue)
      const now = Date.now()
      
      // 检查是否过期
      if (now - parsed.timestamp > parsed.ttl) {
        this.removeItem(key)
        return defaultValue
      }
      
      // 更新访问时间
      this.timestamps.set(key, now)
      
      return parsed.data
    } catch (error: any) {
      console.error('[MemoryStorage] 获取数据失败:', error)
      return defaultValue
    }
  }

  /**
   * 删除数据
   */
  removeItem(key: string): boolean {
    const deleted = this.data.delete(key)
    this.timestamps.delete(key)
    
    // 同时从 sessionStorage 删除
    if (typeof window !== 'undefined' && window.sessionStorage) {
      try {
        window.sessionStorage.removeItem(`memory_${key}`)
      } catch (error: any) {
        console.warn('[MemoryStorage] sessionStorage 删除失败:', error)
      }
    }
    
    return deleted
  }

  /**
   * 检查键是否存在
   */
  hasItem(key: string): boolean {
    return this.data.has(key) && this.getItem(key) !== null
  }

  /**
   * 获取所有键
   */
  getKeys(): string[] {
    return Array.from(this.data.keys())
  }

  /**
   * 清空所有数据
   */
  clear(): void {
    this.data.clear()
    this.timestamps.clear()
    
    // 清空 sessionStorage 中的内存数据
    if (typeof window !== 'undefined' && window.sessionStorage) {
      try {
        const keysToRemove: string[] = []
        for (let i = 0; i < window.sessionStorage.length; i++) {
          const key = window.sessionStorage.key(i)
          if (key && key.startsWith('memory_')) {
            keysToRemove.push(key)
          }
        }
        keysToRemove.forEach(key => window.sessionStorage.removeItem(key))
      } catch (error: any) {
        console.warn('[MemoryStorage] 清空 sessionStorage 失败:', error)
      }
    }
  }

  /**
   * 获取存储统计信息
   */
  getStats(): MemoryStorageStats {
    const now = Date.now()
    let expiredCount = 0
    let totalSize = 0
    
    this.data.forEach((value, key) => {
      try {
        const parsed: StoredDataWrapper = JSON.parse(value)
        if (now - parsed.timestamp > parsed.ttl) {
          expiredCount++
        }
        totalSize += new Blob([key + value]).size
      } catch (error) {
        // 忽略解析错误
      }
    })
    
    return {
      totalItems: this.data.size,
      expiredItems: expiredCount,
      maxItems: this.maxItems,
      memoryUsage: totalSize,
      memoryUsageFormatted: this.formatBytes(totalSize),
      usagePercentage: Math.round((this.data.size / this.maxItems) * 100)
    }
  }

  /**
   * 清理过期数据
   */
  cleanup(): number {
    const now = Date.now()
    const keysToRemove: string[] = []
    
    this.data.forEach((value, key) => {
      try {
        const parsed: StoredDataWrapper = JSON.parse(value)
        if (now - parsed.timestamp > parsed.ttl) {
          keysToRemove.push(key)
        }
      } catch (error) {
        // 解析失败的数据也删除
        keysToRemove.push(key)
      }
    })
    
    keysToRemove.forEach(key => this.removeItem(key))
    
    if (keysToRemove.length > 0) {
      console.log(`[MemoryStorage] 清理了 ${keysToRemove.length} 个过期项目`)
    }
    
    return keysToRemove.length
  }

  /**
   * 驱逐最旧的数据
   */
  private evictOldest(): void {
    let oldestKey: string | null = null
    let oldestTime = Date.now()
    
    this.timestamps.forEach((timestamp, key) => {
      if (timestamp < oldestTime) {
        oldestTime = timestamp
        oldestKey = key
      }
    })
    
    if (oldestKey) {
      this.removeItem(oldestKey)
      console.log(`[MemoryStorage] 驱逐最旧数据: ${oldestKey}`)
    }
  }

  /**
   * 启动定期清理
   */
  private startCleanup(): void {
    if (this.cleanupTimer) {
      clearInterval(this.cleanupTimer)
    }
    
    this.cleanupTimer = setInterval(() => {
      this.cleanup()
    }, this.cleanupInterval)
  }

  /**
   * 停止定期清理
   */
  stopCleanup(): void {
    if (this.cleanupTimer) {
      clearInterval(this.cleanupTimer)
      this.cleanupTimer = null
    }
  }

  /**
   * 从 sessionStorage 恢复数据
   */
  private recoverFromSession(): void {
    if (typeof window === 'undefined' || !window.sessionStorage) {
      return
    }
    
    try {
      const recovered: string[] = []
      for (let i = 0; i < window.sessionStorage.length; i++) {
        const key = window.sessionStorage.key(i)
        if (key && key.startsWith('memory_')) {
          const memoryKey = key.replace('memory_', '')
          const value = window.sessionStorage.getItem(key)
          
          if (value) {
            try {
              const parsed: StoredDataWrapper = JSON.parse(value)
              const now = Date.now()
              
              // 只恢复未过期的持久化数据
              if (parsed.persist && now - parsed.timestamp <= parsed.ttl) {
                this.data.set(memoryKey, value)
                this.timestamps.set(memoryKey, parsed.timestamp)
                recovered.push(memoryKey)
              }
            } catch (error) {
              // 清理损坏的数据
              window.sessionStorage.removeItem(key)
            }
          }
        }
      }
      
      if (recovered.length > 0) {
        console.log(`[MemoryStorage] 从会话恢复了 ${recovered.length} 个项目`)
      }
    } catch (error: any) {
      console.warn('[MemoryStorage] 会话恢复失败:', error)
    }
  }

  /**
   * 格式化字节数
   */
  private formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
  }

  /**
   * 销毁存储实例
   */
  destroy(): void {
    this.stopCleanup()
    this.clear()
  }
}

// ==================== 全局实例 ====================

// 创建全局实例
const memoryStorage = new MemoryStorage()

// 页面卸载时清理
if (typeof window !== 'undefined') {
  window.addEventListener('beforeunload', () => {
    memoryStorage.stopCleanup()
  })
}

// ==================== 便捷的存储函数 ====================

export const setMemoryItem = (key: string, value: any, options?: MemoryStorageOptions): boolean => 
  memoryStorage.setItem(key, value, options)
export const getMemoryItem = <T = any>(key: string, defaultValue?: T): T | null => 
  memoryStorage.getItem(key, defaultValue)
export const removeMemoryItem = (key: string): boolean => memoryStorage.removeItem(key)
export const hasMemoryItem = (key: string): boolean => memoryStorage.hasItem(key)
export const getMemoryKeys = (): string[] => memoryStorage.getKeys()
export const clearMemoryStorage = (): void => memoryStorage.clear()
export const getMemoryStats = (): MemoryStorageStats => memoryStorage.getStats()
export const cleanupMemoryStorage = (): number => memoryStorage.cleanup()

// ==================== 群聊数据专用存储函数 ====================

export const setGroupChatData = (channelId: string | null, groupChats: any[], options: MemoryStorageOptions = {}): boolean => {
  const key = `cluster_actions_group_chats_${channelId || 'global'}`
  return setMemoryItem(key, groupChats, { persist: true, ...options })
}

export const getGroupChatData = (channelId: string | null, defaultValue: any[] = []): any[] => {
  const key = `cluster_actions_group_chats_${channelId || 'global'}`
  return getMemoryItem(key, defaultValue) || defaultValue
}

export const setActiveGroupId = (channelId: string | null, actionId: string | null, options: MemoryStorageOptions = {}): boolean => {
  const key = `cluster_actions_active_id_${channelId || 'global'}`
  return setMemoryItem(key, actionId, { persist: true, ...options })
}

export const getActiveGroupId = (channelId: string | null, defaultValue: string | null = null): string | null => {
  const key = `cluster_actions_active_id_${channelId || 'global'}`
  return getMemoryItem(key, defaultValue)
}

// ==================== DIAP身份专用存储函数 ====================

export const setDiapIdentity = (sessionId: string, identity: DiapIdentity | string, options: MemoryStorageOptions = {}): boolean => {
  const key = `diap_identity_${sessionId}`
  
  // 验证身份数据的完整性
  let parsedIdentity: DiapIdentity
  if (typeof identity === 'string') {
    try {
      parsedIdentity = JSON.parse(identity)
    } catch (e: any) {
      console.error('[MemoryStorage] DIAP身份数据解析失败:', e)
      return false
    }
  } else {
    parsedIdentity = identity
  }
  
  // 验证必要字段
  if (!parsedIdentity || !parsedIdentity.did || !parsedIdentity.cid || !parsedIdentity.ipns) {
    console.error('[MemoryStorage] DIAP身份数据不完整:', parsedIdentity)
    return false
  }
  
  // 添加时间戳
  const identityWithTimestamp: DiapIdentity = {
    ...parsedIdentity,
    stored_at: Date.now(),
    session_id: sessionId
  }
  
  console.log('[MemoryStorage] 保存DIAP身份:', {
    sessionId,
    did: parsedIdentity.did,
    cid: parsedIdentity.cid,
    ipns: parsedIdentity.ipns
  })
  
  return setMemoryItem(key, identityWithTimestamp, { persist: true, ttl: 7 * 24 * 60 * 60 * 1000, ...options }) // 7天过期
}

export const getDiapIdentity = (sessionId: string, defaultValue: DiapIdentity | undefined = undefined): DiapIdentity | null => {
  const key = `diap_identity_${sessionId}`
  const identity = getMemoryItem<DiapIdentity>(key, defaultValue)
  
  if (identity) {
    console.log('[MemoryStorage] 获取DIAP身份成功:', {
      sessionId,
      did: identity.did,
      stored_at: identity.stored_at ? new Date(identity.stored_at).toISOString() : 'unknown'
    })
  } else {
    console.log('[MemoryStorage] DIAP身份不存在:', sessionId)
  }
  
  return identity
}

export const removeDiapIdentity = (sessionId: string): boolean => {
  const key = `diap_identity_${sessionId}`
  console.log('[MemoryStorage] 删除DIAP身份:', sessionId)
  return removeMemoryItem(key)
}

export const hasDiapIdentity = (sessionId: string): boolean => {
  const key = `diap_identity_${sessionId}`
  const exists = hasMemoryItem(key)
  console.log('[MemoryStorage] 检查DIAP身份存在性:', { sessionId, exists })
  return exists
}

export const getAllDiapIdentities = (): Record<string, DiapIdentity> => {
  const allKeys = getMemoryKeys()
  const diapKeys = allKeys.filter(key => key.startsWith('diap_identity_'))
  const identities: Record<string, DiapIdentity> = {}
  
  console.log('[MemoryStorage] 获取所有DIAP身份，找到', diapKeys.length, '个')
  
  diapKeys.forEach(key => {
    const sessionId = key.replace('diap_identity_', '')
    const identity = getDiapIdentity(sessionId)
    if (identity) {
      identities[sessionId] = identity
    }
  })
  
  return identities
}

export const cleanupExpiredDiapIdentities = (): number => {
  const allKeys = getMemoryKeys()
  const diapKeys = allKeys.filter(key => key.startsWith('diap_identity_'))
  let cleanedCount = 0
  
  diapKeys.forEach(key => {
    const identity = getMemoryItem(key)
    if (!identity) {
      cleanedCount++
    }
  })
  
  console.log('[MemoryStorage] 清理过期DIAP身份，清理了', cleanedCount, '个')
  return cleanedCount
}

// ==================== 钱包数据专用存储函数 ====================

export const setWalletData = (address: string, walletType: string, chainId: string): void => {
  setMemoryItem('wallet_address', address, { persist: true })
  setMemoryItem('wallet_type', walletType, { persist: true })
  setMemoryItem('wallet_chain_id', chainId, { persist: true })
}

export const getWalletData = (): WalletData => {
  return {
    address: getMemoryItem('wallet_address'),
    type: getMemoryItem('wallet_type'),
    chainId: getMemoryItem('wallet_chain_id')
  }
}

// ==================== 窗口扩展 ====================

declare global {
  interface Window {
    AlouMemoryStorage: {
      set: typeof setMemoryItem
      get: typeof getMemoryItem
      remove: typeof removeMemoryItem
      has: typeof hasMemoryItem
      keys: typeof getMemoryKeys
      clear: typeof clearMemoryStorage
      stats: typeof getMemoryStats
      cleanup: typeof cleanupMemoryStorage
      setGroupChat: typeof setGroupChatData
      getGroupChat: typeof getGroupChatData
      setActiveGroup: typeof setActiveGroupId
      getActiveGroup: typeof getActiveGroupId
      setWallet: typeof setWalletData
      getWallet: typeof getWalletData
      setDiapIdentity: typeof setDiapIdentity
      getDiapIdentity: typeof getDiapIdentity
      removeDiapIdentity: typeof removeDiapIdentity
      hasDiapIdentity: typeof hasDiapIdentity
      getAllDiapIdentities: typeof getAllDiapIdentities
      cleanupDiapIdentities: typeof cleanupExpiredDiapIdentities
      help: () => void
    }
  }
}

// ==================== 控制台工具 ====================

if (typeof window !== 'undefined') {
  window.AlouMemoryStorage = {
    // 基础操作
    set: setMemoryItem,
    get: getMemoryItem,
    remove: removeMemoryItem,
    has: hasMemoryItem,
    keys: getMemoryKeys,
    clear: clearMemoryStorage,
    
    // 统计信息
    stats: getMemoryStats,
    cleanup: cleanupMemoryStorage,
    
    // 专用函数
    setGroupChat: setGroupChatData,
    getGroupChat: getGroupChatData,
    setActiveGroup: setActiveGroupId,
    getActiveGroup: getActiveGroupId,
    setWallet: setWalletData,
    getWallet: getWalletData,
    setDiapIdentity: setDiapIdentity,
    getDiapIdentity: getDiapIdentity,
    removeDiapIdentity: removeDiapIdentity,
    hasDiapIdentity: hasDiapIdentity,
    getAllDiapIdentities: getAllDiapIdentities,
    cleanupDiapIdentities: cleanupExpiredDiapIdentities,
    
    // 帮助
    help: () => {
      console.group('📚 AlouMemoryStorage 使用帮助')
      console.log('基础操作:')
      console.log('  AlouMemoryStorage.set(key, value, options) - 设置数据')
      console.log('  AlouMemoryStorage.get(key, defaultValue) - 获取数据')
      console.log('  AlouMemoryStorage.remove(key) - 删除数据')
      console.log('  AlouMemoryStorage.has(key) - 检查键是否存在')
      console.log('  AlouMemoryStorage.keys() - 获取所有键')
      console.log('  AlouMemoryStorage.clear() - 清空所有数据')
      console.log('')
      console.log('统计信息:')
      console.log('  AlouMemoryStorage.stats() - 获取存储统计')
      console.log('  AlouMemoryStorage.cleanup() - 清理过期数据')
      console.log('')
      console.log('专用函数:')
      console.log('  AlouMemoryStorage.setGroupChat(channelId, data) - 设置群聊数据')
      console.log('  AlouMemoryStorage.getGroupChat(channelId) - 获取群聊数据')
      console.log('  AlouMemoryStorage.setActiveGroup(channelId, id) - 设置活跃群聊')
      console.log('  AlouMemoryStorage.getActiveGroup(channelId) - 获取活跃群聊')
      console.log('  AlouMemoryStorage.setWallet(address, type, chainId) - 设置钱包数据')
      console.log('  AlouMemoryStorage.getWallet() - 获取钱包数据')
      console.log('')
      console.log('DIAP身份管理:')
      console.log('  AlouMemoryStorage.setDiapIdentity(sessionId, identity) - 设置DIAP身份')
      console.log('  AlouMemoryStorage.getDiapIdentity(sessionId) - 获取DIAP身份')
      console.log('  AlouMemoryStorage.removeDiapIdentity(sessionId) - 删除DIAP身份')
      console.log('  AlouMemoryStorage.hasDiapIdentity(sessionId) - 检查DIAP身份是否存在')
      console.log('  AlouMemoryStorage.getAllDiapIdentities() - 获取所有DIAP身份')
      console.log('  AlouMemoryStorage.cleanupDiapIdentities() - 清理过期DIAP身份')
      console.groupEnd()
    }
  }
  
  console.log('🎉 AlouMemoryStorage 已加载!')
  console.log('💡 输入 AlouMemoryStorage.help() 查看使用帮助')
  console.log('📊 输入 AlouMemoryStorage.stats() 查看存储状态')
}

export default memoryStorage
