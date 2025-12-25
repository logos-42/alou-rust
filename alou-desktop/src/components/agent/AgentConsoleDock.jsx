import React, { forwardRef, useImperativeHandle, useRef, useMemo } from 'react'
import ChatInput from '@/components/ChatInput'
import { useI18n } from '@/hooks/useI18n'
import './AgentConsoleDock.css'

const AgentConsoleDock = forwardRef(
  (
    { value, onChange, isLoading, style, showOpenButton, onSend, onCancel, onNewLine, onOpenConversation, inputTargetMode, showGroupChat },
    ref,
  ) => {
    const { t } = useI18n()

    // 根据输入目标模式选择占位符文本
    const placeholder = useMemo(() => {
      if (!showGroupChat) {
        return t('agent.chat.inputPlaceholder')
      }
      if (inputTargetMode === 'groupChat') {
        return t('agent.chat.inputPlaceholder.groupChat')
      }
      return t('agent.chat.inputPlaceholder.remote')
    }, [showGroupChat, inputTargetMode, t])
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
            placeholder={placeholder}
          />
        </div>
      </div>
    )
  },
)

AgentConsoleDock.displayName = 'AgentConsoleDock'

export default AgentConsoleDock
