import { useCallback, useEffect, useState } from 'react'
import pubsubService from '@/services/pubsubService'
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
}) => {
  // 输入目标模式：'agent' = 发送到智能体（遥控模式），'groupChat' = 发送到群聊
  // 只有在群聊面板打开时才启用遥控功能
  const [inputTargetMode, setInputTargetMode] = useState('agent')

  // 监听群聊面板状态，关闭时重置输入目标模式
  useEffect(() => {
    if (!showGroupChat) {
      setInputTargetMode('agent')
    } else {
      // 群聊面板打开时，默认设置为 'agent' 模式（遥控模式）
      setInputTargetMode('agent')
    }
  }, [showGroupChat])

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

        if (isLocalGroupChat) {
          // 本地群聊：直接将消息添加到 store
          const { addGroupChatMessage } = useClusterActionStore.getState()
          const message = {
            id: `msg_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`,
            type: 'chat',
            from: from,
            fromName: userName || from,
            content: text,
            timestamp: Date.now(),
          }
          addGroupChatMessage(actionId, message)
          console.log('[useGroupChatRemoteControl] 本地群聊消息已添加:', actionId)
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
                id: `msg_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`,
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
    [userName],
  )

  // 处理群聊面板点击，切换输入目标模式
  const handleGroupChatPanelClick = useCallback(() => {
    if (showGroupChat) {
      setInputTargetMode('groupChat')
    }
  }, [showGroupChat])

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


