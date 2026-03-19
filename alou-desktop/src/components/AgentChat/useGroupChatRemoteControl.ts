import { useCallback, useEffect, useState } from 'react'
import pubsubService from '@/services/pubsubService'
import localIpfsGroupChatService from '@/services/localIpfsGroupChatService'
import useClusterActionStore from '@/stores/clusterActionStore'

/**
 * useGroupChatRemoteControl - 群聊遥控功能 Hook
 * 管理群聊模式下的输入目标模式和消息发送逻辑
 */
export const useGroupChatRemoteControl = ({
  showGroupChat,
  activeActionId,
  activeChannelId,
  currentMessage,
  setCurrentMessage,
  sendMessageToAgent,
  selectedAgent,
  isAgentLoading,
  isSessionReady,
  createSession,
  setSessionReady,
  userName,
  openConversationPanel,
}: {
  showGroupChat: boolean
  activeActionId: string | null
  activeChannelId: string | null
  currentMessage: string
  setCurrentMessage: (msg: string) => void
  sendMessageToAgent: (channelId: string | null, text: string, agent: any) => Promise<void>
  selectedAgent: any
  isAgentLoading: (channelId: string | null) => boolean
  isSessionReady: boolean
  createSession: () => Promise<void>
  setSessionReady: (ready: boolean) => void
  userName: string
  openConversationPanel: () => void
}) => {
  // 输入目标模式：'agent' = 发送到智能体（遥控模式），'groupChat' = 发送到群聊
  // 只有在群聊面板打开时才启用遥控功能
  const [inputTargetMode, setInputTargetMode] = useState('agent')

  // 监听群聊面板状态，关闭时重置输入目标模式
  useEffect(() => {
    if (!showGroupChat) {
      console.log('[useGroupChatRemoteControl] 群聊面板关闭，设置输入目标为 agent 模式')
      setInputTargetMode('agent')
    } else {
      // 群聊面板打开时，强制设置为 'groupChat' 模式
      // 只在模式不等于 groupChat 时才更新，避免不必要的状态变化
      setInputTargetMode(prevMode => {
        if (prevMode !== 'groupChat') {
          console.log('[useGroupChatRemoteControl] 群聊面板打开，强制设置输入目标为 groupChat 模式', {
            showGroupChat,
            activeActionId
          })
          
          // 使用 setTimeout 确保状态更新后再触发事件
          setTimeout(() => {
            window.dispatchEvent(new CustomEvent('switch-input-target', {
              detail: { target: 'groupChat' }
            }))
            console.log('[useGroupChatRemoteControl] 触发切换事件到 groupChat')
          }, 0)
          
          return 'groupChat'
        }
        return prevMode
      })
    }
  }, [showGroupChat]) // 移除 activeActionId 依赖，避免输入时反复触发

  // 监听输入目标切换事件 - 只添加一次，避免内存泄漏
  useEffect(() => {
    const handleSwitchInputTarget = (event: Event) => {
      const customEvent = event as CustomEvent<{ target: string }>
      const { target } = customEvent.detail
      console.log('[useGroupChatRemoteControl] 收到切换事件:', {
        target,
        currentMode: inputTargetMode,
        showGroupChat
      })

      if (target === 'groupChat' || target === 'agent') {
        setInputTargetMode(target)
        console.log('[useGroupChatRemoteControl] 输入目标切换到:', target)
      } else {
        console.warn('[useGroupChatRemoteControl] 未知的切换目标:', target)
      }
    }

    window.addEventListener('switch-input-target', handleSwitchInputTarget as EventListener)
    console.log('[useGroupChatRemoteControl] 事件监听器已设置')

    return () => {
      console.log('[useGroupChatRemoteControl] 清理事件监听器')
      window.removeEventListener('switch-input-target', handleSwitchInputTarget as EventListener)
    }
  }, []) // 空依赖数组，只执行一次

  // 发送消息到群聊
  const sendMessageToGroupChat = useCallback(
    async (text: string, actionId: string) => {
      if (!actionId) {
        console.warn('[useGroupChatRemoteControl] 无法发送到群聊：缺少 actionId')
        return
      }

      console.log('[useGroupChatRemoteControl] sendMessageToGroupChat 被调用:', { actionId, text: text.slice(0, 30) })

      try {
        const topic = `diap/cluster_action/${actionId}`
        const isLocalGroupChat = actionId.startsWith('local_group_')

        console.log('[useGroupChatRemoteControl] 群聊类型判断:', { actionId, isLocalGroupChat, startsWith: actionId.startsWith('local_group_') })

        // 获取用户标识（优先使用 wallet_address，否则使用 user_id）
        const walletAddress =
          typeof window !== 'undefined' ? localStorage.getItem('wallet_address') : null
        const userId = typeof window !== 'undefined' ? localStorage.getItem('user_id') : null
        const from = walletAddress || userId || 'user'

        // 创建用户消息
        const userMessage = {
          id: `msg_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`,
          type: 'user', // 确保用户消息类型正确
          from: from,
          fromName: userName || from,
          content: text,
          timestamp: Date.now(),
        }

        if (isLocalGroupChat) {
          // 本地群聊：优先使用新的本地 IPFS 群聊服务
          try {
            // 检查是否有对应的本地群聊
            const localGroup = localIpfsGroupChatService.getGroup(actionId)
            if (localGroup) {
              // 使用本地 IPFS 群聊服务发送消息
              await localIpfsGroupChatService.sendMessage(text, actionId, localGroup.topic)
              console.log('[useGroupChatRemoteControl] 使用本地 IPFS 群聊服务发送消息成功')
            } else {
              // 降级到原有的 store 方式
              const { addGroupChatMessage, getActiveAction } = useClusterActionStore.getState()
              addGroupChatMessage(actionId, userMessage)
              console.log('[useGroupChatRemoteControl] 本地群聊消息已添加到 store:', actionId)
            }
          } catch (localError) {
            console.warn('[useGroupChatRemoteControl] 本地 IPFS 群聊服务发送失败，降级到 store:', localError)
            // 降级到原有的 store 方式
            const { addGroupChatMessage } = useClusterActionStore.getState()
            addGroupChatMessage(actionId, userMessage)
          }

          // 关键改进：通知所有智能体处理群聊消息
          const { getActiveAction } = useClusterActionStore.getState()
          // 使用 activeChannelId 获取正确的活跃行动
          const activeAction = activeChannelId ? getActiveAction(activeChannelId) : null
          
          if (activeAction && activeAction.agents && activeAction.agents.length > 0) {
            // 广播消息给所有参与群聊的智能体
            const agentIds = activeAction.agents.map(agent => agent.id || agent.agent_id).filter(Boolean)
            
            // 使用多智能体协调器广播消息
            try {
              const { broadcastToAgents } = await import('@/hooks/useMultiAgentChat')
              if (broadcastToAgents) {
                // 使用第一个智能体作为发送方，或者使用系统标识
                const systemAgentId = 'system-group-chat'
                await broadcastToAgents(
                  systemAgentId,  // fromAgentId: 发送方 ID
                  {
                    content: text,
                    from: from,
                    fromName: userName || from,
                    timestamp: Date.now(),
                    groupId: actionId,
                    type: 'group_chat_message'
                  },
                  from  // excludeAgentId: 排除发送者（用户）
                )
                console.log('[useGroupChatRemoteControl] 已广播消息给所有智能体:', agentIds)
              }
            } catch (error) {
              console.warn('[useGroupChatRemoteControl] 广播消息失败:', error)
            }

            // 备用方案：直接调用每个智能体的消息处理
            for (const agentId of agentIds) {
              try {
                // 触发智能体处理消息的事件
                window.dispatchEvent(new CustomEvent('agent-group-message', {
                  detail: {
                    agentId,
                    message: {
                      ...userMessage,
                      groupId: actionId,
                      type: 'group_chat_message'
                    }
                  }
                }))
              } catch (error) {
                console.warn(`[useGroupChatRemoteControl] 通知智能体 ${agentId} 失败:`, error)
              }
            }
          }
        } else {
          // 远程群聊：尝试使用 pubsubService，如果失败则使用后端 API，最后降级到本地 store
          try {
            const success = await pubsubService.sendGroupMessage(topic, text, {
              type: 'user_message',
              from_name: userName,
              timestamp: Date.now(),
            })
            if (!success) {
              throw new Error('pubsubService 发送失败')
            }
            console.log('[useGroupChatRemoteControl] 消息已通过 PubSub 发送到群聊:', topic)
          } catch (pubsubError) {
            // 降级：使用后端 API
            console.warn('[useGroupChatRemoteControl] PubSub 发送失败，尝试使用后端 API:', pubsubError)
            try {
              const apiClient = (await import('@/services/api')).default
              await apiClient.post('/pubsub/publish', {
                topic,
                message: {
                  id: userMessage.id,
                  msg_type: 'chat',
                  from: from,
                  content: text,
                  topic,
                  timestamp: Date.now(),
                  metadata: {
                    type: 'user_message',
                    from_name: userName,
                  },
                },
              })
              console.log('[useGroupChatRemoteControl] 消息已通过后端 API 发送到群聊:', topic)
            } catch (apiError) {
              // 最终降级：保存到本地 store，即使远程服务都不可用
              console.warn('[useGroupChatRemoteControl] 后端 API 也失败，降级到本地 store:', apiError)
              const { addGroupChatMessage } = useClusterActionStore.getState()
              addGroupChatMessage(actionId, userMessage)
              console.log('[useGroupChatRemoteControl] 消息已保存到本地 store:', actionId)
            }
          }
        }
      } catch (error) {
        console.error('[useGroupChatRemoteControl] 发送消息到群聊失败:', error)
        // 可以考虑显示错误提示给用户
      }
    },
    [userName, activeChannelId],
  )

  // 处理群聊面板点击，切换输入目标模式
  const handleGroupChatPanelClick = useCallback(() => {
    console.log('[useGroupChatRemoteControl] 群聊面板点击，当前状态:', {
      showGroupChat,
      currentMode: inputTargetMode
    })
    
    if (showGroupChat) {
      console.log('[useGroupChatRemoteControl] 强制设置输入目标为群聊模式')
      setInputTargetMode('groupChat')
      
      // 触发切换事件，确保 UI 同步更新
      window.dispatchEvent(new CustomEvent('switch-input-target', {
        detail: { target: 'groupChat' }
      }))
    } else {
      console.log('[useGroupChatRemoteControl] 群聊面板未显示，忽略点击')
    }
  }, [showGroupChat, inputTargetMode, setInputTargetMode])

  // 零状态处理：检测是否需要创建智能体
  const handleZeroStateMessage = useCallback(async (text: string) => {
    console.log('[useGroupChatRemoteControl] 零状态处理，检测意图:', text)
    
    // 检测是否是创建智能体的意图
    const createKeywords = ['创建智能体', '新建智能体', 'create agent', 'new agent', '创建一个', '建一个', '帮我创建一个']
    const isCreateIntent = createKeywords.some(keyword => text.toLowerCase().includes(keyword.toLowerCase()))
    
    if (isCreateIntent) {
      console.log('[useGroupChatRemoteControl] 检测到创建智能体意图')
      // 触发创建智能体的事件，让上层组件处理
      window.dispatchEvent(new CustomEvent('create-agent-from-message', {
        detail: { message: text }
      }))
      return true
    }
    
    // 不是创建智能体，显示引导提示
    console.log('[useGroupChatRemoteControl] 显示引导提示')
    window.dispatchEvent(new CustomEvent('show-zero-state-hint', {
      detail: { message: text }
    }))
    return false
  }, [])

  // 发送消息（整合了遥控功能的逻辑）
  const sendMessage = useCallback(async (text?: string) => {
    // 如果传入了 text 参数，使用它；否则从 currentMessage 获取
    const messageText = text !== undefined ? text : currentMessage
    const trimmed = messageText?.trim()
    if (!trimmed) {
      return
    }

    // 添加调试日志
    console.log('[useGroupChatRemoteControl] sendMessage 被调用:', {
      showGroupChat,
      activeActionId,
      activeChannelId,
      selectedAgent: selectedAgent?.id,
      inputTargetMode,
      text: trimmed.slice(0, 30)
    })

    // 零状态处理：没有活跃频道或智能体时
    if (!activeChannelId || !selectedAgent) {
      console.log('[useGroupChatRemoteControl] 零状态处理:', { activeChannelId, selectedAgent, showGroupChat, activeActionId })

      // 关键修复：如果群聊面板打开且有活跃群聊，直接发送到群聊
      // 不再依赖 inputTargetMode，确保群聊消息能正确发送
      if (showGroupChat && activeActionId) {
        try {
          await sendMessageToGroupChat(trimmed, activeActionId)
          console.log('[useGroupChatRemoteControl] 零状态下消息已发送到群聊')
        } catch (sendError) {
          console.error('[useGroupChatRemoteControl] 零状态下发送消息失败:', sendError)
        }
        return
      }

      // 尝试处理零状态消息
      const handled = await handleZeroStateMessage(trimmed)
      if (handled) {
        return
      }
      // 如果没有成功处理，仍然允许发送（可能会有其他逻辑处理）
    }

    // 以下是原有逻辑...
    // 检查当前智能体是否正在执行（仅在发送到智能体时检查）
    const shouldCheckLoading = !showGroupChat || inputTargetMode === 'agent'
    if (shouldCheckLoading && isAgentLoading(activeChannelId)) {
      console.log('[useGroupChatRemoteControl] 智能体正在执行中，跳过:', activeChannelId)
      return
    }

    // 如果群聊面板未打开，保持原有逻辑：直接发送到智能体主对话
    if (!showGroupChat) {
      if (!isSessionReady) {
        await createSession()
        setSessionReady(true)
      }

      // 发送消息后不自动打开对话面板，只显示查看按钮
      await sendMessageToAgent(activeChannelId, trimmed, selectedAgent)
      return
    }

    // 群聊面板打开时，启用遥控功能
    if (inputTargetMode === 'groupChat') {
      // 发送到群聊
      await sendMessageToGroupChat(trimmed, activeActionId)
    } else {
      // 发送到智能体主对话（遥控模式）
      if (!isSessionReady) {
        await createSession()
        setSessionReady(true)
      }

      // 发送消息后不自动打开对话面板，只显示查看按钮
      await sendMessageToAgent(activeChannelId, trimmed, selectedAgent)
    }
  }, [
    handleZeroStateMessage,
    activeChannelId,
    activeActionId,
    createSession,
    currentMessage,
    inputTargetMode,
    isAgentLoading,
    isSessionReady,
    openConversationPanel,
    selectedAgent,
    sendMessageToAgent,
    sendMessageToGroupChat,
    setSessionReady,
    showGroupChat,
  ])

  return {
    inputTargetMode,
    sendMessage,
    handleGroupChatPanelClick,
  }
}
