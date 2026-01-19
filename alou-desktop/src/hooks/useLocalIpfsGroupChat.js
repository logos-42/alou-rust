/**
 * useLocalIpfsGroupChat - 本地IPFS群聊管理Hook
 * 使用本地IPFS节点创建pubsub，消息存储在本地内存和KV中
 */

import { useState, useEffect, useCallback, useRef } from 'react'
import localIpfsGroupChatService from '@/services/localIpfsGroupChatService'

export const useLocalIpfsGroupChat = () => {
  const [isInitialized, setIsInitialized] = useState(false)
  const [isIpfsAvailable, setIsIpfsAvailable] = useState(false)
  const [localIdentity, setLocalIdentity] = useState(null)
  const [groups, setGroups] = useState([])
  const [activeGroup, setActiveGroup] = useState(null)
  const [messages, setMessages] = useState([])
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState(null)
  
  const messageHandlersRef = new Map() // groupId -> Set<callback>

  // 初始化服务
  useEffect(() => {
    const initialize = async () => {
      try {
        setIsLoading(true)
        setError(null)

        // 初始化本地IPFS群聊服务
        await localIpfsGroupChatService.initialize()
        
        // 检查IPFS可用性
        const ipfsAvailable = await localIpfsGroupChatService.checkIpfsAvailability()
        setIsIpfsAvailable(ipfsAvailable)
        
        if (!ipfsAvailable) {
          setError('IPFS节点不可用，请确保IPFS节点正在运行')
          return
        }

        // 获取本地身份（简化版本，不依赖DIAP）
        try {
          // 从localStorage获取用户信息
          const walletAddress = typeof window !== 'undefined' ? localStorage.getItem('wallet_address') : null
          const userId = typeof window !== 'undefined' ? localStorage.getItem('user_id') : null
          
          if (walletAddress || userId) {
            const identity = {
              did: walletAddress || userId,
              name: walletAddress ? `用户${walletAddress.slice(-6)}` : '本地用户'
            }
            setLocalIdentity(identity)
            localIpfsGroupChatService.setLocalIdentity(identity)
          }
        } catch (identityError) {
          console.warn('[useLocalIpfsGroupChat] 获取本地身份失败:', identityError)
          // 即使没有身份也可以继续，只是不能创建/加入群聊
        }

        // 加载已有的群聊
        await localIpfsGroupChatService.loadGroupsFromKV()
        const loadedGroups = localIpfsGroupChatService.getAllGroups()
        setGroups(loadedGroups)

        setIsInitialized(true)
        console.log('[useLocalIpfsGroupChat] 初始化完成')
      } catch (error) {
        console.error('[useLocalIpfsGroupChat] 初始化失败:', error)
        setError(error.message)
      } finally {
        setIsLoading(false)
      }
    }

    initialize()
  }, [])

  // 创建群聊
  const createGroup = useCallback(async (config) => {
    if (!isInitialized || !isIpfsAvailable) {
      throw new Error('服务未初始化或IPFS不可用')
    }

    try {
      setIsLoading(true)
      setError(null)

      const group = await localIpfsGroupChatService.createGroup(config)
      
      // 更新群聊列表
      setGroups(prev => [...prev, group])
      
      console.log('[useLocalIpfsGroupChat] 群聊创建成功:', group)
      return group
    } catch (error) {
      console.error('[useLocalIpfsGroupChat] 创建群聊失败:', error)
      setError(error.message)
      throw error
    } finally {
      setIsLoading(false)
    }
  }, [isInitialized, isIpfsAvailable])

  // 加入群聊
  const joinGroup = useCallback(async (groupId, topic = null) => {
    if (!isInitialized || !isIpfsAvailable) {
      throw new Error('服务未初始化或IPFS不可用')
    }

    try {
      setIsLoading(true)
      setError(null)

      const group = await localIpfsGroupChatService.joinGroup(groupId, topic)
      
      // 更新群聊列表
      setGroups(prev => {
        const index = prev.findIndex(g => g.groupId === groupId)
        if (index >= 0) {
          const newGroups = [...prev]
          newGroups[index] = group
          return newGroups
        }
        return [...prev, group]
      })

      console.log('[useLocalIpfsGroupChat] 加入群聊成功:', group)
      return group
    } catch (error) {
      console.error('[useLocalIpfsGroupChat] 加入群聊失败:', error)
      setError(error.message)
      throw error
    } finally {
      setIsLoading(false)
    }
  }, [isInitialized, isIpfsAvailable])

  // 切换到指定群聊
  const switchToGroup = useCallback(async (groupId) => {
    if (!isInitialized) return

    try {
      setIsLoading(true)
      
      const group = localIpfsGroupChatService.getGroup(groupId)
      if (!group) {
        throw new Error('群聊不存在')
      }

      // 设置为活跃群聊
      setActiveGroup(group)

      // 订阅群聊消息
      await localIpfsGroupChatService.subscribeToGroup(groupId, group.topic)

      // 获取群聊消息
      const groupMessages = localIpfsGroupChatService.getGroupMessages(groupId)
      setMessages(groupMessages)

      // 添加消息处理器
      if (!messageHandlersRef.has(groupId)) {
        messageHandlersRef.set(groupId, new Set())
      }

      const messageHandler = (message) => {
        setMessages(prev => {
          const exists = prev.find(m => m.id === message.id)
          if (!exists) {
            return [...prev, message]
          }
          return prev
        })
      }

      messageHandlersRef.get(groupId).add(messageHandler)
      localIpfsGroupChatService.addMessageHandler(groupId, messageHandler)

      console.log('[useLocalIpfsGroupChat] 切换到群聊:', groupId)
    } catch (error) {
      console.error('[useLocalIpfsGroupChat] 切换群聊失败:', error)
      setError(error.message)
    } finally {
      setIsLoading(false)
    }
  }, [isInitialized])

  // 发送消息
  const sendMessage = useCallback(async (content) => {
    if (!activeGroup || !isInitialized || !isIpfsAvailable) {
      throw new Error('没有活跃群聊或服务不可用')
    }

    try {
      setError(null)
      await localIpfsGroupChatService.sendMessage(content, activeGroup.groupId, activeGroup.topic)
      console.log('[useLocalIpfsGroupChat] 消息发送成功')
    } catch (error) {
      console.error('[useLocalIpfsGroupChat] 发送消息失败:', error)
      setError(error.message)
      throw error
    }
  }, [activeGroup, isInitialized, isIpfsAvailable])

  // 离开群聊
  const leaveGroup = useCallback(async (groupId) => {
    if (!isInitialized) return

    try {
      setIsLoading(true)
      setError(null)

      await localIpfsGroupChatService.leaveGroup(groupId)

      // 从群聊列表中移除
      setGroups(prev => prev.filter(g => g.groupId !== groupId))

      // 如果是当前活跃群聊，清空状态
      if (activeGroup?.groupId === groupId) {
        setActiveGroup(null)
        setMessages([])
      }

      // 清理消息处理器
      if (messageHandlersRef.has(groupId)) {
        messageHandlersRef.delete(groupId)
      }

      console.log('[useLocalIpfsGroupChat] 离开群聊成功:', groupId)
    } catch (error) {
      console.error('[useLocalIpfsGroupChat] 离开群聊失败:', error)
      setError(error.message)
    } finally {
      setIsLoading(false)
    }
  }, [isInitialized, activeGroup])

  // 刷新群聊列表
  const refreshGroups = useCallback(async () => {
    if (!isInitialized) return

    try {
      await localIpfsGroupChatService.loadGroupsFromKV()
      const loadedGroups = localIpfsGroupChatService.getAllGroups()
      setGroups(loadedGroups)
      console.log('[useLocalIpfsGroupChat] 群聊列表已刷新')
    } catch (error) {
      console.error('[useLocalIpfsGroupChat] 刷新群聊列表失败:', error)
      setError(error.message)
    }
  }, [isInitialized])

  // 清理资源
  useEffect(() => {
    return () => {
      localIpfsGroupChatService.cleanup()
      messageHandlersRef.clear()
    }
  }, [])

  return {
    // 状态
    isInitialized,
    isIpfsAvailable,
    localIdentity,
    groups,
    activeGroup,
    messages,
    isLoading,
    error,

    // 方法
    createGroup,
    joinGroup,
    switchToGroup,
    sendMessage,
    leaveGroup,
    refreshGroups,

    // 工具方法
    clearError: () => setError(null),
    getGroupById: (groupId) => localIpfsGroupChatService.getGroup(groupId),
    getGroupMessages: (groupId) => localIpfsGroupChatService.getGroupMessages(groupId),
  }
}
