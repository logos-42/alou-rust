import React, { useMemo, useState } from 'react'
import './AgentStreamPanel.css'

const EVENT_LABELS = {
  'stream.connected': '流式连接已建立',
  'conversation.received': '收到请求',
  'llm.request': '模型推理中',
  'tool.calls': '执行工具调用',
  'tool.result': '工具返回结果',
  'llm.response': '模型生成回复',
  'assistant.persisted': '回复已保存',
  'conversation.completed': '等待用户确认',
  'conversation.error': '执行失败',
  'stream.heartbeat': '连接保活',
}

const EVENT_ICONS = {
  'stream.connected': '🛰️',
  'conversation.received': '📥',
  'llm.request': '🤖',
  'tool.calls': '🛠️',
  'tool.result': '📦',
  'llm.response': '💬',
  'assistant.persisted': '📝',
  'conversation.completed': '✅',
  'conversation.error': '⚠️',
  'stream.heartbeat': '💓',
}

const STATUS_LABELS = {
  idle: '未开始',
  polling: '拉取中',
  active: '执行中',
  completed: '已完成',
  error: '异常',
}

const formatTime = (timestamp) => {
  if (!timestamp) return '--:--:--'
  try {
    return new Date(timestamp).toLocaleTimeString('zh-CN', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    })
  } catch (error) {
    return '--:--:--'
  }
}

const safeStringify = (value) => {
  if (!value) return null
  try {
    return JSON.stringify(value, null, 2)
  } catch (error) {
    return String(value)
  }
}

const AgentStreamPanel = ({ events = [], status = 'idle' }) => {
  const [collapsed, setCollapsed] = useState(true)

  const statusLabel = STATUS_LABELS[status] || STATUS_LABELS.idle

  const items = useMemo(() => {
    if (!Array.isArray(events) || events.length === 0) {
      return []
    }
    return events.map((event, index) => {
      const label = EVENT_LABELS[event.event] || event.label || event.event
      const icon = EVENT_ICONS[event.event] || '•'
      const details =
        safeStringify(event.payload) || safeStringify(event.raw?.payload) || null

      return {
        key: `${event.timestamp || index}_${index}`,
        label,
        icon,
        timestamp: event.timestamp,
        details,
        isFinal: Boolean(event.isFinal),
        openByDefault: index === events.length - 1,
      }
    })
  }, [events])

  return (
    <section className={`agent-stream-panel${collapsed ? ' collapsed' : ''}`}>
      <header>
        <div className="panel-title">
          <span className="emoji">📡</span>
          <span>实时执行</span>
          <span className={`status status-${status}`}>{statusLabel}</span>
        </div>
        <button type="button" className="toggle-btn" onClick={() => setCollapsed((prev) => !prev)}>
          {collapsed ? '展开' : '收起'}
        </button>
      </header>

      {!collapsed && (
        <div className="panel-body">
          {items.length === 0 ? (
            <div className="empty-hint">等待后端响应...</div>
          ) : (
            <ul className="event-list">
              {items.slice(-40).map((item) => (
                <li key={item.key} className={item.isFinal ? 'event final' : 'event'}>
                  <details open={item.openByDefault || item.isFinal}>
                    <summary>
                      <span className="label">
                        <span className="icon">{item.icon}</span>
                        {item.label}
                      </span>
                      <span className="time">{formatTime(item.timestamp)}</span>
                    </summary>
                    {item.details && (
                      <pre className="payload" aria-label="event-payload">
                        {item.details}
                      </pre>
                    )}
                  </details>
                </li>
              ))}
            </ul>
          )}
        </div>
      )}
    </section>
  )
}

export default AgentStreamPanel

