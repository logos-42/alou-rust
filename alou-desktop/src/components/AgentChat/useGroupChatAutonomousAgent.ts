/**
 * useGroupChatAutonomousAgent - 群聊自主智能体 Hook
 * 实现智能体在群聊中的自主响应和协作功能
 * 支持 iroh、pubsub 和 memory 三种群聊模式的持续运行
 */

import { useCallback, useEffect, useRef, useState } from 'react'
import useClusterActionStore from '@/stores/clusterActionStore'
import { isAgentMentioned } from '@/utils/mentionParser'

/**
 * 群聊消息接口
 */
export interface GroupChatMessage {
  id: string
  groupId: string
  from: string
  fromName: string
  content: string
  timestamp: number
  type?: 'user' | 'agent' | 'system'
  metadata?: Record<string, unknown>
}

/**
 * 智能体配置
 */
export interface AgentConfig {
  id: string
  name: string
  role_description?: string
  customPrompt?: string
  avatar?: string
  mode?: 'agent' | 'user'
}

/**
 * Hook 配置
 */
export interface UseGroupChatAutonomousAgentConfig {
  enabled?: boolean // 是否启用自主响应
  mentionOnly?: boolean // 仅在被 @ 时响应
}

/**
 * Hook 返回值
 */
export interface UseGroupChatAutonomousAgentReturn {
  isReady: boolean
  activeGroups: string[]
  triggerAgentResponse: (groupId: string, message: GroupChatMessage, agentId: string) => Promise<void>
  broadcastToGroup: (groupId: string, content: string, fromAgentId: string) => Promise<void>
  getGroupAgents: (groupId: string) => AgentConfig[]
}

/**
 * 群聊自主智能体 Hook（无并发限制）
 */
export const useGroupChatAutonomousAgent = (
  config: UseGroupChatAutonomousAgentConfig = {}
): UseGroupChatAutonomousAgentReturn => {
  const {
    enabled = true,
    mentionOnly = false,
  } = config

  // 状态管理
  const [isReady, setIsReady] = useState(false)
  const [activeGroups, setActiveGroups] = useState<string[]>([])

  // 引用管理
  const agentTasksRef = useRef<Map<string, Promise<void>>>(new Map())
  const groupAgentsRef = useRef<Map<string, AgentConfig[]>>(new Map())
  const activeGroupsRef = useRef<string[]>([])
  const messageQueueRef = useRef<Map<string, string[]>>(new Map())
  const processingMessages = useRef<Map<string, boolean>>(new Map())

  /**
   * 获取群聊中的所有智能体
   */
  const getGroupAgents = useCallback((groupId: string): AgentConfig[] => {
    // 从缓存获取
    const cached = groupAgentsRef.current.get(groupId)
    if (cached) {
      console.log('[useGroupChatAutonomousAgent] 从缓存获取智能体:', groupId, cached.length, '个')
      return cached
    }

    // 从 store 获取
    try {
      const { getActions } = useClusterActionStore.getState()
      
      // 🔥 关键修复：获取所有频道的 actions，而不是只获取空字符串 channelId
      // 先获取所有已知的 channelId
      const state = useClusterActionStore.getState()
      const channelIds = Object.keys(state.actionsByChannel || {})
      
      // 收集所有 actions
      let allActions: any[] = []
      channelIds.forEach(channelId => {
        const actions = getActions(channelId) || []
        if (actions.length > 0) {
          allActions = [...allActions, ...actions]
        }
      })
      
      // 如果还是没有 actions，尝试直接获取所有 action
      if (allActions.length === 0) {
        allActions = Object.values(state.actionsByChannel || {}).flat()
      }

      console.log('[useGroupChatAutonomousAgent] 从 store 获取智能体，groupId:', groupId, 'channelIds:', channelIds, 'actions 数量:', allActions.length)

      // 查找匹配的群聊 - 支持多种 ID 格式
      const groupAction = allActions.find(action => {
        const match = action.action_id === groupId ||
          action.metadata?.diap_group_id === groupId ||
          action.metadata?.local_group_id === groupId
        console.log('[useGroupChatAutonomousAgent] 检查 action:', action.action_id, '匹配:', match)
        return match
      })

      if (groupAction && groupAction.agents) {
        console.log('[useGroupChatAutonomousAgent] 找到群聊 action，agents 数量:', groupAction.agents.length)
        const agents: AgentConfig[] = groupAction.agents
          .filter(agent => agent.mode === 'agent')
          .map(agent => ({
            id: agent.id || agent.agent_id,
            name: agent.name || agent.display_name || '智能体',
            role_description: agent.role_description,
            customPrompt: agent.customPrompt,
            avatar: agent.avatar,
            mode: 'agent'
          }))

        console.log('[useGroupChatAutonomousAgent] 过滤后的智能体数量:', agents.length, agents.map(a => a.id))
        groupAgentsRef.current.set(groupId, agents)
        return agents
      } else {
        console.warn('[useGroupChatAutonomousAgent] 未找到群聊 action 或 agents:', groupId)
      }
    } catch (error) {
      console.error('[useGroupChatAutonomousAgent] 获取群聊智能体失败:', error)
    }

    return []
  }, [])

  /**
   * 判断智能体是否应该响应消息
   */
  const shouldAgentRespond = useCallback((
    agent: AgentConfig,
    message: GroupChatMessage,
    allAgents: AgentConfig[]
  ): boolean => {
    // 系统消息不响应
    if (message.type === 'system') {
      return false
    }
    
    // 如果配置为仅在被 @ 时响应
    if (mentionOnly) {
      // 检查消息是否包含 @ 提及
      if (message.content.includes('@')) {
        return isAgentMentioned(agent.id, message.content, allAgents)
      }
      return false
    }

    // ✅ 关键修复：如果只有一个智能体，始终响应（除非是系统消息）
    if (allAgents.length === 1) {
      return true
    }

    // 🔥 群聊模式：多个智能体时，使用轮询或随机响应
    // 每个智能体有 50% 的概率响应，确保消息能得到回复
    const content = message.content.toLowerCase()
    
    // 1. 检查是否被直接 @ - 必须响应
    if (content.includes('@' + agent.name)) {
      console.log('[shouldAgentRespond] 智能体被@，响应')
      return true
    }

    // 2. 检查是否是问题或请求 - 高概率响应
    const questionPatterns = [
      '?', '？', '吗', '什么', '怎么', '为什么', '是否', '能否', '可以',
      'help', 'what', 'how', 'why', 'can', 'could', 'please'
    ]
    const isQuestion = questionPatterns.some(pattern => content.includes(pattern))
    
    if (isQuestion) {
      // 问题消息，70% 概率响应
      if (Math.random() < 0.7) {
        console.log('[shouldAgentRespond] 智能体响应问题')
        return true
      }
    }

    // 3. 检查是否是问候语或简单消息 - 低概率响应，避免多个智能体同时回复
    const greetingPatterns = ['你好', 'hello', 'hi', '嗨', '早', '好', '在吗', '喂', 'test', '测试']
    const isGreeting = greetingPatterns.some(pattern => content.includes(pattern))
    
    if (isGreeting) {
      // 问候语，30% 概率响应
      if (Math.random() < 0.3) {
        console.log('[shouldAgentRespond] 智能体响应问候')
        return true
      }
    }

    // 4. 检查是否提及智能体相关关键词
    const agentKeywords = [
      agent.name,
      ...(agent.role_description ? [agent.role_description] : [])
    ]
    const mentionsAgent = agentKeywords.some(keyword =>
      keyword && content.includes(keyword.toLowerCase())
    )

    if (mentionsAgent) {
      // 提及智能体，60% 概率响应
      if (Math.random() < 0.6) {
        console.log('[shouldAgentRespond] 智能体响应提及')
        return true
      }
    }

    // 5. 其他消息，20% 概率随机响应（保持对话活跃）
    if (Math.random() < 0.2) {
      console.log('[shouldAgentRespond] 智能体随机响应')
      return true
    }

    return false
  }, [mentionOnly])

  /**
   * 触发智能体响应（无并发限制）
   */
  const triggerAgentResponse = useCallback(async (
    groupId: string,
    message: GroupChatMessage,
    agentId: string
  ): Promise<void> => {
    const taskKey = `${groupId}_${agentId}_${message.id}`

    // ✅ 移除正在处理检查 - 允许同一消息被多个 agent 同时处理
    // if (processingMessages.get(taskKey)) {
    //   console.log('[useGroupChatAutonomousAgent] 消息正在处理中，跳过:', taskKey)
    //   return
    // }

    // ✅ 移除并发限制 - 允许所有 agent 同时响应
    // const activeTasks = Array.from(agentTasksRef.current.values())
    //   .filter(task => task !== undefined).length
    // if (activeTasks >= maxConcurrentTasks) {
    //   console.log('[useGroupChatAutonomousAgent] 达到最大并发任务数，消息已加入队列')
    //   if (!messageQueueRef.current.has(groupId)) {
    //     messageQueueRef.current.set(groupId, [])
    //   }
    //   messageQueueRef.current.get(groupId)!.push(message)
    //   return
    // }

    try {
      processingMessages.current.set(taskKey, true)
      agentTasksRef.current.set(taskKey, Promise.resolve())

      console.log('[useGroupChatAutonomousAgent] 触发智能体响应:', {
        agentId,
        groupId,
        messageContent: message.content.slice(0, 50)
      })

      // 获取智能体信息
      const agents = getGroupAgents(groupId)
      const agent = agents.find(a => a.id === agentId)

      if (!agent) {
        console.warn('[useGroupChatAutonomousAgent] 未找到智能体:', agentId)
        return
      }

      // 判断是否应该响应
      if (!shouldAgentRespond(agent, message, agents)) {
        console.log('[useGroupChatAutonomousAgent] 智能体不需要响应:', agentId)
        return
      }

      // 立即触发智能体响应（无延迟）
      console.log('[useGroupChatAutonomousAgent] 立即触发智能体响应:', agentId)
      const systemPrompt = `你是一个群聊智能体，正在参与群聊 "${groupId}"。

你的角色：${agent.role_description || '助手'}
你的名字：${agent.name}

群聊规则：
1. 只在被@或消息与你相关时响应
2. 保持回复简洁明了
3. 与其他智能体协作完成任务
4. 如果消息是任务分配，确认接收并执行

当前消息来自：${message.fromName}
消息内容：${message.content}

请根据上下文给出合适的回复。`

      // 触发智能体处理消息的事件
      window.dispatchEvent(new CustomEvent('agent-group-message', {
        detail: {
          agentId,
          message: {
            ...message,
            type: 'group_chat_message',
            isMentioned: message.content.includes('@' + agent.name),
            mentionedAgentIds: [agentId],
            metadata: {
              ...message.metadata,
              systemPrompt,
              groupId,
              isGroupChat: true
            }
          }
        }
      }))

      console.log('[useGroupChatAutonomousAgent] 智能体响应已触发:', agentId)

    } catch (error) {
      console.error('[useGroupChatAutonomousAgent] 触发智能体响应失败:', error)
    } finally {
      processingMessages.current.set(taskKey, false)
      agentTasksRef.current.delete(taskKey)
    }
  }, [getGroupAgents, shouldAgentRespond])

  /**
   * 广播消息到群聊
   */
  const broadcastToGroup = useCallback(async (
    groupId: string,
    content: string,
    fromAgentId: string
  ): Promise<void> => {
    try {
      console.log('[useGroupChatAutonomousAgent] 广播消息到群聊:', {
        groupId,
        fromAgentId,
        content: content.slice(0, 50)
      })

      // 创建广播消息
      const broadcastMessage: GroupChatMessage = {
        id: `broadcast_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`,
        groupId,
        from: fromAgentId,
        fromName: '系统',
        content,
        timestamp: Date.now(),
        type: 'system',
        metadata: {
          isBroadcast: true,
          fromAgentId
        }
      }

      // 获取群聊中的所有智能体
      const agents = getGroupAgents(groupId)

      // 通知每个智能体（除了发送者）
      for (const agent of agents) {
        if (agent.id !== fromAgentId) {
          await triggerAgentResponse(groupId, broadcastMessage, agent.id)
        }
      }

      // 保存到群聊消息存储
      try {
        const { addGroupChatMessage } = useClusterActionStore.getState()
        addGroupChatMessage(groupId, {
          id: broadcastMessage.id,
          type: 'agent',
          from: fromAgentId,
          fromName: '智能体',
          content,
          timestamp: broadcastMessage.timestamp,
          metadata: broadcastMessage.metadata
        })
      } catch (storeError) {
        console.warn('[useGroupChatAutonomousAgent] 保存广播消息失败:', storeError)
      }

    } catch (error) {
      console.error('[useGroupChatAutonomousAgent] 广播消息失败:', error)
    }
  }, [getGroupAgents, triggerAgentResponse])

  /**
   * 处理消息队列
   */
  const processMessageQueue = useCallback(async (groupId: string): Promise<void> => {
    const queue = messageQueueRef.current.get(groupId)
    if (!queue || queue.length === 0) return

    console.log('[useGroupChatAutonomousAgent] 处理消息队列:', groupId, queue.length, '条消息')

    while (queue.length > 0) {
      const message = queue.shift()
      if (message) {
        const agents = getGroupAgents(groupId)
        for (const agent of agents) {
          await triggerAgentResponse(groupId, message, agent.id)
        }
      }
      // 每条消息之间添加延迟
      await new Promise(resolve => setTimeout(resolve, 500))
    }
  }, [getGroupAgents, triggerAgentResponse])

  /**
   * 监听群聊消息事件
   */
  useEffect(() => {
    if (!enabled) {
      console.log('[useGroupChatAutonomousAgent] 自主响应已禁用')
      return
    }

    const handleAgentGroupMessage = async (event: Event) => {
      const customEvent = event as CustomEvent<{
        agentId: string
        message: GroupChatMessage
      }>
      const { agentId, message } = customEvent.detail
      
      console.log('[useGroupChatAutonomousAgent] 收到群聊消息事件:', {
        agentId,
        groupId: message.groupId,
        content: message.content.slice(0, 50)
      })

      // 触发智能体响应
      await triggerAgentResponse(message.groupId, message, agentId)
    }

    window.addEventListener('agent-group-message', handleAgentGroupMessage)

    // 设置为就绪状态
    setIsReady(true)

    return () => {
      window.removeEventListener('agent-group-message', handleAgentGroupMessage)
    }
  }, [enabled, triggerAgentResponse])

  /**
   * 监听群聊创建和更新
   */
  useEffect(() => {
    const handleClusterActionCreated = (event: Event) => {
      const customEvent = event as CustomEvent<{ actionId: string }>
      const { actionId } = customEvent.detail
      if (actionId && !activeGroupsRef.current.includes(actionId)) {
        setActiveGroups(prev => {
          const updated = [...prev, actionId]
          activeGroupsRef.current = updated
          return updated
        })
        console.log('[useGroupChatAutonomousAgent] 新群聊已激活:', actionId)
      }
    }

    window.addEventListener('cluster-action-created', handleClusterActionCreated)

    // 初始化时加载现有群聊
    try {
      const { getActions } = useClusterActionStore.getState()
      const actions = getActions("") || []
      const groupIds = actions
        .filter(action => 
          action.action_id?.startsWith('local_group_') ||
          action.action_id?.startsWith('diap_group_') ||
          action.metadata?.type === 'group_chat'
        )
        .map(action => action.action_id)
      
      setActiveGroups(groupIds)
      activeGroupsRef.current = groupIds
      console.log('[useGroupChatAutonomousAgent] 已加载活跃群聊:', groupIds.length)
    } catch (error) {
      console.error('[useGroupChatAutonomousAgent] 加载群聊失败:', error)
    }

    return () => {
      window.removeEventListener('cluster-action-created', handleClusterActionCreated)
    }
  }, [])

  /**
   * 定期处理消息队列
   */
  useEffect(() => {
    const interval = setInterval(async () => {
      for (const groupId of messageQueueRef.current.keys()) {
        await processMessageQueue(groupId)
      }
    }, 2000)

    return () => clearInterval(interval)
  }, [processMessageQueue])

  return {
    isReady,
    activeGroups,
    triggerAgentResponse,
    broadcastToGroup,
    getGroupAgents
  }
}

export default useGroupChatAutonomousAgent
