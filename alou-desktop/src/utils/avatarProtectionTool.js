/**
 * 智能体头像保护工具
 * 用于检测和修复头像被意外替换的问题
 */

import useAgentStore from '@/stores/agentStore'
import { resolveAgentAvatar } from '@/components/AgentChat/agentUtils'

class AvatarProtectionTool {
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
  async checkAllAvatars() {
    console.log('🔍 开始检查所有智能体头像状态...')
    
    try {
      const agents = useAgentStore.getState().agents
      this.protectionResults.checkedAgents = agents.length
      
      const avatarStatus = agents.map(agent => {
        const status = this.analyzeAvatarStatus(agent)
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
  analyzeAvatarStatus(agent) {
    const avatarSources = {
      avatar_cid: agent.avatar_cid,
      avatar_url: agent.avatar_url,
      avatar: agent.avatar,
      avatarCid: agent.avatarCid,
      diapIdentity: agent.diapIdentity?.avatar_cid,
      serviceEndpoint: agent.serviceEndpoint?.avatar_cid,
      didDocument: agent.did_document?.service?.find(s => s.serviceEndpoint?.avatar_cid)?.serviceEndpoint?.avatar_cid,
      meta: agent.meta ? this.checkMetaAvatar(agent.meta) : null
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
  checkMetaAvatar(meta) {
    return {
      avatar_cid: meta.avatar_cid,
      avatar_url: meta.avatar_url,
      avatar: meta.avatar,
      diapIdentity: meta.diapIdentity?.avatar_cid
    }
  }

  /**
   * 检查头像是否损坏
   */
  isAvatarCorrupted(agent) {
    const avatarFields = ['avatar_cid', 'avatar_url', 'avatar', 'avatarCid']
    
    for (const field of avatarFields) {
      const value = agent[field]
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
  isAvatarProtected(agent) {
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
  getPrimaryAvatarSource(sources) {
    const priority = ['avatar_url', 'avatar', 'avatar_cid', 'avatarCid', 'diapIdentity', 'serviceEndpoint', 'didDocument']
    
    for (const source of priority) {
      if (sources[source]) {
        return source
      }
    }
    
    return null
  }

  /**
   * 修复损坏的头像
   */
  async fixCorruptedAvatars() {
    console.log('🔧 开始修复损坏的头像...')
    
    try {
      const agents = useAgentStore.getState().agents
      const updateAgent = useAgentStore.getState().updateAgent
      let fixedCount = 0
      
      for (const agent of agents) {
        const status = this.analyzeAvatarStatus(agent)
        
        if (status.isCorrupted) {
          console.log(`[AvatarProtection] 修复智能体头像: ${agent.name || agent.id}`)
          
          // 清理损坏的头像字段
          const updates = {}
          const avatarFields = ['avatar_cid', 'avatar_url', 'avatar', 'avatarCid']
          
          for (const field of avatarFields) {
            const value = agent[field]
            if (value && typeof value === 'string') {
              if (value.includes('undefined') || value.includes('null') || value.includes('[object Object]')) {
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
  backupAvatarData() {
    console.log('💾 开始备份头像数据...')
    
    try {
      const agents = useAgentStore.getState().agents
      const avatarBackup = agents.map(agent => ({
        id: agent.id,
        name: agent.display_name || agent.name,
        avatarData: {
          avatar_cid: agent.avatar_cid,
          avatar_url: agent.avatar_url,
          avatar: agent.avatar,
          avatarCid: agent.avatarCid,
          diapIdentity: agent.diapIdentity?.avatar_cid,
          serviceEndpoint: agent.serviceEndpoint?.avatar_cid,
          didDocument: agent.did_document?.service?.find(s => s.serviceEndpoint?.avatar_cid)?.serviceEndpoint?.avatar_cid
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
  async restoreAvatarData() {
    console.log('🔄 开始恢复头像数据...')
    
    try {
      const backupData = localStorage.getItem('alou_avatar_backup')
      if (!backupData) {
        console.log('⚠️ 没有找到头像备份数据')
        return 0
      }
      
      const avatarBackup = JSON.parse(backupData)
      const updateAgent = useAgentStore.getState().updateAgent
      let restoredCount = 0
      
      for (const backup of avatarBackup) {
        const currentAgent = useAgentStore.getState().agents.find(a => a.id === backup.id)
        
        if (currentAgent) {
          const updates = {}
          
          // 只恢复缺失的头像字段
          if (!currentAgent.avatar_cid && backup.avatarData.avatar_cid) {
            updates.avatar_cid = backup.avatarData.avatar_cid
          }
          if (!currentAgent.avatar_url && backup.avatarData.avatar_url) {
            updates.avatar_url = backup.avatarData.avatar_url
          }
          if (!currentAgent.avatar && backup.avatarData.avatar) {
            updates.avatar = backup.avatar_data.avatar
          }
          
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
   * 获取保护统计
   */
  getProtectionStats() {
    return {
      ...this.protectionResults,
      timestamp: Date.now()
    }
  }

  /**
   * 重置统计结果
   */
  resetResults() {
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
    getStats: () => avatarProtectionTool.getProtectionStats(),
    resetResults: () => avatarProtectionTool.resetResults(),
    help: () => {
      console.group('🛡️ 智能体头像保护工具使用帮助')
      console.log('检查功能:')
      console.log('  AlouAvatarProtection.checkAllAvatars() - 检查所有头像状态')
      console.log('')
      console.log('修复功能:')
      console.log('  AlouAvatarProtection.fixCorruptedAvatars() - 修复损坏头像')
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
}

export default avatarProtectionTool
