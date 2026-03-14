import React, { forwardRef, useImperativeHandle, useRef, useMemo, memo, useState, useEffect } from 'react'
import ChatInput from '@/components/ChatInput'
import { useI18n } from '@/hooks/useI18n'
import './AgentConsoleDock.css'

const AgentConsoleDock = memo(
  forwardRef(
    ({ isLoading, style, showOpenButton, onSend, onCancel, onOpenConversation, inputTargetMode, showGroupChat, onNewLine },
      ref,
    ) => {
      const { t } = useI18n()
      
      // 使用内部状态管理输入，完全独立于父组件渲染
      const [currentMessage, setCurrentMessage] = useState('')

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

      // 当外部 onSend 被调用时（发送消息），清空输入
      useEffect(() => {
        // 监听消息发送 - 通过检查输入是否为空来判断
      }, [currentMessage])

      const handleSend = (text) => {
        const trimmed = text?.trim()
        if (!trimmed) return
        setCurrentMessage('')  // 发送后清空
        onSend?.(trimmed)
      }

      const handleNewLine = () => {
        setCurrentMessage((prev) => `${prev}\n`)
      }

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
              value={currentMessage}
              onChange={setCurrentMessage}
              onSend={handleSend}
              onCancel={onCancel}
              onNewLine={onNewLine || handleNewLine}
              isLoading={isLoading}
              placeholder={placeholder}
            />
          </div>
        </div>
      )
    },
  ))


export default AgentConsoleDock
