/**
 * useUnifiedGroupChat - 统一群聊 Hook
 *
 * 基于新的适配器架构，统一管理 Memory、PubSub、Iroh 三种群聊模式
 * 提供简化的 API 和一致的使用体验
 *
 * @module hooks/useUnifiedGroupChat
 */

import { useCallback, useEffect, useState, useRef } from 'react'
import {
  GroupChatMode,
  UnifiedGroup,
  UnifiedMessage,
  MessageType,
  AgentInfo,
  GroupChatConfig,
  GroupChatMessage,
} from '@/types/groupchat'
import {
  groupChatAdapterFactory,
  getAdapterForGroup,
  getBestAvailableAdapter,
} from '@/adapters/groupChatAdapter'
import { agentCoordinator } from '@/services/unifiedAgentCoordinator'
import useClusterActionStore from '@/stores/clusterActionStore'
import { parseMentions } from '@/utils/mentionParser'

/**
 * Hook 选项
 */
export interface UseUnifiedGroupChatOptions {
  /** 当前频道 ID */
  activeChannelId?: string | null
  /** 本地身份信息 */
  localIdentity?: {
    did: string
    name?: string
    avatar?: string
  } | null
  /** 是否启用 */
  enabled?: boolean
  /** 群聊创建成功的回调 */
  onGroupCreated?: (group: UnifiedGroup, agents: AgentInfo[]) => void
  /** 收到消息的回调 */
  onMessage?: (groupId: string, message: UnifiedMessage) => void
}

/**
 * Hook 返回值
 */
export interface UseUnifiedGroupChatReturn {
  // 状态
  /** 是否已初始化 */
  isInitialized: boolean
  /** 当前群聊模式 */
  currentMode: GroupChatMode | null
  /** 活跃群聊 ID */
  activeGroupId: string | null
  /** 活跃群聊详情 */
  activeGroup: UnifiedGroup | null
  /** 活跃群聊消息 */
  messages: UnifiedMessage[]
  /** 所有可访问的群聊列表 */
  groups: UnifiedGroup[]
  /** 是否正在加载 */
  isLoading: boolean
  /** 错误信息 */
  error: string | null
  /** 适配器可用性状态 */
  adapterStatus: Record<string, boolean>

  // 操作方法
  /** 创建群聊 */
  createGroup: (config: GroupChatConfig) => Promise<UnifiedGroup>
  /** 加入群聊 */
  joinGroup: (groupId: string) => Promise<UnifiedGroup>
  /** 离开群聊 */
  leaveGroup: (groupId: string) => Promise<void>
  /** 切换到群聊 */
  switchGroup: (groupId: string) => Promise<void>
  /** 发送消息 */
  sendMessage: (groupId: string, content: string) => Promise<void>
  /** 刷新群聊列表 */
  refreshGroups: () => Promise<void>
  /** 加载历史消息 */
  loadHistory: (groupId: string, limit?: number) => Promise<void>

  // 工具方法
  /** 获取群聊详情 */
  getGroupById: (groupId: string) => UnifiedGroup | null
  /** 获取群聊消息 */
  getGroupMessages: (groupId: string) => UnifiedMessage[]
  /** 清除错误 */
  clearError: () => void
  /** 检测最佳适配器 */
  detectBestAdapter: () => Promise<GroupChatMode>
}

/**
 * 统一群聊 Hook
 */
export function useUnifiedGroupChat(
  options: UseUnifiedGroupChatOptions = {}
): UseUnifiedGroupChatReturn {
  const {
    activeChannelId = null,
    localIdentity = null,
    enabled = true,
    onGroupCreated,
    onMessage,
  } = options

  // 状态
  const [isInitialized, setIsInitialized] = useState(false)
  const [currentMode, setCurrentMode] = useState<GroupChatMode | null>(null)
  const [activeGroupId, setActiveGroupId] = useState<string | null>(null)
  const [activeGroup, setActiveGroup] = useState<UnifiedGroup | null>(null)
  const [messages, setMessages] = useState<UnifiedMessage[]>([])
  const [groups, setGroups] = useState<UnifiedGroup[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [adapterStatus, setAdapterStatus] = useState<Record<string, boolean>>({
    [GroupChatMode.MEMORY]: true,
    [GroupChatMode.PUBSUB]: false,
    [GroupChatMode.IROH]: false,
  })

  // 使用 ref 避免重复初始化
  const initializedRef = useRef(false)
  const activeGroupRef = useRef<UnifiedGroup | null>(null)
  const unsubscribeRef = useRef<(() => void) | null>(null)

  // 从 store 获取方法
  const {
    addAction,
    setActiveAction,
    getGroupChatMessages,
    addGroupChatMessage,
  } = useClusterActionStore()

  // ============================================================================
  // 工具方法（定义在前面，避免循环依赖）
  // ============================================================================

  /**
   * 获取群聊详情
   */
  const getGroupById = useCallback(
    (groupId: string): UnifiedGroup | null => {
      return groups.find((g) => g.id === groupId) || null
    },
    [groups]
  )

  /**
   * 清除错误
   */
  const clearError = useCallback(() => {
    setError(null)
  }, [])

  /**
   * 检测最佳适配器
   */
  const detectBestAdapter = useCallback(async (): Promise<GroupChatMode> => {
    const adapter = await getBestAvailableAdapter()
    return adapter.mode
  }, [])

  /**
   * 获取群聊消息
   */
  const getGroupMessages = useCallback(
    (groupId: string): UnifiedMessage[] => {
      if (groupId === activeGroupId) {
        return messages
      }
      // 从 store 获取，需要转换为 UnifiedMessage[]
      const storeMessages = getGroupChatMessages(groupId) || []
      return storeMessages.map((msg: any) => ({
        id: msg.id,
        groupId: msg.groupId || msg.group_id || groupId,
        type: msg.type as MessageType,
        sender: msg.sender || { id: msg.from, name: msg.fromName },
        content: msg.content,
        timestamp: msg.timestamp,
        metadata: msg.metadata,
        delivered: msg.delivered,
      }))
    },
    [activeGroupId, messages, getGroupChatMessages]
  )

  // ============================================================================
  // 内部方法（不导出）
  // ============================================================================

  /**
   * 触发智能体响应（内部方法）
   */
  const triggerAgentResponse = useCallback(
    (message: UnifiedMessage, mentions?: string[]) => {
      if (!activeGroupRef.current) {
        return
      }

      const group = activeGroupRef.current

      // 查找群聊中的智能体
      const agents = group.members.filter((m) => m.mode === 'agent')

      if (mentions && mentions.length > 0) {
        // 有明确提及，只通知被提及的智能体
        for (const mentionedId of mentions) {
          const agent = agents.find((a) => a.id === mentionedId)
          if (agent) {
            agentCoordinator.dispatchToAgent(agent.id, message)
          }
        }
      } else {
        // 没有提及，通知所有智能体
        for (const agent of agents) {
          agentCoordinator.dispatchToAgent(agent.id, message)
        }
      }
    },
    []
  )

  /**
   * 订阅群聊消息（内部方法）
   */
  const subscribeToGroup = useCallback(
    async (group: UnifiedGroup): Promise<void> => {
      const adapter = getAdapterForGroup(group)

      const unsubscribe = adapter.subscribe(group.id, (message: UnifiedMessage | GroupChatMessage) => {
        // 统一转换为 UnifiedMessage
        const unifiedMessage: UnifiedMessage = {
          id: message.id,
          groupId: 'groupId' in message ? (message as UnifiedMessage).groupId : (message as any).group_id,
          type: message.type as MessageType,
          sender: 'sender' in message ? (message as UnifiedMessage).sender : { id: (message as any).from, name: (message as any).fromName },
          content: message.content,
          timestamp: message.timestamp,
          metadata: message.metadata,
        }

        // 更新消息列表
        setMessages((prev) => {
          // 检查是否已存在
          const exists = prev.some((m) => m.id === unifiedMessage.id)
          if (exists) {
            return prev
          }
          return [...prev, unifiedMessage]
        })

        // 添加到 store - 转换为 GroupChatMessage 格式
        const groupChatMessage: any = {
          id: unifiedMessage.id,
          type: unifiedMessage.type,
          from: unifiedMessage.sender.id,
          fromName: unifiedMessage.sender.name,
          content: unifiedMessage.content,
          timestamp: unifiedMessage.timestamp,
          metadata: unifiedMessage.metadata,
        }
        addGroupChatMessage(group.id, groupChatMessage)

        // 回调
        onMessage?.(group.id, unifiedMessage)

        // 触发智能体响应
        triggerAgentResponse(unifiedMessage)
      })

      unsubscribeRef.current = unsubscribe
      console.log('[useUnifiedGroupChat] 已订阅群聊消息:', group.id)
    },
    [onMessage, addGroupChatMessage, triggerAgentResponse]
  )

  // ============================================================================
  // 核心操作方法
  // ============================================================================

  /**
   * 加载历史消息
   */
  const loadHistory = useCallback(async (groupId: string, limit: number = 50): Promise<void> => {
    if (!currentMode) {
      throw new Error('群聊未初始化')
    }

    const group = getGroupById(groupId)
    if (!group) {
      throw new Error(`群聊不存在：${groupId}`)
    }

    setIsLoading(true)

    try {
      const adapter = getAdapterForGroup(group)
      const history = await adapter.getHistory(groupId, limit)

      setMessages(history)
      console.log('[useUnifiedGroupChat] 历史消息已加载:', history.length)
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : '加载历史消息失败'
      console.error('[useUnifiedGroupChat] 加载历史消息失败:', err)
      setError(errorMsg)
      throw err
    } finally {
      setIsLoading(false)
    }
  }, [currentMode, getGroupById])

  /**
   * 切换到群聊
   */
  const switchGroup = useCallback(
    async (groupId: string): Promise<void> => {
      const group = groups.find((g) => g.id === groupId)

      if (!group) {
        throw new Error(`群聊不存在：${groupId}`)
      }

      // 如果已经在该群聊，直接返回
      if (activeGroupId === groupId) {
        return
      }

      // 取消之前的订阅
      if (unsubscribeRef.current) {
        unsubscribeRef.current()
      }

      // 设置活跃群聊
      setActiveGroupId(groupId)
      setActiveGroup(group)
      activeGroupRef.current = group

      // 加载历史消息
      await loadHistory(groupId)

      // 订阅新消息
      await subscribeToGroup(group)

      console.log('[useUnifiedGroupChat] 已切换到群聊:', groupId)
    },
    [activeGroupId, groups, loadHistory, subscribeToGroup]
  )

  /**
   * 发送消息
   */
  const sendMessage = useCallback(
    async (groupId: string, content: string): Promise<void> => {
      if (!currentMode) {
        throw new Error('群聊未初始化')
      }

      const group = getGroupById(groupId)
      if (!group) {
        throw new Error(`群聊不存在：${groupId}`)
      }

      try {
        const adapter = getAdapterForGroup(group)
        const message = await adapter.sendMessage(groupId, content, MessageType.CHAT)

        // 解析 @提及 - parseMentions 需要智能体列表作为第二个参数
        const agents = group.members.map(m => ({
          id: m.id,
          did: m.did,
          name: m.name,
          display_name: m.name,
        }))
        const parseResult = parseMentions(content, agents)
        const mentions = parseResult.rawMentions

        // 添加到本地消息列表
        setMessages((prev) => [...prev, message])
        
        // 添加到 store - 转换为 GroupChatMessage 格式
        const groupChatMessage: any = {
          id: message.id,
          type: message.type,
          from: message.sender.id,
          fromName: message.sender.name,
          content: message.content,
          timestamp: message.timestamp,
          metadata: message.metadata,
        }
        addGroupChatMessage(groupId, groupChatMessage)

        // 触发智能体响应
        if (mentions.length > 0) {
          triggerAgentResponse(message, mentions)
        }

        console.log('[useUnifiedGroupChat] 消息发送成功:', message.id)
      } catch (err) {
        const errorMsg = err instanceof Error ? err.message : '发送消息失败'
        console.error('[useUnifiedGroupChat] 发送消息失败:', err)
        setError(errorMsg)
        throw err
      }
    },
    [currentMode, addGroupChatMessage, getGroupById, triggerAgentResponse]
  )

  /**
   * 初始化 - 检测可用适配器并加载群聊列表
   */
  const initialize = useCallback(async () => {
    if (!enabled || initializedRef.current) {
      return
    }

    setIsLoading(true)
    setError(null)

    try {
      // 检测可用适配器 - 使用 detectAvailableAdapters 方法
      const availableModes = await groupChatAdapterFactory.detectAvailableAdapters()
      const status: Record<string, boolean> = {
        [GroupChatMode.MEMORY]: true, // Memory 始终可用
        [GroupChatMode.PUBSUB]: availableModes.includes(GroupChatMode.PUBSUB),
        [GroupChatMode.IROH]: availableModes.includes(GroupChatMode.IROH),
      }
      setAdapterStatus(status)

      // 选择最佳适配器
      const bestAdapter = await getBestAvailableAdapter()
      setCurrentMode(bestAdapter.mode)

      // 加载群聊列表
      await refreshGroups()

      initializedRef.current = true
      setIsInitialized(true)

      console.log('[useUnifiedGroupChat] 初始化成功，模式:', bestAdapter.mode)
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : '初始化失败'
      console.error('[useUnifiedGroupChat] 初始化失败:', err)
      setError(errorMsg)
    } finally {
      setIsLoading(false)
    }
  }, [enabled])

  /**
   * 刷新群聊列表
   */
  const refreshGroups = useCallback(async () => {
    if (!currentMode) return

    setIsLoading(true)
    setError(null)

    try {
      const adapter = groupChatAdapterFactory.getAdapter(currentMode)
      const groupList = await adapter.listGroups()
      setGroups(groupList)

      console.log('[useUnifiedGroupChat] 群聊列表已刷新:', groupList.length)
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : '刷新群聊列表失败'
      console.error('[useUnifiedGroupChat] 刷新群聊列表失败:', err)
      setError(errorMsg)
      throw err
    } finally {
      setIsLoading(false)
    }
  }, [currentMode])

  /**
   * 创建群聊
   */
  const createGroup = useCallback(
    async (config: GroupChatConfig): Promise<UnifiedGroup> => {
      if (!currentMode) {
        throw new Error('群聊未初始化')
      }

      setIsLoading(true)
      setError(null)

      try {
        const adapter = groupChatAdapterFactory.getAdapter(currentMode)
        const group = await adapter.createGroup(config)

        // 添加到 store（用于 UI 显示）
        const actionId = `${group.mode}_group_${group.id}`
        const action: any = {
          action_id: actionId,
          description: group.description || `${group.name}`,
          status: 'Active',
          created_at: new Date().toISOString(),
          agents: [
            {
              id: localIdentity?.did || 'unknown',
              name: localIdentity?.name || '用户',
              avatar: localIdentity?.avatar,
              mode: 'user',
            },
            ...(config.members || []),
          ],
          metadata: {
            type: `${group.mode}_group_chat`,
            [`${group.mode}_group_id`]: group.id,
            channel_id: activeChannelId,
          },
        }

        addAction(action)
        setActiveAction(actionId, activeChannelId || undefined)

        // 触发创建事件
        window.dispatchEvent(
          new CustomEvent('cluster-action-created', {
            detail: { actionId },
          })
        )

        // 回调
        onGroupCreated?.(group, config.members || [])

        // 更新状态
        setGroups((prev) => [...prev, group])

        console.log('[useUnifiedGroupChat] 群聊创建成功:', group.id)
        return group
      } catch (err) {
        const errorMsg = err instanceof Error ? err.message : '创建群聊失败'
        console.error('[useUnifiedGroupChat] 创建群聊失败:', err)
        setError(errorMsg)
        throw err
      } finally {
        setIsLoading(false)
      }
    },
    [currentMode, localIdentity, activeChannelId, onGroupCreated, addAction, setActiveAction]
  )

  /**
   * 加入群聊
   */
  const joinGroup = useCallback(
    async (groupId: string): Promise<UnifiedGroup> => {
      if (!currentMode) {
        throw new Error('群聊未初始化')
      }

      setIsLoading(true)
      setError(null)

      try {
        const adapter = groupChatAdapterFactory.getAdapter(currentMode)
        const group = await adapter.joinGroup(groupId)

        // 订阅群聊消息
        await subscribeToGroup(group)

        console.log('[useUnifiedGroupChat] 加入群聊成功:', group.id)
        return group
      } catch (err) {
        const errorMsg = err instanceof Error ? err.message : '加入群聊失败'
        console.error('[useUnifiedGroupChat] 加入群聊失败:', err)
        setError(errorMsg)
        throw err
      } finally {
        setIsLoading(false)
      }
    },
    [currentMode, subscribeToGroup]
  )

  /**
   * 离开群聊
   */
  const leaveGroup = useCallback(
    async (groupId: string): Promise<void> => {
      if (!currentMode) {
        throw new Error('群聊未初始化')
      }

      setIsLoading(true)
      setError(null)

      try {
        const adapter = groupChatAdapterFactory.getAdapter(currentMode)
        await adapter.leaveGroup(groupId)

        // 取消订阅
        if (unsubscribeRef.current) {
          unsubscribeRef.current()
          unsubscribeRef.current = null
        }

        // 更新状态
        if (activeGroupId === groupId) {
          setActiveGroupId(null)
          setActiveGroup(null)
          setMessages([])
        }

        setGroups((prev) => prev.filter((g) => g.id !== groupId))

        console.log('[useUnifiedGroupChat] 离开群聊成功:', groupId)
      } catch (err) {
        const errorMsg = err instanceof Error ? err.message : '离开群聊失败'
        console.error('[useUnifiedGroupChat] 离开群聊失败:', err)
        setError(errorMsg)
        throw err
      } finally {
        setIsLoading(false)
      }
    },
    [currentMode, activeGroupId]
  )

  // ============================================================================
  // Effects
  // ============================================================================

  // 初始化
  useEffect(() => {
    if (enabled && !initializedRef.current) {
      initialize()
    }

    return () => {
      // 清理订阅
      if (unsubscribeRef.current) {
        unsubscribeRef.current()
      }
    }
  }, [enabled, initialize])

  // 监听适配器可用性变化
  useEffect(() => {
    const unsubscribe = groupChatAdapterFactory.onAdapterAvailabilityChange(
      (mode: GroupChatMode, available: boolean) => {
        setAdapterStatus((prev) => ({
          ...prev,
          [mode]: available,
        }))

        // 如果当前模式不可用，切换到最佳可用模式
        if (currentMode && !available && mode === currentMode) {
          console.warn('[useUnifiedGroupChat] 当前模式不可用，切换中...')
          getBestAvailableAdapter().then((adapter) => {
            setCurrentMode(adapter.mode)
          })
        }
      }
    )

    return () => unsubscribe()
  }, [currentMode])

  // ============================================================================
  // 返回值
  // ============================================================================

  // 返回值
  return {
    // 状态
    isInitialized,
    currentMode,
    activeGroupId,
    activeGroup,
    messages,
    groups,
    isLoading,
    error,
    adapterStatus,

    // 操作方法
    createGroup,
    joinGroup,
    leaveGroup,
    switchGroup,
    sendMessage,
    refreshGroups,
    loadHistory,

    // 工具方法
    getGroupById,
    getGroupMessages,
    clearError,
    detectBestAdapter,
  }
}
