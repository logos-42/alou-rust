import React, { useMemo } from 'react'
import './GroupChatMessage.css'

const DEFAULT_AVATAR = 'https://avatars.githubusercontent.com/u/16309930?v=4'

/**
 * GroupChatMessage - 群聊消息组件
 * 类似微信的群聊消息样式，显示发送者信息
 */
const GroupChatMessage = ({ message }) => {
  const { id, type, from, fromName, avatar, content, timestamp, metadata } = message

  const formattedTime = useMemo(() => {
    if (!timestamp) return ''
    const date = new Date(timestamp)
    const now = new Date()
    const diff = now - date

    // 如果是今天，只显示时间
    if (diff < 24 * 60 * 60 * 1000 && date.getDate() === now.getDate()) {
      return date.toLocaleTimeString('zh-CN', {
        hour: '2-digit',
        minute: '2-digit',
      })
    }

    // 如果是昨天
    if (diff < 48 * 60 * 60 * 1000) {
      return `昨天 ${date.toLocaleTimeString('zh-CN', {
        hour: '2-digit',
        minute: '2-digit',
      })}`
    }

    // 其他情况显示完整日期时间
    return date.toLocaleString('zh-CN', {
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    })
  }, [timestamp])

  const avatarSrc = avatar || DEFAULT_AVATAR
  const displayName = fromName || from || '未知'

  // 格式化消息内容
  const formatContent = (text) => {
    if (!text) return ''
    return text
      .replace(/\n/g, '<br>')
      .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
      .replace(/\*(.*?)\*/g, '<em>$1</em>')
      .replace(/`(.*?)`/g, '<code>$1</code>')
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
            <span className="task-type">{isRequest ? '任务请求' : '任务完成'}</span>
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

  // 普通消息样式（类似微信）
  const isUser = type === 'user'
  const isAgent = type === 'agent'

  return (
    <div className={`group-chat-message ${isUser ? 'user-message' : 'agent-message'}`}>
      <div className="message-avatar">
        <img src={avatarSrc} alt={displayName} />
      </div>
      <div className="message-content-wrapper">
        <div className="message-header">
          <span className="message-sender">{displayName}</span>
          {formattedTime && <span className="message-time">{formattedTime}</span>}
        </div>
        <div className={`message-bubble ${isUser ? 'user-bubble' : 'agent-bubble'}`}>
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

