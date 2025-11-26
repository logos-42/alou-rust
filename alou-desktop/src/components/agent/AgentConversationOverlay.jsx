import React, { forwardRef, useImperativeHandle, useRef } from 'react'
import MessageList from '@/components/MessageList'
import AgentStreamPanel from '@/components/agent/AgentStreamPanel'
import './AgentConversationOverlay.css'

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

    const header = (
      <header>
        <div className="title">
          <span className="emoji">💬</span>
          <div className="title-text">
            <span>{title}</span>
            {subtitle && <small>{subtitle}</small>}
          </div>
        </div>
        <div className="header-actions">
          <div className={`status ${connectionStatus}`}>
            <span className="dot" />
            <span>{connectionStatusLabel}</span>
          </div>
          {actions}
          {!embedded && (
            <button type="button" className="close-btn" onClick={onClose}>
              ✕
            </button>
          )}
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
