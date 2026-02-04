/**
 * 智能体创建诊断工具
 * 用于诊断DIAP身份创建失败和头像同化问题
 */

import { getDiapIdentitySafe, hasDiapIdentitySafe } from '@/utils/diapIdentityManager'

// Suppress unused import warning
// type DiapIdentity = import('@/utils/diapIdentityManager').DiapIdentity
import useAgentStore from '@/stores/agentStore'
import { resolveAgentAvatar, Agent } from '@/components/AgentChat/agentUtils'

// 诊断结果接口
export interface DiagnosticResults {
  diapIdentityCreated: boolean
  diapIdentityStored: boolean
  avatarCorrect: boolean
  avatarSources: Record<string, unknown>
  errors: string[]
}

// 诊断报告接口
export interface DiagnosticReport {
  sessionId: string
  timestamp: number
  status: 'success' | 'error'
  results: DiagnosticResults
  recommendations: Recommendation[]
}

// 修复建议接口
export interface Recommendation {
  issue: string
  solution: string
  action: string
}

// 全局窗口扩展
declare global {
  interface Window {
    AlouAgentDiagnostic: {
      diagnoseAgentCreation: (sessionId: string) => Promise<DiagnosticReport>
      checkDiapIdentity: (sessionId: string) => Promise<void>
      checkAvatar: (sessionId: string) => Promise<void>
      getRecommendations: () => Recommendation[]
      resetResults: () => void
      help: () => void
    }
  }
}

class AgentCreationDiagnosticTool {
  private diagnosticResults: DiagnosticResults

  constructor() {
    this.diagnosticResults = {
      diapIdentityCreated: false,
      diapIdentityStored: false,
      avatarCorrect: false,
      avatarSources: {},
      errors: []
    }
  }

  /**
   * 诊断智能体创建过程中的问题
   */
  async diagnoseAgentCreation(sessionId: string): Promise<DiagnosticReport> {
    console.log('🔍 开始诊断智能体创建过程...')
    
    try {
      // 1. 检查DIAP身份创建
      await this.checkDiapIdentityCreation(sessionId)
      
      // 2. 检查头像状态
      await this.checkAvatarStatus(sessionId)
      
      // 3. 检查数据存储状态
      await this.checkDataStorage(sessionId)
      
      // 4. 生成诊断报告
      const report = this.generateDiagnosticReport(sessionId)
      
      console.log('📊 诊断报告:', report)
      return report
      
    } catch (error) {
      console.error('❌ 诊断失败:', error)
      this.diagnosticResults.errors.push((error as Error).message)
      return this.generateDiagnosticReport(sessionId)
    }
  }

  /**
   * 检查DIAP身份创建状态
   */
  async checkDiapIdentityCreation(sessionId: string): Promise<void> {
    console.log('🔍 检查DIAP身份创建状态...')
    
    try {
      // 检查统一存储中是否有DIAP身份
      const hasIdentity = await hasDiapIdentitySafe(sessionId)
      this.diagnosticResults.diapIdentityStored = hasIdentity
      
      if (hasIdentity) {
        const identity = await getDiapIdentitySafe(sessionId)
        console.log('✅ DIAP身份已创建并存储:', {
          sessionId,
          did: identity?.did,
          ipns: identity?.ipns,
          cid: identity?.cid,
          publicKey: identity?.public_key ? '***' : null
        })
        this.diagnosticResults.diapIdentityCreated = true
      } else {
        console.log('❌ DIAP身份未创建或未存储')
        this.diagnosticResults.errors.push('DIAP身份未创建或未存储')
      }
    } catch (error) {
      console.error('❌ 检查DIAP身份失败:', error)
      this.diagnosticResults.errors.push(`DIAP身份检查失败: ${(error as Error).message}`)
    }
  }

  /**
   * 检查头像状态
   */
  async checkAvatarStatus(sessionId: string): Promise<void> {
    console.log('🔍 检查头像状态...')

    try {
      // 获取智能体信息
      const agents = useAgentStore.getState().agents
      const agent = agents.find((a) => a.sessionId === sessionId)

      if (!agent) {
        console.log('❌ 未找到对应的智能体')
        this.diagnosticResults.errors.push('未找到对应的智能体')
        return
      }

      // 分析头像源（使用类型断言来处理AgentMetadata）
      const agentData = agent as Agent
      this.diagnosticResults.avatarSources = {
        avatar: agentData.avatar || undefined,
        avatar_url: agentData.avatar_url || undefined,
        avatar_cid: agentData.avatar_cid,
        avatarCid: agentData.avatarCid || undefined,
        diapIdentity: agentData.diapIdentity?.avatar_cid,
        serviceEndpoint: agentData.serviceEndpoint?.avatar_cid,
        didDocument: agentData.did_document?.service?.find((s: { serviceEndpoint?: { avatar_cid?: string } }) => s.serviceEndpoint?.avatar_cid)?.serviceEndpoint?.avatar_cid
      }

      // 解析头像
      const resolvedAvatar = resolveAgentAvatar(agentData)
      console.log('📊 头像解析结果:', {
        resolvedAvatar,
        sources: this.diagnosticResults.avatarSources,
        isFallback: resolvedAvatar === this.getFallbackAvatar()
      })

      // 检查头像是否正确
      this.diagnosticResults.avatarCorrect = resolvedAvatar !== this.getFallbackAvatar()
      
      if (!this.diagnosticResults.avatarCorrect) {
        console.log('⚠️ 头像使用默认值，可能存在问题')
        this.diagnosticResults.errors.push('头像使用默认值，可能存在同化问题')
      } else {
        console.log('✅ 头像解析正确')
      }
      
    } catch (error) {
      console.error('❌ 检查头像状态失败:', error)
      this.diagnosticResults.errors.push(`头像状态检查失败: ${(error as Error).message}`)
    }
  }

  /**
   * 检查数据存储状态
   */
  async checkDataStorage(sessionId: string): Promise<void> {
    console.log('🔍 检查数据存储状态...')

    try {
      // 检查agentStore中的数据
      const agents = useAgentStore.getState().agents
      const agent = agents.find((a) => a.sessionId === sessionId)
      
      if (agent) {
        console.log('📊 AgentStore数据:', {
          id: agent.id,
          name: agent.name,
          display_name: agent.display_name,
          avatar_cid: agent.avatar_cid,
          avatar_url: agent.avatar_url,
          diapIdentity: agent.diapIdentity,
          ipns: agent.ipns,
          cid: agent.cid,
          did: agent.did
        })
        
        // 检查数据一致性
        const hasDiapIdentity = await hasDiapIdentitySafe(sessionId)
        if (hasDiapIdentity && !agent.diapIdentity) {
          console.log('⚠️ 数据不一致：统一存储有DIAP身份但agentStore中没有')
          this.diagnosticResults.errors.push('数据不一致：统一存储有DIAP身份但agentStore中没有')
        }
      }
      
    } catch (error) {
      console.error('❌ 检查数据存储失败:', error)
      this.diagnosticResults.errors.push(`数据存储检查失败: ${(error as Error).message}`)
    }
  }

  /**
   * 生成诊断报告
   */
  generateDiagnosticReport(sessionId: string): DiagnosticReport {
    const report: DiagnosticReport = {
      sessionId,
      timestamp: Date.now(),
      status: this.diagnosticResults.errors.length === 0 ? 'success' : 'error',
      results: this.diagnosticResults,
      recommendations: this.getRecommendations()
    }
    
    return report
  }

  /**
   * 获取修复建议
   */
  getRecommendations(): Recommendation[] {
    const recommendations: Recommendation[] = []
    
    if (!this.diagnosticResults.diapIdentityCreated) {
      recommendations.push({
        issue: 'DIAP身份创建失败',
        solution: '检查IPFS节点状态，确保Tauri服务正常运行',
        action: '在控制台运行: AlouDiapCleanup.fullCleanup()'
      })
    }
    
    if (!this.diagnosticResults.avatarCorrect) {
      recommendations.push({
        issue: '头像同化问题',
        solution: '检查头像数据源，确保每个智能体有独立的头像',
        action: '在控制台运行: AlouAvatarProtection.checkAllAvatars()'
      })
    }
    
    if (this.diagnosticResults.errors.length > 0) {
      recommendations.push({
        issue: '数据存储问题',
        solution: '检查数据同步状态，清理损坏数据',
        action: '在控制台运行: AlouDiapCleanup.fixCorruptedAvatars()'
      })
    }
    
    return recommendations
  }

  /**
   * 获取默认头像
   */
  getFallbackAvatar(): string {
    return '/assets/avatar-placeholder.png'
  }

  /**
   * 重置诊断结果
   */
  resetResults(): void {
    this.diagnosticResults = {
      diapIdentityCreated: false,
      diapIdentityStored: false,
      avatarCorrect: false,
      avatarSources: {},
      errors: []
    }
  }
}

// 创建全局实例
const agentCreationDiagnosticTool = new AgentCreationDiagnosticTool()

// 控制台工具
if (typeof window !== 'undefined') {
  window.AlouAgentDiagnostic = {
    diagnoseAgentCreation: (sessionId: string) => agentCreationDiagnosticTool.diagnoseAgentCreation(sessionId),
    checkDiapIdentity: (sessionId: string) => agentCreationDiagnosticTool.checkDiapIdentityCreation(sessionId),
    checkAvatar: (sessionId: string) => agentCreationDiagnosticTool.checkAvatarStatus(sessionId),
    getRecommendations: () => agentCreationDiagnosticTool.getRecommendations(),
    resetResults: () => agentCreationDiagnosticTool.resetResults(),
    help: () => {
      console.group('🔍 智能体创建诊断工具使用帮助')
      console.log('完整诊断:')
      console.log('  AlouAgentDiagnostic.diagnoseAgentCreation(sessionId) - 完整诊断智能体创建')
      console.log('')
      console.log('单项检查:')
      console.log('  AlouAgentDiagnostic.checkDiapIdentity(sessionId) - 检查DIAP身份')
      console.log('  AlouAgentDiagnostic.checkAvatar(sessionId) - 检查头像状态')
      console.log('')
      console.log('工具:')
      console.log('  AlouAgentDiagnostic.getRecommendations() - 获取修复建议')
      console.log('  AlouAgentDiagnostic.resetResults() - 重置诊断结果')
      console.groupEnd()
    }
  }
  
  console.log('🔍 AlouAgentDiagnostic 诊断工具已加载!')
  console.log('💡 输入 AlouAgentDiagnostic.help() 查看使用帮助')
  console.log('🔧 输入 AlouAgentDiagnostic.diagnoseAgentCreation(sessionId) 诊断智能体创建')
}

export default agentCreationDiagnosticTool
