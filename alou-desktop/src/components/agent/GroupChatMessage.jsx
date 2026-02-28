import React, { useMemo } from 'react'
import { useI18n } from '@/hooks/useI18n'
import DOMPurify from 'dompurify'
import './GroupChatMessage.css'

const DEFAULT_AVATAR = 'https://avatars.githubusercontent.com/u/16309930?v=4'

/**
 * GroupChatMessage - 群聊消息组件
 * 类似微信的群聊消息样式，显示发送者信息
 */
const GroupChatMessage = ({ message }) => {
  const { t, currentLanguage } = useI18n()
  const { id, type, from, fromName, avatar, content, timestamp, metadata } = message

  const formattedTime = useMemo(() => {
    if (!timestamp) return ''
    const date = new Date(timestamp)
    const now = new Date()
    const diff = now - date

    const locale = currentLanguage === 'zh' ? 'zh-CN' : 'en-US'

    // 如果是今天，只显示时间
    if (diff < 24 * 60 * 60 * 1000 && date.getDate() === now.getDate()) {
      return date.toLocaleTimeString(locale, {
        hour: '2-digit',
        minute: '2-digit',
      })
    }

    // 如果是昨天
    if (diff < 48 * 60 * 60 * 1000) {
      return `${t('agent.groupChat.yesterday')} ${date.toLocaleTimeString(locale, {
        hour: '2-digit',
        minute: '2-digit',
      })}`
    }

    // 其他情况显示完整日期时间
    return date.toLocaleString(locale, {
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    })
  }, [timestamp, currentLanguage, t])

  const avatarSrc = avatar || DEFAULT_AVATAR
  const displayName = fromName || from || t('agent.groupChat.unknown')

  // 格式化消息内容 - 使用 DOMPurify 防止 XSS
  const formatContent = (text) => {
    if (!text) return ''
    const formatted = text
      .replace(/\n/g, '<br>')
      .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
      .replace(/\*(.*?)\*/g, '<em>$1</em>')
      .replace(/`(.*?)`/g, '<code>$1</code>')
    // 使用 DOMPurify 清理 HTML，防止 XSS 攻击
    return DOMPurify.sanitize(formatted, {
      ALLOWED_TAGS: ['br', 'strong', 'em', 'code'],
      ALLOWED_ATTR: []
    })
  }

  // 系统消息样式
  if (type === 'system') {
    return (
      <div className="group-chat-message system-message">
        <div className="system-message-content">
          <span className="system-icon">ℹ️</span>
          <span className="system-text">{content}</span>
          {formattedTime && <span className="system-time">{formattedTime}</span>}
        </div>
      </div>
    )
  }

  // 任务请求消息样式
  if (type === 'task_request' || type === 'task_result') {
    const isRequest = type === 'task_request'
    return (
      <div className={`group-chat-message task-message ${isRequest ? 'task-request' : 'task-result'}`}>
        <div className="task-message-content">
          <div className="task-header">
            <span className="task-icon">{isRequest ? '📋' : '✅'}</span>
            <span className="task-type">{isRequest ? t('agent.groupChat.taskRequest') : t('agent.groupChat.taskComplete')}</span>
            {metadata?.task_id && (
              <span className="task-id">#{metadata.task_id.slice(-8)}</span>
            )}
          </div>
          <div className="task-body">
            <div className="task-description">{content}</div>
            {metadata?.task_type && (
              <div className="task-meta">
                <span className="task-type-tag">{metadata.task_type}</span>
                {metadata?.status && (
                  <span className={`task-status status-${metadata.status.toLowerCase()}`}>
                    {metadata.status}
                  </span>
                )}
              </div>
            )}
          </div>
          <div className="task-footer">
            <span className="task-from">{displayName}</span>
            {formattedTime && <span className="task-time">{formattedTime}</span>}
          </div>
        </div>
      </div>
    )
  }

  // 普通消息样式（用户消息与对话页面保持一致）
  const isUser = type === 'user'
  const isAgent = type === 'agent'

  // 用户消息使用与对话页面一致的格式
  if (isUser) {
    return (
      <div className="message-wrapper user">
        <div className="message-bubble">
          <div
            className="message-content"
            dangerouslySetInnerHTML={{ __html: formatContent(content) }}
          />
          <div className="message-footer">
            <span className="timestamp">{formattedTime}</span>
            <div className="message-actions">
              <button
                type="button"
                className="copy-btn"
                onClick={(e) => {
                  e.stopPropagation()
                  navigator.clipboard?.writeText(content).catch(() => {})
                }}
                title="复制内容"
              >
                📋
              </button>
            </div>
          </div>
        </div>
      </div>
    )
  }

  // 智能体消息保持群聊样式（显示头像和发送者信息）
  return (
    <div className="group-chat-message agent-message">
      <div className="message-avatar">
        <img src={avatarSrc} alt={displayName} />
      </div>
      <div className="message-content-wrapper">
        <div className="message-header">
          <span className="message-sender">{displayName}</span>
          {formattedTime && <span className="message-time">{formattedTime}</span>}
        </div>
        <div className="message-bubble agent-bubble">
          <div
            className="message-text"
            dangerouslySetInnerHTML={{ __html: formatContent(content) }}
          />
        </div>
      </div>
    </div>
  )
}

export default GroupChatMessage

