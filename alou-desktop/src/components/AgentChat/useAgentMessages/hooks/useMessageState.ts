/**
 * 消息状态管理 Hook
 * 
 * @module components/AgentChat/useAgentMessages/hooks/useMessageState
 */

import { useState, useCallback, useMemo, useRef } from 'react'
import { Message } from '../types'

interface UseMessageStateReturn {
  messagesByChannel: Record<string, Message[]>
  currentMessage: string
  isLoading: boolean
  loadingByAgent: Record<string, boolean>
  sessionsByAgent: Record<string, string>
  systemPromptCache: Record<string, string>
  setCurrentMessage: (msg: string) => void
  setIsLoading: (loading: boolean) => void
  setSessionsByAgent: React.Dispatch<React.SetStateAction<Record<string, string>>>
  setSystemPromptCache: React.Dispatch<React.SetStateAction<Record<string, string>>>
  isAgentLoading: (agentId: string) => boolean
  setAgentLoading: (agentId: string, loading: boolean) => void
  appendMessage: (message: Message, channelId?: string) => void
  setMessagesForChannel: (channelId: string, msgs: Message[]) => void
  clearMessagesForChannel: (channelId: string) => void
}

/**
 * 消息状态管理
 */
export function useMessageState(): UseMessageStateReturn {
  const [messagesByChannel, setMessagesByChannel] = useState<Record<string, Message[]>>({})
  const [currentMessage, setCurrentMessage] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [loadingByAgent, setLoadingByAgent] = useState<Record<string, boolean>>({})
  const [sessionsByAgent, setSessionsByAgent] = useState<Record<string, string>>({})
  const [systemPromptCache, setSystemPromptCache] = useState<Record<string, string>>({})

  const savedMessageCountRef = useRef<Record<string, number>>({})

  const isAgentLoading = useCallback((agentId: string) => {
    return !!loadingByAgent[agentId]
  }, [loadingByAgent])

  const setAgentLoading = useCallback((agentId: string, loading: boolean) => {
    setLoadingByAgent(prev => ({ ...prev, [agentId]: loading }))
  }, [])

  const appendMessage = useCallback((message: Message, channelId?: string) => {
    if (!channelId) {
      channelId = 'default'
    }

    setMessagesByChannel(prev => {
      const channelMessages = prev[channelId] || []
      return {
        ...prev,
        [channelId]: [...channelMessages, message],
      }
    })
  }, [])

  const setMessagesForChannel = useCallback((channelId: string, msgs: Message[]) => {
    setMessagesByChannel(prev => ({
      ...prev,
      [channelId]: msgs,
    }))
  }, [])

  const clearMessagesForChannel = useCallback((channelId: string) => {
    setMessagesByChannel(prev => ({
      ...prev,
      [channelId]: [],
    }))
    savedMessageCountRef.current[channelId] = 0
  }, [])

  return {
    messagesByChannel,
    currentMessage,
    isLoading,
    loadingByAgent,
    sessionsByAgent,
    systemPromptCache,
    setCurrentMessage,
    setIsLoading,
    setSessionsByAgent,
    setSystemPromptCache,
    isAgentLoading,
    setAgentLoading,
    appendMessage,
    setMessagesForChannel,
    clearMessagesForChannel,
  }
}
