import { useCallback, useEffect, useMemo, useState, useRef } from 'react'
import { useDiapGroupChat } from '@/hooks/useDiapGroupChat'
import useClusterActionStore from '@/stores/clusterActionStore'
import { parseMentions } from '@/utils/mentionParser'
import { useGroupChatAutonomousAgent } from './useGroupChatAutonomousAgent'

/**
 * useGroupChatManager - 基于 DIAP PubSub 的群聊管理 Hook
 * 统一管理群聊相关的状态、事件监听和 UI 控制
 * 支持 iroh、pubsub 和 memory 三种群聊模式的自主智能体响应
 */
export const useGroupChatManager = ({ openConversationPanel, activeChannelId, localIdentity }: {
  openConversationPanel: () => void
  activeChannelId: string | null
  localIdentity: any
}) => {
  // 从 store 获取集群行动相关状态
  const {
    getActions,
    setActiveAction,
    getActiveAction,
    getGroupChatMessages,
    getActionStatus,
    loadChannelGroupChats,
    addAction,
  } = useClusterActionStore()

  // 使用 ref 跟踪已加载的频道，避免重复加载
  const loadedChannelRef = useRef(null)
  const openConversationPanelRef = useRef(openConversationPanel)

  // 更新 ref
  useEffect(() => {
    openConversationPanelRef.current = openConversationPanel
  }, [openConversationPanel])

  // DIAP 群聊 Hook
  const diapGroupChat = useDiapGroupChat({
    localIdentity,
    onGroupCreated: (group: any, agents: any[]) => {
      console.log('[useGroupChatManager] DIAP 群聊创建成功:', group.groupName)

      // 创建对应的集群行动（用于 UI 显示）
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
          local: false // 标记为 DIAP 群聊，不是本地群聊
        }
      }

      // 添加到 store
      setActiveAction(actionId, activeChannelId)

      // 触发群聊显示事件
      window.dispatchEvent(
        new CustomEvent('cluster-action-created', {
          detail: { actionId },
        }),
      )
    },
    onMessage: (groupId: string, message: any) => {
      console.log('[useGroupChatManager] 收到 DIAP 消息:', groupId, message.content)
      
      // 关键改进：触发智能体自主响应
      // 当收到新消息时，通知所有群聊中的智能体
      try {
        const { getActions } = useClusterActionStore.getState()
        const actions = getActions("") || []
        
        // 查找群聊对应的行动
        const groupAction = actions.find(action => 
          action.action_id === groupId || 
          action.action_id === `diap_group_${groupId}`
        )
        
        if (groupAction && groupAction.agents && groupAction.agents.length > 0) {
          // 通知每个智能体处理消息
          groupAction.agents
            .filter(agent => agent.mode === 'agent')
            .forEach(agent => {
              const agentId = agent.id || agent.agent_id || agent.did
              if (agentId) {
                window.dispatchEvent(new CustomEvent('agent-group-message', {
                  detail: {
                    agentId,
                    message: {
                      id: message.id || `msg_${Date.now()}`,
                      groupId,
                      from: message.from || 'unknown',
                      fromName: message.fromName || '群聊成员',
                      content: message.content,
                      timestamp: message.timestamp || Date.now(),
                      type: 'group_chat_message',
                      isMentioned: message.content?.includes('@'),
                      metadata: {
                        isGroupChat: true,
                        groupType: 'diap_pubsub'
                      }
                    }
                  }
                }))
              }
            })
          
          console.log('[useGroupChatManager] 已通知', groupAction.agents.length, '个智能体处理消息')
        }
      } catch (error) {
        console.error('[useGroupChatManager] 触发智能体响应失败:', error)
      }
    },
    onError: (error: Error) => {
      console.error('[useGroupChatManager] DIAP 群聊错误:', error)
    }
  })

  // 自主智能体 Hook - 支持群聊中的智能体自主响应（无并发限制）
  const autonomousAgent = useGroupChatAutonomousAgent({
    enabled: true, // 启用自主响应
    mentionOnly: false, // 不只在被@时响应（允许自主判断）
    // ✅ 移除 maxConcurrentTasks - 允许所有 agent 同时响应
  })

  // 更新 ref
  useEffect(() => {
    openConversationPanelRef.current = openConversationPanel
  }, [openConversationPanel])

  // 状态管理
  const [showGroupChat, setShowGroupChat] = useState(false)
  const [activeGroupId, setActiveGroupId] = useState<string | null>(null)

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
        setActiveGroupId(storedActiveActionId as string)
        return
      }
      const firstDiapGroup = diapGroupChat.groups.find(isGroupInActiveChannel)
      if (firstDiapGroup) {
        // eslint-disable-next-line react-hooks/setState-in-effect
        setActiveGroupId(firstDiapGroup.groupId as string)
        return
      }
      if (actions.length > 0) {
        // eslint-disable-next-line react-hooks/setState-in-effect
        setActiveGroupId(actions[0].action_id as string)
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
        setActiveGroupId(firstGroup.groupId as string)
      }
    }
  }, [diapGroupChat.groups, activeGroupId])

  // 获取当前活跃的群聊（优先从 DIAP 群聊查找，然后从本地 store 查找）
  const activeGroup = useMemo(() => {
    if (!activeGroupId) return null

    console.log('[useGroupChatManager] 获取活跃群聊，activeGroupId:', activeGroupId)

    // 先从 DIAP 群聊查找
    const diapGroup = diapGroupChat.groups.find(g => g.groupId === activeGroupId)
    if (diapGroup) {
      console.log('[useGroupChatManager] 从 DIAP 找到群聊:', diapGroup)
      return diapGroup
    }

    // 从本地 store 查找
    const localActions = getActions(activeChannelId) || []
    const matchedAction = localActions.find((action) => action.action_id === activeGroupId)
    if (matchedAction) {
      console.log('[useGroupChatManager] 从本地 store 找到群聊:', matchedAction)
      return {
        groupId: matchedAction.action_id,
        groupName: matchedAction.description?.replace('群聊：', '').split(' + ')[0] || '本地群聊',
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
        groupName: activeAction.description?.replace('群聊：', '').split(' + ')[0] || '本地群聊',
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

  // 获取当前活跃群聊的消息（优先从 DIAP 群聊查找，然后从本地 store 查找）
  const [lastMessageCount, setLastMessageCount] = useState(0)
  
  const activeGroupMessages = useMemo(() => {
    if (!activeGroupId) {
      console.log('[useGroupChatManager] activeGroupMessages: activeGroupId 为空')
      return []
    }

    // 先从 DIAP 群聊获取消息（仅当当前活跃群聊匹配）
    if (diapGroupChat.activeGroup?.groupId === activeGroupId) {
      return diapGroupChat.messages
    }

    // 从本地 store 获取消息
    const messages = getGroupChatMessages(activeGroupId)
    return messages || []
  }, [diapGroupChat.activeGroup?.groupId, diapGroupChat.messages, activeGroupId, lastMessageCount])
  
  // 监听 store 中的消息变化，只在消息数量变化时强制更新
  useEffect(() => {
    if (!activeGroupId) return
    
    const messages = getGroupChatMessages(activeGroupId)
    const count = messages?.length || 0
    
    if (count !== lastMessageCount) {
      console.log('[useGroupChatManager] 检测到消息数量变化:', lastMessageCount, '->', count)
      setLastMessageCount(count)
    }
  }, [activeGroupId, lastMessageCount])

  // 获取当前频道的群聊列表（合并 DIAP 群聊和本地群聊）
  const groupChatList = useMemo(() => {
    if (!activeChannelId) return []

    // 获取 DIAP 群聊
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

  // 创建群聊 - 默认添加当前 channel 的所有智能体
  const createGroupChat = useCallback(async (groupName: string, agents: any[] = []) => {
    if (!localIdentity) {
      // 创建一个默认身份
      localIdentity = {
        did: `user_${Date.now()}`,
        name: '本地用户'
      }
    }

    // 如果没有频道，创建一个默认频道 ID
    let channelId = activeChannelId
    if (!channelId) {
      channelId = `channel_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`
      console.log('[useGroupChatManager] 自动创建默认频道:', channelId)
    }

    // 如果没有传入 agents，从当前 channel 获取所有智能体
    let channelAgents = agents
    if (!channelAgents || channelAgents.length === 0) {
      // 从 store 获取当前频道的智能体
      const { getActions } = useClusterActionStore.getState()
      const actions = getActions(channelId) || []
      channelAgents = actions
        .filter(action => action.agents && action.agents.length > 0)
        .flatMap(action => action.agents)
        .filter(agent => agent.mode === 'agent')
      
      console.log('[useGroupChatManager] 从 channel 获取智能体:', channelAgents.length, '个')
    }

    try {
      console.log('[useGroupChatManager] 创建群聊:', groupName, '频道:', channelId, '智能体:', channelAgents.length)

      // 获取频道信息
      const channel = {
        id: channelId,
        name: `频道 ${channelId.slice(-8)}`,
      }

      // 尝试使用 DIAP 创建群聊
      const group = await diapGroupChat.createGroupWithAgents({
        groupName,
        description: `${channel.name} 的群聊`,
        agents: channelAgents,
        channel: channel,
        metadata: {
          channelId: channelId,
          channelName: channel.name,
          createdAt: Date.now()
        }
      })

      // 设置为活跃群聊
      setActiveGroupId(group.groupId as string)
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
          description: `${groupName} (${channelAgents.length}个智能体)`,
          status: 'Active',
          created_at: new Date().toISOString(),
          agents: [
            {
              id: localIdentity?.did || 'user',
              did: localIdentity?.did,
              name: localIdentity?.name || '用户',
              mode: 'user'
            },
            ...channelAgents.map(agent => ({
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

        // 保存到 store
        addAction(action)
        setActiveAction(actionId, channelId)

        // 设置为活跃群聊
        setActiveGroupId(actionId as string)
        setShowGroupChat(true)
        openConversationPanelRef.current?.()

        console.log('[useGroupChatManager] 本地群聊创建成功:', actionId)
        return { groupId: actionId, groupName, local: true }
      } catch (localError) {
        console.error('[useGroupChatManager] 本地群聊也创建失败:', localError)
        throw new Error('无法创建群聊，请确保应用已正常初始化')
      }
    }
  }, [localIdentity, activeChannelId, diapGroupChat, openConversationPanelRef, setActiveAction, addAction, getActions])

  // 发送消息 - 健壮的群聊消息发送逻辑
  const sendMessage = useCallback(async (groupId: string, content: string, options?: {
    mentions?: Array<{ agentId: string; agentName: string }>,
    hasMentions?: boolean,
    isPrivateMention?: boolean,
    targetAgentId?: string
  }): Promise<void> => {
    if (!groupId) {
      console.error('[useGroupChatManager] 群聊 ID 缺失')
      throw new Error('群聊 ID 缺失')
    }

    if (!content || content.trim() === '') {
      console.warn('[useGroupChatManager] 消息内容为空')
      return
    }

    try {
      console.log('[useGroupChatManager] sendMessage 被调用:', { 
        groupId, 
        content: content.slice(0, 50), 
        activeChannelId,
        options 
      })

      // 判断群聊类型
      const isLocalGroupChat = groupId.startsWith('local_group_') || groupId.startsWith('action_')
      const isDiapGroupChat = groupId.startsWith('diap_group_') || !isLocalGroupChat

      // 获取用户标识
      const walletAddress = typeof window !== 'undefined' ? localStorage.getItem('wallet_address') : null
      const userId = typeof window !== 'undefined' ? localStorage.getItem('user_id') : null
      const from = walletAddress || userId || 'user'

      // 创建用户消息对象
      const userMessage = {
        id: `msg_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`,
        type: 'user' as const,
        from: from,
        fromName: localIdentity?.name || from,
        content: content,
        timestamp: Date.now(),
        mentions: options?.mentions || [],
        hasMentions: options?.hasMentions || false,
      }

      let sendSuccess = false
      let sendError: Error | null = null

      // 根据群聊类型选择发送方式
      if (isLocalGroupChat) {
        // === 本地群聊 ===
        console.log('[useGroupChatManager] 检测到本地群聊，保存到本地 store')

        try {
          // 1. 保存到本地 store
          const { addGroupChatMessage } = useClusterActionStore.getState()
          addGroupChatMessage(groupId, userMessage)
          console.log('[useGroupChatManager] 本地群聊消息已添加到 store:', groupId, userMessage.id)
          sendSuccess = true
        } catch (error) {
          console.error('[useGroupChatManager] 保存到本地 store 失败:', error)
          sendError = error as Error
        }
      }

      if (isDiapGroupChat && !sendSuccess) {
        // === DIAP 群聊 ===
        console.log('[useGroupChatManager] 使用 DIAP 群聊发送')

        try {
          await diapGroupChat.sendMessage(groupId, content)
          console.log('[useGroupChatManager] DIAP 群聊发送成功')
          sendSuccess = true
        } catch (error) {
          console.error('[useGroupChatManager] DIAP 群聊发送失败:', error)
          sendError = error as Error
          // 降级：即使 DIAP 失败，也保存到本地 store
          try {
            const { addGroupChatMessage } = useClusterActionStore.getState()
            addGroupChatMessage(groupId, userMessage)
            console.log('[useGroupChatManager] 降级：消息已保存到本地 store')
            sendSuccess = true
            sendError = null
          } catch (fallbackError) {
            console.error('[useGroupChatManager] 降级保存也失败:', fallbackError)
          }
        }
      }

      // 无论哪种群聊类型，都通知智能体
      const { getActiveAction, getActions } = useClusterActionStore.getState()
      const activeAction = activeChannelId ? getActiveAction(activeChannelId) : null
      
      // 获取群聊中的所有智能体（包括非活跃的）
      const actions = getActions("") || []
      const groupAction = actions.find(action => 
        action.action_id === groupId || 
        action.action_id === `diap_group_${groupId}` ||
        action.action_id?.startsWith('local_group_')
      )
      const allAgents = groupAction?.agents || activeAction?.agents || []

      if (allAgents && allAgents.length > 0) {
        console.log('[useGroupChatManager] 准备通知智能体:', allAgents.length, '个')

        // 解析@提及
        const { mentionedAgents, cleanContent, hasMention } = parseMentions(content, allAgents)

        // 确定要通知的智能体列表
        let targetAgents = allAgents

        if (hasMention && mentionedAgents.length > 0) {
          // 如果有@提及，只通知被@的智能体
          targetAgents = mentionedAgents
          console.log('[useGroupChatManager] @提及检测：只通知被@的智能体:', mentionedAgents.map(a => a.name || a.id))
        } else if (hasMention && mentionedAgents.length === 0) {
          // 有@但没有匹配到智能体，跳过通知
          console.log('[useGroupChatManager] @提及但未匹配到智能体，不通知')
          // 仍然返回成功，因为消息已经发送，只是没有智能体可通知
          return
        } else {
          // 没有@，广播给所有智能体（启用自主响应）
          console.log('[useGroupChatManager] 无@提及，广播给所有智能体（启用自主响应）')
        }

        // 通知目标智能体
        const agentIds = targetAgents
          .filter(agent => agent.mode === 'agent') // 只通知智能体，不通知用户
          .map(agent => agent.id || agent.agent_id || agent.did)
          .filter(Boolean)

        if (agentIds.length === 0) {
          console.warn('[useGroupChatManager] 没有可通知的智能体 ID')
          return
        }

        // 使用自主智能体系统触发响应（如果可用）
        if (autonomousAgent && autonomousAgent.isReady) {
          console.log('[useGroupChatManager] 使用自主智能体系统触发响应')
          
          for (const agentId of agentIds) {
            try {
              // 使用自主智能体系统触发响应
              await autonomousAgent.triggerAgentResponse(groupId, {
                id: userMessage.id,
                groupId,
                from: from,
                fromName: localIdentity?.name || from,
                content: hasMention ? cleanContent : content,
                timestamp: Date.now(),
                type: 'user',
                metadata: {
                  isMentioned: hasMention,
                  mentionedAgentIds: mentionedAgents.map(a => a.id || a.agent_id),
                  rawContent: content
                }
              }, agentId)
              console.log('[useGroupChatManager] 自主智能体响应已触发:', agentId)
            } catch (error) {
              console.warn(`[useGroupChatManager] 触发自主智能体 ${agentId} 失败:`, error)
            }
          }
        } else {
          // 降级：直接发送事件通知
          console.log('[useGroupChatManager] 自主智能体系统未就绪，使用事件通知')
          
          for (const agentId of agentIds) {
            try {
              console.log('[useGroupChatManager] 通知智能体:', agentId)
              // 触发智能体处理消息的事件
              window.dispatchEvent(new CustomEvent('agent-group-message', {
                detail: {
                  agentId,
                  message: {
                    id: userMessage.id,
                    content: hasMention ? cleanContent : content,
                    rawContent: content,
                    from: from,
                    fromName: localIdentity?.name || from,
                    timestamp: Date.now(),
                    groupId: groupId,
                    type: 'group_chat_message' as const,
                    isMentioned: hasMention,
                    mentionedAgentIds: mentionedAgents.map(a => a.id)
                  }
                }
              }))
              console.log('[useGroupChatManager] 智能体通知成功:', agentId)
            } catch (error) {
              console.warn(`[useGroupChatManager] 通知智能体 ${agentId} 失败:`, error)
              // 单个智能体通知失败不影响整体
            }
          }
        }
      } else {
        console.warn('[useGroupChatManager] 没有找到活跃的群聊或智能体，但消息已发送')
      }

      if (!sendSuccess && sendError) {
        throw sendError
      }

      console.log('[useGroupChatManager] 消息发送成功:', groupId, content)
    } catch (error) {
      console.error('[useGroupChatManager] 发送消息失败:', error)
      throw error
    }
  }, [diapGroupChat, activeChannelId, localIdentity, autonomousAgent])

  // 切换群聊
  const switchGroupChat = useCallback(async (groupId: string) => {
    if (!groupId) return
    setActiveGroupId(groupId)
    if (activeChannelId) {
      setActiveAction(groupId, activeChannelId)
    }
    if (diapGroupChat.groups.some(g => g.groupId === groupId)) {
      try {
        await diapGroupChat.switchToGroup(groupId)
      } catch (error) {
        console.warn('[useGroupChatManager] 切换 DIAP 群聊失败，将继续使用本地存档消息:', error)
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
          console.warn('[useGroupChatManager] 打开群聊时切换活跃 DIAP 群失败:', error)
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

  // 获取 action 状态
  const actionStatus = useMemo(() => {
    if (!activeGroupId) return null
    return getActionStatus(activeGroupId) || 'Active'
  }, [activeGroupId, getActionStatus])

  // 删除群聊
  const deleteGroupChat = useCallback(async (groupId: string) => {
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

  // 监听 DIAP 群聊创建事件
  useEffect(() => {
    const handleClusterActionCreated = (event) => {
      const { actionId } = event.detail
      console.log('[useGroupChatManager] 收到群聊创建事件:', actionId)

      // 处理 DIAP 群聊和本地群聊
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
            console.warn('[useGroupChatManager] 事件切换 DIAP 群聊失败:', error)
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
    activeActionId: activeGroupId, // 兼容性：将 activeGroupId 作为 activeActionId
    activeAction,
    activeGroup,
    activeGroupMessages,
    groupChatList,
    isLoading: diapGroupChat.isLoading,
    actionStatus,
    splitPosition,

    // DIAP 群聊状态
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
