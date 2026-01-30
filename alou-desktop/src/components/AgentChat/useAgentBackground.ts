import { useState, useEffect, useCallback } from 'react'

/**
 * Hook for managing agent-specific background images
 * 管理智能体专属背景图片
 * 
 * @param activeChannelId - 当前活动的频道ID
 * @returns 背景相关的状态和方法
 */
export const useAgentBackground = (activeChannelId: string | null = null) => {
  const [chatBackground, setChatBackground] = useState<string>('')
  
  // 根据 activeChannelId 获取存储键名
  const getBackgroundStorageKey = useCallback((channelId: string | null): string => {
    if (!channelId) return 'alou-chat-background'
    return `alou-chat-background-${channelId}`
  }, [])
  
  // 当 activeChannelId 变化时，加载对应的背景
  useEffect(() => {
    if (typeof localStorage !== 'undefined') {
      const storageKey = getBackgroundStorageKey(activeChannelId)
      const stored = localStorage.getItem(storageKey) || ''
      setChatBackground(stored)
    } else {
      setChatBackground('')
    }
  }, [activeChannelId, getBackgroundStorageKey])
  
  // 更新背景图片
  const updateBackground = useCallback((backgroundUrl: string | null | undefined) => {
    setChatBackground(backgroundUrl || '')
  }, [])
  
  return {
    chatBackground,
    updateBackground,
    getBackgroundStorageKey,
  }
}

export default useAgentBackground
