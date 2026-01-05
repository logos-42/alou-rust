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
   * 更新智能体头像和名称
   * @param {string} agentId - 智能体ID
   * @param {string} avatar - 新头像URL（可以是 http URL、data URL 或 IPFS CID）
   * @param {string} name - 新名称（可选）
   * @returns {Object} 更新后的智能体
   */
  updateAvatar(agentId, avatar, name = null) {
    console.log('[AvatarManager] 更新头像和名称:', { agentId, avatar, name })
    
    try {
      // 检查是否是data URL，如果是则记录警告
      if (avatar && avatar.startsWith('data:image/')) {
        console.warn('[AvatarManager] 检测到data URL头像，建议使用IPFS存储:', avatar.length)
        // 对于data URL，我们仍然存储，但会尝试压缩
        avatar = this.compressAvatarIfNeeded(avatar)
      }
      
      // 准备更新数据
      const updates = {
        avatar: avatar, // 存储实际头像
        avatar_url: avatar, // 保持兼容性
        avatar_cid: this.extractCidFromUrl(avatar), // 提取CID
        updated_at: Date.now()
      }
      
      // 如果提供了名称，也更新名称
      if (name) {
        updates.display_name = name
        updates.name = name
      }
      
      // 更新 store
      const updateAgent = useAgentStore.getState().updateAgent
      const updatedAgent = updateAgent(agentId, updates)

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
    } catch (error) {
      console.error('[AvatarManager] 更新头像失败:', error)
      return null
    }
  }

  /**
   * 批量更新头像和名称（用于频道列表等）
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
          name: updatedAgent.display_name || updatedAgent.name || channel.name,
          display_name: updatedAgent.display_name || channel.display_name
        }
        console.log('[AvatarManager] 更新频道头像和名称:', {
          channelId: channel.id,
          oldAvatar: channel.avatar,
          newAvatar: updatedChannel.avatar,
          oldName: channel.name,
          newName: updatedChannel.name
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
        name: updatedAgent.display_name || updatedAgent.name,
        agent: updatedAgent
      }
    }))
    
    // 同时触发名称更新事件（如果需要）
    if (updatedAgent.display_name || updatedAgent.name) {
      window.dispatchEvent(new CustomEvent('agent-name-updated', {
        detail: {
          agentId: updatedAgent.id,
          name: updatedAgent.display_name || updatedAgent.name,
          agent: updatedAgent
        }
      }))
    }
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
   * 从URL中提取CID
   * @param {string} url - 头像URL
   * @returns {string|null} CID或null
   */
  extractCidFromUrl(url) {
    if (!url) return null
    
    // 检查IPFS URL格式
    const ipfsMatch = url.match(/ipfs:\/\/([a-zA-Z0-9]+)/)
    if (ipfsMatch) {
      return ipfsMatch[1]
    }
    
    // 检查包含ipfs/的URL
    const ipfsPathMatch = url.match(/ipfs\/([a-zA-Z0-9]+)/)
    if (ipfsPathMatch) {
      return ipfsPathMatch[1]
    }
    
    // 检查gateway URL
    const gatewayMatch = url.match(/\/(Qm[1-9A-HJ-NP-Za-km-z]{44,}|b[A-Za-z2-7]{58,}|B[A-Z2-7]{58,}|z[1-9A-HJ-NP-Za-km-z]{48,}|F[0-9A-F]{50,})/)
    if (gatewayMatch) {
      return gatewayMatch[1]
    }
    
    return null
  }

  /**
   * 压缩头像数据（如果需要）
   * @param {string} dataUrl - data URL 格式的头像
   * @returns {string} 压缩后的头像URL
   */
  compressAvatarIfNeeded(dataUrl) {
    // 如果是小图片，直接返回
    if (dataUrl.length < 50000) { // 小于50KB
      return dataUrl
    }
    
    console.warn('[AvatarManager] 头像数据过大，尝试压缩:', dataUrl.length)
    
    try {
      // 创建一个临时图片元素来获取图片尺寸
      const img = new Image()
      img.src = dataUrl
      
      // 异步压缩（这里简化处理，实际应该使用canvas压缩）
      // 暂时返回原始数据，但记录警告
      return dataUrl
    } catch (error) {
      console.error('[AvatarManager] 压缩头像失败:', error)
      return dataUrl
    }
  }

  /**
   * 处理头像数据，避免存储过大
   * @param {string} dataUrl - data URL 格式的头像
   * @returns {string} 处理后的头像URL
   */
  processAvatarData(dataUrl) {
    // 如果是小图片，直接返回
    if (dataUrl.length < 100000) { // 小于100KB
      return dataUrl
    }
    
    console.warn('[AvatarManager] 头像数据过大，尝试压缩或使用占位符:', dataUrl.length)
    
    // 方案1：压缩图片（这里简化处理，实际应该使用canvas压缩）
    // 方案2：使用占位符或缩略图
    // 方案3：存储到IndexedDB而不是localStorage
    
    // 暂时返回一个占位符，避免存储错误
    return this.getFallbackAvatar()
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
