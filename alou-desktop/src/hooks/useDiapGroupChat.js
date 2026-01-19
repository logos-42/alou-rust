/**
 * useDiapGroupChat - DIAP群聊Hook
 * 为了向后兼容而保留的简单实现
 * 实际功能已迁移到useLocalIpfsGroupChat
 */

import { useState, useEffect, useCallback } from 'react'
import { useLocalIpfsGroupChat } from './useLocalIpfsGroupChat'

export const useDiapGroupChat = (options = {}) => {
  // 使用新的本地IPFS群聊服务
  const localIpfsGroupChat = useLocalIpfsGroupChat()

  // 兼容性状态
  const [status, setStatus] = useState('idle')
  const [error, setError] = useState(null)

  // 状态同步
  useEffect(() => {
    if (localIpfsGroupChat.isLoading) {
      setStatus('loading')
    } else if (localIpfsGroupChat.error) {
      setStatus('error')
      setError(localIpfsGroupChat.error)
    } else if (localIpfsGroupChat.isInitialized) {
      setStatus('ready')
      setError(null)
    } else {
      setStatus('idle')
      setError(null)
    }
  }, [localIpfsGroupChat.isLoading, localIpfsGroupChat.error, localIpfsGroupChat.isInitialized])

  // 兼容性方法
  const createGroup = useCallback(async (config) => {
    try {
      setStatus('loading')
      const group = await localIpfsGroupChat.createGroup(config)
      setStatus('ready')
      return group
    } catch (error) {
      setStatus('error')
      setError(error.message)
      throw error
    }
  }, [localIpfsGroupChat.createGroup])

  const joinGroup = useCallback(async (groupId, topic) => {
    try {
      setStatus('loading')
      const group = await localIpfsGroupChat.joinGroup(groupId, topic)
      setStatus('ready')
      return group
    } catch (error) {
      setStatus('error')
      setError(error.message)
      throw error
    }
  }, [localIpfsGroupChat.joinGroup])

  const sendMessage = useCallback(async (groupId, content) => {
    try {
      await localIpfsGroupChat.sendMessage(content)
    } catch (error) {
      setStatus('error')
      setError(error.message)
      throw error
    }
  }, [localIpfsGroupChat.sendMessage])

  const leaveGroup = useCallback(async (groupId) => {
    try {
      setStatus('loading')
      await localIpfsGroupChat.leaveGroup(groupId)
      setStatus('ready')
    } catch (error) {
      setStatus('error')
      setError(error.message)
      throw error
    }
  }, [localIpfsGroupChat.leaveGroup])

  // 创建带智能体的群聊（兼容性方法）
  const createGroupWithAgents = useCallback(async (config) => {
    try {
      setStatus('loading')
      const group = await localIpfsGroupChat.createGroup(config)
      setStatus('ready')
      return group
    } catch (error) {
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
