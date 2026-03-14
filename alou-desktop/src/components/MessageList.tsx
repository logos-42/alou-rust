import React, { forwardRef, memo, useCallback, useImperativeHandle, useMemo, useRef } from 'react'
import { useI18n } from '@/hooks/useI18n'
import './MessageList.css'
import LoadingIcon from '@/assets/加载0.2.png'

const sourceMap: Record<string, string> = {
  'wasm-core': 'WASM',
  'edge-worker-proxy': 'Edge',
  'http-backend-fallback': 'Backend',
  system: 'System',
  error: 'Error',
}

export interface Message {
  id: string
  type: string
  content: string
  html?: string
  timestamp: number
  formattedTime?: string
  source?: string
  formattedSource?: string | null
  isNew?: boolean
}

export interface MessageListProps {
  messages?: Message[]
  isLoading?: boolean
  loadingContent?: React.ReactNode
  onMessageSelect?: (message: Message) => void
}

export interface MessageListRef {
  container: HTMLDivElement | null
  scrollToBottom: () => void
}

const formatMessage = (content: string | null | undefined): string => {
  // 如果 content 是 null 或 undefined，返回空字符串
  if (content == null) {
    return ''
  }

  // 确保 content 是字符串类型
  const contentStr = String(content)

  // 如果字符串为空，直接返回
  if (!contentStr.trim()) {
    return contentStr
  }

  return contentStr
    .replace(/\n/g, '<br>')
    .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
    .replace(/\*(.*?)\*/g, '<em>$1</em>')
    .replace(/`(.*?)`/g, '<code>$1</code>')
    .replace(/•/g, '<span class="bullet">•</span>')
}

const formatTime = (timestamp: number): string =>
  new Date(timestamp).toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
  })

interface MessageItemProps {
  message: Message & { isNew?: boolean }
  isInteractive: boolean
  onMessageSelect?: (message: Message) => void
  handleCopy: (content: string, messageId: string) => void
  isNew?: boolean
}

// 优化：单个消息组件，使用 memo 防止不必要的重渲染
const MessageItem = memo<MessageItemProps>(({ message, isInteractive, onMessageSelect, handleCopy, isNew }) => {
  return (
    <div
      className={`message-wrapper ${message.type}${isNew ? ' message-enter' : ''}`}
      role={isInteractive ? 'button' : undefined}
      tabIndex={isInteractive ? 0 : undefined}
      onClick={() => onMessageSelect?.(message)}
      onKeyDown={(event: React.KeyboardEvent) => {
        if (!isInteractive) return
        if (event.key === 'Enter' || event.key === ' ') {
          onMessageSelect(message)
        }
      }}
    >
      <div className="message-bubble">
        <div
          className="message-content"
          dangerouslySetInnerHTML={{ __html: message.html || '' }}
        />
        <div className="message-footer">
          <span className="timestamp">{message.formattedTime}</span>
          <div className="message-actions">
            <button
              type="button"
              className="copy-btn"
              onClick={(e: React.MouseEvent) => {
                e.stopPropagation()
                handleCopy(message.content, message.id)
              }}
              title="复制内容"
            >
              📋
            </button>
            {message.formattedSource && (
              <span className="source-tag">{message.formattedSource}</span>
            )}
          </div>
        </div>
      </div>
    </div>
  )
})

MessageItem.displayName = 'MessageItem'

const MessageList = forwardRef<MessageListRef, MessageListProps>(
  ({ messages = [], isLoading = false, loadingContent = null, onMessageSelect }, ref) => {
  const containerRef = useRef<HTMLDivElement>(null)
  const { t } = useI18n()
  // 跟踪已渲染过的消息 ID，用于判断哪些是新消息
  const renderedMessageIds = useRef<Set<string>>(new Set())

  useImperativeHandle(
    ref,
    () => ({
      container: containerRef.current,
      scrollToBottom: () => {
        // 简化的滚动逻辑，只负责滚动自己的容器
        const container = containerRef.current
        if (container) {
          container.scrollTop = container.scrollHeight
        }
      },
    }),
    [],
  )

  const handleCopy = useCallback(async (content: string, messageId: string) => {
    try {
      await navigator.clipboard.writeText(content)
    } catch (error) {
      console.error('复制失败:', error)
    }
  }, [])

  // 跟踪上一条消息的 ID，用于判断是否有新消息
  const prevLastMessageIdRef = useRef<string | null>(null)

  const renderedMessages = useMemo(() => {
    const lastMessage = messages[messages.length - 1]
    const lastMessageId = lastMessage?.id || null

    // 检查是否有新消息（通过比较最后一条消息的 ID）
    const hasNewMessage = lastMessageId && lastMessageId !== prevLastMessageIdRef.current

    // 更新上一条消息 ID
    if (hasNewMessage) {
      prevLastMessageIdRef.current = lastMessageId
    }

    return messages.map((message, index) => ({
      ...message,
      html: formatMessage(message.content),
      formattedTime: formatTime(message.timestamp),
      formattedSource: message.source ? sourceMap[message.source] || message.source : null,
      // 只有最后一条消息是新消息
      isNew: hasNewMessage && index === messages.length - 1,
    }))
  }, [messages])

  // 移除了自动滚动逻辑，由父容器 AgentConversationOverlay 统一控制

  const isInteractive = typeof onMessageSelect === 'function'

  return (
    <div className="messages-area" ref={containerRef}>
      <div>
        {renderedMessages.map((message) => (
          <MessageItem
            key={message.id}
            message={message}
            isInteractive={isInteractive}
            onMessageSelect={onMessageSelect}
            handleCopy={handleCopy}
            isNew={message.isNew}
          />
        ))}
      </div>

      {isLoading &&
        (loadingContent || (
          <div className="message-wrapper assistant">
            <div className="message-bubble loading">
              <div className="typing-animation">
                <img src={LoadingIcon} alt="加载中" className="loading-icon" />
                <span className="typing-text">{t('thinking')}</span>
              </div>
            </div>
          </div>
        ))}
    </div>
  )
})

MessageList.displayName = 'MessageList'

export default MessageList
