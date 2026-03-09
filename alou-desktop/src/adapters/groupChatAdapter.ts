/**
 * 统一群聊适配器接口 - 前端薄适配层
 *
 * 注意：这是前端的薄适配层，核心逻辑在 Rust 后端 (tools/adapters/group_adapter.rs)
 * 前端只负责：
 * 1. 调用 Rust 后端 API (通过 invoke)
 * 2. 本地状态缓存
 * 3. 事件订阅管理
 *
 * @module adapters/groupChatAdapter
 * @author Alou Team
 * @since 1.0.0
 */

import { invoke } from '@tauri-apps/api/core'
import {
  GroupChatAdapter,
  GroupChatAdapterFactory,
  GroupChatConfig,
  GroupChatMode,
  GroupChatError,
  GroupChatErrorCode,
  MessageHandler,
  UnifiedGroup,
  UnifiedMessage,
  UnsubscribeFunction,
  AgentInfo,
  MessageType,
} from '../types/groupchat'

// Rust 后端工具 ID
const TOOL_ID = 'group_adapter'

/**
 * 调用 Rust 后端工具
 */
async function callTool<T>(action: string, params: Record<string, unknown>): Promise<T> {
  const result = await invoke<{ success: boolean; data?: T; error?: string }>('execute_tool', {
    toolId: TOOL_ID,
    args: JSON.stringify({ action, ...params }),
  })

  if (!result.success) {
    throw new GroupChatError(
      GroupChatErrorCode.UNKNOWN,
      result.error || '操作失败'
    )
  }

  return result.data as T
}

/**
 * 基础适配器实现 - 薄调用层
 * 所有模式共享同一套实现，只是 mode 标识不同
 */
export class BaseAdapter implements GroupChatAdapter {
  readonly mode: GroupChatMode
  public available: boolean = true

  protected localIdentity: AgentInfo | null = null
  protected subscriptions: Map<string, Set<MessageHandler>> = new Map()
  protected messageCache: Map<string, UnifiedMessage[]> = new Map()

  constructor(mode: GroupChatMode) {
    this.mode = mode
  }

  setLocalIdentity(identity: AgentInfo): void {
    this.localIdentity = identity
    // 同步到后端
    callTool('set_local_identity', {
      id: identity.id,
      name: identity.name,
      did: identity.did,
      avatar: identity.avatar,
    }).catch(console.error)
  }

  getLocalIdentity(): AgentInfo | null {
    return this.localIdentity
  }

  async createGroup(config: GroupChatConfig): Promise<UnifiedGroup> {
    const group = await callTool<UnifiedGroup>('create_group', {
      name: config.name,
      description: config.description,
      members: config.members,
      mode: config.mode ?? this.mode,
      topic: config.topic,
    })

    // 缓存群聊信息
    this.messageCache.set(group.id, [])

    return group
  }

  async joinGroup(groupIdOrTicket: string): Promise<UnifiedGroup> {
    const group = await callTool<UnifiedGroup>('join_group', {
      group_id: groupIdOrTicket,
    })

    // 缓存群聊信息
    if (!this.messageCache.has(group.id)) {
      this.messageCache.set(group.id, [])
    }

    return group
  }

  async leaveGroup(groupId: string): Promise<void> {
    await callTool('leave_group', { group_id: groupId })
    this.messageCache.delete(groupId)
    this.subscriptions.delete(groupId)
  }

  async listGroups(): Promise<UnifiedGroup[]> {
    const groups = await callTool<UnifiedGroup[]>('list_groups', {})

    // 过滤当前模式
    return groups.filter(g => g.mode.toLowerCase() === this.mode.toLowerCase())
  }

  async getGroupInfo(groupId: string): Promise<UnifiedGroup | null> {
    try {
      const group = await callTool<UnifiedGroup>('get_group_info', { group_id: groupId })
      return group
    } catch {
      return null
    }
  }

  async sendMessage(groupId: string, content: string, messageType?: MessageType): Promise<UnifiedMessage> {
    const message = await callTool<UnifiedMessage>('send_message', {
      group_id: groupId,
      content,
      message_type: messageType,
    })

    // 缓存消息
    const cache = this.messageCache.get(groupId) || []
    cache.push(message)
    if (cache.length > 100) cache.shift()
    this.messageCache.set(groupId, cache)

    // 触发本地订阅
    this.notifySubscribers(groupId, message)

    return message
  }

  subscribe(groupId: string, handler: MessageHandler): UnsubscribeFunction {
    if (!this.subscriptions.has(groupId)) {
      this.subscriptions.set(groupId, new Set())
    }

    this.subscriptions.get(groupId)!.add(handler)

    // 返回取消订阅
    return () => {
      const handlers = this.subscriptions.get(groupId)
      if (handlers) {
        handlers.delete(handler)
        if (handlers.size === 0) {
          this.subscriptions.delete(groupId)
        }
      }
    }
  }

  async getHistory(groupId: string, limit: number = 50): Promise<UnifiedMessage[]> {
    // 优先从缓存返回
    const cached = this.messageCache.get(groupId)
    if (cached && cached.length > 0) {
      return cached.slice(-limit)
    }

    // 从后端获取
    const messages = await callTool<UnifiedMessage[]>('get_history', {
      group_id: groupId,
      limit,
    })

    this.messageCache.set(groupId, messages)
    return messages
  }

  protected notifySubscribers(groupId: string, message: UnifiedMessage): void {
    const handlers = this.subscriptions.get(groupId)
    if (handlers) {
      handlers.forEach(handler => {
        try {
          handler(message)
        } catch (error) {
          console.error('[GroupAdapter] 消息处理错误:', error)
        }
      })
    }
  }

  /**
   * 检测可用模式（静态方法）
   */
  static async detectAvailableModes(): Promise<GroupChatMode[]> {
    try {
      const modes = await callTool<string[]>('detect_available_modes', {})
      return modes.map(m => m.toLowerCase() as GroupChatMode)
    } catch {
      return [GroupChatMode.MEMORY]
    }
  }
}

/**
 * Memory 模式适配器
 */
export class MemoryGroupChatAdapter extends BaseAdapter {
  readonly mode: GroupChatMode = GroupChatMode.MEMORY
  readonly available: boolean = true

  constructor() {
    super(GroupChatMode.MEMORY)
  }
}

/**
 * PubSub 模式适配器
 */
export class PubSubGroupChatAdapter extends BaseAdapter {
  readonly mode: GroupChatMode = GroupChatMode.PUBSUB

  constructor() {
    super(GroupChatMode.PUBSUB)
  }
}

/**
 * Iroh 模式适配器
 */
export class IrohGroupChatAdapter extends BaseAdapter {
  readonly mode: GroupChatMode = GroupChatMode.IROH

  constructor() {
    super(GroupChatMode.IROH)
  }
}

// ============================================================================
// 适配器工厂
// ============================================================================

/**
 * 群聊适配器工厂 - 单例实现
 */
export class DefaultGroupChatAdapterFactory implements GroupChatAdapterFactory {
  private adapters: Map<GroupChatMode, GroupChatAdapter> = new Map()

  getAdapter(mode: GroupChatMode): GroupChatAdapter {
    if (this.adapters.has(mode)) {
      return this.adapters.get(mode)!
    }

    let adapter: GroupChatAdapter

    switch (mode) {
      case GroupChatMode.MEMORY:
        adapter = new MemoryGroupChatAdapter()
        break
      case GroupChatMode.PUBSUB:
        adapter = new PubSubGroupChatAdapter()
        break
      case GroupChatMode.IROH:
        adapter = new IrohGroupChatAdapter()
        break
      default:
        adapter = new MemoryGroupChatAdapter()
    }

    this.adapters.set(mode, adapter)
    return adapter
  }

  async getBestAdapter(): Promise<GroupChatAdapter> {
    // 尝试从后端获取可用模式
    try {
      const available = await BaseAdapter.detectAvailableModes()

      if (available.includes(GroupChatMode.IROH)) {
        return this.getAdapter(GroupChatMode.IROH)
      }
      if (available.includes(GroupChatMode.PUBSUB)) {
        return this.getAdapter(GroupChatMode.PUBSUB)
      }
    } catch {
      // 降级到 Memory
    }

    return this.getAdapter(GroupChatMode.MEMORY)
  }

  async detectAvailableAdapters(): Promise<GroupChatMode[]> {
    return BaseAdapter.detectAvailableModes()
  }

  onAdapterAvailabilityChange(_callback: (mode: GroupChatMode, available: boolean) => void): UnsubscribeFunction {
    // TODO: 实现可用性变化监听
    return () => {}
  }
}

// 导出工厂单例
export const groupChatAdapterFactory = new DefaultGroupChatAdapterFactory()

// ============================================================================
// 便捷函数
// ============================================================================

/**
 * 根据群聊自动选择适配器
 */
export function getAdapterForGroup(group: UnifiedGroup): GroupChatAdapter {
  const mode = group.mode.toLowerCase()
  if (mode === 'memory') {
    return groupChatAdapterFactory.getAdapter(GroupChatMode.MEMORY)
  } else if (mode === 'pubsub' || mode === 'pubsub') {
    return groupChatAdapterFactory.getAdapter(GroupChatMode.PUBSUB)
  } else if (mode === 'iroh') {
    return groupChatAdapterFactory.getAdapter(GroupChatMode.IROH)
  }
  return groupChatAdapterFactory.getAdapter(GroupChatMode.MEMORY)
}

/**
 * 根据模式名称获取适配器
 */
export function getAdapterByMode(mode: string): GroupChatAdapter {
  const normalizedMode = mode.toLowerCase()
  if (normalizedMode === 'memory') {
    return groupChatAdapterFactory.getAdapter(GroupChatMode.MEMORY)
  } else if (normalizedMode === 'pubsub' || normalizedMode === 'pubsub') {
    return groupChatAdapterFactory.getAdapter(GroupChatMode.PUBSUB)
  } else if (normalizedMode === 'iroh') {
    return groupChatAdapterFactory.getAdapter(GroupChatMode.IROH)
  }
  return groupChatAdapterFactory.getAdapter(GroupChatMode.MEMORY)
}

/**
 * 自动检测最佳适配器
 */
export async function getBestAvailableAdapter(): Promise<GroupChatAdapter> {
  return groupChatAdapterFactory.getBestAdapter()
}

/**
 * 检测最佳群聊模式
 */
export async function detectBestGroupChatMode(): Promise<GroupChatMode> {
  const available = await groupChatAdapterFactory.detectAvailableAdapters()

  if (available.includes(GroupChatMode.IROH)) {
    return GroupChatMode.IROH
  }

  if (available.includes(GroupChatMode.PUBSUB)) {
    return GroupChatMode.PUBSUB
  }

  return GroupChatMode.MEMORY
}

/**
 * 创建群聊的便捷函数
 */
export async function createUnifiedGroup(config: GroupChatConfig): Promise<UnifiedGroup> {
  let mode = config.mode ?? 'auto'

  if (mode === 'auto') {
    mode = await detectBestGroupChatMode()
  }

  const adapter = groupChatAdapterFactory.getAdapter(mode as GroupChatMode)
  return adapter.createGroup(config)
}

/**
 * 发送消息的便捷函数
 */
export async function sendUnifiedMessage(
  group: UnifiedGroup,
  content: string
): Promise<UnifiedMessage> {
  const adapter = getAdapterForGroup(group)
  return adapter.sendMessage(group.id, content)
}

// 重新导出类型
export type {
  GroupChatAdapter,
  GroupChatAdapterFactory,
  GroupChatConfig,
  GroupChatMode,
  MessageHandler,
  UnifiedGroup,
  UnifiedMessage,
  UnsubscribeFunction,
}
