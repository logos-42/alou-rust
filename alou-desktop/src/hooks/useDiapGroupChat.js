/**
 * useDiapGroupChat - DIAP群聊Hook
 * 整合DIAP SDK的群聊创建、管理、消息传递功能
 * 支持智能体邀请和群聊协作
 */

import { useCallback, useEffect, useRef, useState } from 'react'
import diapGroupChatService, { DiapGroupConfig, DiapGroup, DiapGroupMessage } from '@/services/diapGroupChatService'
import { MessageType } from '@/services/pubsubService'
import useClusterActionStore from '@/stores/clusterActionStore'
import clusterActionService from '@/services/clusterActionService'
import apiClient from '@/services/api'
import { i18n } from '@/hooks/useI18n'

/**
 * 群聊状态枚举
 */
export const GroupChatStatus = {
  IDLE: 'idle',
  CREATING: 'creating',
  JOINING: 'joining',
  ACTIVE: 'active',
  LEAVING: 'leaving',
  ERROR: 'error'
}

/**
 * DIAP群聊Hook
 */
export const useDiapGroupChat = ({
  localIdentity,
  onMessage,
  onGroupCreated,
  onAgentJoined,
  onAgentLeft,
  onError,
  // 新增：兼容原有useGroupChat的参数
  actionId,
  enabled = true
} = {}) => {
  // 状态管理
  const [status, setStatus] = useState(GroupChatStatus.IDLE)
  const [groups, setGroups] = useState(new Map()) // groupId -> DiapGroup
  const [messages, setMessages] = useState(new Map()) // groupId -> DiapGroupMessage[]
  const [activeGroupId, setActiveGroupId] = useState(null)
  const [error, setError] = useState(null)

  // 引用管理
  const messageHandlersRef = useRef(new Map()) // groupId -> Set<callback>
  const subscriptionsRef = useRef(new Map()) // groupId -> unsubscribe function

  // Store相关
  const { addAction, setActiveAction, getActiveAction, addGroupChatMessage, addGroupChatMessages, getGroupChatMessages, updateActionStatus, setActionDetails, getActionStatus } = useClusterActionStore()

  // 检查是否是本地群聊（兼容原有逻辑）
  const isLocalGroupChat = useCallback((id) => {
    return id && id.startsWith('local_group_')
  }, [])

  // 转换 PubSub 消息格式为群聊消息格式（兼容原有逻辑）
  const transformPubSubMessage = useCallback((pubSubMsg) => {
    return {
      id: pubSubMsg.id || `msg_${Date.now()}_${Math.random()}`,
      type: pubSubMsg.msg_type || 'system',
      from: pubSubMsg.from || 'system',
      fromName: pubSubMsg.from === 'system' ? i18n.t('agent.groupChat.system') : pubSubMsg.from,
      avatar: pubSubMsg.metadata?.avatar || null,
      content: pubSubMsg.content || '',
      timestamp: pubSubMsg.timestamp || Date.now(),
      metadata: pubSubMsg.metadata || {},
    }
  }, [])

  // 初始化：设置本地身份
  useEffect(() => {
    if (localIdentity) {
      diapGroupChatService.setLocalIdentity(localIdentity)
      console.log('[useDiapGroupChat] 本地身份已设置:', localIdentity.did)
    }
  }, [localIdentity])

  // 兼容原有useGroupChat的消息订阅逻辑
  const subscriptionRef = useRef(null)
  const pollIntervalRef = useRef(null)
  const lastTimestampRef = useRef(Date.now())
  const unsubscribeMessagesRef = useRef(null)
  const unsubscribeStatusRef = useRef(null)

  // 订阅群聊消息（轮询方式）- 兼容原有逻辑
  const subscribeToGroupChat = useCallback(
    async (actionId, onMessage) => {
      if (!actionId) return () => {} // 返回空的清理函数而不是 null

      // 本地群聊不需要轮询后端
      if (isLocalGroupChat(actionId)) {
        return () => {} // 返回空的清理函数
      }

      const topic = `diap/cluster_action/${actionId}`
      let lastTimestamp = lastTimestampRef.current

      const poll = async () => {
        try {
          // 获取新消息
          const response = await apiClient.get(`/pubsub/messages`, {
            params: {
              topic,
              since: lastTimestamp,
            },
          })

          if (response.data?.messages) {
            const newMessages = response.data.messages
            if (newMessages.length > 0) {
              // 转换消息格式
              const transformedMessages = newMessages.map(transformPubSubMessage)
              
              // 添加到 store（兼容原有逻辑）
              addGroupChatMessages(actionId, transformedMessages)

              // 更新最后时间戳
              newMessages.forEach((msg) => {
                if (msg.timestamp > lastTimestamp) {
                  lastTimestamp = msg.timestamp
                }
              })
              lastTimestampRef.current = lastTimestamp

              // 调用回调
              transformedMessages.forEach((msg) => {
                onMessage?.(msg)
              })
            }
          }
        } catch (error) {
          // 静默处理 404 错误（可能是本地群聊或后端未准备好）
          if (error.response?.status !== 404) {
            console.error('[useDiapGroupChat] 获取群聊消息失败:', error)
          }
        }
      }

      // 立即执行一次
      await poll()

      // 设置轮询间隔（1秒）
      const interval = setInterval(poll, 1000)
      subscriptionRef.current = interval

      return () => {
        if (interval) {
          clearInterval(interval)
        }
      }
    },
    [addGroupChatMessages, transformPubSubMessage, isLocalGroupChat],
  )

  // 加载历史消息 - 兼容原有逻辑
  const loadGroupMessages = useCallback(
    async (actionId) => {
      if (!actionId) return

      // 本地群聊：添加欢迎消息
      if (isLocalGroupChat(actionId)) {
        // 检查是否已经添加过欢迎消息
        const existingMessages = getGroupChatMessages(actionId)
        const hasWelcome = existingMessages.some(msg => msg.id === `welcome_${actionId}`)
        
        if (!hasWelcome) {
          const action = getActiveAction()
          const description = action ? (action.description || actionId) : actionId
          const welcomeMessage = {
            id: `welcome_${actionId}`,
            type: 'system',
            from: 'system',
            fromName: i18n.t('agent.groupChat.system'),
            avatar: null,
            content: i18n.t('agent.groupChat.created', { description }),
            timestamp: Date.now(),
            metadata: {},
          }
          addGroupChatMessage(actionId, welcomeMessage)
        }
        return
      }

      try {
        const topic = `diap/cluster_action/${actionId}`
        const response = await apiClient.get(`/pubsub/messages`, {
          params: {
            topic,
          },
        })

        if (response.data?.messages) {
          const messages = response.data.messages.map(transformPubSubMessage)
          addGroupChatMessages(actionId, messages)

          // 更新最后时间戳
          if (messages.length > 0) {
            const latestTimestamp = Math.max(...messages.map((m) => m.timestamp))
            lastTimestampRef.current = latestTimestamp
          }
        }
      } catch (error) {
        // 静默处理 404 错误
        if (error.response?.status !== 404) {
          console.error('[useDiapGroupChat] 加载历史消息失败:', error)
        }
      }
    },
    [addGroupChatMessages, addGroupChatMessage, transformPubSubMessage, isLocalGroupChat, getActiveAction, getGroupChatMessages],
  )

  // 轮询行动状态 - 兼容原有逻辑
  const pollActionStatus = useCallback(
    async (actionId) => {
      if (!actionId) return

      // 本地群聊：使用本地状态，不轮询后端
      if (isLocalGroupChat(actionId)) {
        const action = getActiveAction()
        if (action) {
          updateActionStatus(actionId, action.status || 'Active')
        }
        return () => {} // 返回空的清理函数
      }

      // 检查是否是集群行动（action_ 开头）还是 AI 任务（task_ 开头）
      const isClusterAction = actionId.startsWith('action_')
      const isAiTask = actionId.startsWith('task_')

      if (!isClusterAction && !isAiTask) {
        console.warn('[useDiapGroupChat] 未知的行动类型:', actionId)
        return () => {}
      }

      // 如果是 AI 任务，不在这里轮询（由 useAsyncTaskPolling 处理）
      if (isAiTask) {
        console.log('[useDiapGroupChat] AI 任务状态由 useAsyncTaskPolling 处理，跳过群聊轮询:', actionId)
        return () => {}
      }

      const poll = async () => {
        try {
          const status = await clusterActionService.getActionStatus(actionId)
          updateActionStatus(actionId, status.status)

          // 如果行动完成，获取最终结果
          if (status.status === 'Completed' || status.status === 'Failed' || status.status === 'Cancelled') {
            try {
              const results = await clusterActionService.getActionResults(actionId)
              setActionDetails(actionId, results)
            } catch (error) {
              // 静默处理错误
              if (error.response?.status !== 404) {
                console.error('[useDiapGroupChat] 获取行动结果失败:', error)
              }
            }
          }
        } catch (error) {
          // 静默处理 404 错误
          if (error.response?.status !== 404) {
            console.error('[useDiapGroupChat] 轮询行动状态失败:', error)
          }
        }
      }

      // 立即执行一次
      await poll()

      // 设置轮询间隔（2秒）
      const interval = setInterval(poll, 2000)
      pollIntervalRef.current = interval

      return () => {
        if (interval) {
          clearInterval(interval)
        }
      }
    },
    [updateActionStatus, setActionDetails, isLocalGroupChat, getActiveAction]
  )

  // 兼容原有useGroupChat的初始化逻辑
  useEffect(() => {
    // 清理函数必须始终返回，即使条件不满足
    const cleanup = () => {
      if (unsubscribeMessagesRef.current && typeof unsubscribeMessagesRef.current === 'function') {
        unsubscribeMessagesRef.current()
        unsubscribeMessagesRef.current = null
      }
      if (unsubscribeStatusRef.current && typeof unsubscribeStatusRef.current === 'function') {
        unsubscribeStatusRef.current()
        unsubscribeStatusRef.current = null
      }
      if (subscriptionRef.current) {
        clearInterval(subscriptionRef.current)
        subscriptionRef.current = null
      }
      if (pollIntervalRef.current) {
        clearInterval(pollIntervalRef.current)
        pollIntervalRef.current = null
      }
    }

    if (!enabled || !actionId) {
      return cleanup
    }

    // 加载历史消息
    loadGroupMessages(actionId)

    // 订阅新消息
    subscribeToGroupChat(actionId, (message) => {
      console.log('[useDiapGroupChat] 收到新消息:', message)
    }).then((unsub) => {
      unsubscribeMessagesRef.current = unsub
    }).catch(() => {
      unsubscribeMessagesRef.current = () => {}
    })

    // 轮询状态
    pollActionStatus(actionId).then((unsub) => {
      unsubscribeStatusRef.current = unsub
    }).catch(() => {
      unsubscribeStatusRef.current = () => {}
    })

    // 返回清理函数
    return cleanup
  }, [enabled, actionId, subscribeToGroupChat, pollActionStatus, loadGroupMessages])

  // 获取当前消息列表 - 兼容原有逻辑
  const messagesFromStore = actionId ? getGroupChatMessages(actionId) : []

  // 获取当前状态 - 兼容原有逻辑
  const statusFromStore = actionId ? getActionStatus(actionId) : null

  /**
   * 创建群聊并邀请智能体
   * @param {Object} options - 创建选项
   * @param {string} options.groupName - 群聊名称
   * @param {string} options.description - 群聊描述
   * @param {Array} options.agents - 要邀请的智能体列表
   * @param {Object} options.channel - 频道信息（可选）
   * @param {Object} options.metadata - 额外元数据
   * @returns {Promise<DiapGroup>} 创建的群聊
   */
  const createGroupWithAgents = useCallback(async (options) => {
    const {
      groupName,
      description = '',
      agents = [],
      channel = null,
      metadata = {}
    } = options

    if (!localIdentity) {
      const error = new Error('未设置本地身份，无法创建群聊')
      setError(error.message)
      onError?.(error)
      throw error
    }

    if (!groupName || groupName.trim().length === 0) {
      const error = new Error('群聊名称不能为空')
      setError(error.message)
      onError?.(error)
      throw error
    }

    setStatus(GroupChatStatus.CREATING)
    setError(null)

    try {
      console.log('[useDiapGroupChat] 开始创建群聊:', {
        groupName,
        description,
        agentsCount: agents.length,
        channel: channel?.name
      })

      // 构建智能体DID列表
      const agentDids = agents.map(agent => 
        agent.did || agent.ipns || agent.cid || agent.id
      ).filter(Boolean)

      // 创建DIAP群聊配置
      const config = new DiapGroupConfig({
        groupName: groupName.trim(),
        description: description.trim(),
        members: agentDids,
        isPublic: metadata.isPublic || false,
        requireAuth: metadata.requireAuth !== false,
        maxMembers: metadata.maxMembers || 50,
        enableZkp: metadata.enableZkp !== false,
        topicPrefix: metadata.topicPrefix || 'diap/group'
      })

      // 创建群聊
      const group = await diapGroupChatService.createGroup(config)

      // 扩展群聊元数据
      group.metadata = {
        ...group.metadata,
        ...metadata,
        channel,
        invitedAgents: agents,
        createdAt: Date.now()
      }

      // 缓存群聊信息
      setGroups(prev => new Map(prev).set(group.groupId, group))
      setMessages(prev => new Map(prev).set(group.groupId, []))

      // 设置消息处理器
      setupMessageHandlers(group.groupId, group.topic)

      // 订阅群聊消息
      await subscribeToGroup(group.groupId, group.topic)

      // 创建对应的集群行动（用于UI显示）
      const actionId = `diap_group_${group.groupId}`
      const action = {
        action_id: actionId,
        description: group.description || `${group.groupName} (${agents.length}个智能体)`,
        status: 'Active',
        created_at: new Date().toISOString(),
        agents: [
          // 添加当前用户
          {
            id: localIdentity.did,
            did: localIdentity.did,
            name: localIdentity.name || '用户',
            avatar: localIdentity.avatar,
            mode: 'user'
          },
          // 添加被邀请的智能体
          ...agents.map(agent => ({
            id: agent.did || agent.ipns || agent.cid || agent.id,
            did: agent.did,
            ipns: agent.ipns,
            cid: agent.cid,
            name: agent.name || agent.display_name || '智能体',
            avatar: agent.avatar || agent.avatar_url,
            avatar_url: agent.avatar || agent.avatar_url,
            avatar_cid: agent.avatar_cid,
            mode: agent.mode || 'agent'
          }))
        ],
        metadata: {
          type: 'diap_group_chat',
          diap_group_id: group.groupId,
          diap_topic: group.topic,
          channel: channel,
          local: false // 标记为DIAP群聊，不是本地群聊
        }
      }

      // 添加到store
      addAction(action)
      
      // 如果有频道，设置活跃action
      if (channel) {
        setActiveAction(actionId, channel.id)
      }

      // 发送邀请消息给每个智能体
      for (const agent of agents) {
        await inviteAgentToGroup(group, agent)
      }

      // 设置为活跃群聊
      setActiveGroupId(group.groupId)

      setStatus(GroupChatStatus.ACTIVE)

      console.log('[useDiapGroupChat] 群聊创建成功:', group)

      // 触发回调
      onGroupCreated?.(group, agents)

      return group

    } catch (error) {
      console.error('[useDiapGroupChat] 创建群聊失败:', error)
      setStatus(GroupChatStatus.ERROR)
      setError(error.message)
      onError?.(error)
      throw error
    }
  }, [localIdentity, addAction, setActiveAction, onGroupCreated, onError])

  /**
   * 邀请智能体加入群聊
   * @param {DiapGroup} group - 群聊对象
   * @param {Object} agent - 智能体信息
   */
  const inviteAgentToGroup = useCallback(async (group, agent) => {
    try {
      const agentDid = agent.did || agent.ipns || agent.cid || agent.id
      const agentName = agent.name || agent.display_name || '智能体'

      // 发送邀请消息
      const inviteMessage = new DiapGroupMessage({
        groupId: group.groupId,
        topic: group.topic,
        from: localIdentity.did,
        fromName: localIdentity.name || '用户',
        to: agentDid,
        content: `邀请 ${agentName} 加入群聊 "${group.groupName}"`,
        type: MessageType.JOIN,
        metadata: {
          type: 'agent_invitation',
          invited_agent: {
            id: agent.id,
            did: agentDid,
            name: agentName,
            avatar: agent.avatar || agent.avatar_url
          },
          inviter: localIdentity.did
        }
      })

      await diapGroupChatService.sendMessage(inviteMessage)

      console.log('[useDiapGroupChat] 智能体邀请已发送:', { agent: agentName, group: group.groupName })

      // 触发回调
      onAgentJoined?.(group.groupId, agent)

    } catch (error) {
      console.error('[useDiapGroupChat] 邀请智能体失败:', error)
      onError?.(error)
    }
  }, [localIdentity, onAgentJoined, onError])

  /**
   * 设置消息处理器
   */
  const setupMessageHandlers = useCallback((groupId, topic) => {
    const handler = (message) => {
      console.log('[useDiapGroupChat] 收到消息:', message)

      // 添加到消息列表
      setMessages(prev => {
        const newMessages = new Map(prev)
        const groupMessages = newMessages.get(groupId) || []
        newMessages.set(groupId, [...groupMessages, message])
        return newMessages
      })

      // 触发外部回调
      onMessage?.(groupId, message)

      // 处理特殊消息类型
      if (message.type === MessageType.JOIN && message.metadata?.type === 'agent_invitation') {
        const invitedAgent = message.metadata.invited_agent
        if (invitedAgent) {
          onAgentJoined?.(groupId, invitedAgent)
        }
      } else if (message.type === MessageType.LEAVE) {
        const leavingAgent = message.metadata?.member
        if (leavingAgent) {
          onAgentLeft?.(groupId, leavingAgent)
        }
      }
    }

    // 添加处理器到服务
    diapGroupChatService.addMessageHandler(groupId, handler)

    // 缓存处理器
    if (!messageHandlersRef.current.has(groupId)) {
      messageHandlersRef.current.set(groupId, new Set())
    }
    messageHandlersRef.current.get(groupId).add(handler)
  }, [onMessage, onAgentJoined, onAgentLeft])

  /**
   * 订阅群聊消息
   */
  const subscribeToGroup = useCallback(async (groupId, topic) => {
    try {
      const unsubscribe = await diapGroupChatService.subscribeToGroup(groupId, topic)
      subscriptionsRef.current.set(groupId, unsubscribe)
      console.log('[useDiapGroupChat] 订阅群聊成功:', groupId)
    } catch (error) {
      console.error('[useDiapGroupChat] 订阅群聊失败:', error)
      throw error
    }
  }, [])

  /**
   * 发送消息到群聊
   * @param {string} groupId - 群聊ID
   * @param {string} content - 消息内容
   * @param {Object} metadata - 额外元数据
   */
  const sendMessage = useCallback(async (groupId, content, metadata = {}) => {
    try {
      const group = groups.get(groupId)
      if (!group) {
        throw new Error(`群聊 ${groupId} 不存在`)
      }

      const message = new DiapGroupMessage({
        groupId,
        topic: group.topic,
        from: localIdentity.did,
        fromName: localIdentity.name || '用户',
        content,
        type: MessageType.CHAT,
        metadata
      })

      await diapGroupChatService.sendMessage(message)

      console.log('[useDiapGroupChat] 消息发送成功:', { groupId, content })

    } catch (error) {
      console.error('[useDiapGroupChat] 发送消息失败:', error)
      onError?.(error)
      throw error
    }
  }, [groups, localIdentity, onError])

  /**
   * 加入现有群聊
   * @param {string} groupId - 群聊ID
   * @param {string} topic - 群聊主题（可选）
   */
  const joinGroup = useCallback(async (groupId, topic = null) => {
    setStatus(GroupChatStatus.JOINING)
    setError(null)

    try {
      const result = await diapGroupChatService.joinGroup(groupId, topic)
      
      // 构建群聊对象
      const group = new DiapGroup({
        groupId,
        topic: result.topic,
        members: [localIdentity.did]
      })

      // 缓存群聊信息
      setGroups(prev => new Map(prev).set(groupId, group))
      setMessages(prev => new Map(prev).set(groupId, []))

      // 设置消息处理器和订阅
      setupMessageHandlers(groupId, result.topic)
      await subscribeToGroup(groupId, result.topic)

      setActiveGroupId(groupId)
      setStatus(GroupChatStatus.ACTIVE)

      console.log('[useDiapGroupChat] 加入群聊成功:', groupId)

    } catch (error) {
      console.error('[useDiapGroupChat] 加入群聊失败:', error)
      setStatus(GroupChatStatus.ERROR)
      setError(error.message)
      onError?.(error)
      throw error
    }
  }, [localIdentity, setupMessageHandlers, subscribeToGroup, onError])

  /**
   * 离开群聊
   * @param {string} groupId - 群聊ID
   */
  const leaveGroup = useCallback(async (groupId) => {
    setStatus(GroupChatStatus.LEAVING)

    try {
      await diapGroupChatService.leaveGroup(groupId)

      // 清理本地状态
      setGroups(prev => {
        const newGroups = new Map(prev)
        newGroups.delete(groupId)
        return newGroups
      })

      setMessages(prev => {
        const newMessages = new Map(prev)
        newMessages.delete(groupId)
        return newMessages
      })

      // 清理处理器和订阅
      const unsubscribe = subscriptionsRef.current.get(groupId)
      if (unsubscribe) {
        unsubscribe()
        subscriptionsRef.current.delete(groupId)
      }

      messageHandlersRef.current.delete(groupId)

      if (activeGroupId === groupId) {
        setActiveGroupId(null)
      }

      setStatus(GroupChatStatus.IDLE)

      console.log('[useDiapGroupChat] 离开群聊成功:', groupId)

    } catch (error) {
      console.error('[useDiapGroupChat] 离开群聊失败:', error)
      setStatus(GroupChatStatus.ERROR)
      setError(error.message)
      onError?.(error)
    }
  }, [activeGroupId, onError])

  /**
   * 获取群聊消息列表
   * @param {string} groupId - 群聊ID
   * @returns {DiapGroupMessage[]} 消息列表
   */
  const getGroupMessages = useCallback((groupId) => {
    return messages.get(groupId) || []
  }, [messages])

  /**
   * 获取群聊信息
   * @param {string} groupId - 群聊ID
   * @returns {DiapGroup|null} 群聊信息
   */
  const getGroup = useCallback((groupId) => {
    return groups.get(groupId) || null
  }, [groups])

  /**
   * 切换活跃群聊
   * @param {string} groupId - 群聊ID
   */
  const switchActiveGroup = useCallback((groupId) => {
    if (groups.has(groupId)) {
      setActiveGroupId(groupId)
      console.log('[useDiapGroupChat] 切换活跃群聊:', groupId)
    }
  }, [groups])

  // 清理函数
  useEffect(() => {
    return () => {
      // 清理所有订阅
      subscriptionsRef.current.forEach((unsubscribe, groupId) => {
        try {
          unsubscribe()
        } catch (error) {
          console.error('[useDiapGroupChat] 清理订阅失败:', groupId, error)
        }
      })

      // 清理服务
      diapGroupChatService.cleanup()
    }
  }, [])

  return {
    // DIAP群聊状态
    status,
    error,
    activeGroupId,
    groups: Array.from(groups.values()),
    isLoading: status === GroupChatStatus.CREATING || status === GroupChatStatus.JOINING,

    // DIAP群聊操作
    createGroupWithAgents,
    joinGroup,
    leaveGroup,
    switchActiveGroup,

    // 消息操作
    sendMessage,
    getGroupMessages,
    getGroup,

    // 智能体邀请
    inviteAgentToGroup,

    // 兼容原有useGroupChat的返回值
    messages: messagesFromStore.length > 0 ? messagesFromStore : (activeGroupId ? Array.from(messages.get(activeGroupId) || []) : []),
    status: statusFromStore || status,
    subscribeToGroupChat,
    loadGroupMessages,
    pollActionStatus,

    // 工具方法
    clearError: () => setError(null)
  }
}

export default useDiapGroupChat
