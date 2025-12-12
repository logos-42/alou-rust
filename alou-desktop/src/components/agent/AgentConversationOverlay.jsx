import React, { forwardRef, useImperativeHandle, useRef } from 'react'
import MessageList from '@/components/MessageList'
import AgentStreamPanel from '@/components/agent/AgentStreamPanel'
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
      streamEvents = [],
      streamStatus = 'idle',
      embedded = false,
      title = '会话',
      subtitle,
      actions,
      emptyState,
      avatar,
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

    const avatarSrc = avatar || DEFAULT_AVATAR
    const avatarAlt = typeof title === 'string' ? title : '智能体'

    const header = (
      <header>
        <div className="title">
          <span className="agent-avatar">
            <img src={avatarSrc} alt={avatarAlt} />
          </span>
          <div className="title-text">
            <span>{title}</span>
            {subtitle && <small>{subtitle}</small>}
          </div>
        </div>
        <div className="header-actions">
          {actions}
          <button type="button" className="close-btn" onClick={onClose} title="关闭对话">
            ✕
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

    if (embedded) {
      return (
        <section className="conversation-panel embedded" style={style}>
          {header}
          {body}
        </section>
      )
    }

    return (
      <div className="conversation-overlay" style={style}>
        <section className="conversation-panel">
          {header}
          {body}
        </section>
      </div>
    )
  },
)

AgentConversationOverlay.displayName = 'AgentConversationOverlay'

export default AgentConversationOverlay
