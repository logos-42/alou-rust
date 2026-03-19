/**
 * 本地IPFS PubSub群聊服务
 * 使用本地IPFS节点创建pubsub，消息存储在本地内存、后端KV和LocalStorage中
 */

import { invoke } from '@tauri-apps/api/core'

const DEFAULT_IPFS_API = import.meta.env.VITE_IPFS_API_URL || 'http://127.0.0.1:5001'

/**
 * 日志级别
 */
const LogLevel = {
  DEBUG: 'DEBUG',
  INFO: 'INFO',
  WARN: 'WARN',
  ERROR: 'ERROR',
} as const

type LogLevelType = typeof LogLevel[keyof typeof LogLevel]

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
  ACK: 'ack',                         // 消息确认
  REPLAY: 'replay',                   // 消息重放请求
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
  // 消息状态跟踪
  delivered: boolean
  acked: boolean
  retryCount: number

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
    // 初始化消息状态
    this.delivered = metadata?.delivered || false
    this.acked = metadata?.acked || false
    this.retryCount = metadata?.retryCount || 0
  }

  toJSON(): LocalGroupMessageConfig & { delivered: boolean; acked: boolean; retryCount: number } {
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
      metadata: this.metadata,
      delivered: this.delivered,
      acked: this.acked,
      retryCount: this.retryCount,
    }
  }

  static fromJSON(json: LocalGroupMessageConfig & { delivered?: boolean; acked?: boolean; retryCount?: number }): LocalGroupMessage {
    const msg = new LocalGroupMessage(json)
    msg.delivered = json.delivered || false
    msg.acked = json.acked || false
    msg.retryCount = json.retryCount || 0
    return msg
  }
}

/**
 * 智能体信息接口
 */
export interface AgentInfo {
  id?: string
  name?: string
  did?: string
  ipns?: string
  cid?: string
  avatar?: string
  agent_id?: string
  agent_name?: string
  mode?: string
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
  agents?: AgentInfo[]
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
  agents: AgentInfo[]

  constructor({
    groupId,
    groupName,
    description = '',
    topic,
    members = [],
    creator,
    createdAt,
    metadata = {},
    agents = []
  }: LocalGroupConfig) {
    this.groupId = groupId
    this.groupName = groupName
    this.description = description
    this.topic = topic
    this.members = members
    this.creator = creator
    this.createdAt = createdAt
    this.metadata = metadata
    this.agents = agents
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
      metadata: this.metadata,
      agents: this.agents
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
  agents?: AgentInfo[]
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
 * 分页配置
 */
export interface PaginationConfig {
  page: number
  pageSize: number
}

/**
 * 消息查询结果
 */
export interface MessageQueryResult {
  messages: LocalGroupMessage[]
  total: number
  hasMore: boolean
}

/**
 * 本地IPFS群聊服务类
 * 专注于使用本地IPFS节点进行PubSub通信
 * 包含消息去重、顺序保证、错误重试和消息确认机制
 */
class LocalIpfsGroupChatService {
  private localIdentity: LocalIdentity | null = null
  private groups: Map<string, LocalGroup> = new Map() // groupId -> LocalGroup (内存存储)
  private messages: Map<string, LocalGroupMessage[]> = new Map() // groupId -> LocalGroupMessage[] (内存存储)
  private subscriptions: Map<string, UnsubscribeFunction> = new Map() // groupId -> unsubscribe function
  private messageHandlers: Map<string, Set<MessageHandler>> = new Map() // groupId -> Set<callback>
  private ipfsAvailable: boolean = false
  private initialized: boolean = false
  
  // 消息去重相关
  private messageIdSet: Map<string, Set<string>> = new Map() // groupId -> Set<messageId>
  private messageHashSet: Map<string, Set<string>> = new Map() // groupId -> Set<contentHash>
  
  // 错误重试相关
  private retryQueue: Map<string, { message: LocalGroupMessage; retries: number; lastAttempt: number }> = new Map()
  private readonly MAX_RETRY_ATTEMPTS = 3
  private readonly RETRY_DELAY_MS = 2000
  private retryTimer: NodeJS.Timeout | null = null
  
  // 消息确认相关
  private pendingAcks: Map<string, { resolve: () => void; reject: (error: Error) => void; timeout: NodeJS.Timeout }> = new Map()
  private readonly ACK_TIMEOUT_MS = 10000
  
  // 日志相关
  private logLevel: LogLevelType = LogLevel.INFO
  private logs: Array<{ level: LogLevelType; message: string; timestamp: number; data?: any }> = []
  private readonly MAX_LOGS = 1000

  /**
   * 记录日志
   */
  private log(level: LogLevelType, message: string, data?: any): void {
    const logEntry = {
      level,
      message,
      timestamp: Date.now(),
      data
    }
    
    this.logs.push(logEntry)
    
    // 限制日志数量
    if (this.logs.length > this.MAX_LOGS) {
      this.logs.shift()
    }
    
    // 控制台输出
    const prefix = `[LocalIpfsGroupChatService][${level}]`
    switch (level) {
      case LogLevel.DEBUG:
        if (this.logLevel === LogLevel.DEBUG) {
          console.debug(prefix, message, data || '')
        }
        break
      case LogLevel.INFO:
        console.log(prefix, message, data || '')
        break
      case LogLevel.WARN:
        console.warn(prefix, message, data || '')
        break
      case LogLevel.ERROR:
        console.error(prefix, message, data || '')
        break
    }
  }

  /**
   * 设置日志级别
   */
  setLogLevel(level: LogLevelType): void {
    this.logLevel = level
    this.log(LogLevel.INFO, '日志级别设置为:', level)
  }

  /**
   * 获取日志
   */
  getLogs(level?: LogLevelType, limit: number = 100): Array<{ level: LogLevelType; message: string; timestamp: number; data?: any }> {
    let filteredLogs = this.logs
    if (level) {
      filteredLogs = this.logs.filter(log => log.level === level)
    }
    return filteredLogs.slice(-limit)
  }

  /**
   * 生成消息内容的哈希值（用于去重）
   */
  private generateMessageHash(message: LocalGroupMessage): string {
    const content = `${message.from}:${message.content}:${message.timestamp}`
    // 简单的哈希算法
    let hash = 0
    for (let i = 0; i < content.length; i++) {
      const char = content.charCodeAt(i)
      hash = ((hash << 5) - hash) + char
      hash = hash & hash // 转换为32位整数
    }
    return `hash_${hash}_${message.timestamp}`
  }

  /**
   * 检查消息是否重复
   */
  private isDuplicateMessage(groupId: string, message: LocalGroupMessage): boolean {
    const idSet = this.messageIdSet.get(groupId)
    if (idSet && idSet.has(message.id)) {
      this.log(LogLevel.DEBUG, '消息ID重复，忽略消息:', { groupId, messageId: message.id })
      return true
    }
    
    // 检查内容哈希（短时间内相同内容的消息）
    const hash = this.generateMessageHash(message)
    const hashSet = this.messageHashSet.get(groupId)
    if (hashSet) {
      // 清理超过1分钟的旧哈希
      const now = Date.now()
      for (const entry of Array.from(hashSet.entries())) {
        const key = entry[0]
        const timestampStr = key.split('_').slice(-1)[0]
        const timestamp = parseInt(timestampStr)
        if (now - (timestamp as number) > 60000) {
          hashSet.delete(key as string)
        }
      }
      
      if (hashSet.has(hash)) {
        this.log(LogLevel.DEBUG, '消息内容重复，忽略消息:', { groupId, hash })
        return true
      }
    }
    
    return false
  }

  /**
   * 记录消息ID和哈希
   */
  private recordMessageId(groupId: string, message: LocalGroupMessage): void {
    if (!this.messageIdSet.has(groupId)) {
      this.messageIdSet.set(groupId, new Set())
    }
    this.messageIdSet.get(groupId)!.add(message.id)
    
    if (!this.messageHashSet.has(groupId)) {
      this.messageHashSet.set(groupId, new Set())
    }
    const hash = this.generateMessageHash(message)
    this.messageHashSet.get(groupId)!.add(hash)
  }

  /**
   * 初始化服务
   * 注意：此方法不再自动加载群聊数据，需要手动调用 loadGroupsFromKV() 或 loadGroup() 按需加载
   */
  async initialize(): Promise<void> {
    if (this.initialized) {
      return
    }

    this.log(LogLevel.INFO, '初始化本地IPFS群聊服务')
    // 检查 IPFS 节点可用性
    await this.checkIpfsAvailability()

    // 从LocalStorage加载备份数据
    await this.loadFromLocalStorage()
    
    // 启动重试处理器
    this.startRetryProcessor()
    
    this.initialized = true
    this.log(LogLevel.INFO, '初始化完成', { 
      groupsCount: this.groups.size,
      ipfsAvailable: this.ipfsAvailable 
    })
  }

  /**
   * 设置本地身份
   */
  setLocalIdentity(identity: LocalIdentity | null): void {
    this.localIdentity = identity
    this.log(LogLevel.INFO, '设置本地身份:', { did: identity?.did })
  }

  /**
   * 检查IPFS可用性
   */
  async checkIpfsAvailability(): Promise<boolean> {
    try {
      // 尝试获取IPFS节点信息
      const result = await invoke('get_ipfs_info')
      this.ipfsAvailable = !!result
      this.log(LogLevel.INFO, 'IPFS节点可用性检查:', { available: this.ipfsAvailable })
      return this.ipfsAvailable
    } catch (error: any) {
      this.ipfsAvailable = false
      this.log(LogLevel.WARN, 'IPFS节点不可用:', { error: error.message })
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
   * @param forceMemoryMode - 是否强制使用内存模式（当IPFS不可用时）
   * @returns 创建的群聊信息
   */
  async createGroup(config: CreateGroupConfig, forceMemoryMode: boolean = false): Promise<LocalGroup> {
    this.log(LogLevel.INFO, '创建群聊:', { groupName: config.groupName, forceMemoryMode })
    
    // 如果不是强制内存模式且IPFS不可用，则抛出错误
    if (!forceMemoryMode && !this.ipfsAvailable) {
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
        memoryMode: forceMemoryMode || !this.ipfsAvailable, // 标记是否为内存模式
        ...config.metadata
      },
      agents: config.agents || []
    })

    try {
      // 保存到内存（不依赖IPFS）
      this.groups.set(groupId, group)
      this.messages.set(groupId, [])
      this.messageIdSet.set(groupId, new Set())
      this.messageHashSet.set(groupId, new Set())

      // 保存到KV存储（需要后端，如果失败则跳过）
      try {
        await this.saveGroupToKV(group)
      } catch (kvError) {
        this.log(LogLevel.WARN, '保存到KV失败，使用内存模式:', { error: kvError })
      }
      
      // 保存到LocalStorage备份（不依赖IPFS）
      try {
        await this.saveToLocalStorage()
      } catch (lsError) {
        this.log(LogLevel.WARN, '保存到LocalStorage失败:', { error: lsError })
      }

      // 仅在IPFS可用时发送群聊创建消息到IPFS PubSub
      if (this.ipfsAvailable) {
        try {
          await this._sendSystemMessage(group, `群聊 "${config.groupName}" 已创建`)
        } catch (pubsubError) {
          this.log(LogLevel.WARN, '发送PubSub消息失败:', { error: pubsubError })
        }
      }

      this.log(LogLevel.INFO, '群聊创建成功:', { groupId, groupName: group.groupName, memoryMode: forceMemoryMode || !this.ipfsAvailable })
      return group

    } catch (error: any) {
      this.log(LogLevel.ERROR, '创建群聊失败:', { error: error.message })
      // 清理内存中的数据
      this.groups.delete(groupId)
      this.messages.delete(groupId)
      this.messageIdSet.delete(groupId)
      this.messageHashSet.delete(groupId)
      throw new Error(`创建群聊失败: ${error.message}`)
    }
  }

  /**
   * 加入群聊
   * @param groupId - 群聊ID
   * @param topic - 群聊主题（可选）
   * @param forceMemoryMode - 是否强制使用内存模式
   * @returns 加入的群聊信息
   */
  async joinGroup(groupId: string, topic?: string | null, forceMemoryMode: boolean = false): Promise<LocalGroup> {
    this.log(LogLevel.INFO, '加入群聊:', { groupId, forceMemoryMode })

    if (!this.localIdentity) {
      throw new Error('未设置本地身份，无法加入群聊')
    }

    // 1. 首先尝试从内存获取群聊
    let group: LocalGroup | undefined = this.groups.get(groupId)
    
    // 2. 如果内存中没有，尝试从 KV 加载
    if (!group) {
      this.log(LogLevel.INFO, '内存中未找到群聊，尝试从 KV 加载:', { groupId })
      try {
        const groupFromKV = await this.loadGroupFromKV(groupId)
        if (groupFromKV) {
          group = groupFromKV
        }
      } catch (kvError) {
        this.log(LogLevel.WARN, '从KV加载群聊失败:', { error: kvError })
      }
    }
    
    // 3. 如果 KV 中也没有，创建新的群聊对象（用于加入外部创建的群聊）
    if (!group) {
      this.log(LogLevel.INFO, 'KV 中也未找到群聊，创建新群聊对象:', { groupId })
      const groupTopic = topic || `diap/cluster_action/${groupId}`
      group = new LocalGroup({
        groupId,
        groupName: `群聊 ${groupId.slice(-8)}`,
        description: '',
        topic: groupTopic,
        members: [],
        creator: 'unknown',
        createdAt: Date.now(),
        metadata: {
          isPublic: true,
          externalGroup: true,
          memoryMode: forceMemoryMode || !this.ipfsAvailable
        }
      })
      
      // 保存到新创建的群聊到 KV（失败则跳过）
      try {
        await this.saveGroupToKV(group)
      } catch (kvError) {
        this.log(LogLevel.WARN, '保存群聊到KV失败:', { error: kvError })
      }
    }

    const groupTopic = topic || group.topic

    try {
      // 仅在IPFS可用时订阅群聊消息
      if (this.ipfsAvailable) {
        try {
          await this.subscribeToGroup(groupId, groupTopic)
        } catch (subError) {
          this.log(LogLevel.WARN, '订阅群聊消息失败:', { error: subError })
        }
      }

      // 如果用户不在群聊成员列表中，添加进去
      if (!group.members.includes(this.localIdentity.did)) {
        group.members.push(this.localIdentity.did)
        try {
          await this.saveGroupToKV(group)
        } catch (kvError) {
          this.log(LogLevel.WARN, '更新群聊成员到KV失败:', { error: kvError })
        }
      }

      // 仅在IPFS可用时发送加入消息
      if (this.ipfsAvailable) {
        try {
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
        } catch (msgError) {
          this.log(LogLevel.WARN, '发送加入消息失败:', { error: msgError })
        }
      }

      // 保存到内存
      this.groups.set(groupId, group)

      this.log(LogLevel.INFO, '加入群聊成功:', { groupId, member: this.localIdentity.did })
      return group

    } catch (error: any) {
      this.log(LogLevel.ERROR, '加入群聊失败:', { groupId, error: error.message })
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
      this.log(LogLevel.DEBUG, '群聊已订阅，返回现有订阅:', { groupId })
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
          for (const msgStr of messages) {
            try {
              const msgData = typeof msgStr === 'string' ? JSON.parse(msgStr) : msgStr
              const message = LocalGroupMessage.fromJSON(msgData)
              
              this.log(LogLevel.DEBUG, '收到PubSub消息:', { 
                groupId, 
                messageId: message.id,
                type: message.type 
              })
              
              // 消息去重检查
              if (this.isDuplicateMessage(groupId, message)) {
                continue
              }
              
              // 记录消息ID
              this.recordMessageId(groupId, message)
              
              // 处理确认消息
              if (message.type === MessageTypes.ACK) {
                this.handleAckMessage(message)
                continue
              }
              
              // 发送确认（如果不是自己的消息且不是系统消息）
              if (message.from !== this.localIdentity?.did && message.type !== MessageTypes.SYSTEM) {
                this.sendAck(message)
              }
              
              // 保存到内存
              this.saveMessageToMemory(groupId, message)
              
              // 保存到KV存储
              this.saveMessageToKV(groupId, message)
              
              // 保存到LocalStorage备份
              this.saveMessageToLocalStorage(groupId, message)
              
              // 通知所有处理器
              messageHandlers.forEach((handler: MessageHandler) => {
                try {
                  handler(message)
                } catch (error: any) {
                  this.log(LogLevel.ERROR, '消息处理器错误:', { error: error.message })
                }
              })

              // 关键改进：触发智能体自主响应
              // 当收到新消息时，通知群聊中的所有智能体
              try {
                // 动态导入 clusterActionStore 以避免循环依赖
                const clusterActionStore = await import('@/stores/clusterActionStore').then(m => m.default)
                const { getActions } = clusterActionStore.getState()
                const actions = getActions('') || []
                
                // 查找群聊对应的行动
                const groupAction = actions.find(action => 
                  action.action_id === groupId || 
                  action.action_id?.includes(groupId)
                )
                
                if (groupAction && groupAction.agents && groupAction.agents.length > 0) {
                  // 通知每个智能体处理消息
                  const agentCount = groupAction.agents
                    .filter(agent => agent.mode === 'agent')
                    .length
                  
                  this.log(LogLevel.INFO, '触发群聊智能体响应:', {
                    groupId,
                    agentCount,
                    totalAgents: groupAction.agents.length
                  })
                  
                  groupAction.agents
                    .filter(agent => agent.mode === 'agent')
                    .forEach(agent => {
                      const agentId = agent.id || agent.agent_id || agent.did
                      if (agentId && typeof window !== 'undefined') {
                        // 触发智能体处理消息的事件
                        window.dispatchEvent(new CustomEvent('agent-group-message', {
                          detail: {
                            agentId,
                            message: {
                              id: message.id,
                              groupId,
                              from: message.from,
                              fromName: message.fromName || message.from,
                              content: message.content,
                              timestamp: message.timestamp,
                              type: message.type,
                              isMentioned: message.content?.includes('@'),
                              metadata: {
                                isGroupChat: true,
                                groupType: 'local_ipfs_pubsub',
                                topic: message.topic
                              }
                            }
                          }
                        }))
                      }
                    })
                  
                  this.log(LogLevel.INFO, '已通知智能体处理群聊消息:', { agentCount })
                }
              } catch (agentError) {
                this.log(LogLevel.WARN, '触发智能体响应失败:', { error: agentError })
              }
            } catch (error: any) {
              this.log(LogLevel.ERROR, '解析消息失败:', { error: error.message, msgStr })
            }
          }
        }
      } catch (error: any) {
        // IPFS错误时静默处理，避免频繁日志
        if (error.message?.includes('timeout')) {
          // 超时是正常的，不记录
        } else {
          this.log(LogLevel.DEBUG, 'PubSub轮询错误:', { error: error.message })
        }
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
      this.log(LogLevel.INFO, '取消订阅群聊:', { groupId })
    }

    this.subscriptions.set(groupId, unsubscribe)

    this.log(LogLevel.INFO, '订阅群聊成功:', { groupId, topic })
    return unsubscribe
  }

  /**
   * 发送确认消息
   */
  private async sendAck(originalMessage: LocalGroupMessage): Promise<void> {
    if (!this.localIdentity) return
    
    const ackMessage = new LocalGroupMessage({
      groupId: originalMessage.groupId,
      topic: originalMessage.topic,
      from: this.localIdentity.did,
      fromName: this.localIdentity.name || this.localIdentity.did,
      content: 'ack',
      type: MessageTypes.ACK,
      metadata: {
        ackedMessageId: originalMessage.id,
        ackedBy: this.localIdentity.did
      }
    })
    
    try {
      await invoke('ipfs_pubsub_publish', {
        topic: ackMessage.topic,
        message: JSON.stringify(ackMessage.toJSON()),
        ipfs_api_url: DEFAULT_IPFS_API
      })
    } catch (error: any) {
      this.log(LogLevel.DEBUG, '发送确认消息失败:', { error: error.message })
    }
  }

  /**
   * 处理确认消息
   */
  private handleAckMessage(ackMessage: LocalGroupMessage): void {
    const ackedMessageId = ackMessage.metadata?.ackedMessageId
    if (ackedMessageId && this.pendingAcks.has(ackedMessageId)) {
      const pending = this.pendingAcks.get(ackedMessageId)!
      clearTimeout(pending.timeout)
      this.pendingAcks.delete(ackedMessageId)
      pending.resolve()
      this.log(LogLevel.DEBUG, '收到消息确认:', { messageId: ackedMessageId })
    }
  }

  /**
   * 发送消息到群聊（带确认机制）
   * @param message - 消息对象或内容字符串
   * @param groupId - 群聊ID（如果message是字符串）
   * @param topic - 群聊主题（如果message是字符串）
   * @param waitForAck - 是否等待确认
   * @returns 发送是否成功
   */
  async sendMessage(
    message: LocalGroupMessage | string,
    groupId?: string | null,
    topic?: string | null,
    waitForAck: boolean = false
  ): Promise<boolean> {
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

    this.log(LogLevel.INFO, '发送消息:', { 
      messageId: messageObj.id, 
      groupId: messageObj.groupId,
      type: messageObj.type,
      ipfsAvailable: this.ipfsAvailable
    })

    try {
      // 如果 IPFS 可用，发布到 IPFS PubSub
      if (this.ipfsAvailable) {
        try {
          await invoke('ipfs_pubsub_publish', {
            topic: messageObj.topic,
            message: JSON.stringify(messageObj.toJSON()),
            ipfs_api_url: DEFAULT_IPFS_API
          })
          this.log(LogLevel.INFO, 'IPFS 发布成功:', { messageId: messageObj.id })
        } catch (ipfsError: any) {
          this.log(LogLevel.WARN, 'IPFS 发布失败，降级到内存模式:', { 
            messageId: messageObj.id, 
            error: ipfsError.message 
          })
          // 继续执行，保存到内存和 KV
        }
      } else {
        this.log(LogLevel.INFO, 'IPFS 不可用，使用内存模式:', { messageId: messageObj.id })
      }

      // 无论 IPFS 是否可用，都保存到本地
      // 保存到内存
      this.saveMessageToMemory(messageObj.groupId, messageObj)
      
      // 保存到KV存储
      await this.saveMessageToKV(messageObj.groupId, messageObj)
      
      // 保存到LocalStorage备份
      this.saveMessageToLocalStorage(messageObj.groupId, messageObj)
      
      // 记录消息ID
      this.recordMessageId(messageObj.groupId, messageObj)

      // 标记为已发送
      messageObj.delivered = true

      // 通知所有处理器 - 确保发送消息后立即更新UI
      const messageHandlers = this.messageHandlers.get(messageObj.groupId)
      if (messageHandlers) {
        messageHandlers.forEach((handler: MessageHandler) => {
          try {
            handler(messageObj)
          } catch (error: any) {
            this.log(LogLevel.ERROR, '消息处理器错误:', { error: error.message })
          }
        })
      }

      // 如果需要等待确认（仅在 IPFS 可用时）
      if (waitForAck && this.ipfsAvailable) {
        await this.waitForAck(messageObj.id)
      }

      this.log(LogLevel.INFO, '消息发送成功:', { 
        messageId: messageObj.id,
        mode: this.ipfsAvailable ? 'IPFS' : '内存'
      })
      return true

    } catch (error: any) {
      this.log(LogLevel.ERROR, '发送消息失败:', { 
        messageId: messageObj.id, 
        error: error.message 
      })
      
      // 如果 IPFS 可用，加入重试队列
      if (this.ipfsAvailable) {
        this.addToRetryQueue(messageObj)
      }
      
      throw new Error(`发送消息失败: ${error.message}`)
    }
  }

  /**
   * 等待消息确认
   */
  private waitForAck(messageId: string): Promise<void> {
    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        this.pendingAcks.delete(messageId)
        reject(new Error('消息确认超时'))
      }, this.ACK_TIMEOUT_MS)
      
      this.pendingAcks.set(messageId, { resolve, reject, timeout })
    })
  }

  /**
   * 添加消息到重试队列
   */
  private addToRetryQueue(message: LocalGroupMessage): void {
    this.retryQueue.set(message.id, {
      message,
      retries: 0,
      lastAttempt: Date.now()
    })
    this.log(LogLevel.INFO, '消息加入重试队列:', { messageId: message.id })
  }

  /**
   * 启动重试处理器
   */
  private startRetryProcessor(): void {
    if (this.retryTimer) {
      clearInterval(this.retryTimer)
    }
    
    this.retryTimer = setInterval(() => {
      this.processRetryQueue()
    }, this.RETRY_DELAY_MS)
    
    this.log(LogLevel.INFO, '重试处理器已启动')
  }

  /**
   * 处理重试队列
   */
  private async processRetryQueue(): Promise<void> {
    if (this.retryQueue.size === 0) return
    if (!this.ipfsAvailable) return
    
    const now = Date.now()
    const toRemove: string[] = []
    
    for (const [messageId, item] of this.retryQueue.entries()) {
      // 检查是否到达重试间隔
      if (now - item.lastAttempt < this.RETRY_DELAY_MS) {
        continue
      }
      
      // 检查是否超过最大重试次数
      if (item.retries >= this.MAX_RETRY_ATTEMPTS) {
        this.log(LogLevel.ERROR, '消息重试次数超限，放弃发送:', { messageId, retries: item.retries })
        toRemove.push(messageId)
        continue
      }
      
      try {
        item.retries++
        item.lastAttempt = now
        item.message.retryCount = item.retries
        
        this.log(LogLevel.INFO, '重试发送消息:', { messageId, attempt: item.retries })
        
        // 重新发送
        await invoke('ipfs_pubsub_publish', {
          topic: item.message.topic,
          message: JSON.stringify(item.message.toJSON()),
          ipfs_api_url: DEFAULT_IPFS_API
        })
        
        // 发送成功，从队列移除
        toRemove.push(messageId)
        
        // 保存到存储
        this.saveMessageToMemory(item.message.groupId, item.message)
        await this.saveMessageToKV(item.message.groupId, item.message)
        this.saveMessageToLocalStorage(item.message.groupId, item.message)
        
        this.log(LogLevel.INFO, '消息重试发送成功:', { messageId })
        
      } catch (error: any) {
        this.log(LogLevel.WARN, '消息重试发送失败:', { 
          messageId, 
          attempt: item.retries, 
          error: error.message 
        })
      }
    }
    
    // 移除已处理的消息
    for (const messageId of toRemove) {
      this.retryQueue.delete(messageId)
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
      this.log(LogLevel.WARN, '系统消息发送失败:', { error: error.message })
    }
  }

  /**
   * 保存消息到内存（带去重和排序）
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
      
      // 按时间戳排序，确保消息顺序
      messages.sort((a: LocalGroupMessage, b: LocalGroupMessage) => a.timestamp - b.timestamp)
      
      // 限制内存中的消息数量（保留最新的200条）
      if (messages.length > 200) {
        messages.splice(0, messages.length - 200)
      }
      
      this.log(LogLevel.DEBUG, '消息保存到内存:', { 
        groupId, 
        messageId: message.id,
        totalMessages: messages.length 
      })
    }
  }

  /**
   * 保存群聊到 Tauri KV 存储
   */
  async saveGroupToKV(group: LocalGroup): Promise<void> {
    try {
      await invoke('kv_set', {
        key: `group:${group.groupId}`,
        value: JSON.stringify(group.toJSON())
      })
      this.log(LogLevel.DEBUG, '群聊已保存到KV:', { groupId: group.groupId })
    } catch (error: any) {
      this.log(LogLevel.WARN, '保存群聊到KV失败:', { 
        groupId: group.groupId, 
        error: error.message 
      })
    }
  }

  /**
   * 保存消息到 Tauri KV 存储
   */
  async saveMessageToKV(groupId: string, message: LocalGroupMessage): Promise<void> {
    try {
      await invoke('kv_set', {
        key: `message:${groupId}:${message.id}`,
        value: JSON.stringify(message.toJSON())
      })
    } catch (error: any) {
      this.log(LogLevel.WARN, '保存消息到KV失败:', { 
        groupId, 
        messageId: message.id, 
        error: error.message 
      })
    }
  }

  /**
   * 保存消息到 LocalStorage（关键信息备份）
   */
  private saveMessageToLocalStorage(groupId: string, message: LocalGroupMessage): void {
    try {
      const storageKey = `alou:groupchat:messages:${groupId}`
      const existing = localStorage.getItem(storageKey)
      let messages: any[] = existing ? JSON.parse(existing) : []
      
      // 检查是否已存在
      if (!messages.find((m: any) => m.id === message.id)) {
        messages.push({
          id: message.id,
          groupId: message.groupId,
          from: message.from,
          content: message.content.substring(0, 500), // 限制内容长度
          type: message.type,
          timestamp: message.timestamp
        })
        
        // 限制数量
        if (messages.length > 500) {
          messages = messages.slice(-500)
        }
        
        localStorage.setItem(storageKey, JSON.stringify(messages))
      }
    } catch (error: any) {
      this.log(LogLevel.WARN, '保存消息到LocalStorage失败:', { error: error.message })
    }
  }

  /**
   * 保存群聊信息到 LocalStorage
   */
  private async saveToLocalStorage(): Promise<void> {
    try {
      const groups = Array.from(this.groups.values()).map(g => ({
        groupId: g.groupId,
        groupName: g.groupName,
        description: g.description,
        topic: g.topic,
        memberCount: g.members.length,
        createdAt: g.createdAt
      }))
      
      localStorage.setItem('alou:groupchat:groups', JSON.stringify(groups))
      this.log(LogLevel.DEBUG, '群聊信息已保存到LocalStorage:', { count: groups.length })
    } catch (error: any) {
      this.log(LogLevel.WARN, '保存群聊到LocalStorage失败:', { error: error.message })
    }
  }

  /**
   * 从 Tauri KV 存储加载群聊数据
   */
  
  /**
   * 从 Tauri KV 存储加载单个群聊
   * @param groupId - 群聊 ID
   * @returns 加载的群聊信息，如果不存在则返回 null
   */
  async loadGroupFromKV(groupId: string): Promise<LocalGroup | null> {
    try {
      const key = `group:${groupId}`
      const value: string | null = await invoke('kv_get', { key })
      
      if (value) {
        const group = LocalGroup.fromJSON(JSON.parse(value))
        this.log(LogLevel.INFO, '从 KV 加载群聊成功:', { groupId })
        
        // 保存到内存
        this.groups.set(groupId, group)
        
        // 初始化消息 ID 集合
        this.messageIdSet.set(groupId, new Set())
        this.messageHashSet.set(groupId, new Set())
        
        return group
      }
      
      this.log(LogLevel.DEBUG, 'KV 中未找到群聊:', { groupId })
      return null
    } catch (error: any) {
      this.log(LogLevel.WARN, '从 KV 加载群聊失败:', { groupId, error: error.message })
      return null
    }
  }

async loadGroupsFromKV(): Promise<void> {
    try {
      const keys: string[] = await invoke('kv_keys', { prefix: 'group:' })
      this.log(LogLevel.INFO, '从KV加载群聊数据:', { count: keys.length })

      for (const key of keys) {
        try {
          const value: string | null = await invoke('kv_get', { key })
          if (value) {
            const group = LocalGroup.fromJSON(JSON.parse(value))
            this.groups.set(group.groupId, group)
            
            // 初始化消息ID集合
            this.messageIdSet.set(group.groupId, new Set())
            this.messageHashSet.set(group.groupId, new Set())
            
            await this.loadMessagesFromKV(group.groupId)
          }
        } catch (error: any) {
          this.log(LogLevel.WARN, `加载群聊失败:`, { key, error: error.message })
        }
      }

      this.log(LogLevel.INFO, '群聊数据加载完成:', { count: this.groups.size })
    } catch (error: any) {
      this.log(LogLevel.WARN, '从KV加载群聊数据失败:', { error: error.message })
    }
  }

  /**
   * 从 Tauri KV 存储加载群聊消息（只加载最新的 N 条）
   * @param groupId - 群聊 ID
   * @param limit - 最大加载数量，默认 100 条
   */
  async loadMessagesFromKV(groupId: string, limit: number = 100): Promise<void> {
    try {
      const keys: string[] = await invoke('kv_keys', { prefix: `message:${groupId}:` })
      this.log(LogLevel.INFO, '加载群聊消息:', { groupId, totalKeys: keys.length, limit })

      const messages: LocalGroupMessage[] = []
      for (const key of keys) {
        try {
          const value: string | null = await invoke('kv_get', { key })
          if (value) {
            const message = LocalGroupMessage.fromJSON(JSON.parse(value))
            messages.push(message)
          }
        } catch (error: any) {
          this.log(LogLevel.WARN, `加载消息失败:`, { key, error: error.message })
        }
      }

      // 按时间戳排序（从新到旧）
      messages.sort((a: LocalGroupMessage, b: LocalGroupMessage) => b.timestamp - a.timestamp)

      // 只保留最新的 N 条
      const latestMessages = messages.slice(0, limit)

      // 重新按时间戳排序（从旧到新，保证时间线正确）
      latestMessages.sort((a: LocalGroupMessage, b: LocalGroupMessage) => a.timestamp - b.timestamp)

      this.messages.set(groupId, latestMessages)

      // 初始化消息 ID 集合
      const idSet = new Set<string>()
      const hashSet = new Set<string>()
      latestMessages.forEach(msg => {
        idSet.add(msg.id)
        hashSet.add(this.generateMessageHash(msg))
      })
      this.messageIdSet.set(groupId, idSet)
      this.messageHashSet.set(groupId, hashSet)

      this.log(LogLevel.INFO, '群聊消息加载完成:', { 
        groupId, 
        loaded: latestMessages.length, 
        total: messages.length,
        limit 
      })
    } catch (error: any) {
      this.log(LogLevel.WARN, '加载群聊消息失败:', { groupId, error: error.message })
    }
  }

  /**
   * 加载更多历史消息（用于滚动加载）
   * @param groupId - 群聊 ID
   * @param beforeTimestamp - 在此时间戳之前的消息
   * @param limit - 加载数量，默认 50 条
   * @returns 加载的历史消息
   */
  async loadMoreMessages(groupId: string, beforeTimestamp: number, limit: number = 50): Promise<LocalGroupMessage[]> {
    try {
      const keys: string[] = await invoke('kv_keys', { prefix: `message:${groupId}:` })
      
      const messages: LocalGroupMessage[] = []
      for (const key of keys) {
        try {
          const value: string | null = await invoke('kv_get', { key })
          if (value) {
            const message = LocalGroupMessage.fromJSON(JSON.parse(value))
            // 只加载指定时间戳之前的消息
            if (message.timestamp < beforeTimestamp) {
              messages.push(message)
            }
          }
        } catch (error: any) {
          this.log(LogLevel.WARN, `加载历史消息失败:`, { key, error: error.message })
        }
      }

      // 按时间戳排序（从新到旧）
      messages.sort((a: LocalGroupMessage, b: LocalGroupMessage) => b.timestamp - a.timestamp)

      // 只保留指定的数量
      const olderMessages = messages.slice(0, limit)

      // 重新按时间戳排序（从旧到新）
      olderMessages.sort((a: LocalGroupMessage, b: LocalGroupMessage) => a.timestamp - b.timestamp)

      this.log(LogLevel.INFO, '加载更多历史消息:', { 
        groupId, 
        loaded: olderMessages.length,
        beforeTimestamp 
      })

      return olderMessages
    } catch (error: any) {
      this.log(LogLevel.WARN, '加载更多历史消息失败:', { groupId, error: error.message })
      return []
    }
  }


  /**
   * 从 LocalStorage 加载备份数据
   */
  private async loadFromLocalStorage(): Promise<void> {
    try {
      // 加载群聊信息
      const groupsData = localStorage.getItem('alou:groupchat:groups')
      if (groupsData) {
        const groups = JSON.parse(groupsData)
        this.log(LogLevel.INFO, '从LocalStorage加载群聊信息:', { count: groups.length })
      }
    } catch (error: any) {
      this.log(LogLevel.WARN, '从LocalStorage加载数据失败:', { error: error.message })
    }
  }

  /**
   * 分页获取群聊消息
   */
  getGroupMessagesPaginated(
    groupId: string, 
    pagination: PaginationConfig = { page: 1, pageSize: 50 }
  ): MessageQueryResult {
    const allMessages = this.messages.get(groupId) || []
    const { page, pageSize } = pagination
    
    const startIndex = (page - 1) * pageSize
    const endIndex = startIndex + pageSize
    const paginatedMessages = allMessages.slice(startIndex, endIndex)
    
    return {
      messages: paginatedMessages,
      total: allMessages.length,
      hasMore: endIndex < allMessages.length
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
    this.log(LogLevel.DEBUG, '添加消息处理器:', { groupId })
  }

  /**
   * 移除消息处理器
   */
  removeMessageHandler(groupId: string, handler: MessageHandler): void {
    const handlers = this.messageHandlers.get(groupId)
    if (handlers) {
      handlers.delete(handler)
      this.log(LogLevel.DEBUG, '移除消息处理器:', { groupId })
    }
  }

  /**
   * 获取群聊消息（向后兼容）
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
    this.log(LogLevel.INFO, '离开群聊:', { groupId })
    
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
      
      // 清理消息ID集合
      this.messageIdSet.delete(groupId)
      this.messageHashSet.delete(groupId)

      this.log(LogLevel.INFO, '离开群聊成功:', { groupId })

    } catch (error: any) {
      this.log(LogLevel.ERROR, '离开群聊失败:', { groupId, error: error.message })
      throw new Error(`离开群聊失败: ${error.message}`)
    }
  }

  /**
   * 获取重试队列状态
   */
  getRetryQueueStatus(): { size: number; messages: Array<{ id: string; retries: number }> } {
    return {
      size: this.retryQueue.size,
      messages: Array.from(this.retryQueue.entries()).map(([id, item]) => ({
        id,
        retries: item.retries
      }))
    }
  }

  /**
   * 清理所有订阅
   */
  cleanup(): void {
    this.log(LogLevel.INFO, '开始清理群聊服务资源')
    
    // 停止重试处理器
    if (this.retryTimer) {
      clearInterval(this.retryTimer)
      this.retryTimer = null
    }
    
    // 清理待处理确认
    for (const [messageId, pending] of this.pendingAcks.entries()) {
      clearTimeout(pending.timeout)
      pending.reject(new Error('服务已清理'))
    }
    this.pendingAcks.clear()

    // 取消所有订阅
    this.subscriptions.forEach((unsubscribe: UnsubscribeFunction, groupId: string) => {
      try {
        unsubscribe()
      } catch (error: any) {
        this.log(LogLevel.ERROR, '清理订阅失败:', { groupId, error: error.message })
      }
    })

    // 清理所有数据
    this.subscriptions.clear()
    this.messageHandlers.clear()
    this.groups.clear()
    this.messages.clear()
    this.messageIdSet.clear()
    this.messageHashSet.clear()
    this.retryQueue.clear()

    this.log(LogLevel.INFO, '群聊服务资源已清理')
  }
}

export default new LocalIpfsGroupChatService()
