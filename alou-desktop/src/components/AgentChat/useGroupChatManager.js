import { useCallback, useEffect, useMemo, useState, useRef } from 'react'
import { useDiapGroupChat } from '@/hooks/useDiapGroupChat'
import useClusterActionStore from '@/stores/clusterActionStore'

/**
 * useGroupChatManager - 基于DIAP PubSub的群聊管理 Hook
 * 统一管理群聊相关的状态、事件监听和UI控制
 */
export const useGroupChatManager = ({ openConversationPanel, activeChannelId, localIdentity }) => {
  // 从 store 获取集群行动相关状态
  const {
    getActions,
    getActiveActionId,
    setActiveAction,
    getActiveAction,
    getGroupChatMessages,
    getActionStatus,
    loadChannelGroupChats,
  } = useClusterActionStore()

  // 使用 ref 跟踪已加载的频道，避免重复加载
  const loadedChannelRef = useRef(null)
  const openConversationPanelRef = useRef(openConversationPanel)
  
  // 更新 ref
  useEffect(() => {
    openConversationPanelRef.current = openConversationPanel
  }, [openConversationPanel])

  // DIAP群聊Hook
  const diapGroupChat = useDiapGroupChat({
    localIdentity,
    onGroupCreated: (group, agents) => {
      console.log('[useGroupChatManager] DIAP群聊创建成功:', group.groupName)
      
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
            id: localIdentity?.did,
            did: localIdentity?.did,
            name: localIdentity?.name || '用户',
            avatar: localIdentity?.avatar,
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
          channel: activeChannelId ? { id: activeChannelId } : null,
          local: false // 标记为DIAP群聊，不是本地群聊
        }
      }

      // 添加到store
      setActiveAction(actionId, activeChannelId)
      
      // 触发群聊显示事件
      window.dispatchEvent(
        new CustomEvent('cluster-action-created', {
          detail: { actionId },
        }),
      )
    },
    onMessage: (groupId, message) => {
      console.log('[useGroupChatManager] 收到DIAP消息:', groupId, message.content)
    },
    onError: (error) => {
      console.error('[useGroupChatManager] DIAP群聊错误:', error)
    }
  })
  
  // 更新 ref
  useEffect(() => {
    openConversationPanelRef.current = openConversationPanel
  }, [openConversationPanel])

  // 状态管理
  const [showGroupChat, setShowGroupChat] = useState(false)
  const [activeGroupId, setActiveGroupId] = useState(null)

  // 获取当前活跃的群聊（优先从DIAP群聊查找，然后从本地store查找）
  const activeGroup = useMemo(() => {
    if (!activeGroupId) return null
    
    // 先从DIAP群聊查找
    const diapGroup = diapGroupChat.groups.find(g => g.groupId === activeGroupId)
    if (diapGroup) return diapGroup
    
    // 从本地store查找
    const activeAction = getActiveAction(activeChannelId)
    if (activeAction && (activeAction.action_id === activeGroupId || activeAction.action_id?.includes(activeGroupId))) {
      return {
        groupId: activeGroupId,
        groupName: activeAction.description?.replace('群聊: ', '').split(' + ')[0] || '本地群聊',
        description: activeAction.description,
        agents: activeAction.agents || [],
        metadata: {
          type: 'local_group',
          channel: { id: activeChannelId }
        }
      }
    }
    
    return null
  }, [diapGroupChat.groups, activeGroupId, getActiveAction, activeChannelId])

  // 获取当前活跃群聊的消息（优先从DIAP群聊查找，然后从本地store查找）
  const activeGroupMessages = useMemo(() => {
    if (!activeGroupId) return []
    
    // 先从DIAP群聊获取消息
    if (diapGroupChat.messages && diapGroupChat.messages.length > 0) {
      return diapGroupChat.messages
    }
    
    // 从本地store获取消息
    const messages = getGroupChatMessages(activeGroupId)
    return messages || []
  }, [diapGroupChat.messages, activeGroupId, getGroupChatMessages])

  // 获取当前频道的群聊列表（合并DIAP群聊和本地群聊）
  const groupChatList = useMemo(() => {
    if (!activeChannelId) return []
    
    // 获取DIAP群聊
    const diapGroups = diapGroupChat.groups.filter(group => 
      group.metadata?.channel?.id === activeChannelId
    )
    
    // 获取本地群聊
    const localActions = getActions(activeChannelId) || []
    const localGroups = localActions
      .filter(action => 
        action.action_id?.startsWith('local_group_') || 
        action.action_id?.startsWith('action_') ||
        action.metadata?.type === 'group_chat'
      )
      .map(action => ({
        groupId: action.action_id,
        groupName: action.description?.replace('群聊: ', '').split(' + ')[0] || '本地群聊',
        description: action.description,
        agents: action.agents || [],
        metadata: {
          type: 'local_group',
          channel: { id: activeChannelId }
        }
      }))
    
    // 合并群聊列表
    return [...diapGroups, ...localGroups]
  }, [diapGroupChat.groups, activeChannelId, getActions])

  // 创建群聊
  const createGroupChat = useCallback(async (groupName, agents = []) => {
    if (!localIdentity) {
      throw new Error('未设置本地身份，无法创建群聊')
    }

    if (!activeChannelId) {
      throw new Error('未选择频道，无法创建群聊')
    }

    try {
      console.log('[useGroupChatManager] 创建DIAP群聊:', groupName, agents.length)

      // 获取频道信息
      const channel = {
        id: activeChannelId,
        name: `频道 ${activeChannelId.slice(-8)}`,
      }

      // 使用DIAP创建群聊
      const group = await diapGroupChat.createGroupWithAgents({
        groupName,
        description: `${channel.name} 的群聊`,
        agents: agents,
        channel: channel,
        metadata: {
          channelId: activeChannelId,
          channelName: channel.name,
          createdAt: Date.now()
        }
      })

      // 设置为活跃群聊
      setActiveGroupId(group.groupId)
      setShowGroupChat(true)

      // 打开对话面板
      openConversationPanelRef.current?.()

      return group

    } catch (error) {
      console.error('[useGroupChatManager] 创建群聊失败:', error)
      throw error
    }
  }, [localIdentity, activeChannelId, diapGroupChat, openConversationPanelRef])

  // 发送消息
  const sendMessage = useCallback(async (groupId, content) => {
    try {
      await diapGroupChat.sendMessage(groupId, content)
      console.log('[useGroupChatManager] 消息发送成功:', groupId, content)
    } catch (error) {
      console.error('[useGroupChatManager] 发送消息失败:', error)
      throw error
    }
  }, [diapGroupChat])

  // 切换群聊
  const switchGroupChat = useCallback((groupId) => {
    if (diapGroupChat.groups.some(g => g.groupId === groupId)) {
      setActiveGroupId(groupId)
      console.log('[useGroupChatManager] 切换群聊:', groupId)
    }
  }, [diapGroupChat.groups])

  // 打开群聊
  const openGroupChat = useCallback(async () => {
    console.log('[useGroupChatManager] openGroupChat 被调用，当前 activeGroupId:', activeGroupId)
    
    if (activeGroupId) {
      // 如果已有活跃群聊，直接显示
      setShowGroupChat(true)
      openConversationPanelRef.current?.()
      console.log('[useGroupChatManager] 显示现有群聊:', activeGroupId)
    } else {
      // 如果没有活跃群聊，创建一个新群聊
      try {
        console.log('[useGroupChatManager] 创建新群聊...')
        const groupName = `群聊 ${new Date().toLocaleTimeString()}`
        await createGroupChat(groupName, [])
        console.log('[useGroupChatManager] 新群聊创建成功')
      } catch (error) {
        console.error('[useGroupChatManager] 创建群聊失败:', error)
        // 即使创建失败，也尝试显示面板
        setShowGroupChat(true)
        openConversationPanelRef.current?.()
      }
    }
  }, [activeGroupId, createGroupChat])

  // 关闭群聊
  const closeGroupChat = useCallback(() => {
    setShowGroupChat(false)
  }, [])

  // 完全关闭群聊（清除所有状态）
  const closeGroupChatCompletely = useCallback(() => {
    setShowGroupChat(false)
    setActiveGroupId(null)
    console.log('[useGroupChatManager] 完全关闭群聊')
  }, [])

  // 切换群聊显示状态
  const toggleGroupChat = useCallback(() => {
    if (showGroupChat) {
      closeGroupChatCompletely()
    } else if (activeGroupId) {
      setShowGroupChat(true)
      openConversationPanelRef.current?.()
    }
  }, [showGroupChat, activeGroupId, closeGroupChatCompletely])

  // 获取action状态
  const actionStatus = useMemo(() => {
    if (!activeGroupId) return null
    return getActionStatus(activeGroupId) || 'Active'
  }, [activeGroupId, getActionStatus])

  // 删除群聊
  const deleteGroupChat = useCallback(async (groupId) => {
    try {
      await diapGroupChat.leaveGroup(groupId)
      
      // 如果删除的是当前活跃群聊，清除活跃状态
      if (activeGroupId === groupId) {
        setActiveGroupId(null)
        setShowGroupChat(false)
      }
      
      console.log('[useGroupChatManager] 群聊已删除:', groupId)
    } catch (error) {
      console.error('[useGroupChatManager] 删除群聊失败:', error)
    }
  }, [activeGroupId, diapGroupChat])

  // 分割位置状态
  const [splitPosition, setSplitPosition] = useState(50)

  // 监听DIAP群聊创建事件
  useEffect(() => {
    const handleClusterActionCreated = (event) => {
      const { actionId } = event.detail
      console.log('[useGroupChatManager] 收到群聊创建事件:', actionId)
      
      // 处理DIAP群聊和本地群聊
      if (actionId && (actionId.startsWith('diap_group_') || actionId.startsWith('action_') || actionId.startsWith('local_group_'))) {
        let groupId
        if (actionId.startsWith('diap_group_')) {
          groupId = actionId.replace('diap_group_', '')
        } else if (actionId.startsWith('local_group_')) {
          groupId = actionId.replace('local_group_', '')
        } else {
          // 对于action_开头的，直接使用完整ID作为groupId
          groupId = actionId
        }
        
        console.log('[useGroupChatManager] 设置活跃群聊:', groupId)
        setActiveGroupId(groupId)
        setShowGroupChat(true)
        openConversationPanelRef.current?.()
      }
    }

    window.addEventListener('cluster-action-created', handleClusterActionCreated)
    return () => {
      window.removeEventListener('cluster-action-created', handleClusterActionCreated)
    }
  }, [openConversationPanelRef])

  return {
    // 状态
    showGroupChat,
    activeGroupId,
    activeActionId: activeGroupId, // 兼容性：将activeGroupId作为activeActionId
    activeGroup,
    activeGroupMessages,
    groupChatList,
    isLoading: diapGroupChat.isLoading,
    actionStatus,
    splitPosition,

    // DIAP群聊状态
    diapGroupChat,

    // 操作方法
    createGroupChat,
    sendMessage,
    switchGroupChat,
    openGroupChat,
    closeGroupChat,
    closeGroupChatCompletely,
    toggleGroupChat,
    deleteGroupChat,
    setSplitPosition,
    hasActiveAction: !!activeGroupId,
    canOpenGroupChat: !!activeChannelId,

    // 兼容原有接口
    getActions,
    getActiveActionId: () => activeGroupId,
    setActiveAction,
    getActiveAction,
    getGroupChatMessages,
    getActionStatus,
  }
}

export default useGroupChatManager


