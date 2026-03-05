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

  // 按频道加载群聊存档，并尽量恢复活跃群聊
  const isGroupInActiveChannel = useCallback((group) => {
    if (!activeChannelId) return false
    const metadata = group?.metadata || {}
    const groupChannelId = metadata?.channel?.id || metadata?.channelId || metadata?.channel_id
    return groupChannelId === activeChannelId
  }, [activeChannelId])

  useEffect(() => {
    if (!activeChannelId) {
      loadedChannelRef.current = null
      // eslint-disable-next-line react-hooks/setState-in-effect
      setShowGroupChat(false)
      return
    }

    if (loadedChannelRef.current === activeChannelId) {
      return
    }
    loadedChannelRef.current = activeChannelId

    const { actions, activeActionId: storedActiveActionId } = loadChannelGroupChats(activeChannelId)
    if (!activeGroupId) {
      if (storedActiveActionId) {
        // eslint-disable-next-line react-hooks/setState-in-effect
        setActiveGroupId(storedActiveActionId)
        return
      }
      const firstDiapGroup = diapGroupChat.groups.find(isGroupInActiveChannel)
      if (firstDiapGroup) {
        // eslint-disable-next-line react-hooks/setState-in-effect
        setActiveGroupId(firstDiapGroup.groupId)
        return
      }
      if (actions.length > 0) {
        // eslint-disable-next-line react-hooks/setState-in-effect
        setActiveGroupId(actions[0].action_id)
      }
    }
  }, [activeChannelId, activeGroupId, diapGroupChat.groups, isGroupInActiveChannel, loadChannelGroupChats])

  // 兜底：首次启动时若已有群聊但尚未激活，默认选第一个
  useEffect(() => {
    if (!activeGroupId && diapGroupChat.groups.length > 0) {
      const firstGroup = diapGroupChat.groups[0]
      if (firstGroup) {
        console.log('[useGroupChatManager] 自动恢复活跃群聊:', firstGroup.groupId)
        // eslint-disable-next-line react-hooks/setState-in-effect
        setActiveGroupId(firstGroup.groupId)
      }
    }
  }, [diapGroupChat.groups, activeGroupId])

  // 获取当前活跃的群聊（优先从DIAP群聊查找，然后从本地store查找）
  const activeGroup = useMemo(() => {
    if (!activeGroupId) return null

    console.log('[useGroupChatManager] 获取活跃群聊，activeGroupId:', activeGroupId)

    // 先从DIAP群聊查找
    const diapGroup = diapGroupChat.groups.find(g => g.groupId === activeGroupId)
    if (diapGroup) {
      console.log('[useGroupChatManager] 从DIAP找到群聊:', diapGroup)
      return diapGroup
    }

    // 从本地store查找
    const localActions = getActions(activeChannelId) || []
    const matchedAction = localActions.find((action) => action.action_id === activeGroupId)
    if (matchedAction) {
      console.log('[useGroupChatManager] 从本地store找到群聊:', matchedAction)
      return {
        groupId: matchedAction.action_id,
        groupName: matchedAction.description?.replace('群聊: ', '').split(' + ')[0] || '本地群聊',
        description: matchedAction.description,
        agents: matchedAction.agents || [],
        metadata: {
          type: 'local_group',
          channel: { id: activeChannelId }
        }
      }
    }

    const activeAction = getActiveAction(activeChannelId)
    if (activeAction?.action_id === activeGroupId) {
      console.log('[useGroupChatManager] 从活跃行动找到群聊:', activeAction)
      return {
        groupId: activeAction.action_id,
        groupName: activeAction.description?.replace('群聊: ', '').split(' + ')[0] || '本地群聊',
        description: activeAction.description,
        agents: activeAction.agents || [],
        metadata: {
          type: 'local_group',
          channel: { id: activeChannelId }
        }
      }
    }

    console.log('[useGroupChatManager] 未找到活跃群聊')
    return null
  }, [diapGroupChat.groups, activeGroupId, getActiveAction, getActions, activeChannelId])

  // 获取当前活跃群聊的消息（优先从DIAP群聊查找，然后从本地store查找）
  const activeGroupMessages = useMemo(() => {
    if (!activeGroupId) return []
    
    // 先从DIAP群聊获取消息（仅当当前活跃群聊匹配）
    if (diapGroupChat.activeGroup?.groupId === activeGroupId) {
      return diapGroupChat.messages
    }
    
    // 从本地store获取消息
    const messages = getGroupChatMessages(activeGroupId)
    return messages || []
  }, [diapGroupChat.activeGroup?.groupId, diapGroupChat.messages, activeGroupId, getGroupChatMessages])

  // 获取当前频道的群聊列表（合并DIAP群聊和本地群聊）
  const groupChatList = useMemo(() => {
    if (!activeChannelId) return []
    
    // 获取DIAP群聊
    const diapGroups = diapGroupChat.groups
      .filter(isGroupInActiveChannel)
      .map((group) => ({
        action_id: group.groupId,
        description: group.description || group.groupName || `群聊 #${group.groupId?.slice(-8) || 'N/A'}`,
        agents: group.agents || [],
        metadata: {
          ...(group.metadata || {}),
          type: group.metadata?.type || 'diap_group_chat',
          channel_id: activeChannelId,
        },
      }))
    
    // 获取本地群聊
    const localActions = getActions(activeChannelId) || []
    const localGroups = localActions
      .filter(action => 
        action.action_id?.startsWith('local_group_') || 
        action.action_id?.startsWith('action_') ||
        action.metadata?.type === 'group_chat'
      )
    
    // 合并群聊列表
    return [...diapGroups, ...localGroups]
  }, [activeChannelId, diapGroupChat.groups, getActions, isGroupInActiveChannel])

  // 创建群聊
  const createGroupChat = useCallback(async (groupName, agents = []) => {
    if (!localIdentity) {
      // 创建一个默认身份
      localIdentity = {
        did: `user_${Date.now()}`,
        name: '本地用户'
      }
    }

    // 如果没有频道，创建一个默认频道ID
    let channelId = activeChannelId
    if (!channelId) {
      channelId = `channel_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`
      console.log('[useGroupChatManager] 自动创建默认频道:', channelId)
    }

    try {
      console.log('[useGroupChatManager] 创建群聊:', groupName, '频道:', channelId)

      // 获取频道信息
      const channel = {
        id: channelId,
        name: `频道 ${channelId.slice(-8)}`,
      }

      // 尝试使用DIAP创建群聊
      const group = await diapGroupChat.createGroupWithAgents({
        groupName,
        description: `${channel.name} 的群聊`,
        agents: agents,
        channel: channel,
        metadata: {
          channelId: channelId,
          channelName: channel.name,
          createdAt: Date.now()
        }
      })

      // 设置为活跃群聊
      setActiveGroupId(group.groupId)
      await diapGroupChat.switchToGroup(group.groupId)
      setShowGroupChat(true)

      // 打开对话面板
      openConversationPanelRef.current?.()

      return group

    } catch (error) {
      console.error('[useGroupChatManager] 创建群聊失败，尝试本地模式:', error)
      
      // 降级到本地内存模式
      try {
        const localGroupId = `local_group_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`
        const channel = {
          id: channelId,
          name: `频道 ${channelId.slice(-8)}`,
        }
        
        // 创建本地群聊行动
        const actionId = `local_group_${localGroupId}`
        const action = {
          action_id: actionId,
          description: `${groupName} (${agents.length}个智能体)`,
          status: 'Active',
          created_at: new Date().toISOString(),
          agents: [
            {
              id: localIdentity?.did || 'user',
              did: localIdentity?.did,
              name: localIdentity?.name || '用户',
              mode: 'user'
            },
            ...agents.map(agent => ({
              id: agent.did || agent.id,
              did: agent.did,
              name: agent.name || '智能体',
              mode: 'agent'
            }))
          ],
          metadata: {
            type: 'local_group_chat',
            channel: { id: channelId },
            channelName: channel.name,
            local: true
          }
        }
        
        // 保存到store
        setActiveAction(actionId, channelId)
        
        // 设置为活跃群聊
        setActiveGroupId(actionId)
        setShowGroupChat(true)
        openConversationPanelRef.current?.()
        
        console.log('[useGroupChatManager] 本地群聊创建成功:', actionId)
        return { groupId: actionId, groupName, local: true }
      } catch (localError) {
        console.error('[useGroupChatManager] 本地群聊也创建失败:', localError)
        throw new Error('无法创建群聊，请确保应用已正常初始化')
      }
    }
  }, [localIdentity, activeChannelId, diapGroupChat, openConversationPanelRef, setActiveAction])

  // 发送消息
  const sendMessage = useCallback(async (groupId, content) => {
    try {
      await diapGroupChat.sendMessage(groupId, content)
      console.log('[useGroupChatManager] 消息发送成功:', groupId, content)
      
      // 关键：通知所有智能体处理群聊消息
      const { getActiveAction } = useClusterActionStore.getState()
      const activeAction = activeChannelId ? getActiveAction(activeChannelId) : null
      
      if (activeAction && activeAction.agents && activeAction.agents.length > 0) {
        console.log('[useGroupChatManager] 准备通知智能体:', activeAction.agents.length, '个')
        
        // 获取用户标识
        const walletAddress = typeof window !== 'undefined' ? localStorage.getItem('wallet_address') : null
        const userId = typeof window !== 'undefined' ? localStorage.getItem('user_id') : null
        const from = walletAddress || userId || 'user'
        
        // 广播消息给所有参与群聊的智能体
        const agentIds = activeAction.agents.map(agent => agent.id || agent.agent_id).filter(Boolean)
        
        for (const agentId of agentIds) {
          try {
            console.log('[useGroupChatManager] 通知智能体:', agentId)
            // 触发智能体处理消息的事件
            window.dispatchEvent(new CustomEvent('agent-group-message', {
              detail: {
                agentId,
                message: {
                  id: `msg_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`,
                  content: content,
                  from: from,
                  fromName: localIdentity?.name || from,
                  timestamp: Date.now(),
                  groupId: groupId,
                  type: 'group_chat_message'
                }
              }
            }))
          } catch (error) {
            console.warn(`[useGroupChatManager] 通知智能体 ${agentId} 失败:`, error)
          }
        }
      } else {
        console.warn('[useGroupChatManager] 没有找到活跃的群聊或智能体')
      }
    } catch (error) {
      console.error('[useGroupChatManager] 发送消息失败:', error)
      throw error
    }
  }, [diapGroupChat, activeChannelId, localIdentity])

  // 切换群聊
  const switchGroupChat = useCallback(async (groupId) => {
    if (!groupId) return
    setActiveGroupId(groupId)
    if (activeChannelId) {
      setActiveAction(groupId, activeChannelId)
    }
    if (diapGroupChat.groups.some(g => g.groupId === groupId)) {
      try {
        await diapGroupChat.switchToGroup(groupId)
      } catch (error) {
        console.warn('[useGroupChatManager] 切换DIAP群聊失败，将继续使用本地存档消息:', error)
      }
    }
    setShowGroupChat(true)
    openConversationPanelRef.current?.()
    console.log('[useGroupChatManager] 切换群聊:', groupId)
  }, [activeChannelId, diapGroupChat, setActiveAction])

  // 打开群聊
  const openGroupChat = useCallback(async () => {
    console.log('[useGroupChatManager] openGroupChat 被调用，当前 activeGroupId:', activeGroupId)
    console.log('[useGroupChatManager] 可用群聊数量:', groupChatList.length)

    if (activeGroupId) {
      if (diapGroupChat.groups.some((g) => g.groupId === activeGroupId)) {
        try {
          await diapGroupChat.switchToGroup(activeGroupId)
        } catch (error) {
          console.warn('[useGroupChatManager] 打开群聊时切换活跃DIAP群失败:', error)
        }
      }
      setShowGroupChat(true)
      openConversationPanelRef.current?.()
      console.log('[useGroupChatManager] 显示现有群聊:', activeGroupId)
    } else if (groupChatList.length > 0) {
      const firstGroup = groupChatList[0]
      const firstGroupId = firstGroup.groupId || firstGroup.action_id
      console.log('[useGroupChatManager] 选择第一个群聊作为活跃群聊:', firstGroupId)
      await switchGroupChat(firstGroupId)
    } else {
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
  }, [activeGroupId, createGroupChat, diapGroupChat, groupChatList, switchGroupChat, openConversationPanelRef])

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
    } else if (groupChatList.length > 0) {
      const firstGroup = groupChatList[0]
      const firstGroupId = firstGroup.groupId || firstGroup.action_id
      console.log('[useGroupChatManager] 切换时选择第一个群聊作为活跃群聊:', firstGroupId)
      setActiveGroupId(firstGroupId)
      setShowGroupChat(true)
      openConversationPanelRef.current?.()
    }
  }, [activeGroupId, closeGroupChatCompletely, groupChatList, showGroupChat])

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
        } else {
          groupId = actionId
        }
        
        console.log('[useGroupChatManager] 设置活跃群聊:', groupId)
        setActiveGroupId(groupId)
        if (activeChannelId) {
          setActiveAction(groupId, activeChannelId)
        }
        setShowGroupChat(true)
        openConversationPanelRef.current?.()
        if (diapGroupChat.groups.some((g) => g.groupId === groupId)) {
          diapGroupChat.switchToGroup(groupId).catch((error) => {
            console.warn('[useGroupChatManager] 事件切换DIAP群聊失败:', error)
          })
        }
      }
    }

    window.addEventListener('cluster-action-created', handleClusterActionCreated)
    return () => {
      window.removeEventListener('cluster-action-created', handleClusterActionCreated)
    }
  }, [activeChannelId, diapGroupChat, setActiveAction])

  const activeAction = useMemo(() => {
    if (!activeChannelId || !activeGroupId) return null
    const actions = getActions(activeChannelId) || []
    return actions.find((action) => action.action_id === activeGroupId) || getActiveAction(activeChannelId)
  }, [activeChannelId, activeGroupId, getActions, getActiveAction])

  // 检查是否可以打开群聊 - 基于实际状态验证
  const canOpenGroupChat = useMemo(() => {
    // 检查是否有活跃的群聊
    const hasActiveGroup = activeGroupId && typeof activeGroupId === 'string' && activeGroupId.startsWith('local_group_')
    // 检查 IPFS 是否可用
    const isIpfsReady = diapGroupChat.isInitialized && diapGroupChat.isIpfsAvailable
    // 检查身份是否存在
    const hasIdentity = localIdentity !== null
    
    return hasActiveGroup && isIpfsReady && hasIdentity
  }, [activeGroupId, diapGroupChat.isInitialized, diapGroupChat.isIpfsAvailable, localIdentity])

  return {
    // 状态
    showGroupChat,
    activeGroupId,
    activeActionId: activeGroupId, // 兼容性：将activeGroupId作为activeActionId
    activeAction,
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
    // 基于实际状态验证是否可以打开群聊
    canOpenGroupChat,

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

