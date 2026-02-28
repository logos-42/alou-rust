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
}) => {
  // 输入目标模式：'agent' = 发送到智能体（遥控模式），'groupChat' = 发送到群聊
  // 只有在群聊面板打开时才启用遥控功能
  const [inputTargetMode, setInputTargetMode] = useState('agent')

  // 监听群聊面板状态，关闭时重置输入目标模式
  useEffect(() => {
    if (!showGroupChat) {
      console.log('[useGroupChatRemoteControl] 群聊面板关闭，设置输入目标为agent模式')
      setInputTargetMode('agent')
    } else {
      // 群聊面板打开时，默认设置为 'groupChat' 模式
      console.log('[useGroupChatRemoteControl] 群聊面板打开，设置输入目标为groupChat模式')
      setInputTargetMode('groupChat')
    }
  }, [showGroupChat])

  // 监听输入目标切换事件 - 只添加一次，避免内存泄漏
  useEffect(() => {
    const handleSwitchInputTarget = (event: CustomEvent<{ target: string }>) => {
      const { target } = event.detail
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

    window.addEventListener('switch-input-target', handleSwitchInputTarget)
    console.log('[useGroupChatRemoteControl] 事件监听器已设置')

    return () => {
      console.log('[useGroupChatRemoteControl] 清理事件监听器')
      window.removeEventListener('switch-input-target', handleSwitchInputTarget)
    }
  }, []) // 空依赖数组，只执行一次

  // 发送消息到群聊
  const sendMessageToGroupChat = useCallback(
    async (text, actionId) => {
      if (!actionId) {
        console.warn('[useGroupChatRemoteControl] 无法发送到群聊：缺少 actionId')
        return
      }

      try {
        const topic = `diap/cluster_action/${actionId}`
        const isLocalGroupChat = actionId.startsWith('local_group_')

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
          // 本地群聊：优先使用新的本地IPFS群聊服务
          try {
            // 检查是否有对应的本地群聊
            const localGroup = localIpfsGroupChatService.getGroup(actionId)
            if (localGroup) {
              // 使用本地IPFS群聊服务发送消息
              await localIpfsGroupChatService.sendMessage(text, actionId, localGroup.topic)
              console.log('[useGroupChatRemoteControl] 使用本地IPFS群聊服务发送消息成功')
            } else {
              // 降级到原有的store方式
              const { addGroupChatMessage, getActiveAction } = useClusterActionStore.getState()
              addGroupChatMessage(actionId, userMessage)
              console.log('[useGroupChatRemoteControl] 本地群聊消息已添加到store:', actionId)
            }
          } catch (localError) {
            console.warn('[useGroupChatRemoteControl] 本地IPFS群聊服务发送失败，降级到store:', localError)
            // 降级到原有的store方式
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
                await broadcastToAgents({
                  content: text,
                  from: from,
                  fromName: userName || from,
                  timestamp: Date.now(),
                  groupId: actionId,
                  type: 'group_chat_message'
                }, from) // 排除发送者（用户）
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
          // 远程群聊：尝试使用 pubsubService，如果失败则使用后端 API
          try {
            const success = await pubsubService.sendGroupMessage(topic, text, {
              type: 'user_message',
              from_name: userName,
              timestamp: Date.now(),
            })
            if (!success) {
              throw new Error('pubsubService发送失败')
            }
            console.log('[useGroupChatRemoteControl] 消息已通过PubSub发送到群聊:', topic)
          } catch (pubsubError) {
            // 降级：使用后端 API
            console.warn('[useGroupChatRemoteControl] PubSub发送失败，尝试使用后端API:', pubsubError)
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
            console.log('[useGroupChatRemoteControl] 消息已通过后端API发送到群聊:', topic)
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
      
      // 触发切换事件，确保UI同步更新
      window.dispatchEvent(new CustomEvent('switch-input-target', {
        detail: { target: 'groupChat' }
      }))
    } else {
      console.log('[useGroupChatRemoteControl] 群聊面板未显示，忽略点击')
    }
  }, [showGroupChat, inputTargetMode, setInputTargetMode])

  // 发送消息（整合了遥控功能的逻辑）
  const sendMessage = useCallback(async () => {
    const text = currentMessage.trim()
    if (!text || !activeChannelId) {
      return
    }

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

      setCurrentMessage('')
      // 发送消息后自动打开对话面板
      openConversationPanel?.()
      await sendMessageToAgent(activeChannelId, text, selectedAgent)
      return
    }

    // 群聊面板打开时，启用遥控功能
    if (inputTargetMode === 'groupChat') {
      // 发送到群聊
      setCurrentMessage('')
      await sendMessageToGroupChat(text, activeActionId)
    } else {
      // 发送到智能体主对话（遥控模式）
      if (!isSessionReady) {
        await createSession()
        setSessionReady(true)
      }

      setCurrentMessage('')
      // 发送消息后自动打开对话面板
      openConversationPanel?.()
      await sendMessageToAgent(activeChannelId, text, selectedAgent)
    }
  }, [
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
    setCurrentMessage,
    setSessionReady,
    showGroupChat,
  ])

  return {
    inputTargetMode,
    sendMessage,
    handleGroupChatPanelClick,
  }
}


