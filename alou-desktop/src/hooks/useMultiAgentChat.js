/**
 * useMultiAgentChat - 多智能体聊天和群聊 Hook
 * 提供群聊、智能体间通信、智能路由等功能
 */
import { useCallback, useEffect, useRef, useState } from 'react'
import pubsubService, { PubSubMessage, MessageType } from '@/services/pubsubService'
import agentCoordinatorService from '@/services/agentCoordinatorService'

/**
 * 群聊状态
 */
const GroupStatus = {
  IDLE: 'idle',
  JOINING: 'joining',
  ACTIVE: 'active',
  LEAVING: 'leaving',
  ERROR: 'error',
}

/**
 * 多智能体聊天 Hook
 */
export const useMultiAgentChat = ({
  localIdentity,
  registeredAgents = [],
  onAgentMessage,
  onGroupMessage,
}) => {
  // 群聊状态
  const [groups, setGroups] = useState([]) // 已加入的群聊列表
  const [activeGroupId, setActiveGroupId] = useState(null)
  const [groupMessages, setGroupMessages] = useState({}) // groupId -> messages[]
  const [groupStatus, setGroupStatus] = useState(GroupStatus.IDLE)

  // 智能体协调状态
  const [isCoordinatorReady, setCoordinatorReady] = useState(false)

  // 取消订阅函数引用
  const unsubscribeRefs = useRef({})

  // 初始化
  useEffect(() => {
    if (localIdentity) {
      pubsubService.setLocalIdentity(localIdentity)
    }
  }, [localIdentity])

  // 注册智能体到协调器
  useEffect(() => {
    if (registeredAgents.length > 0) {
      registeredAgents.forEach(agent => {
        agentCoordinatorService.registerAgent(agent)
      })
      setCoordinatorReady(true)
    }

    return () => {
      // 清理时注销智能体
      registeredAgents.forEach(agent => {
        const agentId = agent.id || agent.did || agent.ipns
        if (agentId) {
          agentCoordinatorService.unregisterAgent(agentId)
        }
      })
    }
  }, [registeredAgents])

  /**
   * 创建群聊
   */
  const createGroup = useCallback(async (groupName, members = []) => {
    setGroupStatus(GroupStatus.JOINING)
    try {
      const group = await pubsubService.createGroup(groupName, members)
      
      // 订阅群聊消息
      const unsubscribe = pubsubService.subscribe(group.topic, (message) => {
        handleGroupMessage(group.groupId, message)
      })
      unsubscribeRefs.current[group.groupId] = unsubscribe

      setGroups(prev => [...prev, group])
      setActiveGroupId(group.groupId)
      setGroupMessages(prev => ({ ...prev, [group.groupId]: [] }))
      setGroupStatus(GroupStatus.ACTIVE)

      return group
    } catch (error) {
      console.error('[useMultiAgentChat] 创建群聊失败:', error)
      setGroupStatus(GroupStatus.ERROR)
      throw error
    }
  }, [])

  /**
   * 加入群聊
   */
  const joinGroup = useCallback(async (topic, groupName = '群聊') => {
    setGroupStatus(GroupStatus.JOINING)
    try {
      // 从 topic 提取 groupId
      const groupId = topic.split('/').pop() || `group_${Date.now()}`

      const unsubscribe = await pubsubService.joinGroup(topic, (message) => {
        handleGroupMessage(groupId, message)
      })
      unsubscribeRefs.current[groupId] = unsubscribe

      const group = { groupId, topic, groupName, members: [] }
      setGroups(prev => [...prev, group])
      setActiveGroupId(groupId)
      setGroupMessages(prev => ({ ...prev, [groupId]: [] }))
      setGroupStatus(GroupStatus.ACTIVE)

      return group
    } catch (error) {
      console.error('[useMultiAgentChat] 加入群聊失败:', error)
      setGroupStatus(GroupStatus.ERROR)
      throw error
    }
  }, [])

  /**
   * 离开群聊
   */
  const leaveGroup = useCallback(async (groupId) => {
    setGroupStatus(GroupStatus.LEAVING)
    try {
      const group = groups.find(g => g.groupId === groupId)
      if (group) {
        await pubsubService.leaveGroup(group.topic)
      }

      // 取消订阅
      const unsubscribe = unsubscribeRefs.current[groupId]
      if (unsubscribe) {
        unsubscribe()
        delete unsubscribeRefs.current[groupId]
      }

      setGroups(prev => prev.filter(g => g.groupId !== groupId))
      setGroupMessages(prev => {
        const newMessages = { ...prev }
        delete newMessages[groupId]
        return newMessages
      })

      if (activeGroupId === groupId) {
        setActiveGroupId(null)
      }

      setGroupStatus(GroupStatus.IDLE)
    } catch (error) {
      console.error('[useMultiAgentChat] 离开群聊失败:', error)
      setGroupStatus(GroupStatus.ERROR)
    }
  }, [activeGroupId, groups])

  /**
   * 发送群聊消息
   */
  const sendGroupMessage = useCallback(async (groupId, content, metadata = {}) => {
    const group = groups.find(g => g.groupId === groupId)
    if (!group) {
      console.warn('[useMultiAgentChat] 群聊不存在:', groupId)
      return false
    }

    const success = await pubsubService.sendGroupMessage(group.topic, content, metadata)
    
    if (success) {
      // 本地添加消息（不等待 PubSub 回传）
      const localMessage = new PubSubMessage({
        type: MessageType.CHAT,
        from: localIdentity?.did,
        content,
        topic: group.topic,
        metadata: { ...metadata, isLocal: true },
      })
      
      setGroupMessages(prev => ({
        ...prev,
        [groupId]: [...(prev[groupId] || []), localMessage],
      }))
    }

    return success
  }, [groups, localIdentity])

  /**
   * 处理群聊消息
   */
  const handleGroupMessage = useCallback((groupId, message) => {
    // 跳过自己发送的消息（已经本地添加了）
    if (message.metadata?.isLocal) {
      return
    }

    setGroupMessages(prev => ({
      ...prev,
      [groupId]: [...(prev[groupId] || []), message],
    }))

    // 调用外部回调
    if (onGroupMessage) {
      onGroupMessage(groupId, message)
    }
  }, [onGroupMessage])

  /**
   * 智能路由消息到合适的智能体
   */
  const routeMessageToAgent = useCallback(async (message) => {
    if (!isCoordinatorReady) {
      console.warn('[useMultiAgentChat] 协调器未就绪')
      return null
    }

    const targetAgent = await agentCoordinatorService.routeMessage(message)
    return targetAgent
  }, [isCoordinatorReady])

  /**
   * 发送消息给指定智能体
   */
  const sendToAgent = useCallback(async (fromAgentId, toAgentId, message, metadata = {}) => {
    return agentCoordinatorService.sendAgentMessage(fromAgentId, toAgentId, message, metadata)
  }, [])

  /**
   * 请求智能体响应（带等待）
   */
  const requestFromAgent = useCallback(async (fromAgentId, toAgentId, message, timeoutMs = 30000) => {
    return agentCoordinatorService.requestFromAgent(fromAgentId, toAgentId, message, timeoutMs)
  }, [])

  /**
   * 广播消息给所有智能体
   */
  const broadcastToAgents = useCallback(async (message, excludeAgentId = null) => {
    return agentCoordinatorService.broadcastToAgents(message, excludeAgentId)
  }, [])

  /**
   * 设置智能体消息处理器
   */
  const setAgentMessageHandler = useCallback((agentId, handler) => {
    agentCoordinatorService.setMessageHandler(agentId, handler)
  }, [])

  /**
   * 获取已注册的智能体列表
   */
  const getRegisteredAgents = useCallback(() => {
    return agentCoordinatorService.getRegisteredAgents()
  }, [])

  /**
   * 分析用户意图
   */
  const analyzeIntent = useCallback((message) => {
    return agentCoordinatorService.analyzeIntent(message)
  }, [])

  // 清理
  useEffect(() => {
    return () => {
      // 取消所有订阅
      Object.values(unsubscribeRefs.current).forEach(fn => fn?.())
      unsubscribeRefs.current = {}
    }
  }, [])

  return {
    // 群聊
    groups,
    activeGroupId,
    setActiveGroupId,
    groupMessages,
    groupStatus,
    createGroup,
    joinGroup,
    leaveGroup,
    sendGroupMessage,

    // 智能体协调
    isCoordinatorReady,
    routeMessageToAgent,
    sendToAgent,
    requestFromAgent,
    broadcastToAgents,
    setAgentMessageHandler,
    getRegisteredAgents,
    analyzeIntent,
  }
}

export default useMultiAgentChat
