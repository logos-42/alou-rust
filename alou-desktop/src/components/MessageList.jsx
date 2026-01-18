import React, { forwardRef, useCallback, useImperativeHandle, useMemo, useRef, useEffect } from 'react'
import { useI18n } from '@/hooks/useI18n'
import './MessageList.css'
import LoadingIcon from '@/assets/加载0.2.png'

const sourceMap = {
  'wasm-core': 'WASM',
  'edge-worker-proxy': 'Edge',
  'http-backend-fallback': 'Backend',
  system: 'System',
  error: 'Error',
}

const formatMessage = (content) => {
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

const formatTime = (timestamp) =>
  new Date(timestamp).toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
  })

const MessageList = forwardRef(
  ({ messages = [], isLoading = false, loadingContent = null, onMessageSelect }, ref) => {
  const containerRef = useRef(null)
  const { t } = useI18n()

  useImperativeHandle(
    ref,
    () => ({
      container: containerRef.current,
      scrollToBottom: () => {
        const container = containerRef.current
        if (container) {
          container.scrollTop = container.scrollHeight
        }
      },
    }),
    [],
  )

  const handleCopy = useCallback(async (content, messageId) => {
    try {
      await navigator.clipboard.writeText(content)
    } catch (error) {
      console.error('复制失败:', error)
    }
  }, [])

  const renderedMessages = useMemo(
    () =>
      messages.map((message) => ({
        ...message,
        html: formatMessage(message.content),
        formattedTime: formatTime(message.timestamp),
        formattedSource: message.source ? sourceMap[message.source] || message.source : null,
      })),
    [messages],
  )

  // 自动滚动到底部
  useEffect(() => {
    const container = containerRef.current
    if (container) {
      // 检查是否已经接近底部（用户可能在手动查看历史消息）
      const isNearBottom = container.scrollHeight - container.scrollTop - container.clientHeight < 100

      // 如果接近底部或者正在加载，则自动滚动到底部
      if (isNearBottom || isLoading) {
        // 使用 requestAnimationFrame 确保 DOM 更新后再滚动
        requestAnimationFrame(() => {
          container.scrollTop = container.scrollHeight
        })
      }
    }
  }, [renderedMessages, isLoading])

  // 当新消息到达时，强制滚动到底部（除非用户正在查看历史消息）
  useEffect(() => {
    const container = containerRef.current
    if (container && messages.length > 0) {
      const lastMessage = messages[messages.length - 1]
      if (lastMessage && (lastMessage.type === 'assistant' || lastMessage.type === 'user')) {
        // 对于新消息，总是尝试滚动到底部
        setTimeout(() => {
          container.scrollTop = container.scrollHeight
        }, 50)
      }
    }
  }, [messages])

  return (
    <div className="messages-area" ref={containerRef}>
      <div>
        {renderedMessages.map((message) => {
          const isInteractive = typeof onMessageSelect === 'function'
          return (
            <div
              key={message.id}
              className={`message-wrapper ${message.type}`}
              role={isInteractive ? 'button' : undefined}
              tabIndex={isInteractive ? 0 : undefined}
              onClick={() => onMessageSelect?.(message)}
              onKeyDown={(event) => {
                if (!isInteractive) return
                if (event.key === 'Enter' || event.key === ' ') {
                  onMessageSelect(message)
                }
              }}
            >
              <div className="message-bubble">
                <div
                  className="message-content"
                  dangerouslySetInnerHTML={{ __html: message.html }}
                />
                <div className="message-footer">
                  <span className="timestamp">{message.formattedTime}</span>
                  <div className="message-actions">
                    <button
                      type="button"
                      className="copy-btn"
                      onClick={(e) => {
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
        })}
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
