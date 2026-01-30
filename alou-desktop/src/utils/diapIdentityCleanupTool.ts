/**
 * DIAP身份数据清理工具
 * 用于清理重复和损坏的DIAP身份数据
 */

import { 
  getAllDiapIdentitiesSafe, 
  cleanupExpiredDiapIdentitiesSafe,
  getDiapIdentityStats,
  clearDiapIdentityCache,
  setDiapIdentitySafe,
  getDiapIdentitySafe,
  DiapIdentity,
  IdentityStats
} from '@/utils/diapIdentityManager'

import { getMemoryKeys, removeMemoryItem, getMemoryItem } from '@/utils/memoryStorage'

// 清理结果接口
export interface CleanupResults {
  duplicatesRemoved: number
  expiredRemoved: number
  corruptedRemoved: number
  localStorageMigrated: number
  agentStoreFixed: number
}

// 全局窗口扩展
declare global {
  interface Window {
    AlouDiapCleanup: {
      fullCleanup: () => Promise<CleanupResults>
      cleanupExpired: () => Promise<void>
      cleanupDuplicates: () => Promise<void>
      cleanupCorrupted: () => Promise<void>
      migrateLocalStorage: () => Promise<void>
      fixAgentStore: () => Promise<void>
      getStats: () => Promise<(IdentityStats & { cleanupResults: CleanupResults }) | null>
      resetResults: () => void
      help: () => void
    }
    useAgentStore?: {
      getState?: () => {
        agents: Array<{ id: string; diapIdentity?: DiapIdentity; ipns?: string; cid?: string; did?: string }>
        updateAgent: (id: string, updates: Record<string, unknown>) => void
      }
    }
  }
}

class DiapIdentityCleanupTool {
  private cleanupResults: CleanupResults

  constructor() {
    this.cleanupResults = {
      duplicatesRemoved: 0,
      expiredRemoved: 0,
      corruptedRemoved: 0,
      localStorageMigrated: 0,
      agentStoreFixed: 0
    }
  }

  /**
   * 执行完整的DIAP身份数据清理
   */
  async performFullCleanup(): Promise<CleanupResults> {
    console.log('🧹 开始DIAP身份数据完整清理...')
    
    try {
      // 1. 清理过期数据
      await this.cleanupExpiredData()
      
      // 2. 清理重复数据
      await this.cleanupDuplicateData()
      
      // 3. 清理损坏数据
      await this.cleanupCorruptedData()
      
      // 4. 迁移localStorage数据
      await this.migrateLocalStorageData()
      
      // 5. 修复agentStore数据
      await this.fixAgentStoreData()
      
      // 6. 清理缓存
      clearDiapIdentityCache()
      
      console.log('✅ DIAP身份数据清理完成:', this.cleanupResults)
      return this.cleanupResults
      
    } catch (error) {
      console.error('❌ DIAP身份数据清理失败:', error)
      throw error
    }
  }

  /**
   * 清理过期数据
   */
  async cleanupExpiredData(): Promise<void> {
    console.log('🕐 清理过期DIAP身份数据...')
    const expiredCount = await cleanupExpiredDiapIdentitiesSafe()
    this.cleanupResults.expiredRemoved = expiredCount
    console.log(`✅ 清理了 ${expiredCount} 个过期DIAP身份`)
  }

  /**
   * 清理重复数据
   */
  async cleanupDuplicateData(): Promise<void> {
    console.log('🔄 清理重复DIAP身份数据...')
    
    const allIdentities = await getAllDiapIdentitiesSafe()
    const seenIdentities = new Map<string, string>()
    const duplicatesToRemove: string[] = []
    
    for (const [sessionId, identity] of Object.entries(allIdentities)) {
      // 使用did、cid、ipns作为唯一标识
      const uniqueKey = `${identity.did || ''}-${identity.cid || ''}-${identity.ipns || ''}`
      
      if (seenIdentities.has(uniqueKey)) {
        // 发现重复，保留最新的，删除旧的
        const existingSessionId = seenIdentities.get(uniqueKey)!
        const existingTimestamp = allIdentities[existingSessionId]?.stored_at || 0
        const currentTimestamp = identity.stored_at || 0
        
        if (currentTimestamp > existingTimestamp) {
          // 当前数据更新，删除旧的
          duplicatesToRemove.push(existingSessionId)
          seenIdentities.set(uniqueKey, sessionId)
        } else {
          // 现有数据更新，删除当前的
          duplicatesToRemove.push(sessionId)
        }
      } else {
        seenIdentities.set(uniqueKey, sessionId)
      }
    }
    
    // 删除重复数据
    for (const sessionId of duplicatesToRemove) {
      removeMemoryItem(`diap_identity_${sessionId}`)
      this.cleanupResults.duplicatesRemoved++
    }
    
    console.log(`✅ 清理了 ${duplicatesToRemove.length} 个重复DIAP身份`)
  }

  /**
   * 清理损坏数据
   */
  async cleanupCorruptedData(): Promise<void> {
    console.log('🔧 清理损坏DIAP身份数据...')
    
    const allKeys = getMemoryKeys()
    const diapKeys = allKeys.filter(key => key.startsWith('diap_identity_'))
    const corruptedKeys: string[] = []
    
    for (const key of diapKeys) {
      try {
        const identityData = getMemoryItem(key)
        
        if (!identityData) {
          corruptedKeys.push(key)
          continue
        }
        
        // 尝试解析JSON数据
        const identity: DiapIdentity = JSON.parse(identityData)
        
        // 检查必要字段
        if (!identity || typeof identity !== 'object') {
          corruptedKeys.push(key)
          continue
        }
        
        // 检查是否有基本标识符
        if (!identity.did && !identity.cid && !identity.ipns) {
          corruptedKeys.push(key)
          continue
        }
        
        // 检查JSON格式
        JSON.stringify(identity)
        
      } catch {
        corruptedKeys.push(key)
      }
    }
    
    // 删除损坏数据
    for (const key of corruptedKeys) {
      removeMemoryItem(key)
      this.cleanupResults.corruptedRemoved++
    }
    
    console.log(`✅ 清理了 ${corruptedKeys.length} 个损坏DIAP身份`)
  }

  /**
   * 迁移localStorage数据
   */
  async migrateLocalStorageData(): Promise<void> {
    console.log('📦 迁移localStorage数据到统一存储...')
    
    if (typeof window === 'undefined' || !window.localStorage) {
      console.log('⚠️ localStorage不可用，跳过迁移')
      return
    }
    
    const migratedCount: string[] = []
    const keysToRemove: string[] = []
    
    // 查找所有localStorage中的DIAP身份
    for (let i = 0; i < window.localStorage.length; i++) {
      const key = window.localStorage.key(i)
      if (key && key.startsWith('diap_identity_')) {
        try {
          const sessionId = key.replace('diap_identity_', '')
          const identityData = window.localStorage.getItem(key)
          
          if (identityData) {
            const identity: DiapIdentity = JSON.parse(identityData)
            
            // 检查统一存储中是否已存在
            const existingIdentity = await getDiapIdentitySafe(sessionId)
            if (!existingIdentity) {
              // 迁移到统一存储
              await setDiapIdentitySafe(sessionId, identity)
              migratedCount.push(sessionId)
            }
            
            keysToRemove.push(key)
          }
        } catch (error) {
          console.warn(`迁移localStorage数据失败 ${key}:`, error)
          keysToRemove.push(key) // 删除损坏的数据
        }
      }
    }
    
    // 清理localStorage
    for (const key of keysToRemove) {
      window.localStorage.removeItem(key)
    }
    
    this.cleanupResults.localStorageMigrated = migratedCount.length
    console.log(`✅ 迁移了 ${migratedCount.length} 个localStorage DIAP身份`)
  }

  /**
   * 修复agentStore数据
   */
  async fixAgentStoreData(): Promise<void> {
    console.log('🔧 修复agentStore中的DIAP身份引用...')
    
    try {
      // 获取agentStore实例
      const agentStore = window.useAgentStore?.getState?.()
      if (!agentStore) {
        console.log('⚠️ agentStore不可用，跳过修复')
        return
      }
      
      const agents = agentStore.agents || []
      const fixedCount: string[] = []
      
      for (const agent of agents) {
        let needsUpdate = false
        const updates: Record<string, unknown> = {}
        
        // 如果agent中有完整的diapIdentity，清理它
        if (agent.diapIdentity && typeof agent.diapIdentity === 'object') {
          // 提取引用信息
          if (agent.diapIdentity.ipns && !agent.ipns) {
            updates.ipns = agent.diapIdentity.ipns
            needsUpdate = true
          }
          if (agent.diapIdentity.cid && !agent.cid) {
            updates.cid = agent.diapIdentity.cid
            needsUpdate = true
          }
          if (agent.diapIdentity.did && !agent.did) {
            updates.did = agent.diapIdentity.did
            needsUpdate = true
          }
          
          // 清理完整的diapIdentity
          updates.diapIdentity = null
          needsUpdate = true
        }
        
        if (needsUpdate) {
          agentStore.updateAgent(agent.id, updates)
          fixedCount.push(agent.id)
        }
      }
      
      this.cleanupResults.agentStoreFixed = fixedCount.length
      console.log(`✅ 修复了 ${fixedCount.length} 个agentStore中的DIAP身份引用`)
      
    } catch (error) {
      console.warn('修复agentStore数据失败:', error)
    }
  }

  /**
   * 获取清理统计
   */
  async getCleanupStats(): Promise<(IdentityStats & { cleanupResults: CleanupResults }) | null> {
    try {
      const stats = await getDiapIdentityStats()
      return {
        ...stats,
        cleanupResults: this.cleanupResults
      }
    } catch (error) {
      console.error('获取清理统计失败:', error)
      return null
    }
  }

  /**
   * 重置清理结果
   */
  resetResults(): void {
    this.cleanupResults = {
      duplicatesRemoved: 0,
      expiredRemoved: 0,
      corruptedRemoved: 0,
      localStorageMigrated: 0,
      agentStoreFixed: 0
    }
  }
}

// 创建全局实例
const diapCleanupTool = new DiapIdentityCleanupTool()

// 控制台工具
if (typeof window !== 'undefined') {
  window.AlouDiapCleanup = {
    fullCleanup: () => diapCleanupTool.performFullCleanup(),
    cleanupExpired: () => diapCleanupTool.cleanupExpiredData(),
    cleanupDuplicates: () => diapCleanupTool.cleanupDuplicateData(),
    cleanupCorrupted: () => diapCleanupTool.cleanupCorruptedData(),
    migrateLocalStorage: () => diapCleanupTool.migrateLocalStorageData(),
    fixAgentStore: () => diapCleanupTool.fixAgentStoreData(),
    getStats: () => diapCleanupTool.getCleanupStats(),
    resetResults: () => diapCleanupTool.resetResults(),
    help: () => {
      console.group('🧹 DIAP身份清理工具使用帮助')
      console.log('完整清理:')
      console.log('  AlouDiapCleanup.fullCleanup() - 执行完整清理')
      console.log('')
      console.log('单项清理:')
      console.log('  AlouDiapCleanup.cleanupExpired() - 清理过期数据')
      console.log('  AlouDiapCleanup.cleanupDuplicates() - 清理重复数据')
      console.log('  AlouDiapCleanup.cleanupCorrupted() - 清理损坏数据')
      console.log('  AlouDiapCleanup.migrateLocalStorage() - 迁移localStorage数据')
      console.log('  AlouDiapCleanup.fixAgentStore() - 修复agentStore数据')
      console.log('')
      console.log('工具:')
      console.log('  AlouDiapCleanup.getStats() - 获取清理统计')
      console.log('  AlouDiapCleanup.resetResults() - 重置清理结果')
      console.groupEnd()
    }
  }
  
  console.log('🧹 AlouDiapCleanup 清理工具已加载!')
  console.log('💡 输入 AlouDiapCleanup.help() 查看使用帮助')
  console.log('🔧 输入 AlouDiapCleanup.fullCleanup() 执行完整清理')
}

export default diapCleanupTool
