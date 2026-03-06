/**
 * 智能体头像切换工具
 * 允许智能体通过 MCP 工具切换自己的头像
 */

import avatarService from '@/services/avatarService'
import useAgentStore from '@/stores/agentStore'

/**
 * 切换头像参数
 */
export interface SwitchAvatarParams {
  /** 智能体 ID 或会话 ID */
  agentId: string
  /** 新头像 URL 或 IPFS CID */
  avatarUrl?: string
  /** 使用 Dicebear 风格（可选） */
  dicebearStyle?: string
  /** 使用默认头像 */
  useDefault?: boolean
  /** 上传头像文件（base64 编码） */
  avatarFileBase64?: string
  /** 文件名 */
  fileName?: string
}

/**
 * 切换头像结果
 */
export interface SwitchAvatarResult {
  success: boolean
  avatarUrl?: string
  cid?: string
  source?: 'ipfs' | 'local' | 'dicebear'
  error?: string
  message?: string
}

/**
 * 智能体头像切换工具类
 */
class AgentAvatarSwitchTool {
  /**
   * 切换智能体头像
   * @param params - 切换参数
   * @returns 切换结果
   */
  async switchAvatar(params: SwitchAvatarParams): Promise<SwitchAvatarResult> {
    const { agentId, avatarUrl, dicebearStyle, useDefault, avatarFileBase64, fileName } = params

    if (!agentId) {
      return {
        success: false,
        error: '缺少 agentId 参数',
      }
    }

    try {
      const store = useAgentStore.getState()
      const agent = store.agents.find((a) => a.id === agentId || a.sessionId === agentId)

      if (!agent) {
        return {
          success: false,
          error: `未找到智能体：${agentId}`,
        }
      }

      let newAvatarUrl: string
      let cid: string | undefined
      let source: 'ipfs' | 'local' | 'dicebear'

      // 1. 上传文件（如果提供了 base64）
      if (avatarFileBase64) {
        try {
          // 将 base64 转换为 Blob
          const response = await fetch(avatarFileBase64)
          const blob = await response.blob()
          const file = new File([blob], fileName || 'avatar.png', {
            type: blob.type,
          })

          // 上传头像
          const uploadResult = await avatarService.uploadAvatar(file, {
            sessionId: agentId,
          })

          if (!uploadResult.success) {
            return {
              success: false,
              error: uploadResult.error || '头像上传失败',
            }
          }

          newAvatarUrl = uploadResult.avatarUrl
          cid = uploadResult.cid
          source = uploadResult.source as 'ipfs' | 'local'
        } catch (error) {
          return {
            success: false,
            error: `头像上传失败：${(error as Error).message}`,
          }
        }
      }
      // 2. 使用 Dicebear 风格
      else if (dicebearStyle) {
        const seed = agent.name || agent.display_name || agentId
        newAvatarUrl = avatarService.generateDicebearAvatar(seed, dicebearStyle)
        source = 'dicebear'
      }
      // 3. 使用默认头像
      else if (useDefault) {
        newAvatarUrl = avatarService.getFallbackAvatar()
        source = 'local'
      }
      // 4. 使用提供的 URL
      else if (avatarUrl) {
        // 验证头像 URL
        const validation = await avatarService.validateAvatar(avatarUrl)
        if (!validation.isValid) {
          return {
            success: false,
            error: `无效的头像 URL: ${validation.error}`,
          }
        }

        newAvatarUrl = avatarService.sanitizeAvatar(avatarUrl)
        source = avatarUrl.startsWith('data:') ? 'local' : 'ipfs'
      } else {
        return {
          success: false,
          error: '必须提供 avatarUrl、dicebearStyle、useDefault 或 avatarFileBase64 之一',
        }
      }

      // 更新智能体头像
      store.updateAgent(agentId, {
        avatar_url: newAvatarUrl,
        avatar_cid: cid || null,
      })

      console.log('[AgentAvatarSwitchTool] 头像切换成功:', {
        agentId,
        newAvatarUrl,
        source,
      })

      return {
        success: true,
        avatarUrl: newAvatarUrl,
        cid,
        source,
        message: '头像切换成功',
      }
    } catch (error) {
      console.error('[AgentAvatarSwitchTool] 头像切换失败:', error)
      return {
        success: false,
        error: `头像切换失败：${(error as Error).message}`,
      }
    }
  }

  /**
   * 获取可用的 Dicebear 风格列表
   * @returns 风格列表
   */
  getAvailableDicebearStyles(): string[] {
    return [
      'identicon',
      'avataaars',
      'bottts',
      'lorelei',
      'notionists',
      'fun-emoji',
      'shapes',
    ]
  }

  /**
   * 验证头像 URL
   * @param avatarUrl - 头像 URL
   * @returns 验证结果
   */
  async validateAvatar(avatarUrl: string): Promise<{ isValid: boolean; error?: string }> {
    const result = await avatarService.validateAvatar(avatarUrl)
    return {
      isValid: result.isValid,
      error: result.error,
    }
  }
}

// 创建单例实例
const agentAvatarSwitchTool = new AgentAvatarSwitchTool()

export default agentAvatarSwitchTool
export { AgentAvatarSwitchTool }
