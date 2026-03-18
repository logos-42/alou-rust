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

/**
 * 提取工具调用结果中的 AI 回复内容
 */
function extractToolResultSummary(event) {
  if (!event || event.event !== 'tool.result') return null
  
  const payload = event.payload || event.raw?.payload
  if (!payload) return null
  
  // 尝试从工具结果中提取 AI 生成的内容
  if (payload.result) {
    const result = payload.result
    if (typeof result === 'string') {
      return result.slice(0, 200) // 限制长度
    }
    if (result.content) {
      return typeof result.content === 'string' ? result.content.slice(0, 200) : null
    }
    if (result.response) {
      return typeof result.response === 'string' ? result.response.slice(0, 200) : null
    }
    if (result.data) {
      return typeof result.data === 'string' ? result.data.slice(0, 200) : null
    }
  }
  
  // 尝试从 preview 字段获取
  if (payload.preview) {
    return typeof payload.preview === 'string' ? payload.preview.slice(0, 200) : null
  }
  
  return null
}

const AgentStreamPanel = ({ events = [], status = 'idle' }) => {
  const [collapsed, setCollapsed] = useState(true)
  // 跟踪每个事件的展开/折叠状态
  const [expandedEvents, setExpandedEvents] = useState<Record<string, boolean>>({})

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
      
      // 提取工具调用结果的 AI 回复摘要
      const resultSummary = extractToolResultSummary(event)
      
      // 工具调用结果默认折叠，显示 AI 回复摘要
      const isToolResult = event.event === 'tool.result'
      const isExpanded = expandedEvents[event.key || index] !== undefined 
        ? expandedEvents[event.key || index]
        : (isToolResult ? false : index === events.length - 1)

      return {
        key: `${event.timestamp || index}_${index}`,
        label,
        icon,
        timestamp: event.timestamp,
        details,
        resultSummary,
        isToolResult,
        isFinal: Boolean(event.isFinal),
        isExpanded,
        eventKey: event.key || index,
      }
    })
  }, [events, expandedEvents])

  const toggleEventExpand = (eventKey: number | string) => {
    setExpandedEvents(prev => ({
      ...prev,
      [eventKey]: !prev[eventKey]
    }))
  }

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
                  <details open={item.isExpanded}>
                    <summary onClick={(e) => {
                      e.preventDefault()
                      toggleEventExpand(item.eventKey)
                    }}>
                      <span className="label">
                        <span className="icon">{item.icon}</span>
                        {item.label}
                        {item.isToolResult && item.resultSummary && (
                          <span className="result-preview">
                            {item.resultSummary.length > 50 
                              ? `${item.resultSummary.slice(0, 50)}...` 
                              : item.resultSummary}
                          </span>
                        )}
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

