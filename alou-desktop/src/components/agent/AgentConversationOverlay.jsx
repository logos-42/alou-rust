import React, { forwardRef, useImperativeHandle, useRef, useEffect, memo } from 'react'
import MessageList from '@/components/MessageList'
import CloseIcon from '@/assets/关闭0.3.png'
import EditIcon from '@/assets/修改.png'
import './AgentConversationOverlay.css'

const DEFAULT_AVATAR = 'https://avatars.githubusercontent.com/u/16309930?v=4'

const AgentConversationOverlay = memo(forwardRef(
  (
    {
      style,
      connectionStatus,
      connectionStatusLabel,
      messages,
      isLoading,
      onClose,
      onInspectMessage,
      onEdit,
      streamEvents = [],
      streamStatus = 'idle',
      embedded = false,
      title = '会话',
      subtitle,
      actions,
      emptyState,
      avatar,
      backgroundImage,
    },
    ref,
  ) => {
    const messageListRef = useRef(null)
    const conversationBodyRef = useRef(null)

    useImperativeHandle(
      ref,
      () => ({
        scrollToBottom: () => {
          // 优先滚动 conversation-body 容器
          const conversationBody = conversationBodyRef.current
          if (conversationBody) {
            conversationBody.scrollTop = conversationBody.scrollHeight
            return
          }
          
          // 备用方案：滚动 messageList 容器
          messageListRef.current?.scrollToBottom?.()
        },
      }),
      [],
    )

    // 自动滚动逻辑 - 使用 scrollIntoView 确保滚动到底部
    useEffect(() => {
      if (messages.length === 0) return

      // 使用 setTimeout 确保 DOM 完全渲染后再滚动
      const timer = setTimeout(() => {
        // 查找最后一条消息元素并滚动到可见区域
        const messageElements = document.querySelectorAll('.message-wrapper')
        if (messageElements.length > 0) {
          const lastMessage = messageElements[messageElements.length - 1]
          lastMessage.scrollIntoView({ behavior: 'auto', block: 'end', inline: 'nearest' })
        }
      }, 100)

      return () => clearTimeout(timer)
    }, [messages])

    const avatarSrc = avatar || DEFAULT_AVATAR
    const avatarAlt = typeof title === 'string' ? title : '智能体'

    const header = (
      <header>
        <div className="title">
          <div className="avatar-container">
            <span className="agent-avatar">
              <img src={avatarSrc} alt={avatarAlt} />
            </span>
            {onEdit && (
              <button 
                type="button" 
                className="edit-btn" 
                onClick={onEdit} 
                title="编辑智能体"
              >
                <img src={EditIcon} alt="编辑" />
              </button>
            )}
          </div>
          <div className="title-text">
            <span>{title}</span>
            {subtitle && <small>{subtitle}</small>}
          </div>
        </div>
        <div className="header-actions">
          {actions}
          <button type="button" className="close-btn" onClick={onClose} title="关闭对话">
            <img src={CloseIcon} alt="关闭" />
          </button>
        </div>
      </header>
    )

    const body = (
      <div className="conversation-body" ref={conversationBodyRef}>
        {messages.length === 0 && !isLoading && emptyState ? (
          emptyState
        ) : (
          <MessageList
            ref={messageListRef}
            messages={messages}
            isLoading={isLoading}
            loadingContent={
              <div className="typing-indicator-wrapper">
                <div className="typing-indicator">
                  <span className="typing-dot" />
                  <span className="typing-dot" />
                  <span className="typing-dot" />
                </div>
              </div>
            }
            onMessageSelect={onInspectMessage}
          />
        )}
      </div>
    )

    const panelClassName = `conversation-panel${embedded ? ' embedded' : ''}${backgroundImage ? ' has-background' : ''}`
    
    const panelStyle = backgroundImage
      ? {
          ...style,
          backgroundImage: `url(${backgroundImage})`,
          backgroundSize: 'cover',
          backgroundPosition: 'center',
          backgroundRepeat: 'no-repeat',
        }
      : style

    if (embedded) {
      return (
        <section className={panelClassName} style={panelStyle}>
          {header}
          {body}
        </section>
      )
    }

    return (
      <div className="conversation-overlay" style={style}>
        <section className={panelClassName} style={panelStyle}>
          {header}
          {body}
        </section>
      </div>
    )
  },
))

AgentConversationOverlay.displayName = 'AgentConversationOverlay'

export default AgentConversationOverlay
