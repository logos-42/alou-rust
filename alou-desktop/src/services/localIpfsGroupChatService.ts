/**
 * 本地IPFS PubSub群聊服务
 * 使用本地IPFS节点创建pubsub，消息存储在本地内存和后端KV中
 */

import { invoke } from '@tauri-apps/api/core'
import apiClient from './api'

const DEFAULT_IPFS_API = import.meta.env.VITE_IPFS_API_URL || 'http://127.0.0.1:5001'

/**
 * 消息类型常量
 */
export const MessageTypes = {
  CHAT: 'chat',
  SYSTEM: 'system',
  JOIN: 'join',
  LEAVE: 'leave',
  AGENT_MESSAGE: 'agent_message',
  TASK_ASSIGN: 'task_assign',       // 任务分配
  TASK_PROGRESS: 'task_progress',     // 任务进度
  TASK_COMPLETE: 'task_complete',     // 任务完成
} as const

export type MessageType = typeof MessageTypes[keyof typeof MessageTypes]

/**
 * 本地身份接口
 */
export interface LocalIdentity {
  did: string
  name?: string
  [key: string]: any
}

/**
 * 本地群聊消息配置
 */
export interface LocalGroupMessageConfig {
  id?: string
  groupId: string
  topic: string
  from: string
  fromName?: string
  to?: string | null
  content: string
  type?: MessageType
  timestamp?: number | null
  metadata?: Record<string, any>
}

/**
 * 本地群聊消息类
 */
export class LocalGroupMessage {
  id: string
  groupId: string
  topic: string
  from: string
  fromName: string
  to: string | null
  content: string
  type: MessageType
  timestamp: number
  metadata: Record<string, any>

  constructor({
    id,
    groupId,
    topic,
    from,
    fromName = '',
    to = null,
    content,
    type = MessageTypes.CHAT,
    timestamp = null,
    metadata = {}
  }: LocalGroupMessageConfig) {
    this.id = id || `msg_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`
    this.groupId = groupId
    this.topic = topic
    this.from = from
    this.fromName = fromName || from
    this.to = to
    this.content = content
    this.type = type
    this.timestamp = timestamp || Date.now()
    this.metadata = metadata
  }

  toJSON(): LocalGroupMessageConfig {
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

  static fromJSON(json: LocalGroupMessageConfig): LocalGroupMessage {
    return new LocalGroupMessage(json)
  }
}

/**
 * 本地群聊配置
 */
export interface LocalGroupConfig {
  groupId: string
  groupName: string
  description?: string
  topic: string
  members?: string[]
  creator: string
  createdAt: number
  metadata?: Record<string, any>
}

/**
 * 本地群聊信息类
 */
export class LocalGroup {
  groupId: string
  groupName: string
  description: string
  topic: string
  members: string[]
  creator: string
  createdAt: number
  metadata: Record<string, any>

  constructor({
    groupId,
    groupName,
    description = '',
    topic,
    members = [],
    creator,
    createdAt,
    metadata = {}
  }: LocalGroupConfig) {
    this.groupId = groupId
    this.groupName = groupName
    this.description = description
    this.topic = topic
    this.members = members
    this.creator = creator
    this.createdAt = createdAt
    this.metadata = metadata
  }

  toJSON(): LocalGroupConfig {
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

  static fromJSON(json: LocalGroupConfig): LocalGroup {
    return new LocalGroup(json)
  }
}

/**
 * 创建群聊配置
 */
export interface CreateGroupConfig {
  groupName: string
  description?: string
  members?: string[]
  isPublic?: boolean
  maxMembers?: number
  metadata?: Record<string, any>
}

/**
 * 消息处理器类型
 */
export type MessageHandler = (message: LocalGroupMessage) => void

/**
 * 取消订阅函数类型
 */
export type UnsubscribeFunction = () => void

/**
 * KV存储响应类型
 */
interface KVResponse {
  success: boolean
  value?: string
  keys?: string[]
  error?: string
}

/**
 * 本地IPFS群聊服务类
 * 专注于使用本地IPFS节点进行PubSub通信
 */
class LocalIpfsGroupChatService {
  private localIdentity: LocalIdentity | null = null
  private groups: Map<string, LocalGroup> = new Map() // groupId -> LocalGroup (内存存储)
  private messages: Map<string, LocalGroupMessage[]> = new Map() // groupId -> LocalGroupMessage[] (内存存储)
  private subscriptions: Map<string, UnsubscribeFunction> = new Map() // groupId -> unsubscribe function
  private messageHandlers: Map<string, Set<MessageHandler>> = new Map() // groupId -> Set<callback>
  private ipfsAvailable: boolean = false
  private initialized: boolean = false

  /**
   * 初始化服务
   */
  async initialize(): Promise<void> {
    if (this.initialized) {
      return
    }

    console.log('[LocalIpfsGroupChatService] 初始化本地IPFS群聊服务')
    
    // 检查IPFS节点可用性
    await this.checkIpfsAvailability()
    
    // 从KV存储加载群聊数据
    await this.loadGroupsFromKV()
    
    this.initialized = true
    console.log('[LocalIpfsGroupChatService] 初始化完成')
  }

  /**
   * 设置本地身份
   */
  setLocalIdentity(identity: LocalIdentity | null): void {
    this.localIdentity = identity
    console.log('[LocalIpfsGroupChatService] 设置本地身份:', identity?.did)
  }

  /**
   * 检查IPFS可用性
   */
  async checkIpfsAvailability(): Promise<boolean> {
    try {
      // 尝试获取IPFS节点信息
      const result = await invoke('get_ipfs_info')
      this.ipfsAvailable = !!result
      console.log('[LocalIpfsGroupChatService] IPFS节点可用性:', this.ipfsAvailable)
      return this.ipfsAvailable
    } catch (error: any) {
      this.ipfsAvailable = false
      console.warn('[LocalIpfsGroupChatService] IPFS节点不可用:', error.message)
      return false
    }
  }

  /**
   * 生成群聊ID
   */
  generateGroupId(): string {
    return `local_group_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`
  }

  /**
   * 生成群聊主题
   */
  generateGroupTopic(groupId: string): string {
    return `alou/group/${groupId}`
  }

  /**
   * 创建本地群聊
   * @param config - 群聊配置
   * @returns 创建的群聊信息
   */
  async createGroup(config: CreateGroupConfig): Promise<LocalGroup> {
    if (!this.ipfsAvailable) {
      throw new Error('IPFS节点不可用，无法创建群聊')
    }

    if (!this.localIdentity) {
      throw new Error('未设置本地身份，无法创建群聊')
    }

    const groupId = this.generateGroupId()
    const topic = this.generateGroupTopic(groupId)
    const now = Date.now()

    // 创建群聊对象
    const group = new LocalGroup({
      groupId,
      groupName: config.groupName,
      description: config.description || '',
      topic,
      members: [this.localIdentity.did, ...(config.members || [])],
      creator: this.localIdentity.did,
      createdAt: now,
      metadata: {
        isPublic: config.isPublic || false,
        maxMembers: config.maxMembers || 50,
        ...config.metadata
      }
    })

    try {
      // 保存到内存
      this.groups.set(groupId, group)
      this.messages.set(groupId, [])

      // 保存到KV存储
      await this.saveGroupToKV(group)

      // 发送群聊创建消息到IPFS PubSub
      await this._sendSystemMessage(group, `群聊 "${config.groupName}" 已创建`)

      console.log('[LocalIpfsGroupChatService] 本地群聊创建成功:', group)
      return group

    } catch (error: any) {
      console.error('[LocalIpfsGroupChatService] 创建群聊失败:', error)
      // 清理内存中的数据
      this.groups.delete(groupId)
      this.messages.delete(groupId)
      throw new Error(`创建群聊失败: ${error.message}`)
    }
  }

  /**
   * 加入群聊
   * @param groupId - 群聊ID
   * @param topic - 群聊主题（可选）
   * @returns 加入的群聊信息
   */
  async joinGroup(groupId: string, topic?: string | null): Promise<LocalGroup> {
    if (!this.localIdentity) {
      throw new Error('未设置本地身份，无法加入群聊')
    }

    const group = this.groups.get(groupId)
    if (!group) {
      throw new Error('群聊不存在')
    }

    const groupTopic = topic || group.topic

    try {
      // 订阅群聊消息
      await this.subscribeToGroup(groupId, groupTopic)

      // 如果用户不在群聊成员列表中，添加进去
      if (!group.members.includes(this.localIdentity.did)) {
        group.members.push(this.localIdentity.did)
        await this.saveGroupToKV(group)
      }

      // 发送加入消息
      const joinMessage = new LocalGroupMessage({
        groupId,
        topic: groupTopic,
        from: this.localIdentity.did,
        fromName: this.localIdentity.name || this.localIdentity.did,
        content: `${this.localIdentity.did} 加入了群聊`,
        type: MessageTypes.JOIN,
        metadata: {
          type: 'member_joined',
          member: this.localIdentity.did
        }
      })

      await this.sendMessage(joinMessage)

      console.log('[LocalIpfsGroupChatService] 加入群聊成功:', groupId)
      return group

    } catch (error: any) {
      console.error('[LocalIpfsGroupChatService] 加入群聊失败:', error)
      throw new Error(`加入群聊失败: ${error.message}`)
    }
  }

  /**
   * 订阅群聊消息
   * @param groupId - 群聊ID
   * @param topic - 群聊主题
   * @returns 取消订阅函数
   */
  async subscribeToGroup(groupId: string, topic: string): Promise<UnsubscribeFunction> {
    if (this.subscriptions.has(groupId)) {
      // 已经订阅，返回现有的取消订阅函数
      return this.subscriptions.get(groupId)!
    }

    const messageHandlers = new Set<MessageHandler>()
    this.messageHandlers.set(groupId, messageHandlers)

    let isSubscribed = true

    const poll = async () => {
      if (!isSubscribed) return

      try {
        const messages = await invoke<string[]>('ipfs_pubsub_subscribe_once', {
          topic,
          timeoutMs: 1000,
          ipfs_api_url: DEFAULT_IPFS_API
        })

        if (messages && messages.length > 0) {
          messages.forEach((msgStr: string) => {
            try {
              const msgData = typeof msgStr === 'string' ? JSON.parse(msgStr) : msgStr
              const message = LocalGroupMessage.fromJSON(msgData)
              
              // 保存到内存
              this.saveMessageToMemory(groupId, message)
              
              // 保存到KV存储
              this.saveMessageToKV(groupId, message)
              
              // 通知所有处理器
              messageHandlers.forEach((handler: MessageHandler) => {
                try {
                  handler(message)
                } catch (error: any) {
                  console.error('[LocalIpfsGroupChatService] 消息处理器错误:', error)
                }
              })
            } catch (error: any) {
              console.error('[LocalIpfsGroupChatService] 解析消息失败:', error)
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
    const unsubscribe: UnsubscribeFunction = () => {
      isSubscribed = false
      clearInterval(interval)
      console.log('[LocalIpfsGroupChatService] 取消订阅:', groupId)
    }

    this.subscriptions.set(groupId, unsubscribe)

    console.log('[LocalIpfsGroupChatService] 订阅群聊成功:', groupId)
    return unsubscribe
  }

  /**
   * 发送消息到群聊
   * @param message - 消息对象或内容字符串
   * @param groupId - 群聊ID（如果message是字符串）
   * @param topic - 群聊主题（如果message是字符串）
   * @returns 发送是否成功
   */
  async sendMessage(
    message: LocalGroupMessage | string,
    groupId?: string | null,
    topic?: string | null
  ): Promise<boolean> {
    if (!this.ipfsAvailable) {
      throw new Error('IPFS节点不可用，无法发送消息')
    }

    if (!this.localIdentity) {
      throw new Error('未设置本地身份，无法发送消息')
    }

    let messageObj: LocalGroupMessage

    // 如果传入的是字符串，创建消息对象
    if (typeof message === 'string') {
      if (!groupId || !topic) {
        throw new Error('发送字符串消息需要提供groupId和topic')
      }
      messageObj = new LocalGroupMessage({
        groupId,
        topic,
        from: this.localIdentity.did,
        fromName: this.localIdentity.name || this.localIdentity.did,
        content: message,
        type: MessageTypes.CHAT
      })
    } else {
      messageObj = message
    }

    try {
      // 发布到IPFS PubSub
      await invoke('ipfs_pubsub_publish', {
        topic: messageObj.topic,
        message: JSON.stringify(messageObj.toJSON()),
        ipfs_api_url: DEFAULT_IPFS_API
      })

      // 保存到内存
      this.saveMessageToMemory(messageObj.groupId, messageObj)
      
      // 保存到KV存储
      await this.saveMessageToKV(messageObj.groupId, messageObj)

      console.log('[LocalIpfsGroupChatService] 消息发送成功:', messageObj.id)
      return true

    } catch (error: any) {
      console.error('[LocalIpfsGroupChatService] 发送消息失败:', error)
      throw new Error(`发送消息失败: ${error.message}`)
    }
  }

  /**
   * 发送系统消息
   */
  private async _sendSystemMessage(group: LocalGroup, content: string): Promise<void> {
    const systemMessage = new LocalGroupMessage({
      groupId: group.groupId,
      topic: group.topic,
      from: 'system',
      fromName: '系统',
      content,
      type: MessageTypes.SYSTEM,
      metadata: {
        type: 'system_message',
        group: group.groupId
      }
    })

    try {
      await this.sendMessage(systemMessage)
    } catch (error: any) {
      // 系统消息发送失败不影响群聊创建
      console.warn('[LocalIpfsGroupChatService] 系统消息发送失败:', error.message)
    }
  }

  /**
   * 保存消息到内存
   */
  saveMessageToMemory(groupId: string, message: LocalGroupMessage): void {
    if (!this.messages.has(groupId)) {
      this.messages.set(groupId, [])
    }
    
    const messages = this.messages.get(groupId)!
    
    // 检查消息是否已存在（避免重复）
    const existingIndex = messages.findIndex((msg: LocalGroupMessage) => msg.id === message.id)
    if (existingIndex === -1) {
      messages.push(message)
      
      // 限制内存中的消息数量（保留最新的100条）
      if (messages.length > 100) {
        messages.splice(0, messages.length - 100)
      }
    }
  }

  /**
   * 保存群聊到后端KV存储
   */
  async saveGroupToKV(group: LocalGroup): Promise<void> {
    try {
      // 使用后端API存储群聊数据
      const response = await apiClient.post<KVResponse>('/kv/set', {
        key: `group:${group.groupId}`,
        value: JSON.stringify(group.toJSON())
      })
      
      if (response.data.success) {
        console.log('[LocalIpfsGroupChatService] 群聊已保存到后端KV:', group.groupId)
      } else {
        throw new Error(response.data.error || '保存失败')
      }
    } catch (error: any) {
      console.warn('[LocalIpfsGroupChatService] 保存群聊到后端KV失败:', error.message)
    }
  }

  /**
   * 保存消息到后端KV存储
   */
  async saveMessageToKV(groupId: string, message: LocalGroupMessage): Promise<void> {
    try {
      // 使用后端API存储消息数据
      const response = await apiClient.post<KVResponse>('/kv/set', {
        key: `message:${groupId}:${message.id}`,
        value: JSON.stringify(message.toJSON())
      })
      
      if (response.data.success) {
        console.log('[LocalIpfsGroupChatService] 消息已保存到后端KV:', message.id)
      } else {
        throw new Error(response.data.error || '保存失败')
      }
    } catch (error: any) {
      console.warn('[LocalIpfsGroupChatService] 保存消息到后端KV失败:', error.message)
    }
  }

  /**
   * 从后端KV存储加载群聊数据
   */
  async loadGroupsFromKV(): Promise<void> {
    try {
      // 获取所有群聊相关的键
      const response = await apiClient.get<KVResponse>('/kv/keys', {
        params: { prefix: 'group:' }
      })
      
      if (response.data.success && response.data.keys) {
        console.log(`[LocalIpfsGroupChatService] 找到 ${response.data.keys.length} 个群聊数据`)
        
        for (const key of response.data.keys) {
          try {
            const valueResponse = await apiClient.get<KVResponse>('/kv/get', {
              params: { key }
            })
            
            if (valueResponse.data.success && valueResponse.data.value) {
              const group = LocalGroup.fromJSON(JSON.parse(valueResponse.data.value))
              this.groups.set(group.groupId, group)
              
              // 同时加载该群聊的消息
              await this.loadMessagesFromKV(group.groupId)
            }
          } catch (error: any) {
            console.warn(`[LocalIpfsGroupChatService] 加载群聊 ${key} 失败:`, error.message)
          }
        }
        
        console.log(`[LocalIpfsGroupChatService] 成功加载 ${this.groups.size} 个群聊`)
      }
    } catch (error: any) {
      console.warn('[LocalIpfsGroupChatService] 从后端KV加载群聊数据失败:', error.message)
    }
  }

  /**
   * 从后端KV存储加载群聊消息
   */
  async loadMessagesFromKV(groupId: string): Promise<void> {
    try {
      // 获取该群聊的所有消息键
      const response = await apiClient.get<KVResponse>('/kv/keys', {
        params: { prefix: `message:${groupId}:` }
      })
      
      if (response.data.success && response.data.keys) {
        const messages: LocalGroupMessage[] = []
        for (const key of response.data.keys) {
          try {
            const valueResponse = await apiClient.get<KVResponse>('/kv/get', {
              params: { key }
            })
            
            if (valueResponse.data.success && valueResponse.data.value) {
              const message = LocalGroupMessage.fromJSON(JSON.parse(valueResponse.data.value))
              messages.push(message)
            }
          } catch (error: any) {
            console.warn(`[LocalIpfsGroupChatService] 加载消息 ${key} 失败:`, error.message)
          }
        }
        
        // 按时间戳排序
        messages.sort((a: LocalGroupMessage, b: LocalGroupMessage) => a.timestamp - b.timestamp)
        
        // 限制内存中的消息数量
        if (messages.length > 100) {
          messages.splice(0, messages.length - 100)
        }
        
        this.messages.set(groupId, messages)
        console.log(`[LocalIpfsGroupChatService] 加载群聊 ${groupId} 的 ${messages.length} 条消息`)
      }
    } catch (error: any) {
      console.warn(`[LocalIpfsGroupChatService] 加载群聊 ${groupId} 消息失败:`, error.message)
    }
  }

  /**
   * 添加消息处理器
   */
  addMessageHandler(groupId: string, handler: MessageHandler): void {
    if (!this.messageHandlers.has(groupId)) {
      this.messageHandlers.set(groupId, new Set())
    }
    this.messageHandlers.get(groupId)!.add(handler)
  }

  /**
   * 移除消息处理器
   */
  removeMessageHandler(groupId: string, handler: MessageHandler): void {
    const handlers = this.messageHandlers.get(groupId)
    if (handlers) {
      handlers.delete(handler)
    }
  }

  /**
   * 获取群聊消息
   */
  getGroupMessages(groupId: string, limit: number = 50): LocalGroupMessage[] {
    const messages = this.messages.get(groupId) || []
    return messages.slice(-limit) // 返回最新的消息
  }

  /**
   * 获取群聊信息
   */
  getGroup(groupId: string): LocalGroup | null {
    return this.groups.get(groupId) || null
  }

  /**
   * 获取所有群聊
   */
  getAllGroups(): LocalGroup[] {
    return Array.from(this.groups.values())
  }

  /**
   * 离开群聊
   */
  async leaveGroup(groupId: string): Promise<void> {
    try {
      const group = this.groups.get(groupId)
      if (group && this.localIdentity) {
        // 发送离开消息
        const leaveMessage = new LocalGroupMessage({
          groupId,
          topic: group.topic,
          from: this.localIdentity.did,
          fromName: this.localIdentity.name || this.localIdentity.did,
          content: `${this.localIdentity.did} 离开了群聊`,
          type: MessageTypes.LEAVE,
          metadata: {
            type: 'member_left',
            member: this.localIdentity.did
          }
        })

        await this.sendMessage(leaveMessage)

        // 从成员列表中移除
        const memberIndex = group.members.indexOf(this.localIdentity.did)
        if (memberIndex > -1) {
          group.members.splice(memberIndex, 1)
          await this.saveGroupToKV(group)
        }
      }

      // 取消订阅
      const unsubscribe = this.subscriptions.get(groupId)
      if (unsubscribe) {
        unsubscribe()
        this.subscriptions.delete(groupId)
      }

      // 清理消息处理器
      this.messageHandlers.delete(groupId)

      // 从内存中移除群聊（但保留消息历史）
      this.groups.delete(groupId)

      console.log('[LocalIpfsGroupChatService] 离开群聊成功:', groupId)

    } catch (error: any) {
      console.error('[LocalIpfsGroupChatService] 离开群聊失败:', error)
      throw new Error(`离开群聊失败: ${error.message}`)
    }
  }

  /**
   * 清理所有订阅
   */
  cleanup(): void {
    // 取消所有订阅
    this.subscriptions.forEach((unsubscribe: UnsubscribeFunction, groupId: string) => {
      try {
        unsubscribe()
      } catch (error: any) {
        console.error('[LocalIpfsGroupChatService] 清理订阅失败:', groupId, error)
      }
    })

    // 清理所有数据
    this.subscriptions.clear()
    this.messageHandlers.clear()
    this.groups.clear()
    this.messages.clear()

    console.log('[LocalIpfsGroupChatService] 已清理所有群聊订阅')
  }
}

export default new LocalIpfsGroupChatService()
