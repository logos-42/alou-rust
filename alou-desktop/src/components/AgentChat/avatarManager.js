import { resolveAgentAvatar } from './agentUtils'
import useAgentStore from '@/stores/agentStore'

/**
 * 头像管理模块
 * 集中处理头像的更新、同步和缓存
 */
class AvatarManager {
  constructor() {
    this.listeners = new Set()
    this.avatarCache = new Map()
  }

  /**
   * 解析智能体头像
   * @param {Object} agent - 智能体对象
   * @returns {string} 头像URL
   */
  resolveAvatar(agent) {
    if (!agent) {
      return this.getFallbackAvatar()
    }

    // 检查缓存
    const cacheKey = this.getCacheKey(agent)
    if (this.avatarCache.has(cacheKey)) {
      return this.avatarCache.get(cacheKey)
    }

    // 解析头像
    const avatar = resolveAgentAvatar(agent)
    
    // 缓存结果
    this.avatarCache.set(cacheKey, avatar)
    
    return avatar
  }

  /**
   * 更新智能体头像
   * @param {string} agentId - 智能体ID
   * @param {string} avatar - 新头像URL（可以是 http URL 或 data URL）
   * @returns {Object} 更新后的智能体
   */
  updateAvatar(agentId, avatar) {
    console.log('[AvatarManager] 更新头像:', { agentId, avatar })
    
    // 更新 store
    const updateAgent = useAgentStore.getState().updateAgent
    const updatedAgent = updateAgent(agentId, {
      avatar,
      avatar_url: avatar, // 保持兼容性
      updated_at: Date.now()
    })

    if (!updatedAgent) {
      console.warn('[AvatarManager] 更新失败：智能体不存在', agentId)
      return null
    }

    // 清除缓存
    this.clearCache(updatedAgent)
    
    // 通知监听器
    this.notifyListeners(updatedAgent)
    
    // 触发全局事件
    this.dispatchGlobalEvent(updatedAgent)
    
    return updatedAgent
  }

  /**
   * 批量更新头像（用于频道列表等）
   * @param {Array} channels - 频道列表
   * @param {Object} updatedAgent - 更新后的智能体
   * @returns {Array} 更新后的频道列表
   */
  updateChannelsAvatar(channels, updatedAgent) {
    if (!channels || !updatedAgent) {
      return channels || []
    }

    return channels.map(channel => {
      if (channel.meta?.id === updatedAgent.id) {
        const updatedChannel = {
          ...channel,
          meta: updatedAgent,
          avatar: updatedAgent.avatar || channel.avatar,
          name: updatedAgent.display_name || updatedAgent.name || channel.name
        }
        console.log('[AvatarManager] 更新频道头像:', {
          channelId: channel.id,
          oldAvatar: channel.avatar,
          newAvatar: updatedChannel.avatar
        })
        return updatedChannel
      }
      return channel
    })
  }

  /**
   * 添加监听器
   * @param {Function} listener - 监听函数
   */
  addListener(listener) {
    this.listeners.add(listener)
  }

  /**
   * 移除监听器
   * @param {Function} listener - 监听函数
   */
  removeListener(listener) {
    this.listeners.delete(listener)
  }

  /**
   * 通知所有监听器
   * @param {Object} updatedAgent - 更新后的智能体
   */
  notifyListeners(updatedAgent) {
    this.listeners.forEach(listener => {
      try {
        listener(updatedAgent)
      } catch (error) {
        console.error('[AvatarManager] 监听器错误:', error)
      }
    })
  }

  /**
   * 触发全局事件
   * @param {Object} updatedAgent - 更新后的智能体
   */
  dispatchGlobalEvent(updatedAgent) {
    // 触发自定义事件，通知其他组件更新
    window.dispatchEvent(new CustomEvent('agent-avatar-updated', {
      detail: { 
        agentId: updatedAgent.id, 
        avatar: updatedAgent.avatar,
        agent: updatedAgent
      }
    }))
  }

  /**
   * 获取缓存键
   * @param {Object} agent - 智能体对象
   * @returns {string} 缓存键
   */
  getCacheKey(agent) {
    if (!agent) return 'null'
    
    const id = agent.id || agent.sessionId
    const avatar = agent.avatar || agent.avatar_url
    const timestamp = agent.updated_at || 0
    
    return `${id}:${avatar}:${timestamp}`
  }

  /**
   * 清除缓存
   * @param {Object} agent - 智能体对象
   */
  clearCache(agent) {
    const cacheKey = this.getCacheKey(agent)
    this.avatarCache.delete(cacheKey)
  }

  /**
   * 清除所有缓存
   */
  clearAllCache() {
    this.avatarCache.clear()
  }

  /**
   * 获取默认头像
   * @returns {string} 默认头像URL
   */
  getFallbackAvatar() {
    return 'https://avatars.githubusercontent.com/u/16309930?v=4'
  }

  /**
   * 验证头像URL
   * @param {string} avatar - 头像URL
   * @returns {boolean} 是否有效
   */
  validateAvatar(avatar) {
    if (!avatar) return false
    
    // 支持 http/https URL
    if (avatar.startsWith('http://') || avatar.startsWith('https://')) {
      return true
    }
    
    // 支持 data URL（base64图片）
    if (avatar.startsWith('data:image/')) {
      return true
    }
    
    // 支持 IPFS URL
    if (avatar.startsWith('ipfs://') || avatar.includes('ipfs/')) {
      return true
    }
    
    return false
  }

  /**
   * 处理文件上传并生成 data URL
   * @param {File} file - 图片文件
   * @returns {Promise<string>} data URL
   */
  async processFileUpload(file) {
    return new Promise((resolve, reject) => {
      // 验证文件类型
      if (!file.type.startsWith('image/')) {
        reject(new Error('请选择图片文件'))
        return
      }

      // 验证文件大小（5MB限制）
      if (file.size > 5 * 1024 * 1024) {
        reject(new Error('图片大小不能超过5MB'))
        return
      }

      // 创建 data URL
      const reader = new FileReader()
      reader.onload = (event) => {
        resolve(event.target.result)
      }
      reader.onerror = (error) => {
        reject(new Error('文件读取失败: ' + error.message))
      }
      reader.readAsDataURL(file)
    })
  }
}

// 创建单例实例
const avatarManager = new AvatarManager()

// React Hook
export const useAvatarManager = () => {
  return avatarManager
}

// 工具函数
export const useAvatar = (agent) => {
  const [avatar, setAvatar] = React.useState(() => avatarManager.resolveAvatar(agent))
  
  React.useEffect(() => {
    // 初始设置
    setAvatar(avatarManager.resolveAvatar(agent))
    
    // 添加监听器
    const handleAvatarUpdate = (updatedAgent) => {
      if (updatedAgent.id === agent?.id) {
        setAvatar(avatarManager.resolveAvatar(updatedAgent))
      }
    }
    
    avatarManager.addListener(handleAvatarUpdate)
    
    // 监听全局事件
    const handleGlobalEvent = (event) => {
      if (event.detail.agentId === agent?.id) {
        setAvatar(event.detail.avatar)
      }
    }
    
    window.addEventListener('agent-avatar-updated', handleGlobalEvent)
    
    return () => {
      avatarManager.removeListener(handleAvatarUpdate)
      window.removeEventListener('agent-avatar-updated', handleGlobalEvent)
    }
  }, [agent])
  
  return avatar
}

export default avatarManager
