import React, { forwardRef, useCallback, useImperativeHandle, useMemo, useRef } from 'react'
import { useI18n } from '@/hooks/useI18n'
import './MessageList.css'

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
                <div className="typing-dots">
                  <span />
                  <span />
                  <span />
                </div>
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
