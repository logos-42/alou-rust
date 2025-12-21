import { useCallback, useEffect, useRef } from 'react'
import clusterActionService from '@/services/clusterActionService'
import apiClient from '@/services/api'
import useClusterActionStore from '@/stores/clusterActionStore'
import { i18n } from '@/hooks/useI18n'

/**
 * useGroupChat - 群聊状态管理 Hook
 * 管理群聊消息订阅、状态轮询和消息更新
 */
export const useGroupChat = ({ actionId, enabled = true }) => {
  const {
    addGroupChatMessage,
    addGroupChatMessages,
    updateActionStatus,
    setActionDetails,
    getGroupChatMessages,
    getActionStatus,
    getActiveAction,
  } = useClusterActionStore()

  // 检查是否是本地群聊
  const isLocalGroupChat = useCallback((id) => {
    return id && id.startsWith('local_group_')
  }, [])

  const subscriptionRef = useRef(null)
  const pollIntervalRef = useRef(null)
  const lastTimestampRef = useRef(Date.now())
  const unsubscribeMessagesRef = useRef(null)
  const unsubscribeStatusRef = useRef(null)

  // 转换 PubSub 消息格式为群聊消息格式
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

  // 订阅群聊消息（轮询方式）
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
              
              // 添加到 store
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
            console.error('[useGroupChat] 获取群聊消息失败:', error)
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

  // 加载历史消息
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
          console.error('[useGroupChat] 加载历史消息失败:', error)
        }
      }
    },
    [addGroupChatMessages, addGroupChatMessage, transformPubSubMessage, isLocalGroupChat, getActiveAction, getGroupChatMessages],
  )

  // 轮询行动状态
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
                console.error('[useGroupChat] 获取行动结果失败:', error)
              }
            }
          }
        } catch (error) {
          // 静默处理 404 错误
          if (error.response?.status !== 404) {
            console.error('[useGroupChat] 轮询行动状态失败:', error)
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
    [updateActionStatus, setActionDetails, isLocalGroupChat, getActiveAction],
  )

  // 初始化订阅和轮询
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
      console.log('[useGroupChat] 收到新消息:', message)
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

  // 获取当前消息列表
  const messages = actionId ? getGroupChatMessages(actionId) : []

  // 获取当前状态
  const status = actionId ? getActionStatus(actionId) : null

  return {
    messages,
    status,
    subscribeToGroupChat,
    loadGroupMessages,
    pollActionStatus,
  }
}

export default useGroupChat

