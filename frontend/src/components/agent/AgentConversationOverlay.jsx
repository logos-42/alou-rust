import React, { forwardRef, useImperativeHandle, useRef } from 'react'
import MessageList from '@/components/MessageList'
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

    return (
      <div className="conversation-overlay" style={style}>
        <section className="conversation-panel">
          <header>
            <div className="title">
              <span className="emoji">💬</span>
              <span>会话上下文</span>
            </div>
            <div className={`status ${connectionStatus}`}>
              <span className="dot" />
              <span>{connectionStatusLabel}</span>
            </div>
            <button type="button" className="close-btn" onClick={onClose}>
              ✕
            </button>
          </header>
          <div className="conversation-body">
            <MessageList
              ref={messageListRef}
              messages={messages}
              isLoading={isLoading}
              onMessageSelect={onInspectMessage}
            />
          </div>
        </section>
      </div>
    )
  },
)

AgentConversationOverlay.displayName = 'AgentConversationOverlay'

export default AgentConversationOverlay

