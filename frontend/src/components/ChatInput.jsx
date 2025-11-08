import React, { forwardRef, useEffect, useImperativeHandle, useRef } from 'react'
import { useI18n } from '@/hooks/useI18n'
import './ChatInput.css'

const ChatInput = forwardRef(
  ({ value, onChange, onSend, onNewLine, isLoading = false, autoFocus = false }, ref) => {
    const { t } = useI18n()
    const textareaRef = useRef(null)

    const adjustHeight = () => {
      const textarea = textareaRef.current
      if (!textarea) return
      textarea.style.height = 'auto'
      textarea.style.height = `${Math.min(textarea.scrollHeight, 120)}px`
    }

    useImperativeHandle(
      ref,
      () => ({
        adjustHeight,
      }),
      [],
    )

    useEffect(() => {
      adjustHeight()
    }, [value])

    useEffect(() => {
      if (autoFocus && textareaRef.current) {
        textareaRef.current.focus()
      }
    }, [autoFocus])

    const handleInput = (event) => {
      onChange?.(event.target.value)
    }

    const handleKeyDown = (event) => {
      if (event.key === 'Enter' && event.shiftKey) {
        onNewLine?.()
        return
      }

      if (event.key === 'Enter' && !event.shiftKey) {
        event.preventDefault()
        onSend?.()
      }
    }

    const disabled = !value?.trim() || isLoading

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
              placeholder={t('inputPlaceholder')}
            />
            <div className="button-group">
              <button
                type="button"
                className="send-btn"
                onClick={onSend}
                disabled={disabled}
                title={t('send')}
              >
                {!isLoading ? (
                  <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z" />
                  </svg>
                ) : (
                  <svg
                    width="20"
                    height="20"
                    viewBox="0 0 24 24"
                    fill="currentColor"
                    className="loading-icon"
                  >
                    <path d="M12,4V2A10,10 0 0,0 2,12H4A8,8 0 0,1 12,4Z" />
                  </svg>
                )}
              </button>
            </div>
          </div>
        </div>
      </div>
    )
  },
)

ChatInput.displayName = 'ChatInput'

export default ChatInput

