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
  maxConcurrentTasks?: number // 最大并发任务数
}

/**
 * Hook 返回值
 */
export interface UseGroupChatAutonomousAgentReturn {
  isReady: boolean
  activeGroups: string[]
  processingMessages: Map<string, boolean>
  triggerAgentResponse: (groupId: string, message: GroupChatMessage, agentId: string) => Promise<void>
  broadcastToGroup: (groupId: string, content: string, fromAgentId: string) => Promise<void>
  getGroupAgents: (groupId: string) => AgentConfig[]
}

/**
 * 群聊自主智能体 Hook
 */
export const useGroupChatAutonomousAgent = (
  config: UseGroupChatAutonomousAgentConfig = {}
): UseGroupChatAutonomousAgentReturn => {
  const {
    enabled = true,
    mentionOnly = false,
    maxConcurrentTasks = 5,
  } = config

  // 状态管理
  const [isReady, setIsReady] = useState(false)
  const [activeGroups, setActiveGroups] = useState<string[]>([])
  const [processingMessages] = useState<Map<string, boolean>>(new Map())

  // 引用管理
  const agentTasksRef = useRef<Map<string, Promise<void>>>(new Map())
  const groupAgentsRef = useRef<Map<string, AgentConfig[]>>(new Map())
  const messageQueueRef = useRef<Map<string, GroupChatMessage[]>>(new Map())

  /**
   * 获取群聊中的所有智能体
   */
  const getGroupAgents = useCallback((groupId: string): AgentConfig[] => {
    // 从缓存获取
    const cached = groupAgentsRef.current.get(groupId)
    if (cached) {
      return cached
    }

    // 从 store 获取
    try {
      const { getActions } = useClusterActionStore.getState()
      const actions = getActions(null) || []
      
      // 查找匹配的群聊
      const groupAction = actions.find(action => 
        action.action_id === groupId || 
        action.metadata?.diap_group_id === groupId
      )

      if (groupAction && groupAction.agents) {
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

        groupAgentsRef.current.set(groupId, agents)
        return agents
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
    // 如果配置为仅在被 @ 时响应
    if (mentionOnly) {
      // 检查消息是否包含 @ 提及
      if (message.content.includes('@')) {
        return isAgentMentioned(agent.id, message.content, allAgents)
      }
      return false
    }

    // 自主响应模式：智能体根据消息内容判断是否响应
    const content = message.content.toLowerCase()
    
    // 1. 检查是否被直接 @
    if (content.includes('@' + agent.name)) {
      return true
    }

    // 2. 检查是否是问题或请求
    const questionPatterns = [
      '?', '？', '吗', '什么', '怎么', '为什么', '是否', '能否', '可以',
      'help', 'what', 'how', 'why', 'can', 'could', 'please'
    ]
    const isQuestion = questionPatterns.some(pattern => content.includes(pattern))

    // 3. 检查是否提及智能体相关关键词
    const agentKeywords = [
      agent.name,
      ...(agent.role_description ? [agent.role_description] : [])
    ]
    const mentionsAgent = agentKeywords.some(keyword => 
      keyword && content.includes(keyword.toLowerCase())
    )

    // 4. 如果是问题且提及智能体，应该响应
    if (isQuestion && mentionsAgent) {
      return true
    }

    // 5. 如果是直接的问题，智能体可以主动响应
    if (isQuestion && Math.random() > 0.5) {
      return true
    }

    // 6. 如果消息是任务分配或协作请求
    const taskPatterns = ['任务', 'task', '分配', 'assign', '请', '帮忙', 'help']
    const isTaskRequest = taskPatterns.some(pattern => content.includes(pattern))
    if (isTaskRequest && mentionsAgent) {
      return true
    }

    return false
  }, [mentionOnly])

  /**
   * 触发智能体响应
   */
  const triggerAgentResponse = useCallback(async (
    groupId: string,
    message: GroupChatMessage,
    agentId: string
  ): Promise<void> => {
    const taskKey = `${groupId}_${agentId}_${message.id}`
    
    // 检查是否正在处理
    if (processingMessages.get(taskKey)) {
      console.log('[useGroupChatAutonomousAgent] 消息正在处理中，跳过:', taskKey)
      return
    }

    // 检查并发任务数
    const activeTasks = Array.from(agentTasksRef.current.values())
      .filter(task => task !== undefined).length
    
    if (activeTasks >= maxConcurrentTasks) {
      console.log('[useGroupChatAutonomousAgent] 达到最大并发任务数，消息已加入队列')
      // 加入消息队列
      if (!messageQueueRef.current.has(groupId)) {
        messageQueueRef.current.set(groupId, [])
      }
      messageQueueRef.current.get(groupId)!.push(message)
      return
    }

    try {
      processingMessages.set(taskKey, true)

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
      processingMessages.set(taskKey, false)
      agentTasksRef.current.delete(taskKey)
    }
  }, [getGroupAgents, shouldAgentRespond, responseDelay, maxConcurrentTasks, processingMessages])

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
      if (actionId && !activeGroups.includes(actionId)) {
        setActiveGroups(prev => [...prev, actionId])
        console.log('[useGroupChatAutonomousAgent] 新群聊已激活:', actionId)
      }
    }

    window.addEventListener('cluster-action-created', handleClusterActionCreated)

    // 初始化时加载现有群聊
    try {
      const { getActions } = useClusterActionStore.getState()
      const actions = getActions(null) || []
      const groupIds = actions
        .filter(action => 
          action.action_id?.startsWith('local_group_') ||
          action.action_id?.startsWith('diap_group_') ||
          action.metadata?.type === 'group_chat'
        )
        .map(action => action.action_id)
      
      setActiveGroups(groupIds)
      console.log('[useGroupChatAutonomousAgent] 已加载活跃群聊:', groupIds.length)
    } catch (error) {
      console.error('[useGroupChatAutonomousAgent] 加载群聊失败:', error)
    }

    return () => {
      window.removeEventListener('cluster-action-created', handleClusterActionCreated)
    }
  }, [activeGroups])

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
    processingMessages,
    triggerAgentResponse,
    broadcastToGroup,
    getGroupAgents
  }
}

export default useGroupChatAutonomousAgent
