import React, { forwardRef, useImperativeHandle, useRef } from 'react'
import ChatInput from '@/components/ChatInput'
import './AgentConsoleDock.css'

const AgentConsoleDock = forwardRef(
  (
    { value, onChange, isLoading, style, showOpenButton, onSend, onCancel, onNewLine, onOpenConversation },
    ref,
  ) => {
    const chatInputRef = useRef(null)

    useImperativeHandle(
      ref,
      () => ({
        adjustInputHeight: () => chatInputRef.current?.adjustHeight?.(),
      }),
      [],
    )

    return (
      <div className="console-dock" style={style}>
        {showOpenButton && (
          <button type="button" className="reopen-btn" onClick={onOpenConversation}>
            查看对话
          </button>
        )}
        <div className="console-input">
          <ChatInput
            ref={chatInputRef}
            value={value}
            onChange={onChange}
            onSend={onSend}
            onCancel={onCancel}
            onNewLine={onNewLine}
            isLoading={isLoading}
          />
        </div>
      </div>
    )
  },
)

AgentConsoleDock.displayName = 'AgentConsoleDock'

export default AgentConsoleDock
