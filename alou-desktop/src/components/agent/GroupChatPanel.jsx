import React, { useEffect, useRef, useState } from 'react'
import GroupChatMessage from './GroupChatMessage'
import './GroupChatPanel.css'

/**
 * GroupChatPanel - 群聊面板组件
 * 类似微信的群聊界面，显示多智能体协作消息
 */
const GroupChatPanel = ({
  actionId,
  actionDescription,
  agents = [],
  messages = [],
  status,
  onClose,
  onRefresh,
  isLoading = false,
}) => {
  const messagesEndRef = useRef(null)
  const containerRef = useRef(null)
  const [isScrolledToBottom, setIsScrolledToBottom] = useState(true)

  // 滚动到底部
  const scrollToBottom = () => {
    if (messagesEndRef.current) {
      messagesEndRef.current.scrollIntoView({ behavior: 'smooth' })
    }
  }

  // 监听消息变化，自动滚动到底部
  useEffect(() => {
    if (isScrolledToBottom) {
      scrollToBottom()
    }
  }, [messages, isScrolledToBottom])

  // 监听滚动事件，判断是否在底部
  const handleScroll = () => {
    if (!containerRef.current) return
    const { scrollTop, scrollHeight, clientHeight } = containerRef.current
    const isAtBottom = scrollHeight - scrollTop - clientHeight < 50
    setIsScrolledToBottom(isAtBottom)
  }

  // 状态显示已移除

  return (
    <section className="group-chat-panel">
      {/* 头部 */}
      <header className="group-chat-header">
        <div className="header-left">
          {onClose && (
            <button type="button" className="close-btn" onClick={onClose} title="关闭群聊">
              ✕
            </button>
          )}
          <div className="group-icon">👥</div>
          <div className="header-info">
            <div className="group-title">集群行动</div>
            <div className="group-subtitle">
              {actionDescription || `行动 #${actionId?.slice(-8) || 'N/A'}`}
            </div>
          </div>
        </div>
        <div className="header-right">
          {/* 状态显示已移除 */}
        </div>
      </header>

      {/* 参与智能体列表 - 已隐藏 */}
      {false && agents.length > 0 && (
        <div className="group-chat-agents">
          <div className="agents-label">参与智能体 ({agents.length})</div>
          <div className="agents-list">
            {agents.map((agent) => (
              <div key={agent.id || agent.agent_id} className="agent-item">
                <div className="agent-avatar-small">
                  <img src={agent.avatar || 'https://avatars.githubusercontent.com/u/16309930?v=4'} alt={agent.name || agent.agent_name} />
                </div>
                <div className="agent-info">
                  <div className="agent-name">{agent.name || agent.agent_name || '智能体'}</div>
                  {agent.mode && (
                    <div className="agent-mode">{agent.mode === 'agent' ? 'Agent' : 'Alou'}</div>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* 消息列表 */}
      <div className="group-chat-messages" ref={containerRef} onScroll={handleScroll}>
        {messages.length === 0 && !isLoading ? (
          <div className="empty-state">
            <div className="empty-icon">💬</div>
            <div className="empty-text">暂无消息</div>
            <div className="empty-hint">等待智能体开始协作...</div>
          </div>
        ) : (
          <>
            {messages.map((message) => (
              <GroupChatMessage key={message.id} message={message} />
            ))}
            {isLoading && (
              <div className="loading-message">
                <div className="typing-animation">
                  <div className="typing-dots">
                    <span />
                    <span />
                    <span />
                  </div>
                  <span className="typing-text">智能体正在协作...</span>
                </div>
              </div>
            )}
            <div ref={messagesEndRef} />
          </>
        )}
      </div>

      {/* 刷新按钮 - 固定在右下角 */}
      {onRefresh && (
        <button
          type="button"
          className="refresh-btn-fixed"
          onClick={onRefresh}
          title="刷新"
          aria-label="刷新"
        >
          🔄
        </button>
      )}

      {/* 滚动到底部按钮 */}
      {!isScrolledToBottom && messages.length > 0 && (
        <button
          type="button"
          className="scroll-to-bottom-btn"
          onClick={() => {
            setIsScrolledToBottom(true)
            scrollToBottom()
          }}
          title="滚动到底部"
        >
          ↓
        </button>
      )}
    </section>
  )
}

export default GroupChatPanel

