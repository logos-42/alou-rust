/**
 * useDiapGroupChat - DIAP群聊Hook
 * 为了向后兼容而保留的简单实现
 * 实际功能已迁移到useLocalIpfsGroupChat
 */

import { useState, useEffect, useCallback } from 'react'
import { useLocalIpfsGroupChat, UseLocalIpfsGroupChatReturn, LocalGroup, LocalIdentity } from './useLocalIpfsGroupChat'

/**
 * 群聊配置接口
 */
export interface GroupConfig {
  groupName: string
  description?: string
  members?: string[]
  agents?: any[]
  channel?: any
  isPublic?: boolean
  maxMembers?: number
  metadata?: Record<string, any>
}

/**
 * Hook返回结果接口
 */
export interface UseDiapGroupChatReturn {
  status: 'idle' | 'loading' | 'ready' | 'error'
  isLoading: boolean
  error: string | null
  isInitialized: boolean
  groups: LocalGroup[]
  activeGroup: LocalGroup | null
  messages: any[]
  createGroup: (config: GroupConfig) => Promise<LocalGroup>
  createGroupWithAgents: (config: GroupConfig) => Promise<LocalGroup>
  joinGroup: (groupId: string, topic?: string | null) => Promise<LocalGroup>
  sendMessage: (groupId: string, content: string) => Promise<void>
  leaveGroup: (groupId: string) => Promise<void>
  switchToGroup: (groupId: string) => Promise<void>
  refreshGroups: () => Promise<void>
  localIdentity: LocalIdentity | null
  isIpfsAvailable: boolean
  clearError: () => void
}

/**
 * DIAP群聊 Hook
 * 向后兼容的封装，实际使用本地IPFS群聊服务
 */
export const useDiapGroupChat = (options: Record<string, any> = {}): UseDiapGroupChatReturn => {
  // 使用新的本地IPFS群聊服务
  const localIpfsGroupChat: UseLocalIpfsGroupChatReturn = useLocalIpfsGroupChat()

  // 兼容性状态
  const [status, setStatus] = useState<'idle' | 'loading' | 'ready' | 'error'>('idle')
  const [error, setError] = useState<string | null>(null)

  // 状态同步
  useEffect(() => {
    if (localIpfsGroupChat.isLoading) {
      // eslint-disable-next-line react-hooks/setState-in-effect
      setStatus('loading')
    } else if (localIpfsGroupChat.error) {
      // eslint-disable-next-line react-hooks/setState-in-effect
      setStatus('error')
      // eslint-disable-next-line react-hooks/setState-in-effect
      setError(localIpfsGroupChat.error)
    } else if (localIpfsGroupChat.isInitialized) {
      // eslint-disable-next-line react-hooks/setState-in-effect
      setStatus('ready')
      // eslint-disable-next-line react-hooks/setState-in-effect
      setError(null)
    } else {
      // eslint-disable-next-line react-hooks/setState-in-effect
      setStatus('idle')
      // eslint-disable-next-line react-hooks/setState-in-effect
      setError(null)
    }
  }, [localIpfsGroupChat.isLoading, localIpfsGroupChat.error, localIpfsGroupChat.isInitialized])

  // 兼容性方法
  const createGroup = useCallback(async (config: GroupConfig): Promise<LocalGroup> => {
    try {
      setStatus('loading')
      const group = await localIpfsGroupChat.createGroup(config)
      setStatus('ready')
      return group
    } catch (error: any) {
      setStatus('error')
      setError(error.message)
      throw error
    }
  }, [localIpfsGroupChat.createGroup])

  const joinGroup = useCallback(async (groupId: string, topic?: string | null): Promise<LocalGroup> => {
    try {
      setStatus('loading')
      const group = await localIpfsGroupChat.joinGroup(groupId, topic)
      setStatus('ready')
      return group
    } catch (error: any) {
      setStatus('error')
      setError(error.message)
      throw error
    }
  }, [localIpfsGroupChat.joinGroup])

  const sendMessage = useCallback(async (groupId: string, content: string): Promise<void> => {
    try {
      await localIpfsGroupChat.sendMessage(content)
    } catch (error: any) {
      setStatus('error')
      setError(error.message)
      throw error
    }
  }, [localIpfsGroupChat.sendMessage])

  const leaveGroup = useCallback(async (groupId: string): Promise<void> => {
    try {
      setStatus('loading')
      await localIpfsGroupChat.leaveGroup(groupId)
      setStatus('ready')
    } catch (error: any) {
      setStatus('error')
      setError(error.message)
      throw error
    }
  }, [localIpfsGroupChat.leaveGroup])

  // 创建带智能体的群聊（兼容性方法）
  const createGroupWithAgents = useCallback(async (config: GroupConfig): Promise<LocalGroup> => {
    try {
      setStatus('loading')
      const group = await localIpfsGroupChat.createGroup(config)
      setStatus('ready')
      return group
    } catch (error: any) {
      setStatus('error')
      setError(error.message)
      throw error
    }
  }, [localIpfsGroupChat.createGroup])

  // 返回兼容性接口
  return {
    // 状态
    status,
    isLoading: status === 'loading',
    error,
    isInitialized: localIpfsGroupChat.isInitialized,
    groups: localIpfsGroupChat.groups,
    activeGroup: localIpfsGroupChat.activeGroup,
    messages: localIpfsGroupChat.messages,

    // 方法
    createGroup,
    createGroupWithAgents,
    joinGroup,
    sendMessage,
    leaveGroup,
    switchToGroup: localIpfsGroupChat.switchToGroup,
    refreshGroups: localIpfsGroupChat.refreshGroups,

    // 额外属性
    localIdentity: localIpfsGroupChat.localIdentity,
    isIpfsAvailable: localIpfsGroupChat.isIpfsAvailable,
    clearError: () => {
      setError(null)
      localIpfsGroupChat.clearError?.()
    }
  }
}

export default useDiapGroupChat
