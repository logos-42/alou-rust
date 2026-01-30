/**
 * DIAP身份管理器
 * 提供统一的DIAP身份存储、检索和管理功能
 */

import { 
  setDiapIdentity, 
  getDiapIdentity, 
  removeDiapIdentity, 
  hasDiapIdentity,
  getAllDiapIdentities,
  cleanupExpiredDiapIdentities
} from '@/utils/memoryStorage'

import { invoke } from '@tauri-apps/api/core'

// DIAP身份接口 (与 memoryStorage.ts 保持一致)
export interface DiapIdentity {
  did: string
  ipns: string
  cid: string
  stored_at?: number
  updated_at?: number
  session_id?: string
  [key: string]: any
}

// 存储选项接口
export interface IdentityOptions {
  ttl?: number
  persist?: boolean
  priority?: 'high' | 'normal' | 'low'
}

// 缓存条目接口
interface CacheEntry {
  data: DiapIdentity
  timestamp: number
}

// 统计信息接口
export interface IdentityStats {
  totalIdentities: number
  cachedIdentities: number
  identities: Array<{
    sessionId: string
    hasIpns: boolean
    hasCid: boolean
    hasDid: boolean
    storedAt: number
    updatedAt: number
  }>
}

// 全局窗口扩展
declare global {
  interface Window {
    AlouDiapIdentityManager: {
      setIdentity: (sessionId: string, identity: DiapIdentity, options?: IdentityOptions) => Promise<boolean>
      getIdentity: (sessionId: string, defaultValue?: DiapIdentity | null) => Promise<DiapIdentity | null>
      removeIdentity: (sessionId: string) => Promise<boolean>
      hasIdentity: (sessionId: string) => Promise<boolean>
      getAllIdentities: () => Promise<Record<string, DiapIdentity>>
      cleanupExpired: () => Promise<number>
      getStats: () => Promise<IdentityStats>
      clearCache: () => void
      help: () => void
    }
  }
}

class DiapIdentityManager {
  private cache: Map<string, CacheEntry>
  private cacheTimeout: number

  constructor() {
    this.cache = new Map()
    this.cacheTimeout = 5 * 60 * 1000 // 5分钟缓存过期
  }

  /**
   * 设置DIAP身份
   * @param sessionId - 会话ID
   * @param identity - DIAP身份对象
   * @param options - 选项
   */
  async setIdentity(sessionId: string, identity: DiapIdentity, options: IdentityOptions = {}): Promise<boolean> {
    try {
      // 验证身份对象
      if (!identity || typeof identity !== 'object') {
        throw new Error('Invalid identity object')
      }

      // 添加时间戳
      const identityWithTimestamp: DiapIdentity = {
        ...identity,
        stored_at: Date.now(),
        updated_at: Date.now()
      }

      // 存储到统一内存存储
      const success = setDiapIdentity(sessionId, identityWithTimestamp, options)
      
      if (success) {
        // 更新内存缓存
        this.cache.set(sessionId, {
          data: identityWithTimestamp,
          timestamp: Date.now()
        })

        // 同时存储到Tauri后端（如果可用）
        try {
          await invoke('set_diap_identity', {
            sessionId,
            identity: JSON.stringify(identityWithTimestamp)
          })
        } catch (tauriError) {
          console.warn('[DiapIdentityManager] Tauri存储失败，使用前端存储:', (tauriError as Error).message)
        }

        console.log(`[DiapIdentityManager] DIAP身份已存储: ${sessionId}`)
        return true
      }
      
      return false
    } catch (error) {
      console.error('[DiapIdentityManager] 存储DIAP身份失败:', error)
      throw error
    }
  }

  /**
   * 获取DIAP身份
   * @param sessionId - 会话ID
   * @param defaultValue - 默认值
   */
  async getIdentity(sessionId: string, defaultValue: DiapIdentity | null = null): Promise<DiapIdentity | null> {
    try {
      // 首先检查内存缓存
      const cached = this.cache.get(sessionId)
      if (cached && Date.now() - cached.timestamp < this.cacheTimeout) {
        console.log(`[DiapIdentityManager] 从缓存获取DIAP身份: ${sessionId}`)
        return cached.data
      }

      // 从统一内存存储获取
      let identity: DiapIdentity | null = getDiapIdentity(sessionId, defaultValue ?? undefined)

      // 如果前端没有，尝试从Tauri后端获取
      if (!identity && defaultValue === null) {
        try {
          const tauriIdentity = await invoke<string>('get_diap_identity', { sessionId })
          if (tauriIdentity) {
            identity = JSON.parse(tauriIdentity) as DiapIdentity
            // 同步到前端存储
            if (identity && identity.did && identity.ipns && identity.cid) {
              setDiapIdentity(sessionId, identity)
            }
          }
        } catch (tauriError) {
          console.warn('[DiapIdentityManager] Tauri获取失败:', (tauriError as Error).message)
        }
      }

      // 更新缓存
      if (identity) {
        this.cache.set(sessionId, {
          data: identity,
          timestamp: Date.now()
        })
      }

      return identity
    } catch (error) {
      console.error('[DiapIdentityManager] 获取DIAP身份失败:', error)
      return defaultValue
    }
  }

  /**
   * 删除DIAP身份
   * @param sessionId - 会话ID
   */
  async removeIdentity(sessionId: string): Promise<boolean> {
    try {
      // 从统一内存存储删除
      const frontendRemoved = removeDiapIdentity(sessionId)
      
      // 从Tauri后端删除
      let tauriRemoved = false
      try {
        tauriRemoved = await invoke<boolean>('remove_diap_identity', { sessionId })
      } catch (tauriError) {
        console.warn('[DiapIdentityManager] Tauri删除失败:', (tauriError as Error).message)
      }

      // 清除缓存
      this.cache.delete(sessionId)

      const success = frontendRemoved || tauriRemoved
      if (success) {
        console.log(`[DiapIdentityManager] DIAP身份已删除: ${sessionId}`)
      }

      return success
    } catch (error) {
      console.error('[DiapIdentityManager] 删除DIAP身份失败:', error)
      return false
    }
  }

  /**
   * 检查DIAP身份是否存在
   * @param sessionId - 会话ID
   */
  async hasIdentity(sessionId: string): Promise<boolean> {
    try {
      // 检查缓存
      if (this.cache.has(sessionId)) {
        return true
      }

      // 检查前端存储
      if (hasDiapIdentity(sessionId)) {
        return true
      }

      // 检查Tauri后端
      try {
        const tauriIdentity = await invoke<string>('get_diap_identity', { sessionId })
        return !!tauriIdentity
      } catch {
        return false
      }
    } catch (error) {
      console.error('[DiapIdentityManager] 检查DIAP身份失败:', error)
      return false
    }
  }

  /**
   * 获取所有DIAP身份
   */
  async getAllIdentities(): Promise<Record<string, DiapIdentity>> {
    try {
      // 从前端存储获取
      const frontendIdentities = getAllDiapIdentities()

      // 从Tauri后端获取
      let tauriIdentities: Record<string, string> = {}
      try {
        tauriIdentities = await invoke<Record<string, string>>('get_all_diap_identities')
        // 解析JSON字符串
        const parsedIdentities: Record<string, DiapIdentity> = {}
        for (const [sessionId, identityStr] of Object.entries(tauriIdentities)) {
          try {
            parsedIdentities[sessionId] = JSON.parse(identityStr) as DiapIdentity
          } catch (parseError) {
            console.warn(`[DiapIdentityManager] 解析Tauri身份失败: ${sessionId}`, (parseError as Error).message)
          }
        }
        tauriIdentities = parsedIdentities as unknown as Record<string, string>
      } catch (tauriError) {
        console.warn('[DiapIdentityManager] Tauri获取所有身份失败:', (tauriError as Error).message)
      }

      // 合并结果（优先使用前端数据）
      const allIdentities: Record<string, DiapIdentity> = { ...frontendIdentities }
      for (const [sessionId, identity] of Object.entries(tauriIdentities)) {
        if (!allIdentities[sessionId]) {
          allIdentities[sessionId] = identity as unknown as DiapIdentity
        }
      }

      return allIdentities
    } catch (error) {
      console.error('[DiapIdentityManager] 获取所有DIAP身份失败:', error)
      return {}
    }
  }

  /**
   * 清理过期的DIAP身份
   */
  async cleanupExpired(): Promise<number> {
    try {
      // 清理前端过期数据
      const frontendCleaned = cleanupExpiredDiapIdentities()

      // 清理Tauri过期数据
      let tauriCleaned = 0
      try {
        tauriCleaned = await invoke<number>('cleanup_expired_diap_identities')
      } catch (tauriError) {
        console.warn('[DiapIdentityManager] Tauri清理失败:', (tauriError as Error).message)
      }

      // 清理缓存
      this.cache.clear()

      const totalCleaned = frontendCleaned + tauriCleaned
      if (totalCleaned > 0) {
        console.log(`[DiapIdentityManager] 清理了 ${totalCleaned} 个过期DIAP身份`)
      }

      return totalCleaned
    } catch (error) {
      console.error('[DiapIdentityManager] 清理过期DIAP身份失败:', error)
      return 0
    }
  }

  /**
   * 清除缓存
   */
  clearCache(): void {
    this.cache.clear()
    console.log('[DiapIdentityManager] 缓存已清除')
  }

  /**
   * 获取统计信息
   */
  async getStats(): Promise<IdentityStats> {
    try {
      const allIdentities = await this.getAllIdentities()
      const stats: IdentityStats = {
        totalIdentities: Object.keys(allIdentities).length,
        cachedIdentities: this.cache.size,
        identities: Object.keys(allIdentities).map(sessionId => ({
          sessionId,
          hasIpns: !!allIdentities[sessionId].ipns,
          hasCid: !!allIdentities[sessionId].cid,
          hasDid: !!allIdentities[sessionId].did,
          storedAt: allIdentities[sessionId].stored_at || 0,
          updatedAt: allIdentities[sessionId].updated_at || 0
        }))
      }

      return stats
    } catch (error) {
      console.error('[DiapIdentityManager] 获取统计信息失败:', error)
      return {
        totalIdentities: 0,
        cachedIdentities: 0,
        identities: []
      }
    }
  }
}

// 创建全局实例
const diapIdentityManager = new DiapIdentityManager()

// 导出便捷函数
export const setDiapIdentitySafe = (sessionId: string, identity: DiapIdentity, options?: IdentityOptions): Promise<boolean> => 
  diapIdentityManager.setIdentity(sessionId, identity, options)

export const getDiapIdentitySafe = (sessionId: string, defaultValue?: DiapIdentity | null): Promise<DiapIdentity | null> => 
  diapIdentityManager.getIdentity(sessionId, defaultValue)

export const removeDiapIdentitySafe = (sessionId: string): Promise<boolean> => 
  diapIdentityManager.removeIdentity(sessionId)

export const hasDiapIdentitySafe = (sessionId: string): Promise<boolean> => 
  diapIdentityManager.hasIdentity(sessionId)

export const getAllDiapIdentitiesSafe = (): Promise<Record<string, DiapIdentity>> => 
  diapIdentityManager.getAllIdentities()

export const cleanupExpiredDiapIdentitiesSafe = (): Promise<number> => 
  diapIdentityManager.cleanupExpired()

export const getDiapIdentityStats = (): Promise<IdentityStats> => 
  diapIdentityManager.getStats()

export const clearDiapIdentityCache = (): void => 
  diapIdentityManager.clearCache()

// 导出管理器实例
export default diapIdentityManager

// 控制台工具
if (typeof window !== 'undefined') {
  window.AlouDiapIdentityManager = {
    setIdentity: setDiapIdentitySafe,
    getIdentity: getDiapIdentitySafe,
    removeIdentity: removeDiapIdentitySafe,
    hasIdentity: hasDiapIdentitySafe,
    getAllIdentities: getAllDiapIdentitiesSafe,
    cleanupExpired: cleanupExpiredDiapIdentitiesSafe,
    getStats: getDiapIdentityStats,
    clearCache: clearDiapIdentityCache,
    help: () => {
      console.group('🔐 DIAP身份管理器使用帮助')
      console.log('基础操作:')
      console.log('  AlouDiapIdentityManager.setIdentity(sessionId, identity) - 设置DIAP身份')
      console.log('  AlouDiapIdentityManager.getIdentity(sessionId, defaultValue) - 获取DIAP身份')
      console.log('  AlouDiapIdentityManager.removeIdentity(sessionId) - 删除DIAP身份')
      console.log('  AlouDiapIdentityManager.hasIdentity(sessionId) - 检查身份是否存在')
      console.log('')
      console.log('批量操作:')
      console.log('  AlouDiapIdentityManager.getAllIdentities() - 获取所有DIAP身份')
      console.log('  AlouDiapIdentityManager.cleanupExpired() - 清理过期身份')
      console.log('')
      console.log('管理工具:')
      console.log('  AlouDiapIdentityManager.getStats() - 获取统计信息')
      console.log('  AlouDiapIdentityManager.clearCache() - 清除缓存')
      console.groupEnd()
    }
  }
  
  console.log('🔐 AlouDiapIdentityManager 已加载!')
  console.log('💡 输入 AlouDiapIdentityManager.help() 查看使用帮助')
  console.log('📊 输入 AlouDiapIdentityManager.getStats() 查看身份统计')
}
