/**
 * 统一智能体协调器 - 集成群聊适配器（支持 Session 隔离）
 *
 * 统一管理所有群聊模式下的智能体消息分发和协调
 * 支持 Memory、PubSub、Iroh 三种群聊模式
 *
 * 集成特性：
 * - 自动订阅群聊消息
 * - 智能体注册/注销
 * - 消息分发到智能体
 * - 智能体自动回复
 * - 跨群聊智能体协调
 * - 持久化存储（与 agentStore 集成）
 * - Session 隔离（多页面并行支持）
 *
 * @module services/unifiedAgentCoordinator
 */

import {
  UnifiedMessage,
  GroupChatMessage,
  MessageType,
  AgentInfo,
  GroupChatMode,
  UnifiedGroup,
} from '../types/groupchat'

import {
  groupChatAdapterFactory,
  getAdapterForGroup,
  getBestAvailableAdapter,
  MemoryGroupChatAdapter,
  PubSubGroupChatAdapter,
  IrohGroupChatAdapter,
} from '../adapters/groupChatAdapter'

import useAgentStore, { useAgentStoreHydration } from '../stores/agentStore'

import { loadDiapIdentityFromFile } from '@/utils/diapAgentIdentityManager'

/**
 * 智能体消息处理器类型
 */
export type AgentMessageHandler = (message: UnifiedMessage, agentId: string) => void | Promise<void>

/**
 * 智能体注册信息（添加 session 绑定）
 */
export interface RegisteredAgent {
  agent: AgentInfo
  sessionId: string
  groups: Set<string>
  handler?: AgentMessageHandler
}

/**
 * 群聊订阅信息（添加 session 绑定）
 */
export interface GroupSubscription {
  sessionId: string
  groupId: string
  mode: GroupChatMode
  unsubscribe: () => void
  active: boolean
}

/**
 * 统一智能体协调器配置（添加 sessionId）
 */
export interface AgentCoordinatorConfig {
  /** 是否启用自动回复 */
  autoReply?: boolean
  /** 回复延迟（毫秒） */
  replyDelay?: number
  /** 调试模式 */
  debug?: boolean
  /** 是否自动订阅群聊消息 */
  autoSubscribe?: boolean
  /** 消息处理器 */
  onMessage?: (message: UnifiedMessage) => void
  /** 是否从 agentStore 恢复智能体 */
  restoreFromStore?: boolean
  /** Session ID（用于多页面隔离） */
  sessionId?: string
}

/**
 * 统一智能体协调器类
 *
 * 负责：
 * - 智能体注册/注销（支持 Session 隔离）
 * - 群聊订阅管理（支持 Session 隔离）
 * - 消息分发到智能体
 * - 智能体自动回复
 * - 跨群聊智能体协调
 *
 * Session 隔离机制：
 * - 每个 session 有独立的智能体列表
 * - 每个 session 有独立的群聊订阅
 * - 消息只在同一 session 内分发
 */
export class UnifiedAgentCoordinator {
  private static instance: UnifiedAgentCoordinator | null = null

  // Session ID（用于隔离）
  public readonly sessionId: string

  // 注册的智能体（按 session 隔离）
  private registeredAgents: Map<string, RegisteredAgent> = new Map()

  // 群聊到智能体的映射（按 session 隔离）
  private groupAgents: Map<string, Map<string, Set<string>>> = new Map() // sessionId -> (groupId -> Set<agentId>)

  // 群聊订阅管理（按 session 隔离）
  private groupSubscriptions: Map<string, GroupSubscription> = new Map() // key: `${sessionId}::${groupId}`

  // 配置
  private config: Required<AgentCoordinatorConfig>

  // 事件监听器清理函数
  private cleanupFunctions: Array<() => void> = []

  // 是否已恢复持久化数据
  private hasRestored: boolean = false

  private constructor(config: AgentCoordinatorConfig = {}) {
    this.sessionId = config.sessionId || `session_${Date.now()}_${Math.random().toString(36).slice(2, 9)}`
    
    this.config = {
      autoReply: false,
      replyDelay: 1000,
      debug: false,
      autoSubscribe: true,
      onMessage: () => {}, // 默认空函数
      restoreFromStore: true, // 默认启用从 store 恢复
      sessionId: this.sessionId,
      ...config,
    }

    // 设置全局事件监听（带 session 过滤）
    this.setupGlobalListeners()

    // 从 agentStore 恢复智能体（仅恢复当前 session 的）
    if (this.config.restoreFromStore) {
      this.restoreAgentsFromStore()
    }
  }

  /**
   * 获取单例实例（已废弃，建议使用 SessionManager）
   * @deprecated 请使用 SessionManager.getOrCreateSession() 代替
   */
  static getInstance(config?: AgentCoordinatorConfig): UnifiedAgentCoordinator {
    if (!this.instance) {
      this.instance = new UnifiedAgentCoordinator(config)
    }
    return this.instance
  }

  /**
   * 重置单例（用于测试）
   */
  static resetInstance(): void {
    if (this.instance) {
      this.instance.destroy()
      this.instance = null
    }
  }

  /**
   * 创建带 Session ID 的协调器（推荐方式）
   */
  static createWithSession(config: AgentCoordinatorConfig = {}): UnifiedAgentCoordinator {
    return new UnifiedAgentCoordinator(config)
  }

  /**
   * 设置全局事件监听（带 session 过滤）
   */
  private setupGlobalListeners(): void {
    // 监听 agent-group-message 事件（带 session 过滤）
    const handleAgentGroupMessage = (event: Event) => {
      const customEvent = event as CustomEvent
      const { agentId, message, sessionId } = customEvent.detail

      // 只处理当前 session 的消息
      if (sessionId && sessionId !== this.sessionId) {
        return
      }

      if (agentId && message) {
        this.dispatchToAgent(agentId, message)
      }
    }

    window.addEventListener('agent-group-message', handleAgentGroupMessage)
    this.cleanupFunctions.push(() => {
      window.removeEventListener('agent-group-message', handleAgentGroupMessage)
    })

    if (this.config.debug) {
      console.log('[AgentCoordinator] 全局事件监听已设置，sessionId:', this.sessionId)
    }
  }

  /**
   * 从 agentStore 恢复智能体
   */
  private async restoreAgentsFromStore(): Promise<void> {
    try {
      // 等待 agentStore hydration 完成
      const waitForHydration = (): Promise<void> => {
        return new Promise((resolve) => {
          if (useAgentStore.getState()._hasHydrated) {
            resolve()
            return
          }

          const unsubscribe = useAgentStore.subscribe((state) => {
            if (state._hasHydrated) {
              unsubscribe()
              resolve()
            }
          })
        })
      }

      await waitForHydration()

      const agents = useAgentStore.getState().agents
      if (!agents || agents.length === 0) {
        if (this.config.debug) {
          console.log('[AgentCoordinator] agentStore 中没有智能体，跳过恢复')
        }
        return
      }

      // 将 agentStore 中的智能体转换为 AgentInfo 并注册
      for (const agent of agents) {
        const agentId = agent.id
        
        // 尝试从文件加载 DIAP 身份（无论是否已有都尝试加载，以支持重启后的恢复）
        let diapIdentity = agent.diapIdentity
        if (agentId) {
          try {
            const loadedIdentity = await loadDiapIdentityFromFile(agentId)
            if (loadedIdentity) {
              // 如果文件中有但 agentStore 中没有，或者文件中的更完整，使用文件中的
              if (!diapIdentity || (loadedIdentity.did && !diapIdentity.did)) {
                diapIdentity = loadedIdentity
                console.log('[AgentCoordinator] 从文件加载 DIAP 身份:', agentId, loadedIdentity.did)
              }
            } else {
              console.log('[AgentCoordinator] 文件中没有 DIAP 身份:', agentId)
            }
          } catch (loadError) {
            console.warn('[AgentCoordinator] 加载 DIAP 身份失败:', agentId, loadError)
          }
        }
        
        const agentInfo: AgentInfo = {
          id: agentId,
          name: agent.name || agent.display_name || '未命名智能体',
          mode: 'agent',
          avatar: agent.avatar_url || undefined,
          sessionId: agent.sessionId,
          // 保留额外的元数据
          metadata: {
            ipns: agent.ipns,
            cid: agent.cid,
            did: agent.did,
            diapIdentity: diapIdentity,
          },
        }

        // 注册智能体（不传入 handler，因为 handler 是运行时逻辑）
        await this.registerAgent(agentInfo, undefined, false)
      }

      this.hasRestored = true

      if (this.config.debug) {
        console.log('[AgentCoordinator] 已从 agentStore 恢复', agents.length, '个智能体')
      }
    } catch (error) {
      console.error('[AgentCoordinator] 从 agentStore 恢复智能体失败:', error)
    }
  }

  // ============================================================================
  // 智能体管理
  // ============================================================================

  /**
   * 注册智能体（带 Session 绑定）
   * @param agent 智能体信息
   * @param handler 消息处理器
   * @param persistToStore 是否持久化到 agentStore（默认 true）
   */
  async registerAgent(
    agent: AgentInfo,
    handler?: AgentMessageHandler,
    persistToStore: boolean = true
  ): Promise<void> {
    if (this.registeredAgents.has(agent.id)) {
      console.warn('[AgentCoordinator] 智能体已注册:', agent.id)
      return
    }

    const registeredAgent: RegisteredAgent = {
      agent,
      sessionId: this.sessionId,
      groups: new Set(),
      handler,
    }

    this.registeredAgents.set(agent.id, registeredAgent)

    // 持久化到 agentStore（使用 session 隔离的存储）
    if (persistToStore && typeof window !== 'undefined') {
      try {
        const store = useAgentStore.getState()
        store.addAgent({
          id: agent.id,
          sessionId: agent.sessionId || this.sessionId,
          name: agent.name,
          display_name: agent.name,
          avatar_url: agent.avatar,
          agent_type: 'ai_agent_sdk',
          created_at: Date.now(),
          updated_at: Date.now(),
        })
        if (this.config.debug) {
          console.log('[AgentCoordinator] 智能体已持久化到 agentStore:', agent.id, 'sessionId:', this.sessionId)
        }
      } catch (error) {
        console.error('[AgentCoordinator] 持久化到 agentStore 失败:', error)
      }
    }

    // 自动将智能体注册到所有已订阅的群聊（当前 session 的）
    if (this.config.autoSubscribe) {
      for (const [key] of this.groupSubscriptions) {
        // 只处理当前 session 的订阅
        if (key.startsWith(this.sessionId + '::')) {
          const groupId = key.substring(this.sessionId.length + 2)
          await this.registerToGroup(groupId, agent)
        }
      }
    }

    if (this.config.debug) {
      console.log('[AgentCoordinator] 智能体已注册:', {
        id: agent.id,
        sessionId: this.sessionId,
        name: agent.name,
        mode: agent.mode,
      })
    }
  }

  /**
   * 注册智能体到所有群聊
   */
  async registerToAllGroups(agents: AgentInfo[]): Promise<void> {
    for (const agent of agents) {
      await this.registerAgent(agent)

      // 添加到所有群聊
      for (const [groupId] of this.groupAgents) {
        this.registerToGroup(groupId, agent)
      }
    }
  }

  /**
   * 注册智能体到指定群聊（带 Session 隔离）
   */
  async registerToGroup(groupId: string, agent: AgentInfo): Promise<void> {
    // 如果智能体未注册，先注册
    if (!this.registeredAgents.has(agent.id)) {
      await this.registerAgent(agent)
    }

    // 添加到群聊的智能体列表（当前 session 的）
    if (!this.groupAgents.has(this.sessionId)) {
      this.groupAgents.set(this.sessionId, new Map())
    }
    
    const sessionGroupAgents = this.groupAgents.get(this.sessionId)!
    if (!sessionGroupAgents.has(groupId)) {
      sessionGroupAgents.set(groupId, new Set())
    }

    sessionGroupAgents.get(groupId)!.add(agent.id)

    // 更新智能体的群聊列表
    const registeredAgent = this.registeredAgents.get(agent.id)
    if (registeredAgent) {
      registeredAgent.groups.add(groupId)
    }

    if (this.config.debug) {
      console.log('[AgentCoordinator] 智能体已添加到群聊:', {
        sessionId: this.sessionId,
        agentId: agent.id,
        groupId,
      })
    }
  }

  /**
   * 注销智能体（带 Session 隔离）
   */
  async unregisterAgent(agentId: string, removeFromStore: boolean = true): Promise<void> {
    const registeredAgent = this.registeredAgents.get(agentId)

    if (!registeredAgent) {
      console.warn('[AgentCoordinator] 智能体未注册:', agentId)
      return
    }

    // 从所有群聊中移除（当前 session 的）
    const sessionGroupAgents = this.groupAgents.get(this.sessionId)
    if (sessionGroupAgents) {
      for (const [groupId, agents] of sessionGroupAgents) {
        agents.delete(agentId)
      }
    }

    // 从 agentStore 移除（只移除当前 session 的）
    if (removeFromStore && typeof window !== 'undefined') {
      try {
        const store = useAgentStore.getState()
        store.removeAgent(agentId)
        if (this.config.debug) {
          console.log('[AgentCoordinator] 智能体已从 agentStore 移除:', agentId, 'sessionId:', this.sessionId)
        }
      } catch (error) {
        console.error('[AgentCoordinator] 从 agentStore 移除失败:', error)
      }
    }

    // 移除智能体
    this.registeredAgents.delete(agentId)

    if (this.config.debug) {
      console.log('[AgentCoordinator] 智能体已注销:', agentId, 'sessionId:', this.sessionId)
    }
  }

  // ============================================================================
  // 群聊订阅管理 - 集成群聊适配器
  // ============================================================================

  /**
   * 生成 session 隔离的订阅 key
   */
  private getSubscriptionKey(groupId: string): string {
    return `${this.sessionId}::${groupId}`
  }

  /**
   * 从订阅 key 解析 groupId
   */
  private parseGroupIdFromKey(key: string): string {
    const parts = key.split('::')
    return parts.length > 1 ? parts.slice(1).join('::') : key
  }

  /**
   * 订阅群聊消息（带 Session 隔离）
   */
  async subscribeToGroup(groupId: string, mode?: GroupChatMode): Promise<void> {
    const subscriptionKey = this.getSubscriptionKey(groupId)

    // 检查是否已订阅
    if (this.groupSubscriptions.has(subscriptionKey)) {
      console.log('[AgentCoordinator] 群聊已订阅:', groupId, 'sessionId:', this.sessionId)
      return
    }

    try {
      // 获取适配器
      let adapter = mode ? groupChatAdapterFactory.getAdapter(mode) : null

      if (!adapter) {
        // 尝试获取最佳适配器
        adapter = await getBestAvailableAdapter()
      }

      // 订阅群聊消息
      const unsubscribe = adapter.subscribe(groupId, (message) => {
        this.handleGroupMessage(groupId, message)
      })

      // 记录订阅（使用 session 隔离的 key）
      const subscription: GroupSubscription = {
        sessionId: this.sessionId,
        groupId,
        mode: adapter.mode,
        unsubscribe,
        active: true,
      }

      this.groupSubscriptions.set(subscriptionKey, subscription)

      if (this.config.debug) {
        console.log('[AgentCoordinator] 群聊订阅成功:', {
          sessionId: this.sessionId,
          groupId,
          mode: adapter.mode,
        })
      }
    } catch (error) {
      console.error('[AgentCoordinator] 群聊订阅失败:', groupId, error)
    }
  }

  /**
   * 取消订阅群聊（带 Session 隔离）
   */
  unsubscribeFromGroup(groupId: string): void {
    const subscriptionKey = this.getSubscriptionKey(groupId)
    const subscription = this.groupSubscriptions.get(subscriptionKey)

    if (subscription) {
      subscription.unsubscribe()
      subscription.active = false
      this.groupSubscriptions.delete(subscriptionKey)

      if (this.config.debug) {
        console.log('[AgentCoordinator] 群聊取消订阅:', {
          sessionId: this.sessionId,
          groupId,
        })
      }
    }
  }

  /**
   * 处理群聊消息（带 Session 隔离）
   */
  private handleGroupMessage(groupId: string, message: UnifiedMessage | GroupChatMessage): void {
    if (this.config.debug) {
      console.log('[AgentCoordinator] 收到群聊消息:', {
        sessionId: this.sessionId,
        groupId,
        messageId: 'id' in message ? message.id : 'unknown',
        type: message.type,
        content: message.content.slice(0, 50) + '...',
      })
    }

    // 调用自定义消息处理器
    if (this.config.onMessage) {
      try {
        this.config.onMessage(message as UnifiedMessage)
      } catch (error) {
        console.error('[AgentCoordinator] 消息处理器错误:', error)
      }
    }

    // 分发到群聊中的智能体（当前 session 的）
    const agentIds = this.getGroupAgentsForSession(groupId)
    if (agentIds) {
      for (const agentId of agentIds) {
        this.dispatchToAgent(agentId, message as UnifiedMessage)
      }
    }
  }

  /**
   * 获取当前 session 的群聊智能体映射
   */
  private getGroupAgentsForSession(groupId: string): Set<string> | undefined {
    return this.groupAgents.get(this.sessionId)?.get(groupId)
  }

  /**
   * 设置当前 session 的群聊智能体映射
   */
  private setGroupAgentsForSession(groupId: string, agentIds: Set<string>): void {
    if (!this.groupAgents.has(this.sessionId)) {
      this.groupAgents.set(this.sessionId, new Map())
    }
    this.groupAgents.get(this.sessionId)!.set(groupId, agentIds)
  }

  /**
   * 获取所有已订阅的群聊（带 Session 隔离）
   */
  getSubscribedGroups(): string[] {
    return Array.from(this.groupSubscriptions.values())
      .filter(s => s.sessionId === this.sessionId)
      .map(s => s.groupId)
  }

  /**
   * 获取群聊订阅状态（带 Session 隔离）
   */
  getGroupSubscriptionStatus(groupId: string): { subscribed: boolean; mode?: GroupChatMode } {
    const subscriptionKey = this.getSubscriptionKey(groupId)
    const subscription = this.groupSubscriptions.get(subscriptionKey)
    return {
      subscribed: !!subscription && subscription.active && subscription.sessionId === this.sessionId,
      mode: subscription?.mode,
    }
  }

  // ============================================================================
  // 消息分发
  // ============================================================================

  /**
   * 分发事件到智能体（带 session ID）
   */
  dispatchToAgent(agentId: string, message: UnifiedMessage): void {
    const registeredAgent = this.registeredAgents.get(agentId)

    if (!registeredAgent) {
      if (this.config.debug) {
        console.warn('[AgentCoordinator] 智能体未注册，跳过:', agentId)
      }
      return
    }

    // 检查智能体是否在消息的群聊中
    if (message.groupId && !registeredAgent.groups.has(message.groupId)) {
      if (this.config.debug) {
        console.warn('[AgentCoordinator] 智能体不在群聊中:', {
          agentId,
          groupId: message.groupId,
        })
      }
      return
    }

    // 调用处理器
    if (registeredAgent.handler) {
      try {
        const result = registeredAgent.handler(message, agentId)

        // 如果是 Promise，处理错误
        if (result instanceof Promise) {
          result.catch((error) => {
            console.error('[AgentCoordinator] 智能体处理器错误:', error)
          })
        }
      } catch (error) {
        console.error('[AgentCoordinator] 智能体处理器同步错误:', error)
      }
    }

    // 自动回复（如果启用）
    if (this.config.autoReply) {
      this.handleAutoReply(agentId, message)
    }

    // 触发全局事件（带 session ID，用于跨组件通信）
    window.dispatchEvent(new CustomEvent('agent-group-message', {
      detail: {
        agentId,
        message,
        sessionId: this.sessionId, // 添加 session ID
      }
    }))

    if (this.config.debug) {
      console.log('[AgentCoordinator] 消息已分发到智能体:', {
        agentId,
        sessionId: this.sessionId,
        messageType: message.type,
        content: message.content.slice(0, 50) + '...',
      })
    }
  }

  /**
   * 广播消息到所有群聊的智能体
   */
  async broadcastToAgents(message: UnifiedMessage): Promise<void> {
    const agents = this.getActiveAgents(message.groupId)

    for (const agent of agents) {
      this.dispatchToAgent(agent.id, message)
    }
  }

  /**
   * 获取群聊中的活跃智能体（带 Session 隔离）
   */
  getActiveAgents(groupId?: string): AgentInfo[] {
    const agents: AgentInfo[] = []

    if (groupId) {
      // 获取指定群聊的智能体（当前 session 的）
      const agentIds = this.getGroupAgentsForSession(groupId)

      if (agentIds) {
        for (const agentId of agentIds) {
          const registeredAgent = this.registeredAgents.get(agentId)
          if (registeredAgent) {
            agents.push(registeredAgent.agent)
          }
        }
      }
    } else {
      // 获取所有智能体（当前 session 的）
      for (const [agentId, registeredAgent] of this.registeredAgents.entries()) {
        // 只返回当前 session 的智能体
        if (registeredAgent.sessionId === this.sessionId) {
          agents.push(registeredAgent.agent)
        }
      }
    }

    return agents
  }

  // ============================================================================
  // 自动回复
  // ============================================================================

  /**
   * 设置是否启用自动回复
   */
  setAutoReply(enabled: boolean): void {
    this.config.autoReply = enabled
    console.log('[AgentCoordinator] 自动回复已', enabled ? '启用' : '禁用')
  }

  /**
   * 设置自动回复延迟
   */
  setReplyDelay(ms: number): void {
    this.config.replyDelay = ms
    console.log('[AgentCoordinator] 自动回复延迟已设置为:', ms, 'ms')
  }

  /**
   * 处理自动回复
   */
  private async handleAutoReply(agentId: string, message: UnifiedMessage): Promise<void> {
    // 不回复自己发送的消息
    if (message.sender.id === agentId) {
      return
    }

    // 延迟回复
    if (this.config.replyDelay && this.config.replyDelay > 0) {
      await new Promise((resolve) => setTimeout(resolve, this.config.replyDelay))
    }

    // TODO: 实现智能体自动回复逻辑
    // 这里应该调用智能体的回复生成接口
    if (this.config.debug) {
      console.log('[AgentCoordinator] 自动回复触发:', {
        agentId,
        messageId: message.id,
      })
    }
  }

  // ============================================================================
  // 查询方法
  // ============================================================================

  /**
   * 获取所有注册的智能体
   */
  getAllAgents(): AgentInfo[] {
    return Array.from(this.registeredAgents.values()).map((ra) => ra.agent)
  }

  /**
   * 获取智能体详情
   */
  getAgent(agentId: string): AgentInfo | null {
    const registeredAgent = this.registeredAgents.get(agentId)
    return registeredAgent ? registeredAgent.agent : null
  }

  /**
   * 获取智能体所在的群聊
   */
  getAgentGroups(agentId: string): string[] {
    const registeredAgent = this.registeredAgents.get(agentId)
    return registeredAgent ? Array.from(registeredAgent.groups) : []
  }

  /**
   * 获取所有已订阅的群聊（带 Session 隔离）
   */
  getAllSubscribedGroups(): { groupId: string; mode: GroupChatMode }[] {
    return Array.from(this.groupSubscriptions.values())
      .filter(s => s.sessionId === this.sessionId)
      .map((s) => ({
        groupId: s.groupId,
        mode: s.mode,
      }))
  }

  /**
   * 获取协调器状态（带 Session 信息）
   */
  getStatus(): {
    sessionId: string
    registeredAgents: number
    subscribedGroups: number
    autoReply: boolean
    debug: boolean
    hasRestored: boolean
  } {
    // 只统计当前 session 的数据
    const sessionAgentCount = Array.from(this.registeredAgents.values())
      .filter(ra => ra.sessionId === this.sessionId).length
    
    const sessionSubscriptionCount = Array.from(this.groupSubscriptions.values())
      .filter(s => s.sessionId === this.sessionId).length

    return {
      sessionId: this.sessionId,
      registeredAgents: sessionAgentCount,
      subscribedGroups: sessionSubscriptionCount,
      autoReply: this.config.autoReply,
      debug: this.config.debug,
      hasRestored: this.hasRestored,
    }
  }

  /**
   * 手动触发从 agentStore 恢复智能体
   */
  async restoreFromStore(): Promise<void> {
    await this.restoreAgentsFromStore()
  }

  /**
   * 检查是否已完成恢复
   */
  hasRestoredFromStore(): boolean {
    return this.hasRestored
  }

  // ============================================================================
  // 销毁
  // ============================================================================

  /**
   * 销毁协调器，清理资源（带 Session 隔离）
   */
  destroy(): void {
    console.log('[AgentCoordinator] 销毁协调器，sessionId:', this.sessionId)

    // 取消所有群聊订阅（当前 session 的）
    for (const [key, subscription] of this.groupSubscriptions) {
      if (subscription.sessionId === this.sessionId && subscription.active) {
        subscription.unsubscribe()
      }
    }

    // 只删除当前 session 的订阅
    for (const key of this.groupSubscriptions.keys()) {
      if (key.startsWith(this.sessionId + '::')) {
        this.groupSubscriptions.delete(key)
      }
    }

    // 调用所有清理函数
    this.cleanupFunctions.forEach((cleanup) => cleanup())
    this.cleanupFunctions = []

    // 清空数据（当前 session 的）
    this.registeredAgents.clear()
    this.groupAgents.delete(this.sessionId)

    if (this.config.debug) {
      console.log('[AgentCoordinator] 协调器已销毁，sessionId:', this.sessionId)
    }
  }
}

// ============================================================================
// 导出单例和便捷函数（支持 Session）
// ============================================================================

// 导出单例（已废弃，建议使用 SessionManager）
/**
 * @deprecated 请使用 SessionManager 代替
 */
export const agentCoordinator = UnifiedAgentCoordinator.getInstance()

/**
 * 注册智能体（使用默认单例，不推荐用于多页面场景）
 * @deprecated 多页面场景请使用 SessionManager.registerAgentForSession
 */
export function registerAgent(agent: AgentInfo, handler?: AgentMessageHandler): Promise<void> {
  return agentCoordinator.registerAgent(agent, handler)
}

/**
 * 注册智能体到群聊（使用默认单例，不推荐用于多页面场景）
 * @deprecated 多页面场景请使用 SessionManager.registerAgentForSession + registerToGroup
 */
export function registerToGroup(groupId: string, agent: AgentInfo): Promise<void> {
  return agentCoordinator.registerToGroup(groupId, agent)
}

/**
 * 注销智能体
 */
export function unregisterAgent(agentId: string, removeFromStore?: boolean): Promise<void> {
  return agentCoordinator.unregisterAgent(agentId, removeFromStore)
}

/**
 * 订阅群聊消息（使用默认单例，不推荐用于多页面场景）
 * @deprecated 多页面场景请使用 SessionManager.subscribeToGroupForSession
 */
export function subscribeToGroup(groupId: string, mode?: GroupChatMode): Promise<void> {
  return agentCoordinator.subscribeToGroup(groupId, mode)
}

/**
 * 取消订阅群聊
 */
export function unsubscribeFromGroup(groupId: string): void {
  agentCoordinator.unsubscribeFromGroup(groupId)
}

/**
 * 分发事件到智能体
 */
export function dispatchToAgent(agentId: string, message: UnifiedMessage): void {
  agentCoordinator.dispatchToAgent(agentId, message)
}

/**
 * 获取活跃智能体
 */
export function getActiveAgents(groupId?: string): AgentInfo[] {
  return agentCoordinator.getActiveAgents(groupId)
}

/**
 * 设置自动回复
 */
export function setAutoReply(enabled: boolean): void {
  agentCoordinator.setAutoReply(enabled)
}

/**
 * 设置自动回复延迟
 */
export function setReplyDelay(ms: number): void {
  agentCoordinator.setReplyDelay(ms)
}

/**
 * 获取协调器状态
 */
export function getCoordinatorStatus(): ReturnType<typeof agentCoordinator.getStatus> {
  return agentCoordinator.getStatus()
}

/**
 * 创建带 Session ID 的协调器（推荐方式）
 */
export function createSessionCoordinator(config: AgentCoordinatorConfig = {}): UnifiedAgentCoordinator {
  return UnifiedAgentCoordinator.createWithSession(config)
}
