/**
 * PubSub Service - 群聊和智能体间通信
 * 使用 IPFS PubSub 或 WebSocket 实现实时消息传递
 */
import { invoke } from '@tauri-apps/api/core'

const DEFAULT_IPFS_API = import.meta.env.VITE_IPFS_API_URL || 'http://127.0.0.1:5001'

/**
 * 消息类型
 */
export const MessageType = {
  CHAT: 'chat',           // 普通聊天消息
  SYSTEM: 'system',       // 系统消息
  JOIN: 'join',           // 加入频道
  LEAVE: 'leave',         // 离开频道
  AGENT_REQUEST: 'agent_request',   // 智能体请求
  AGENT_RESPONSE: 'agent_response', // 智能体响应
}

/**
 * PubSub 消息结构
 */
export class PubSubMessage {
  constructor({ type, from, to, content, topic, timestamp, metadata }) {
    this.id = `msg_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`
    this.type = type || MessageType.CHAT
    this.from = from        // 发送者 DID/IPNS
    this.to = to || null    // 接收者 DID/IPNS（null 表示广播）
    this.content = content
    this.topic = topic
    this.timestamp = timestamp || Date.now()
    this.metadata = metadata || {}
  }

  toJSON() {
    return {
      id: this.id,
      type: this.type,
      from: this.from,
      to: this.to,
      content: this.content,
      topic: this.topic,
      timestamp: this.timestamp,
      metadata: this.metadata,
    }
  }

  static fromJSON(json) {
    return new PubSubMessage(json)
  }
}

class PubSubService {
  constructor() {
    this.subscriptions = new Map() // topic -> Set<callback>
    this.pollingIntervals = new Map() // topic -> intervalId
    this.lastMessageTimestamp = new Map() // topic -> timestamp
    this.isConnected = false
    this.localIdentity = null
  }

  /**
   * 设置本地身份（用于签名消息）
   */
  setLocalIdentity(identity) {
    this.localIdentity = identity
    console.log('[PubSubService] 设置本地身份:', identity?.did)
  }

  /**
   * 生成群聊主题名称
   */
  generateGroupTopic(groupId) {
    return `diap/group/${groupId}`
  }

  /**
   * 生成智能体间通信主题
   */
  generateAgentTopic(agentDid) {
    const didHash = agentDid.split(':').pop()?.slice(0, 16) || agentDid
    return `diap/agent/${didHash}`
  }

  /**
   * 订阅主题
   * @param {string} topic - 主题名称
   * @param {function} callback - 消息回调 (message: PubSubMessage) => void
   * @returns {function} 取消订阅函数
   */
  subscribe(topic, callback) {
    if (!this.subscriptions.has(topic)) {
      this.subscriptions.set(topic, new Set())
      // 开始轮询该主题
      this._startPolling(topic)
    }

    this.subscriptions.get(topic).add(callback)
    console.log('[PubSubService] 订阅主题:', topic)

    // 返回取消订阅函数
    return () => {
      this.unsubscribe(topic, callback)
    }
  }

  /**
   * 取消订阅
   */
  unsubscribe(topic, callback) {
    const callbacks = this.subscriptions.get(topic)
    if (callbacks) {
      callbacks.delete(callback)
      if (callbacks.size === 0) {
        this.subscriptions.delete(topic)
        this._stopPolling(topic)
      }
    }
    console.log('[PubSubService] 取消订阅:', topic)
  }

  /**
   * 发布消息到主题
   */
  async publish(topic, message) {
    if (!this.localIdentity) {
      console.warn('[PubSubService] 未设置本地身份，无法发布消息')
      return false
    }

    const pubsubMessage = message instanceof PubSubMessage
      ? message
      : new PubSubMessage({
          ...message,
          from: this.localIdentity.did,
          topic,
        })

    try {
      // 尝试通过 IPFS PubSub 发布
      const result = await this._publishToIpfs(topic, pubsubMessage)
      if (result) {
        console.log('[PubSubService] 消息已发布到:', topic)
        return true
      }

      // 降级：通过后端 API 发布
      return await this._publishToBackend(topic, pubsubMessage)
    } catch (error) {
      console.error('[PubSubService] 发布消息失败:', error)
      return false
    }
  }

  /**
   * 通过 IPFS PubSub 发布消息
   */
  async _publishToIpfs(topic, message) {
    try {
      await invoke('ipfs_pubsub_publish', {
        topic,
        message: JSON.stringify(message.toJSON()),
        ipfsApiUrl: DEFAULT_IPFS_API,
      })
      return true
    } catch (error) {
      console.warn('[PubSubService] IPFS PubSub 发布失败，尝试降级:', error)
      return false
    }
  }

  /**
   * 通过后端 API 发布消息（降级方案）
   */
  async _publishToBackend(topic, message) {
    try {
      const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 
        (import.meta.env.DEV ? 'http://localhost:1420' : 'https://alou-edge.yuanjieliu65.workers.dev')
      
      const response = await fetch(`${API_BASE_URL}/api/pubsub/publish`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          topic,
          message: message.toJSON(),
        }),
      })

      return response.ok
    } catch (error) {
      console.error('[PubSubService] 后端发布失败:', error)
      return false
    }
  }

  /**
   * 开始轮询主题消息
   */
  _startPolling(topic) {
    if (this.pollingIntervals.has(topic)) {
      return
    }

    this.lastMessageTimestamp.set(topic, Date.now())

    const poll = async () => {
      try {
        const messages = await this._fetchMessages(topic)
        if (messages && messages.length > 0) {
          const callbacks = this.subscriptions.get(topic)
          if (callbacks) {
            messages.forEach(msg => {
              const pubsubMessage = PubSubMessage.fromJSON(msg)
              callbacks.forEach(cb => cb(pubsubMessage))
            })
          }
          // 更新最后消息时间戳
          const lastMsg = messages[messages.length - 1]
          if (lastMsg.timestamp) {
            this.lastMessageTimestamp.set(topic, lastMsg.timestamp)
          }
        }
      } catch (error) {
        console.error('[PubSubService] 轮询失败:', topic, error)
      }
    }

    // 立即执行一次
    poll()

    // 设置轮询间隔（3秒）
    const intervalId = setInterval(poll, 3000)
    this.pollingIntervals.set(topic, intervalId)
  }

  /**
   * 停止轮询
   */
  _stopPolling(topic) {
    const intervalId = this.pollingIntervals.get(topic)
    if (intervalId) {
      clearInterval(intervalId)
      this.pollingIntervals.delete(topic)
      this.lastMessageTimestamp.delete(topic)
    }
  }

  /**
   * 获取主题消息
   */
  async _fetchMessages(topic) {
    try {
      // 尝试从 IPFS PubSub 获取
      const messages = await invoke('ipfs_pubsub_subscribe_once', {
        topic,
        timeoutMs: 1000,
        ipfsApiUrl: DEFAULT_IPFS_API,
      })
      
      if (messages && messages.length > 0) {
        return messages.map(m => typeof m === 'string' ? JSON.parse(m) : m)
      }
    } catch (error) {
      // IPFS PubSub 不可用，尝试后端
    }

    // 降级：从后端获取
    try {
      const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 
        (import.meta.env.DEV ? 'http://localhost:1420' : 'https://alou-edge.yuanjieliu65.workers.dev')
      
      const since = this.lastMessageTimestamp.get(topic) || 0
      const response = await fetch(
        `${API_BASE_URL}/api/pubsub/messages?topic=${encodeURIComponent(topic)}&since=${since}`,
        { method: 'GET' }
      )

      if (response.ok) {
        const data = await response.json()
        return data.messages || []
      }
    } catch (error) {
      // 静默失败
    }

    return []
  }

  /**
   * 创建群聊
   */
  async createGroup(groupName, members = []) {
    const groupId = `group_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`
    const topic = this.generateGroupTopic(groupId)

    // 发布群聊创建消息
    await this.publish(topic, new PubSubMessage({
      type: MessageType.SYSTEM,
      from: this.localIdentity?.did,
      content: `群聊 "${groupName}" 已创建`,
      topic,
      metadata: {
        groupId,
        groupName,
        members,
        createdAt: Date.now(),
      },
    }))

    return { groupId, topic, groupName, members }
  }

  /**
   * 加入群聊
   */
  async joinGroup(topic, callback) {
    const unsubscribe = this.subscribe(topic, callback)

    // 发布加入消息
    await this.publish(topic, new PubSubMessage({
      type: MessageType.JOIN,
      from: this.localIdentity?.did,
      content: `${this.localIdentity?.did || '用户'} 加入了群聊`,
      topic,
    }))

    return unsubscribe
  }

  /**
   * 离开群聊
   */
  async leaveGroup(topic) {
    // 发布离开消息
    await this.publish(topic, new PubSubMessage({
      type: MessageType.LEAVE,
      from: this.localIdentity?.did,
      content: `${this.localIdentity?.did || '用户'} 离开了群聊`,
      topic,
    }))

    // 取消所有该主题的订阅
    const callbacks = this.subscriptions.get(topic)
    if (callbacks) {
      callbacks.clear()
      this.subscriptions.delete(topic)
      this._stopPolling(topic)
    }
  }

  /**
   * 发送群聊消息
   */
  async sendGroupMessage(topic, content, metadata = {}) {
    return this.publish(topic, new PubSubMessage({
      type: MessageType.CHAT,
      from: this.localIdentity?.did,
      content,
      topic,
      metadata,
    }))
  }

  /**
   * 清理所有订阅
   */
  cleanup() {
    this.pollingIntervals.forEach((intervalId) => {
      clearInterval(intervalId)
    })
    this.pollingIntervals.clear()
    this.subscriptions.clear()
    this.lastMessageTimestamp.clear()
    console.log('[PubSubService] 已清理所有订阅')
  }
}

export default new PubSubService()
