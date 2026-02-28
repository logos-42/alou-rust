/**
 * useLocalIpfsGroupChat - 本地IPFS群聊管理Hook
 * 使用本地IPFS节点创建pubsub，消息存储在本地内存和KV中
 */

import { useState, useEffect, useCallback, useRef } from 'react'
import localIpfsGroupChatService, { LocalGroup, LocalGroupMessage, LocalIdentity } from '@/services/localIpfsGroupChatService'

/**
 * 群聊配置接口
 */
export interface GroupConfig {
  groupName: string
  description?: string
  members?: string[]
  isPublic?: boolean
  maxMembers?: number
  metadata?: Record<string, any>
}

/**
 * Hook返回结果接口
 */
export interface UseLocalIpfsGroupChatReturn {
  isInitialized: boolean
  isIpfsAvailable: boolean
  localIdentity: LocalIdentity | null
  groups: LocalGroup[]
  activeGroup: LocalGroup | null
  messages: LocalGroupMessage[]
  isLoading: boolean
  error: string | null
  createGroup: (config: GroupConfig) => Promise<LocalGroup>
  joinGroup: (groupId: string, topic?: string | null) => Promise<LocalGroup>
  switchToGroup: (groupId: string) => Promise<void>
  sendMessage: (content: string) => Promise<void>
  leaveGroup: (groupId: string) => Promise<void>
  refreshGroups: () => Promise<void>
  clearError: () => void
  getGroupById: (groupId: string) => LocalGroup | null
  getGroupMessages: (groupId: string) => LocalGroupMessage[]
}

/**
 * 本地IPFS群聊 Hook
 */
export const useLocalIpfsGroupChat = (): UseLocalIpfsGroupChatReturn => {
  const [isInitialized, setIsInitialized] = useState(false)
  const [isIpfsAvailable, setIsIpfsAvailable] = useState(false)
  const [localIdentity, setLocalIdentity] = useState<LocalIdentity | null>(null)
  const [groups, setGroups] = useState<LocalGroup[]>([])
  const [activeGroup, setActiveGroup] = useState<LocalGroup | null>(null)
  const [messages, setMessages] = useState<LocalGroupMessage[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  
  const messageHandlersRef = useRef<Map<string, Set<(message: LocalGroupMessage) => void>>>(new Map())

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
        let hasValidIdentity = false
        try {
          // 从localStorage获取用户信息
          const walletAddress = typeof window !== 'undefined' ? localStorage.getItem('wallet_address') : null
          const userId = typeof window !== 'undefined' ? localStorage.getItem('user_id') : null
          
          if (walletAddress || userId) {
            const identity: LocalIdentity = {
              did: walletAddress || userId || '',
              name: walletAddress ? `用户${walletAddress.slice(-6)}` : '本地用户'
            }
            setLocalIdentity(identity)
            localIpfsGroupChatService.setLocalIdentity(identity)
            hasValidIdentity = true
          } else {
            // 身份为空，提示用户设置身份
            setError('请先在设置中创建身份（钱包地址或用户ID）才能使用群聊功能')
          }
        } catch (identityError: any) {
          console.warn('[useLocalIpfsGroupChat] 获取本地身份失败:', identityError)
          setError('获取身份信息失败，请刷新页面重试')
        }

        // 如果身份无效，仍然可以加载群聊但不能发送消息
        if (!hasValidIdentity) {
          console.warn('[useLocalIpfsGroupChat] 身份无效，无法创建/加入群聊')
        }

        // 加载已有的群聊
        await localIpfsGroupChatService.loadGroupsFromKV()
        const loadedGroups = localIpfsGroupChatService.getAllGroups()
        setGroups(loadedGroups)

        setIsInitialized(true)
        console.log('[useLocalIpfsGroupChat] 初始化完成')
      } catch (error: any) {
        console.error('[useLocalIpfsGroupChat] 初始化失败:', error)
        setError(error.message)
      } finally {
        setIsLoading(false)
      }
    }

    initialize()
  }, [])

  // 定期检测IPFS连接状态（每30秒检查一次）
  useEffect(() => {
    if (!isInitialized) return

    const checkIpfsHealth = async () => {
      try {
        const wasAvailable = isIpfsAvailable
        const ipfsAvailable = await localIpfsGroupChatService.checkIpfsAvailability()
        setIsIpfsAvailable(ipfsAvailable)

        // 状态变化时更新错误提示
        if (!ipfsAvailable && wasAvailable) {
          console.warn('[useLocalIpfsGroupChat] IPFS连接已断开')
          setError('IPFS节点已断开连接，请检查IPFS节点是否正在运行')
        } else if (ipfsAvailable && !wasAvailable) {
          console.log('[useLocalIpfsGroupChat] IPFS连接已恢复')
          // 清除之前的IPFS断开错误
          if (error?.includes('IPFS')) {
            setError(null)
          }
        }
      } catch (error) {
        console.error('[useLocalIpfsGroupChat] IPFS健康检查失败:', error)
        if (isIpfsAvailable) {
          setIsIpfsAvailable(false)
          setError('IPFS连接检查失败，请检查IPFS节点状态')
        }
      }
    }

    // 初始检查后设置定期心跳
    const heartbeatInterval = setInterval(checkIpfsHealth, 30000)

    return () => {
      clearInterval(heartbeatInterval)
    }
  }, [isInitialized, isIpfsAvailable, error])

  // 创建群聊
  const createGroup = useCallback(async (config: GroupConfig): Promise<LocalGroup> => {
    if (!isInitialized) {
      throw new Error('服务未初始化')
    }

    // 检查是否有本地身份
    if (!localIdentity) {
      const errorMsg = '请先连接钱包或创建身份'
      setError(errorMsg)
      throw new Error(errorMsg)
    }

    try {
      setIsLoading(true)
      setError(null)

      // 如果IPFS可用，正常创建；否则降级到内存模式
      const forceMemoryMode = !isIpfsAvailable
      if (forceMemoryMode) {
        console.log('[useLocalIpfsGroupChat] IPFS不可用，使用内存模式创建群聊')
      }

      const group = await localIpfsGroupChatService.createGroup(config, forceMemoryMode)
      
      // 更新群聊列表
      setGroups(prev => [...prev, group])
      
      console.log('[useLocalIpfsGroupChat] 群聊创建成功:', group)
      return group
    } catch (error: any) {
      console.error('[useLocalIpfsGroupChat] 创建群聊失败:', error)
      setError(error.message)
      throw error
    } finally {
      setIsLoading(false)
    }
  }, [isInitialized, isIpfsAvailable, localIdentity])

  // 加入群聊
  const joinGroup = useCallback(async (groupId: string, topic?: string | null): Promise<LocalGroup> => {
    if (!isInitialized) {
      throw new Error('服务未初始化')
    }

    try {
      setIsLoading(true)
      setError(null)

      // 如果IPFS不可用，降级到内存模式
      const forceMemoryMode = !isIpfsAvailable
      if (forceMemoryMode) {
        console.log('[useLocalIpfsGroupChat] IPFS不可用，使用内存模式加入群聊')
      }

      const group = await localIpfsGroupChatService.joinGroup(groupId, topic, forceMemoryMode)
      
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
    } catch (error: any) {
      console.error('[useLocalIpfsGroupChat] 加入群聊失败:', error)
      setError(error.message)
      throw error
    } finally {
      setIsLoading(false)
    }
  }, [isInitialized, isIpfsAvailable, localIdentity])

  // 切换到指定群聊
  const switchToGroup = useCallback(async (groupId: string): Promise<void> => {
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

      // 添加消息处理器 - 修复：保存 handler 引用以便后续清理
      if (!messageHandlersRef.current.has(groupId)) {
        messageHandlersRef.current.set(groupId, new Set())
      }

      const messageHandler = (message: LocalGroupMessage) => {
        setMessages(prev => {
          const exists = prev.find(m => m.id === message.id)
          if (!exists) {
            return [...prev, message]
          }
          return prev
        })
      }

      messageHandlersRef.current.get(groupId)!.add(messageHandler)
      localIpfsGroupChatService.addMessageHandler(groupId, messageHandler)

      console.log('[useLocalIpfsGroupChat] 切换到群聊:', groupId)
    } catch (error: any) {
      console.error('[useLocalIpfsGroupChat] 切换群聊失败:', error)
      setError(error.message)
    } finally {
      setIsLoading(false)
    }
  }, [isInitialized])

  // 发送消息
  const sendMessage = useCallback(async (content: string): Promise<void> => {
    if (!activeGroup || !isInitialized || !isIpfsAvailable) {
      throw new Error('没有活跃群聊或服务不可用')
    }

    try {
      setError(null)
      await localIpfsGroupChatService.sendMessage(content, activeGroup.groupId, activeGroup.topic)
      console.log('[useLocalIpfsGroupChat] 消息发送成功')
    } catch (error: any) {
      console.error('[useLocalIpfsGroupChat] 发送消息失败:', error)
      setError(error.message)
      throw error
    }
  }, [activeGroup, isInitialized, isIpfsAvailable])

  // 离开群聊 - 修复消息处理器清理
  const leaveGroup = useCallback(async (groupId: string): Promise<void> => {
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

      // 清理消息处理器 - 修复：同时清理 service 中的处理器
      const handlers = messageHandlersRef.current.get(groupId)
      if (handlers) {
        // 从 service 中移除所有处理器
        handlers.forEach(handler => {
          localIpfsGroupChatService.removeMessageHandler(groupId, handler)
        })
        messageHandlersRef.current.delete(groupId)
      }

      console.log('[useLocalIpfsGroupChat] 离开群聊成功:', groupId)
    } catch (error: any) {
      console.error('[useLocalIpfsGroupChat] 离开群聊失败:', error)
      setError(error.message)
    } finally {
      setIsLoading(false)
    }
  }, [isInitialized, activeGroup])

  // 刷新群聊列表
  const refreshGroups = useCallback(async (): Promise<void> => {
    if (!isInitialized) return

    try {
      await localIpfsGroupChatService.loadGroupsFromKV()
      const loadedGroups = localIpfsGroupChatService.getAllGroups()
      setGroups(loadedGroups)
      console.log('[useLocalIpfsGroupChat] 群聊列表已刷新')
    } catch (error: any) {
      console.error('[useLocalIpfsGroupChat] 刷新群聊列表失败:', error)
      setError(error.message)
    }
  }, [isInitialized])

  // 清理资源
  useEffect(() => {
    return () => {
      localIpfsGroupChatService.cleanup()
      messageHandlersRef.current.clear()
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
    getGroupById: (groupId: string) => localIpfsGroupChatService.getGroup(groupId),
    getGroupMessages: (groupId: string) => localIpfsGroupChatService.getGroupMessages(groupId),
  }
}

export default useLocalIpfsGroupChat
