import React, { forwardRef, useEffect, useImperativeHandle, useRef } from 'react'
import { useI18n } from '@/hooks/useI18n'
import sendIcon from '@/assets/向上·发送 2.png'
import cancelIcon from '@/assets/终止0.2.png'
import './ChatInput.css'

export interface ChatInputProps {
  value: string
  onChange?: (value: string) => void
  onSend?: (value: string) => void
  onNewLine?: () => void
  onCancel?: () => void
  isLoading?: boolean
  autoFocus?: boolean
  placeholder?: string
}

export interface ChatInputRef {
  adjustHeight: () => void
}

const ChatInput = forwardRef<ChatInputRef, ChatInputProps>(
  ({ value, onChange, onSend, onNewLine, onCancel, isLoading = false, autoFocus = false, placeholder }, ref) => {
    const { t } = useI18n()
    const displayPlaceholder = placeholder || t('inputPlaceholder')
    const textareaRef = useRef<HTMLTextAreaElement>(null)

    const adjustHeight = () => {
      const textarea = textareaRef.current
      if (!textarea) return

      // 只在内容变化时调整高度，避免不必要的布局抖动
      const newHeight = `${Math.min(textarea.scrollHeight, 120)}px`
      if (textarea.style.height !== newHeight) {
        textarea.style.height = 'auto'
        textarea.style.height = newHeight
      }
    }

    useImperativeHandle(
      ref,
      () => ({
        adjustHeight,
      }),
      [],
    )

    useEffect(() => {
      // 使用 requestAnimationFrame 优化高度调整，减少布局抖动
      const rafId = requestAnimationFrame(() => {
        adjustHeight()
      })
      return () => cancelAnimationFrame(rafId)
    }, [value])

    useEffect(() => {
      if (autoFocus && textareaRef.current) {
        textareaRef.current.focus()
      }
    }, [autoFocus])

    const handleInput = (event: React.ChangeEvent<HTMLTextAreaElement>) => {
      onChange?.(event.target.value)
    }

    const handleKeyDown = (event: React.KeyboardEvent<HTMLTextAreaElement>) => {
      if (event.key === 'Enter' && event.shiftKey) {
        onNewLine?.()
        return
      }

      if (event.key === 'Enter' && !event.shiftKey) {
        event.preventDefault()
        onSend?.(value)  // 传递当前输入值
      }
    }

    const disabled = !value?.trim() || isLoading
    const showCancel = isLoading && onCancel

    return (
      <div className="input-area">
        <div className="input-container">
          <div className="input-wrapper">
            <textarea
              ref={textareaRef}
              className="message-input"
              rows={1}
              value={value}
              onChange={handleInput}
              onKeyDown={handleKeyDown}
              placeholder={displayPlaceholder}
            />
            <div className="button-group">
              {showCancel ? (
                <button
                  type="button"
                  className="cancel-btn"
                  onClick={onCancel}
                  title="终止执行"
                >
                  <img src={cancelIcon} alt="终止" width="20" height="20" />
                </button>
              ) : (
              <button
                type="button"
                className="send-btn"
                onClick={() => onSend?.(value)}  // 传递当前输入值
                disabled={disabled}
                title={t('send')}
              >
                  <img src={sendIcon} alt="发送" width="20" height="20" />
                </button>
              )}
            </div>
          </div>
        </div>
      </div>
    )
  },
)

ChatInput.displayName = 'ChatInput'

export default ChatInput
