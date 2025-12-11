import React, { useEffect, useRef, useState } from 'react'
import GroupChatMessage from './GroupChatMessage'
import { useI18n } from '@/hooks/useI18n'
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
  const { t } = useI18n()
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
            <button type="button" className="close-btn" onClick={onClose} title={t('agent.groupChat.close')}>
              ✕
            </button>
          )}
          <div className="group-icon">👥</div>
          <div className="header-info">
            <div className="group-title">{t('agent.groupChat.title')}</div>
            <div className="group-subtitle">
              {actionDescription || `${t('agent.groupChat.action')} #${actionId?.slice(-8) || 'N/A'}`}
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
          <div className="agents-label">{t('agent.groupChat.participants')} ({agents.length})</div>
          <div className="agents-list">
            {agents.map((agent) => (
              <div key={agent.id || agent.agent_id} className="agent-item">
                <div className="agent-avatar-small">
                  <img src={agent.avatar || 'https://avatars.githubusercontent.com/u/16309930?v=4'} alt={agent.name || agent.agent_name} />
                </div>
                <div className="agent-info">
                  <div className="agent-name">{agent.name || agent.agent_name || t('agent.type.claude')}</div>
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
            <div className="empty-text">{t('agent.groupChat.empty')}</div>
            <div className="empty-hint">{t('agent.groupChat.emptyHint')}</div>
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
                  <span className="typing-text">{t('agent.groupChat.typing')}</span>
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
          title={t('agent.groupChat.refresh')}
          aria-label={t('agent.groupChat.refresh')}
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
          title={t('agent.groupChat.scrollToBottom')}
        >
          ↓
        </button>
      )}
    </section>
  )
}

export default GroupChatPanel

