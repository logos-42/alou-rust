/**
 * 统一智能体协调器 - 集成群聊适配器
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

/**
 * 智能体消息处理器类型
 */
export type AgentMessageHandler = (message: UnifiedMessage, agentId: string) => void | Promise<void>

/**
 * 智能体注册信息
 */
export interface RegisteredAgent {
  agent: AgentInfo
  groups: Set<string>
  handler?: AgentMessageHandler
}

/**
 * 群聊订阅信息
 */
export interface GroupSubscription {
  groupId: string
  mode: GroupChatMode
  unsubscribe: () => void
  active: boolean
}

/**
 * 统一智能体协调器配置
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
}

/**
 * 统一智能体协调器类
 *
 * 负责：
 * - 智能体注册/注销
 * - 群聊订阅管理
 * - 消息分发到智能体
 * - 智能体自动回复
 * - 跨群聊智能体协调
 */
export class UnifiedAgentCoordinator {
  private static instance: UnifiedAgentCoordinator | null = null

  // 注册的智能体
  private registeredAgents: Map<string, RegisteredAgent> = new Map()

  // 群聊到智能体的映射
  private groupAgents: Map<string, Set<string>> = new Map()

  // 群聊订阅管理
  private groupSubscriptions: Map<string, GroupSubscription> = new Map()

  // 配置
  private config: Required<AgentCoordinatorConfig>

  // 事件监听器清理函数
  private cleanupFunctions: Array<() => void> = []

  private constructor(config: AgentCoordinatorConfig = {}) {
    this.config = {
      autoReply: false,
      replyDelay: 1000,
      debug: false,
      autoSubscribe: true,
      onMessage: () => {}, // 默认空函数
      ...config,
    }

    // 设置全局事件监听
    this.setupGlobalListeners()
  }

  /**
   * 获取单例实例
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
   * 设置全局事件监听
   */
  private setupGlobalListeners(): void {
    // 监听 agent-group-message 事件
    const handleAgentGroupMessage = (event: Event) => {
      const customEvent = event as CustomEvent
      const { agentId, message } = customEvent.detail

      if (agentId && message) {
        this.dispatchToAgent(agentId, message)
      }
    }

    window.addEventListener('agent-group-message', handleAgentGroupMessage)
    this.cleanupFunctions.push(() => {
      window.removeEventListener('agent-group-message', handleAgentGroupMessage)
    })

    if (this.config.debug) {
      console.log('[AgentCoordinator] 全局事件监听已设置')
    }
  }

  // ============================================================================
  // 智能体管理
  // ============================================================================

  /**
   * 注册智能体
   */
  async registerAgent(
    agent: AgentInfo,
    handler?: AgentMessageHandler
  ): Promise<void> {
    if (this.registeredAgents.has(agent.id)) {
      console.warn('[AgentCoordinator] 智能体已注册:', agent.id)
      return
    }

    const registeredAgent: RegisteredAgent = {
      agent,
      groups: new Set(),
      handler,
    }

    this.registeredAgents.set(agent.id, registeredAgent)

    // 自动将智能体注册到所有已订阅的群聊
    if (this.config.autoSubscribe) {
      for (const [groupId] of this.groupSubscriptions) {
        await this.registerToGroup(groupId, agent)
      }
    }

    if (this.config.debug) {
      console.log('[AgentCoordinator] 智能体已注册:', {
        id: agent.id,
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
   * 注册智能体到指定群聊
   */
  async registerToGroup(groupId: string, agent: AgentInfo): Promise<void> {
    // 如果智能体未注册，先注册
    if (!this.registeredAgents.has(agent.id)) {
      await this.registerAgent(agent)
    }

    // 添加到群聊的智能体列表
    if (!this.groupAgents.has(groupId)) {
      this.groupAgents.set(groupId, new Set())
    }

    this.groupAgents.get(groupId)!.add(agent.id)

    // 更新智能体的群聊列表
    const registeredAgent = this.registeredAgents.get(agent.id)
    if (registeredAgent) {
      registeredAgent.groups.add(groupId)
    }

    if (this.config.debug) {
      console.log('[AgentCoordinator] 智能体已添加到群聊:', {
        agentId: agent.id,
        groupId,
      })
    }
  }

  /**
   * 注销智能体
   */
  async unregisterAgent(agentId: string): Promise<void> {
    const registeredAgent = this.registeredAgents.get(agentId)

    if (!registeredAgent) {
      console.warn('[AgentCoordinator] 智能体未注册:', agentId)
      return
    }

    // 从所有群聊中移除
    for (const [groupId, agents] of this.groupAgents) {
      agents.delete(agentId)
    }

    // 移除智能体
    this.registeredAgents.delete(agentId)

    if (this.config.debug) {
      console.log('[AgentCoordinator] 智能体已注销:', agentId)
    }
  }

  // ============================================================================
  // 群聊订阅管理 - 集成群聊适配器
  // ============================================================================

  /**
   * 订阅群聊消息
   */
  async subscribeToGroup(groupId: string, mode?: GroupChatMode): Promise<void> {
    // 检查是否已订阅
    if (this.groupSubscriptions.has(groupId)) {
      console.log('[AgentCoordinator] 群聊已订阅:', groupId)
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

      // 记录订阅
      const subscription: GroupSubscription = {
        groupId,
        mode: adapter.mode,
        unsubscribe,
        active: true,
      }

      this.groupSubscriptions.set(groupId, subscription)

      if (this.config.debug) {
        console.log('[AgentCoordinator] 群聊订阅成功:', {
          groupId,
          mode: adapter.mode,
        })
      }
    } catch (error) {
      console.error('[AgentCoordinator] 群聊订阅失败:', groupId, error)
    }
  }

  /**
   * 取消订阅群聊
   */
  unsubscribeFromGroup(groupId: string): void {
    const subscription = this.groupSubscriptions.get(groupId)

    if (subscription) {
      subscription.unsubscribe()
      subscription.active = false
      this.groupSubscriptions.delete(groupId)

      if (this.config.debug) {
        console.log('[AgentCoordinator] 群聊取消订阅:', groupId)
      }
    }
  }

  /**
   * 处理群聊消息
   */
  private handleGroupMessage(groupId: string, message: UnifiedMessage | GroupChatMessage): void {
    if (this.config.debug) {
      console.log('[AgentCoordinator] 收到群聊消息:', {
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

    // 分发到群聊中的智能体
    const agentIds = this.groupAgents.get(groupId)
    if (agentIds) {
      for (const agentId of agentIds) {
        this.dispatchToAgent(agentId, message as UnifiedMessage)
      }
    }
  }

  /**
   * 获取所有已订阅的群聊
   */
  getSubscribedGroups(): string[] {
    return Array.from(this.groupSubscriptions.keys())
  }

  /**
   * 获取群聊订阅状态
   */
  getGroupSubscriptionStatus(groupId: string): { subscribed: boolean; mode?: GroupChatMode } {
    const subscription = this.groupSubscriptions.get(groupId)
    return {
      subscribed: !!subscription && subscription.active,
      mode: subscription?.mode,
    }
  }

  // ============================================================================
  // 消息分发
  // ============================================================================

  /**
   * 分发事件到智能体
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

    if (this.config.debug) {
      console.log('[AgentCoordinator] 消息已分发到智能体:', {
        agentId,
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
   * 获取群聊中的活跃智能体
   */
  getActiveAgents(groupId?: string): AgentInfo[] {
    const agents: AgentInfo[] = []

    if (groupId && this.groupAgents.has(groupId)) {
      // 获取指定群聊的智能体
      const agentIds = this.groupAgents.get(groupId)!

      for (const agentId of agentIds) {
        const registeredAgent = this.registeredAgents.get(agentId)
        if (registeredAgent) {
          agents.push(registeredAgent.agent)
        }
      }
    } else {
      // 获取所有智能体
      for (const registeredAgent of this.registeredAgents.values()) {
        agents.push(registeredAgent.agent)
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
   * 获取所有已订阅的群聊
   */
  getAllSubscribedGroups(): { groupId: string; mode: GroupChatMode }[] {
    return Array.from(this.groupSubscriptions.values()).map((s) => ({
      groupId: s.groupId,
      mode: s.mode,
    }))
  }

  /**
   * 获取协调器状态
   */
  getStatus(): {
    registeredAgents: number
    subscribedGroups: number
    autoReply: boolean
    debug: boolean
  } {
    return {
      registeredAgents: this.registeredAgents.size,
      subscribedGroups: this.groupSubscriptions.size,
      autoReply: this.config.autoReply,
      debug: this.config.debug,
    }
  }

  // ============================================================================
  // 销毁
  // ============================================================================

  /**
   * 销毁协调器，清理资源
   */
  destroy(): void {
    // 取消所有群聊订阅
    for (const [groupId, subscription] of this.groupSubscriptions) {
      if (subscription.active) {
        subscription.unsubscribe()
      }
    }
    this.groupSubscriptions.clear()

    // 调用所有清理函数
    this.cleanupFunctions.forEach((cleanup) => cleanup())
    this.cleanupFunctions = []

    // 清空数据
    this.registeredAgents.clear()
    this.groupAgents.clear()

    if (this.config.debug) {
      console.log('[AgentCoordinator] 协调器已销毁')
    }
  }
}

// ============================================================================
// 导出单例和便捷函数
// ============================================================================

// 导出单例
export const agentCoordinator = UnifiedAgentCoordinator.getInstance()

/**
 * 注册智能体
 */
export function registerAgent(agent: AgentInfo, handler?: AgentMessageHandler): Promise<void> {
  return agentCoordinator.registerAgent(agent, handler)
}

/**
 * 注册智能体到群聊
 */
export function registerToGroup(groupId: string, agent: AgentInfo): Promise<void> {
  return agentCoordinator.registerToGroup(groupId, agent)
}

/**
 * 注销智能体
 */
export function unregisterAgent(agentId: string): Promise<void> {
  return agentCoordinator.unregisterAgent(agentId)
}

/**
 * 订阅群聊消息
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
