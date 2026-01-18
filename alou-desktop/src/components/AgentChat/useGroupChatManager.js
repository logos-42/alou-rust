import { useCallback, useEffect, useMemo, useState, useRef } from 'react'
import { useGroupChat } from './useGroupChat'
import useClusterActionStore from '@/stores/clusterActionStore'

/**
 * useGroupChatManager - 群聊管理 Hook
 * 统一管理群聊相关的状态、事件监听和UI控制
 */
export const useGroupChatManager = ({ openConversationPanel, activeChannelId }) => {
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

  // 加载当前频道的群聊
  useEffect(() => {
    if (activeChannelId && loadedChannelRef.current !== activeChannelId) {
      loadedChannelRef.current = activeChannelId
      // 使用 setTimeout 延迟执行，避免在渲染过程中更新状态
      setTimeout(() => {
        const result = loadChannelGroupChats(activeChannelId)
        
        // 如果加载了群聊且有保存的显示状态，恢复显示
        if (result.activeActionId) {
          const saved = localStorage.getItem(`agent-chat-show-group-chat-${activeChannelId}`)
          if (saved === 'true') {
            setShowGroupChat(true)
            openConversationPanelRef.current?.()
          }
        }
      }, 0)
    } else if (!activeChannelId) {
      loadedChannelRef.current = null
      setShowGroupChat(false)
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [activeChannelId]) // 只依赖 activeChannelId，避免循环

  // 获取当前频道的 activeActionId（使用 Zustand selector 订阅状态变化）
  const activeActionId = useClusterActionStore((state) => {
    return activeChannelId ? state.activeActionIdByChannel[activeChannelId] || null : null
  })

  // 本地 UI 状态（按频道存储）
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

  // 保存群聊显示状态到 localStorage（按频道）
  useEffect(() => {
    if (typeof window !== 'undefined' && activeChannelId) {
      localStorage.setItem(`agent-chat-show-group-chat-${activeChannelId}`, showGroupChat.toString())
    }
  }, [showGroupChat, activeChannelId])

  // 群聊相关数据
  const activeAction = useMemo(() => {
    const action = activeChannelId ? getActiveAction(activeChannelId) : null
    console.log('[useGroupChatManager] 获取到的activeAction:', action)
    console.log('[useGroupChatManager] activeChannelId:', activeChannelId)
    if (action && action.agents) {
      console.log('[useGroupChatManager] activeAction中的agents:', action.agents)
      console.log('[useGroupChatManager] agents数量:', action.agents.length)
    } else {
      console.log('[useGroupChatManager] activeAction或agents为空')
    }
    
    // 强制绕过缓存，直接获取最新数据
    if (activeChannelId && !action?.agents?.length) {
      console.log('[useGroupChatManager] 尝试绕过缓存，直接获取store状态')
      const currentState = useClusterActionStore.getState()
      const allActions = currentState.actionsByChannel[activeChannelId] || []
      console.log('[useGroupChatManager] 直接获取的actions:', allActions)
      const foundAction = allActions.find((a) => a.action_id === currentState.activeActionIdByChannel[activeChannelId])
      if (foundAction && foundAction.agents?.length) {
        console.log('[useGroupChatManager] 绕过缓存成功，使用新数据:', foundAction)
        return foundAction
      }
    }
    
    return action
  }, [activeChannelId, activeActionId, getActiveAction])
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

  // 完全关闭群聊（只关闭显示，保留记录以便下次打开）
  const closeGroupChatCompletely = useCallback(() => {
    setShowGroupChat(false)
    // 不清除 activeActionId，保留记录以便下次可以重新打开
    // 只清除持久化的显示状态
    if (typeof window !== 'undefined' && activeChannelId) {
      localStorage.removeItem(`agent-chat-show-group-chat-${activeChannelId}`)
    }
  }, [activeChannelId])

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
      if (actionId && activeChannelId) {
        setActiveAction(actionId, activeChannelId)
        setShowGroupChat(true)
        openConversationPanel?.()
      }
    }

    window.addEventListener('cluster-action-created', handleClusterActionCreated)
    return () => {
      window.removeEventListener('cluster-action-created', handleClusterActionCreated)
    }
  }, [setActiveAction, openConversationPanel, activeChannelId])


  // 当 activeActionId 被清除时，自动关闭群聊显示
  useEffect(() => {
    if (!activeActionId && showGroupChat && activeChannelId) {
      setShowGroupChat(false)
      if (typeof window !== 'undefined') {
        localStorage.removeItem(`agent-chat-show-group-chat-${activeChannelId}`)
      }
    }
  }, [activeActionId, showGroupChat, activeChannelId])

  // 获取当前频道的群聊列表（直接使用 getActions，不订阅 store，避免循环）
  const groupChatList = useMemo(() => {
    if (!activeChannelId) return []
    return getActions(activeChannelId) || []
    // 只依赖 activeChannelId，getActions 是稳定的函数引用
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [activeChannelId])

  // 切换群聊
  const switchGroupChat = useCallback((actionId) => {
    if (activeChannelId) {
      setActiveAction(actionId, activeChannelId)
    }
  }, [activeChannelId, setActiveAction])

  return {
    // 状态
    showGroupChat,
    activeActionId,
    activeAction,
    groupChatMessages: groupChatMessagesFromHook.length > 0 ? groupChatMessagesFromHook : groupChatMessages,
    actionStatus: actionStatus || 'Pending',
    splitPosition,
    groupChatList,

    // 操作方法
    openGroupChat,
    closeGroupChat,
    closeGroupChatCompletely,
    toggleGroupChat,
    setSplitPosition,
    switchGroupChat,

    // 计算属性
    hasActiveAction: !!activeActionId,
    canOpenGroupChat: !!activeActionId,
  }
}

export default useGroupChatManager


