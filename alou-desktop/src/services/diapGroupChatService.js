/**
 * DIAP SDK 群聊服务
 * 统一管理基于DIAP SDK的群聊创建、加入、消息传递等功能
 * 支持IPFS PubSub和后端API的双重架构
 */

import { invoke } from '@tauri-apps/api/core'
import { PubSubMessage, MessageType } from './pubsubService'
import apiClient from './api'

const DEFAULT_IPFS_API = import.meta.env.VITE_IPFS_API_URL || 'http://127.0.0.1:5001'

/**
 * DIAP 群聊配置
 */
export class DiapGroupConfig {
  constructor({
    groupName,
    description = '',
    members = [],
    isPublic = false,
    requireAuth = true,
    maxMembers = 50,
    enableZkp = true,
    topicPrefix = 'diap/group'
  } = {}) {
    this.groupName = groupName
    this.description = description
    this.members = members
    this.isPublic = isPublic
    this.requireAuth = requireAuth
    this.maxMembers = maxMembers
    this.enableZkp = enableZkp
    this.topicPrefix = topicPrefix
  }
}

/**
 * DIAP 群聊信息
 */
export class DiapGroup {
  constructor({
    groupId,
    groupName,
    description,
    topic,
    members = [],
    creator,
    createdAt,
    metadata = {}
  } = {}) {
    this.groupId = groupId
    this.groupName = groupName
    this.description = description
    this.topic = topic
    this.members = members
    this.creator = creator
    this.createdAt = createdAt
    this.metadata = metadata
  }

  toJSON() {
    return {
      groupId: this.groupId,
      groupName: this.groupName,
      description: this.description,
      topic: this.topic,
      members: this.members,
      creator: this.creator,
      createdAt: this.createdAt,
      metadata: this.metadata
    }
  }

  static fromJSON(json) {
    return new DiapGroup(json)
  }
}

/**
 * DIAP 群聊消息
 */
export class DiapGroupMessage {
  constructor({
    id,
    groupId,
    topic,
    from,
    fromName,
    to = null,
    content,
    type = MessageType.CHAT,
    timestamp,
    metadata = {}
  } = {}) {
    this.id = id || `msg_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`
    this.groupId = groupId
    this.topic = topic
    this.from = from
    this.fromName = fromName
    this.to = to
    this.content = content
    this.type = type
    this.timestamp = timestamp || Date.now()
    this.metadata = metadata
  }

  toJSON() {
    return {
      id: this.id,
      groupId: this.groupId,
      topic: this.topic,
      from: this.from,
      fromName: this.fromName,
      to: this.to,
      content: this.content,
      type: this.type,
      timestamp: this.timestamp,
      metadata: this.metadata
    }
  }

  static fromJSON(json) {
    return new DiapGroupMessage(json)
  }
}

/**
 * DIAP 群聊服务类
 */
class DiapGroupChatService {
  constructor() {
    this.localIdentity = null
    this.groups = new Map() // groupId -> DiapGroup
    this.subscriptions = new Map() // groupId -> unsubscribe function
    this.messageHandlers = new Map() // groupId -> Set<callback>
    this.ipfsAvailable = null // 缓存IPFS可用性
    this.backendAvailable = null // 缓存后端可用性
  }

  /**
   * 设置本地身份
   */
  setLocalIdentity(identity) {
    this.localIdentity = identity
    console.log('[DiapGroupChatService] 设置本地身份:', identity?.did)
  }

  /**
   * 检查IPFS PubSub可用性
   */
  async checkIpfsAvailability() {
    if (this.ipfsAvailable !== null) {
      return this.ipfsAvailable
    }

    try {
      // 尝试获取IPFS节点信息
      await invoke('get_ipfs_info', { ipfs_api_url: DEFAULT_IPFS_API })
      this.ipfsAvailable = true
      console.log('[DiapGroupChatService] IPFS PubSub 可用')
      return true
    } catch (error) {
      this.ipfsAvailable = false
      console.warn('[DiapGroupChatService] IPFS PubSub 不可用:', error.message)
      return false
    }
  }

  /**
   * 检查后端API可用性
   */
  async checkBackendAvailability() {
    if (this.backendAvailable !== null) {
      return this.backendAvailable
    }

    try {
      const response = await apiClient.get('/pubsub/health', { timeout: 2000 })
      this.backendAvailable = response.status === 200
      console.log('[DiapGroupChatService] 后端API 可用:', this.backendAvailable)
      return this.backendAvailable
    } catch (error) {
      this.backendAvailable = false
      console.warn('[DiapGroupChatService] 后端API 不可用:', error.message)
      return false
    }
  }

  /**
   * 生成群聊ID
   */
  generateGroupId() {
    return `diap_group_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`
  }

  /**
   * 生成群聊主题
   */
  generateGroupTopic(groupId) {
    return `diap/group/${groupId}`
  }

  /**
   * 创建DIAP群聊
   * @param {DiapGroupConfig} config - 群聊配置
   * @returns {Promise<DiapGroup>} 创建的群聊信息
   */
  async createGroup(config) {
    if (!this.localIdentity) {
      throw new Error('未设置本地身份，无法创建群聊')
    }

    if (!(config instanceof DiapGroupConfig)) {
      config = new DiapGroupConfig(config)
    }

    const groupId = this.generateGroupId()
    const topic = this.generateGroupTopic(groupId)
    const now = Date.now()

    // 创建群聊对象
    const group = new DiapGroup({
      groupId,
      groupName: config.groupName,
      description: config.description,
      topic,
      members: [this.localIdentity.did, ...config.members],
      creator: this.localIdentity.did,
      createdAt: now,
      metadata: {
        isPublic: config.isPublic,
        requireAuth: config.requireAuth,
        maxMembers: config.maxMembers,
        enableZkp: config.enableZkp
      }
    })

    try {
      // 尝试通过IPFS PubSub创建
      const ipfsAvailable = await this.checkIpfsAvailability()
      if (ipfsAvailable) {
        await this._createGroupViaIpfs(group, config)
      }

      // 尝试通过后端API创建（作为备份）
      const backendAvailable = await this.checkBackendAvailability()
      if (backendAvailable) {
        await this._createGroupViaBackend(group, config)
      }

      // 缓存群聊信息
      this.groups.set(groupId, group)

      // 发送群聊创建消息
      await this._sendSystemMessage(group, `群聊 "${config.groupName}" 已创建`)

      console.log('[DiapGroupChatService] 群聊创建成功:', group)
      return group

    } catch (error) {
      console.error('[DiapGroupChatService] 创建群聊失败:', error)
      throw new Error(`创建群聊失败: ${error.message}`)
    }
  }

  /**
   * 通过IPFS PubSub创建群聊
   */
  async _createGroupViaIpfs(group, config) {
    // 创建群聊主题的系统消息
    const systemMessage = new DiapGroupMessage({
      groupId: group.groupId,
      topic: group.topic,
      from: 'system',
      fromName: '系统',
      content: `群聊 "${config.groupName}" 已通过IPFS PubSub创建`,
      type: MessageType.SYSTEM,
      metadata: {
        type: 'group_created',
        group: group.toJSON(),
        protocol: 'ipfs_pubsub'
      }
    })

    // 发布到IPFS PubSub
    await invoke('ipfs_pubsub_publish', {
      topic: group.topic,
      message: JSON.stringify(systemMessage.toJSON()),
      ipfs_api_url: DEFAULT_IPFS_API
    })

    console.log('[DiapGroupChatService] IPFS PubSub 群聊创建成功')
  }

  /**
   * 通过后端API创建群聊
   */
  async _createGroupViaBackend(group, config) {
    try {
      const response = await apiClient.post('/diap/group/create', {
        group: group.toJSON(),
        config: config
      })

      if (response.data.success) {
        console.log('[DiapGroupChatService] 后端API 群聊创建成功')
      } else {
        throw new Error(response.data.error || '后端创建失败')
      }
    } catch (error) {
      // 后端创建失败不影响整体流程，只记录警告
      console.warn('[DiapGroupChatService] 后端API群聊创建失败:', error.message)
    }
  }

  /**
   * 加入群聊
   * @param {string} groupId - 群聊ID
   * @param {string} topic - 群聊主题（可选，如果不提供则自动生成）
   * @returns {Promise<DiapGroup>} 加入的群聊信息
   */
  async joinGroup(groupId, topic = null) {
    if (!this.localIdentity) {
      throw new Error('未设置本地身份，无法加入群聊')
    }

    const groupTopic = topic || this.generateGroupTopic(groupId)

    try {
      // 订阅群聊消息
      await this.subscribeToGroup(groupId, groupTopic)

      // 发送加入消息
      const joinMessage = new DiapGroupMessage({
        groupId,
        topic: groupTopic,
        from: this.localIdentity.did,
        fromName: this.localIdentity.name || this.localIdentity.did,
        content: `${this.localIdentity.did} 加入了群聊`,
        type: MessageType.JOIN,
        metadata: {
          type: 'member_joined',
          member: this.localIdentity.did
        }
      })

      await this.sendMessage(joinMessage)

      console.log('[DiapGroupChatService] 加入群聊成功:', groupId)
      return { groupId, topic: groupTopic }

    } catch (error) {
      console.error('[DiapGroupChatService] 加入群聊失败:', error)
      throw new Error(`加入群聊失败: ${error.message}`)
    }
  }

  /**
   * 订阅群聊消息
   * @param {string} groupId - 群聊ID
   * @param {string} topic - 群聊主题
   * @returns {Promise<function>} 取消订阅函数
   */
  async subscribeToGroup(groupId, topic) {
    if (this.subscriptions.has(groupId)) {
      // 已经订阅，返回现有的取消订阅函数
      return this.subscriptions.get(groupId)
    }

    const messageHandlers = new Set()
    this.messageHandlers.set(groupId, messageHandlers)

    let unsubscribe = null

    // 尝试通过IPFS PubSub订阅
    const ipfsAvailable = await this.checkIpfsAvailability()
    if (ipfsAvailable) {
      unsubscribe = await this._subscribeViaIpfs(groupId, topic, messageHandlers)
    }

    // 如果IPFS不可用，尝试通过后端轮询
    if (!unsubscribe) {
      const backendAvailable = await this.checkBackendAvailability()
      if (backendAvailable) {
        unsubscribe = await this._subscribeViaBackend(groupId, topic, messageHandlers)
      }
    }

    if (!unsubscribe) {
      throw new Error('无法订阅群聊消息：IPFS和后端都不可用')
    }

    // 缓存取消订阅函数
    this.subscriptions.set(groupId, unsubscribe)

    console.log('[DiapGroupChatService] 订阅群聊成功:', groupId)
    return unsubscribe
  }

  /**
   * 通过IPFS PubSub订阅
   */
  async _subscribeViaIpfs(groupId, topic, messageHandlers) {
    let isSubscribed = true

    const poll = async () => {
      if (!isSubscribed) return

      try {
        const messages = await invoke('ipfs_pubsub_subscribe_once', {
          topic,
          timeoutMs: 1000,
          ipfs_api_url: DEFAULT_IPFS_API
        })

        if (messages && messages.length > 0) {
          messages.forEach(msgStr => {
            try {
              const msgData = typeof msgStr === 'string' ? JSON.parse(msgStr) : msgStr
              const message = DiapGroupMessage.fromJSON(msgData)
              
              // 通知所有处理器
              messageHandlers.forEach(handler => {
                try {
                  handler(message)
                } catch (error) {
                  console.error('[DiapGroupChatService] 消息处理器错误:', error)
                }
              })
            } catch (error) {
              console.error('[DiapGroupChatService] 解析消息失败:', error)
            }
          })
        }
      } catch (error) {
        // IPFS错误时静默处理，避免频繁日志
      }
    }

    // 立即执行一次
    poll()

    // 设置轮询
    const interval = setInterval(poll, 1000)

    // 返回取消订阅函数
    return () => {
      isSubscribed = false
      clearInterval(interval)
      console.log('[DiapGroupChatService] 取消IPFS订阅:', groupId)
    }
  }

  /**
   * 通过后端轮询订阅
   */
  async _subscribeViaBackend(groupId, topic, messageHandlers) {
    let lastTimestamp = Date.now()
    let isSubscribed = true

    const poll = async () => {
      if (!isSubscribed) return

      try {
        const response = await apiClient.get('/pubsub/messages', {
          params: { topic, since: lastTimestamp }
        })

        if (response.data?.messages) {
          response.data.messages.forEach(msgData => {
            const message = DiapGroupMessage.fromJSON(msgData)
            
            // 通知所有处理器
            messageHandlers.forEach(handler => {
              try {
                handler(message)
              } catch (error) {
                console.error('[DiapGroupChatService] 消息处理器错误:', error)
              }
            })

            // 更新时间戳
            if (message.timestamp > lastTimestamp) {
              lastTimestamp = message.timestamp
            }
          })
        }
      } catch (error) {
        console.error('[DiapGroupChatService] 后端轮询失败:', error)
      }
    }

    // 立即执行一次
    poll()

    // 设置轮询
    const interval = setInterval(poll, 2000)

    // 返回取消订阅函数
    return () => {
      isSubscribed = false
      clearInterval(interval)
      console.log('[DiapGroupChatService] 取消后端订阅:', groupId)
    }
  }

  /**
   * 发送消息到群聊
   * @param {DiapGroupMessage|string} message - 消息对象或内容字符串
   * @param {string} groupId - 群聊ID（如果message是字符串）
   * @param {string} topic - 群聊主题（如果message是字符串）
   * @returns {Promise<boolean>} 发送是否成功
   */
  async sendMessage(message, groupId = null, topic = null) {
    if (!this.localIdentity) {
      throw new Error('未设置本地身份，无法发送消息')
    }

    // 如果传入的是字符串，创建消息对象
    if (typeof message === 'string') {
      if (!groupId || !topic) {
        throw new Error('发送字符串消息需要提供groupId和topic')
      }
      message = new DiapGroupMessage({
        groupId,
        topic,
        from: this.localIdentity.did,
        fromName: this.localIdentity.name || this.localIdentity.did,
        content: message,
        type: MessageType.CHAT
      })
    }

    if (!(message instanceof DiapGroupMessage)) {
      throw new Error('消息必须是DiapGroupMessage对象或字符串')
    }

    try {
      let success = false

      // 尝试通过IPFS PubSub发送
      const ipfsAvailable = await this.checkIpfsAvailability()
      if (ipfsAvailable) {
        try {
          await invoke('ipfs_pubsub_publish', {
            topic: message.topic,
            message: JSON.stringify(message.toJSON()),
            ipfs_api_url: DEFAULT_IPFS_API
          })
          success = true
          console.log('[DiapGroupChatService] IPFS PubSub 消息发送成功')
        } catch (error) {
          console.warn('[DiapGroupChatService] IPFS PubSub 发送失败:', error.message)
        }
      }

      // 如果IPFS失败，尝试通过后端发送
      if (!success) {
        const backendAvailable = await this.checkBackendAvailability()
        if (backendAvailable) {
          try {
            await apiClient.post('/pubsub/publish', {
              topic: message.topic,
              message: message.toJSON()
            })
            success = true
            console.log('[DiapGroupChatService] 后端API 消息发送成功')
          } catch (error) {
            console.warn('[DiapGroupChatService] 后端API 发送失败:', error.message)
          }
        }
      }

      if (!success) {
        throw new Error('无法发送消息：IPFS和后端都不可用')
      }

      return true

    } catch (error) {
      console.error('[DiapGroupChatService] 发送消息失败:', error)
      throw error
    }
  }

  /**
   * 发送系统消息
   */
  async _sendSystemMessage(group, content) {
    const systemMessage = new DiapGroupMessage({
      groupId: group.groupId,
      topic: group.topic,
      from: 'system',
      fromName: '系统',
      content,
      type: MessageType.SYSTEM,
      metadata: {
        type: 'system_message',
        group: group.groupId
      }
    })

    try {
      await this.sendMessage(systemMessage)
    } catch (error) {
      // 系统消息发送失败不影响群聊创建
      console.warn('[DiapGroupChatService] 系统消息发送失败:', error.message)
    }
  }

  /**
   * 添加消息处理器
   * @param {string} groupId - 群聊ID
   * @param {function} handler - 消息处理函数
   */
  addMessageHandler(groupId, handler) {
    if (!this.messageHandlers.has(groupId)) {
      this.messageHandlers.set(groupId, new Set())
    }
    this.messageHandlers.get(groupId).add(handler)
  }

  /**
   * 移除消息处理器
   * @param {string} groupId - 群聊ID
   * @param {function} handler - 消息处理函数
   */
  removeMessageHandler(groupId, handler) {
    const handlers = this.messageHandlers.get(groupId)
    if (handlers) {
      handlers.delete(handler)
    }
  }

  /**
   * 离开群聊
   * @param {string} groupId - 群聊ID
   */
  async leaveGroup(groupId) {
    try {
      // 发送离开消息
      const group = this.groups.get(groupId)
      if (group && this.localIdentity) {
        const leaveMessage = new DiapGroupMessage({
          groupId,
          topic: group.topic,
          from: this.localIdentity.did,
          fromName: this.localIdentity.name || this.localIdentity.did,
          content: `${this.localIdentity.did} 离开了群聊`,
          type: MessageType.LEAVE,
          metadata: {
            type: 'member_left',
            member: this.localIdentity.did
          }
        })

        await this.sendMessage(leaveMessage)
      }

      // 取消订阅
      const unsubscribe = this.subscriptions.get(groupId)
      if (unsubscribe) {
        unsubscribe()
        this.subscriptions.delete(groupId)
      }

      // 清理消息处理器
      this.messageHandlers.delete(groupId)

      // 移除群聊缓存
      this.groups.delete(groupId)

      console.log('[DiapGroupChatService] 离开群聊成功:', groupId)

    } catch (error) {
      console.error('[DiapGroupChatService] 离开群聊失败:', error)
      throw new Error(`离开群聊失败: ${error.message}`)
    }
  }

  /**
   * 获取群聊信息
   * @param {string} groupId - 群聊ID
   * @returns {DiapGroup|null} 群聊信息
   */
  getGroup(groupId) {
    return this.groups.get(groupId) || null
  }

  /**
   * 获取所有群聊
   * @returns {DiapGroup[]} 群聊列表
   */
  getAllGroups() {
    return Array.from(this.groups.values())
  }

  /**
   * 清理所有订阅
   */
  cleanup() {
    // 取消所有订阅
    this.subscriptions.forEach((unsubscribe, groupId) => {
      try {
        unsubscribe()
      } catch (error) {
        console.error('[DiapGroupChatService] 清理订阅失败:', groupId, error)
      }
    })

    // 清理所有数据
    this.subscriptions.clear()
    this.messageHandlers.clear()
    this.groups.clear()

    console.log('[DiapGroupChatService] 已清理所有群聊订阅')
  }
}

export default new DiapGroupChatService()
