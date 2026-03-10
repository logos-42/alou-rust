import React, { createContext, useContext, useMemo, ReactNode } from 'react'

export interface Message {
  id: string
  type: 'user' | 'assistant' | 'system' | 'error'
  content: string
  timestamp: number
  source?: string
  agentId?: string
  isLoading?: boolean
  metadata?: Record<string, unknown>
}

interface MessageContextValue {
  messages: Message[]
  isLoading: boolean
}

const MessageContext = createContext<MessageContextValue | null>(null)

// 消息列表 Hook - 从 Context 获取消息
export const useMessageList = () => {
  const context = useContext(MessageContext)
  if (!context) {
    throw new Error('useMessageList must be used within MessageProvider')
  }
  return context
}

interface MessageProviderProps {
  children: ReactNode
  messages: Message[]
  isLoading: boolean
}

// 消息提供者 - 使用 useMemo 确保只有在 messages 或 isLoading 真正变化时才更新
export const MessageProvider: React.FC<MessageProviderProps> = ({
  children,
  messages,
  isLoading,
}) => {
  // 使用 useMemo 确保只有在 messages 或 isLoading 真正变化时才更新
  const value = useMemo<MessageContextValue>(
    () => ({ messages, isLoading }),
    [messages, isLoading]
  )

  return (
    <MessageContext.Provider value={value}>
      {children}
    </MessageContext.Provider>
  )
}

export default MessageContext
