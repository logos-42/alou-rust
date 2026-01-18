import React, { forwardRef, useImperativeHandle, useRef, useEffect } from 'react'
import MessageList from '@/components/MessageList'
import AgentStreamPanel from '@/components/agent/AgentStreamPanel'
import CloseIcon from '@/assets/关闭0.3.png'
import EditIcon from '@/assets/修改.png'
import './AgentConversationOverlay.css'

const DEFAULT_AVATAR = 'https://avatars.githubusercontent.com/u/16309930?v=4'

const AgentConversationOverlay = forwardRef(
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

    useImperativeHandle(
      ref,
      () => ({
        scrollToBottom: () => {
          messageListRef.current?.scrollToBottom?.()
        },
      }),
      [],
    )

    // 监听消息变化，自动滚动到底部
    useEffect(() => {
      if (messageListRef.current) {
        // 使用 setTimeout 确保 DOM 完全更新后再滚动，避免被中断
        setTimeout(() => {
          messageListRef.current.scrollToBottom()
        }, 100)
      }
    }, [messages, isLoading])

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
      <div className="conversation-body">
        {messages.length === 0 && !isLoading && emptyState ? (
          emptyState
        ) : (
          <MessageList
            ref={messageListRef}
            messages={messages}
            isLoading={isLoading}
            loadingContent={
              <div className="stream-panel-wrapper">
                <AgentStreamPanel events={streamEvents} status={streamStatus} />
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
)

AgentConversationOverlay.displayName = 'AgentConversationOverlay'

export default AgentConversationOverlay
