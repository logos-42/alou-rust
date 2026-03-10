/**
 * 智能体头像保护工具
 * 用于检测和修复头像被意外替换的问题
 */

import useAgentStore from '@/stores/agentStore'
import { resolveAgentAvatar, Agent } from '@/components/AgentChat/agentUtils'

// 保护结果接口
export interface ProtectionResults {
  checkedAgents: number
  protectedAgents: number
  fixedAgents: number
  missingAvatars: number
  corruptedAvatars: number
}

// 头像状态接口
export interface AvatarStatus {
  hasAvatar: boolean
  resolvedAvatar: string | null
  isCorrupted: boolean
  isProtected: boolean
  sources: AvatarSources
  primarySource: string | null
}

// 头像源接口
export interface AvatarSources {
  avatar_cid?: string
  avatar_url?: string
  avatar?: string
  avatarCid?: string
  diapIdentity?: string
  serviceEndpoint?: string
  didDocument?: string
  meta?: AvatarSources | null
}

// 头像备份接口
export interface AvatarBackup {
  id: string
  name: string
  avatarData: AvatarSources & {
    diapIdentity?: string
    serviceEndpoint?: string
    didDocument?: string
  }
  timestamp: number
}

// 全局窗口扩展
declare global {
  interface Window {
    AlouAvatarProtection: {
      checkAllAvatars: () => Promise<AvatarStatus[]>
      fixCorruptedAvatars: () => Promise<number>
      backupAvatarData: () => AvatarBackup[]
      restoreAvatarData: () => Promise<number>
      getStats: () => ProtectionResults & { timestamp: number }
      resetResults: () => void
      help: () => void
    }
  }
}

class AvatarProtectionTool {
  private protectionResults: ProtectionResults

  constructor() {
    this.protectionResults = {
      checkedAgents: 0,
      protectedAgents: 0,
      fixedAgents: 0,
      missingAvatars: 0,
      corruptedAvatars: 0
    }
  }

  /**
   * 检查所有智能体的头像状态
   */
  async checkAllAvatars(): Promise<AvatarStatus[]> {
    console.log('🔍 开始检查所有智能体头像状态...')
    
    try {
      const agents = useAgentStore.getState().agents
      this.protectionResults.checkedAgents = agents.length
      
      const avatarStatus = agents.map(agent => {
        const status = this.analyzeAvatarStatus(agent as Agent)
        return {
          id: agent.id,
          name: agent.display_name || agent.name,
          ...status
        }
      })
      
      console.log('📊 头像状态检查结果:', avatarStatus)
      
      // 统计问题
      this.protectionResults.missingAvatars = avatarStatus.filter(a => a.hasAvatar === false).length
      this.protectionResults.corruptedAvatars = avatarStatus.filter(a => a.isCorrupted === true).length
      this.protectionResults.protectedAgents = avatarStatus.filter(a => a.isProtected === true).length
      
      return avatarStatus
    } catch (error) {
      console.error('❌ 检查头像状态失败:', error)
      throw error
    }
  }

  /**
   * 分析单个智能体的头像状态
   */
  analyzeAvatarStatus(agent: Agent): AvatarStatus {
    const avatarSources: AvatarSources = {
      avatar_cid: agent.avatar_cid || undefined,
      avatar_url: agent.avatar_url || undefined,
      avatar: agent.avatar,
      avatarCid: agent.avatarCid || undefined,
      diapIdentity: agent.diapIdentity?.avatar_cid,
      serviceEndpoint: agent.serviceEndpoint?.avatar_cid,
      didDocument: agent.did_document?.service?.find((s: { serviceEndpoint?: { avatar_cid?: string } }) => s.serviceEndpoint?.avatar_cid)?.serviceEndpoint?.avatar_cid,
      meta: agent.meta ? this.checkMetaAvatar(agent.meta as Record<string, unknown>) : null
    }

    const hasAnyAvatar = Object.values(avatarSources).some(source => source !== null && source !== undefined)
    const resolvedAvatar = resolveAgentAvatar(agent)
    const isCorrupted = this.isAvatarCorrupted(agent)
    const isProtected = this.isAvatarProtected(agent)

    return {
      hasAvatar: hasAnyAvatar,
      resolvedAvatar,
      isCorrupted,
      isProtected,
      sources: avatarSources,
      primarySource: this.getPrimaryAvatarSource(avatarSources)
    }
  }

  /**
   * 检查meta对象中的头像
   */
  checkMetaAvatar(meta: Record<string, unknown>): AvatarSources {
    return {
      avatar_cid: meta.avatar_cid as string | undefined,
      avatar_url: meta.avatar_url as string | undefined,
      avatar: meta.avatar as string | undefined,
      diapIdentity: (meta.diapIdentity as { avatar_cid?: string })?.avatar_cid
    }
  }

  /**
   * 检查头像是否损坏
   */
  isAvatarCorrupted(agent: Agent): boolean {
    const avatarFields = ['avatar_cid', 'avatar_url', 'avatar', 'avatarCid']
    
    for (const field of avatarFields) {
      const value = agent[field as keyof Agent]
      if (value && typeof value === 'string') {
        // 检查是否是无效的URL或CID
        if (value.includes('undefined') || value.includes('null') || value.includes('[object Object]')) {
          return true
        }
      }
    }
    
    return false
  }

  /**
   * 检查头像是否受保护
   */
  isAvatarProtected(agent: Agent): boolean {
    // 检查是否有多个头像源（冗余保护）
    const sources = [
      agent.avatar_cid,
      agent.avatar_url,
      agent.avatar,
      agent.avatarCid,
      agent.diapIdentity?.avatar_cid
    ].filter(Boolean)
    
    return sources.length >= 2
  }

  /**
   * 获取主要头像源
   */
  getPrimaryAvatarSource(sources: AvatarSources): string | null {
    const priority = ['avatar_url', 'avatar', 'avatar_cid', 'avatarCid', 'diapIdentity', 'serviceEndpoint', 'didDocument']
    
    for (const source of priority) {
      if (sources[source as keyof AvatarSources]) {
        return source
      }
    }
    
    return null
  }

  /**
   * 修复损坏的头像
   */
  async fixCorruptedAvatars(): Promise<number> {
    console.log('🔧 开始修复损坏的头像...')
    
    try {
      const agents = useAgentStore.getState().agents
      const updateAgent = useAgentStore.getState().updateAgent
      let fixedCount = 0
      
      for (const agent of agents) {
        const status = this.analyzeAvatarStatus(agent as Agent)
        
        if (status.isCorrupted) {
          console.log(`[AvatarProtection] 修复智能体头像: ${agent.name || agent.id}`)
          
          // 清理损坏的头像字段
          const updates: Record<string, string | null> = {}
          const avatarFields = ['avatar_cid', 'avatar_url', 'avatar', 'avatarCid']

          for (const field of avatarFields) {
            const value = agent as unknown as Record<string, unknown>
            const fieldValue = value[field] as string | undefined
            if (fieldValue && typeof fieldValue === 'string') {
              if (fieldValue.includes('undefined') || fieldValue.includes('null') || fieldValue.includes('[object Object]')) {
                updates[field] = null
              }
            }
          }
          
          // 尝试从其他源恢复头像
          if (status.sources.diapIdentity) {
            updates.avatar_cid = status.sources.diapIdentity
          } else if (status.sources.serviceEndpoint) {
            updates.avatar_cid = status.sources.serviceEndpoint
          }
          
          if (Object.keys(updates).length > 0) {
            updateAgent(agent.id, updates)
            fixedCount++
          }
        }
      }
      
      this.protectionResults.fixedAgents = fixedCount
      console.log(`✅ 修复了 ${fixedCount} 个损坏的头像`)
      
      return fixedCount
    } catch (error) {
      console.error('❌ 修复头像失败:', error)
      throw error
    }
  }

  /**
   * 备份头像数据
   */
  backupAvatarData(): AvatarBackup[] {
    console.log('💾 开始备份头像数据...')

    try {
      const agents = useAgentStore.getState().agents
      const avatarBackup: AvatarBackup[] = agents.map(agent => ({
        id: agent.id,
        name: agent.display_name || agent.name || '',
        avatarData: {
          avatar_cid: agent.avatar_cid || undefined,
          avatar_url: agent.avatar_url || undefined,
          avatar: undefined,
          avatarCid: agent.avatar_cid || undefined,
          diapIdentity: agent.diapIdentity?.avatar_cid,
          serviceEndpoint: undefined,
          didDocument: undefined
        },
        timestamp: Date.now()
      }))
      
      // 保存到localStorage
      localStorage.setItem('alou_avatar_backup', JSON.stringify(avatarBackup))
      
      console.log(`✅ 备份了 ${avatarBackup.length} 个智能体的头像数据`)
      return avatarBackup
    } catch (error) {
      console.error('❌ 备份头像数据失败:', error)
      throw error
    }
  }

  /**
   * 恢复头像数据
   */
  async restoreAvatarData(): Promise<number> {
    console.log('🔄 开始恢复头像数据...')

    try {
      const backupData = localStorage.getItem('alou_avatar_backup')
      if (!backupData) {
        console.log('⚠️ 没有找到头像备份数据')
        return 0
      }

      const avatarBackup: AvatarBackup[] = JSON.parse(backupData)
      const updateAgent = useAgentStore.getState().updateAgent
      const resolveIpfsUrl = useAgentStore.getState().resolveIpfsUrl
      let restoredCount = 0

      for (const backup of avatarBackup) {
        const currentAgent = useAgentStore.getState().agents.find(a => a.id === backup.id)

        if (currentAgent) {
          const updates: Record<string, string | undefined> = {}

          // 只恢复缺失的头像字段
          if (!currentAgent.avatar_cid && backup.avatarData.avatar_cid) {
            updates.avatar_cid = backup.avatarData.avatar_cid
            // 同时解析为 URL
            updates.avatar_url = resolveIpfsUrl(backup.avatarData.avatar_cid) || undefined
          }
          if (!currentAgent.avatar_url && backup.avatarData.avatar_url) {
            updates.avatar_url = backup.avatarData.avatar_url
          }
          // Skip avatar field restoration as it doesn't exist in AgentMetadata
          // if (!currentAgent.avatar && backup.avatarData.avatar) {
          //   updates.avatar = backup.avatarData.avatar
          // }

          if (Object.keys(updates).length > 0) {
            updateAgent(backup.id, updates)
            restoredCount++
          }
        }
      }

      console.log(`✅ 恢复了 ${restoredCount} 个智能体的头像数据`)
      return restoredCount
    } catch (error) {
      console.error('❌ 恢复头像数据失败:', error)
      throw error
    }
  }

  /**
   * 自动检测并修复所有头像 URL（跨系统兼容）
   * 在应用启动时调用，确保所有头像 URL 都是可访问的
   */
  async autoFixAvatarUrls(): Promise<number> {
    console.log('🔧 开始自动修复头像 URL...')

    try {
      const agents = useAgentStore.getState().agents
      const updateAgent = useAgentStore.getState().updateAgent
      const resolveIpfsUrl = useAgentStore.getState().resolveIpfsUrl
      let fixedCount = 0

      for (const agent of agents) {
        const needsFix = !agent.avatar_url || 
          agent.avatar_url.includes('undefined') || 
          agent.avatar_url.includes('null') ||
          (agent.avatar_cid && agent.avatar_url?.includes('gateway.ipfs.io')) // 旧网关可能不可访问

        if (needsFix && agent.avatar_cid) {
          const newAvatarUrl = resolveIpfsUrl(agent.avatar_cid)
          if (newAvatarUrl && newAvatarUrl !== agent.avatar_url) {
            updateAgent(agent.id, {
              avatar_url: newAvatarUrl,
            })
            fixedCount++
            console.log(`[AvatarProtection] 修复智能体头像 URL: ${agent.name || agent.id}`)
          }
        }
      }

      console.log(`✅ 自动修复了 ${fixedCount} 个头像 URL`)
      return fixedCount
    } catch (error) {
      console.error('❌ 自动修复头像 URL 失败:', error)
      throw error
    }
  }

  /**
   * 获取保护统计
   */
  getProtectionStats(): ProtectionResults & { timestamp: number } {
    return {
      ...this.protectionResults,
      timestamp: Date.now()
    }
  }

  /**
   * 重置统计结果
   */
  resetResults(): void {
    this.protectionResults = {
      checkedAgents: 0,
      protectedAgents: 0,
      fixedAgents: 0,
      missingAvatars: 0,
      corruptedAvatars: 0
    }
  }
}

// 创建全局实例
const avatarProtectionTool = new AvatarProtectionTool()

// 控制台工具
if (typeof window !== 'undefined') {
  window.AlouAvatarProtection = {
    checkAllAvatars: () => avatarProtectionTool.checkAllAvatars(),
    fixCorruptedAvatars: () => avatarProtectionTool.fixCorruptedAvatars(),
    backupAvatarData: () => avatarProtectionTool.backupAvatarData(),
    restoreAvatarData: () => avatarProtectionTool.restoreAvatarData(),
    autoFixAvatarUrls: () => avatarProtectionTool.autoFixAvatarUrls(),
    getStats: () => avatarProtectionTool.getProtectionStats(),
    resetResults: () => avatarProtectionTool.resetResults(),
    help: () => {
      console.group('🛡️ 智能体头像保护工具使用帮助')
      console.log('检查功能:')
      console.log('  AlouAvatarProtection.checkAllAvatars() - 检查所有头像状态')
      console.log('')
      console.log('修复功能:')
      console.log('  AlouAvatarProtection.fixCorruptedAvatars() - 修复损坏头像')
      console.log('  AlouAvatarProtection.autoFixAvatarUrls() - 自动修复所有头像 URL（跨系统兼容）')
      console.log('')
      console.log('备份功能:')
      console.log('  AlouAvatarProtection.backupAvatarData() - 备份头像数据')
      console.log('  AlouAvatarProtection.restoreAvatarData() - 恢复头像数据')
      console.log('')
      console.log('工具:')
      console.log('  AlouAvatarProtection.getStats() - 获取保护统计')
      console.log('  AlouAvatarProtection.resetResults() - 重置统计结果')
      console.groupEnd()
    }
  }

  console.log('🛡️ AlouAvatarProtection 头像保护工具已加载!')
  console.log('💡 输入 AlouAvatarProtection.help() 查看使用帮助')
  console.log('🔍 输入 AlouAvatarProtection.checkAllAvatars() 检查头像状态')
  console.log('🔧 输入 AlouAvatarProtection.autoFixAvatarUrls() 自动修复头像 URL')
}

export default avatarProtectionTool
