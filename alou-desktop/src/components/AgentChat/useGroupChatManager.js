import { useCallback, useEffect, useMemo, useState } from 'react'
import { useGroupChat } from './useGroupChat'
import useClusterActionStore from '@/stores/clusterActionStore'

/**
 * useGroupChatManager - 群聊管理 Hook
 * 统一管理群聊相关的状态、事件监听和UI控制
 */
export const useGroupChatManager = ({ openConversationPanel }) => {
  // 从 store 获取集群行动相关状态
  const {
    activeActionId,
    setActiveAction,
    getActiveAction,
    getGroupChatMessages,
    getActionStatus,
  } = useClusterActionStore()

  // 本地 UI 状态
  const [showGroupChat, setShowGroupChat] = useState(false)
  const [splitPosition, setSplitPosition] = useState(() => {
    // 从 localStorage 读取保存的分割位置
    if (typeof window !== 'undefined') {
      const saved = localStorage.getItem('agent-chat-split-position')
      if (saved) {
        const parsed = parseFloat(saved)
        if (!isNaN(parsed) && parsed >= 30 && parsed <= 70) {
          return parsed
        }
      }
    }
    return 50 // 默认 50%
  })

  // 保存分割位置到 localStorage
  useEffect(() => {
    if (typeof window !== 'undefined') {
      localStorage.setItem('agent-chat-split-position', splitPosition.toString())
    }
  }, [splitPosition])

  // 群聊相关数据
  const activeAction = useMemo(() => getActiveAction(), [activeActionId, getActiveAction])
  const groupChatMessages = useMemo(
    () => (activeActionId ? getGroupChatMessages(activeActionId) : []),
    [activeActionId, getGroupChatMessages],
  )
  const actionStatus = useMemo(
    () => (activeActionId ? getActionStatus(activeActionId) : null),
    [activeActionId, getActionStatus],
  )

  // 使用 useGroupChat hook 管理群聊订阅和轮询
  const { messages: groupChatMessagesFromHook } = useGroupChat({
    actionId: activeActionId,
    enabled: showGroupChat && !!activeActionId,
  })

  // 打开群聊
  const openGroupChat = useCallback(() => {
    if (activeActionId) {
      setShowGroupChat(true)
      openConversationPanel()
    }
  }, [activeActionId, openConversationPanel])

  // 关闭群聊
  const closeGroupChat = useCallback(() => {
    setShowGroupChat(false)
    // 注意：不自动清除 activeActionId，以便用户可以重新打开
  }, [])

  // 完全关闭群聊（清除行动）
  const closeGroupChatCompletely = useCallback(() => {
    setShowGroupChat(false)
    setActiveAction(null)
  }, [setActiveAction])

  // 切换群聊显示状态
  const toggleGroupChat = useCallback(() => {
    if (showGroupChat) {
      closeGroupChat()
    } else {
      openGroupChat()
    }
  }, [showGroupChat, openGroupChat, closeGroupChat])

  // 监听集群行动创建事件
  useEffect(() => {
    const handleClusterActionCreated = (event) => {
      const { actionId } = event.detail
      if (actionId) {
        setActiveAction(actionId)
        setShowGroupChat(true)
        openConversationPanel?.()
      }
    }

    window.addEventListener('cluster-action-created', handleClusterActionCreated)
    return () => {
      window.removeEventListener('cluster-action-created', handleClusterActionCreated)
    }
  }, [setActiveAction, openConversationPanel])

  // 当 activeActionId 变化时，如果之前有群聊打开，保持打开状态
  useEffect(() => {
    if (activeActionId && !showGroupChat) {
      // 如果有新的行动但群聊未打开，可以选择自动打开
      // 这里不自动打开，让用户手动控制
    }
  }, [activeActionId, showGroupChat])

  return {
    // 状态
    showGroupChat,
    activeActionId,
    activeAction,
    groupChatMessages: groupChatMessagesFromHook.length > 0 ? groupChatMessagesFromHook : groupChatMessages,
    actionStatus: actionStatus || 'Pending',
    splitPosition,

    // 操作方法
    openGroupChat,
    closeGroupChat,
    closeGroupChatCompletely,
    toggleGroupChat,
    setSplitPosition,

    // 计算属性
    hasActiveAction: !!activeActionId,
    canOpenGroupChat: !!activeActionId,
  }
}

export default useGroupChatManager

