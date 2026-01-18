import React, { useEffect, useRef, useState, useMemo } from 'react'
import GroupChatMessage from './GroupChatMessage'
import GroupChatArchiveList from './GroupChatArchiveList'
import { useI18n } from '@/hooks/useI18n'
import { resolveAgentAvatar } from '../AgentChat/agentUtils'
import GroupIcon from '@/assets/群组.png'
import RefreshIcon from '@/assets/刷新0.2.png'
import CloseIcon from '@/assets/关闭0.3.png'
import LoadingIcon from '@/assets/加载0.2.png'
import './GroupChatPanel.css'

/**
 * AgentAvatar - 智能体头像组件
 * 使用 resolveAgentAvatar 函数来处理头像显示
 */
const AgentAvatar = ({ agent, onAgentClick, t }) => {
  const agentId = agent.id || agent.agent_id || agent.did
  const avatar = resolveAgentAvatar(agent)
  
  // 调试信息
  console.log('[AgentAvatar] 渲染智能体:', {
    agentId,
    originalAgent: agent,
    resolvedAvatar: avatar,
    hasAvatar: !!avatar,
    avatarSource: agent.avatar ? 'agent.avatar' : 
                  agent.avatar_url ? 'agent.avatar_url' :
                  agent.avatar_cid ? 'agent.avatar_cid' : 'unknown'
  })
  
  return (
    <button
      key={agentId}
      className="agent-avatar-button"
      onClick={() => onAgentClick?.(agent)}
      title={`${agent.name || agent.agent_name || t('agent.type.claude')} - 点击打开对话`}
    >
      <div className="agent-avatar-small">
        {avatar ? (
          <img 
            src={avatar} 
            alt={agent.name || agent.agent_name}
            onLoad={() => console.log('[AgentAvatar] 头像加载成功:', avatar)}
            onError={(e) => {
              console.error('[AgentAvatar] 头像加载失败:', avatar, e)
              e.target.style.display = 'none'
            }}
          />
        ) : (
          <div style={{
            width: '32px',
            height: '32px',
            borderRadius: '50%',
            background: '#ccc',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: '12px',
            color: '#666'
          }}>
            {agent.name?.charAt(0)?.toUpperCase() || '?'}
          </div>
        )}
      </div>
      {agent.mode && (
        <div className="agent-mode-badge">{agent.mode === 'agent' ? 'A' : 'L'}</div>
      )}
    </button>
  )
}

/**
 * GroupChatPanel - 基于DIAP PubSub的群聊面板组件
 * 支持去中心化群聊通信和本地持久化
 */
const GroupChatPanel = ({
  activeGroup,
  messages = [],
  agents = [],
  status,
  onClose,
  onRefresh,
  onAgentClick,
  isLoading = false,
  groupChatList = [],
  activeChannelId,
  onSwitchGroupChat,
  onPanelClick,
  onSelectAgent,
  inputTargetMode,
  onSendMessage,
  diapGroupChat,
  // 新增：DIAP群聊管理器的方法
  createGroupChat,
  sendMessage: diapSendMessage,
  switchGroupChat: diapSwitchGroupChat
}) => {
  const { t } = useI18n()
  const messagesEndRef = useRef(null)
  const containerRef = useRef(null)
  const [isScrolledToBottom, setIsScrolledToBottom] = useState(true)
  const [isClicked, setIsClicked] = useState(false)
  const [messageInput, setMessageInput] = useState('')

  // 使用 activeGroup 的 agents，而不是从外部传入
  const actualAgents = useMemo(() => {
    return activeGroup?.agents || agents || []
  }, [activeGroup, agents])

  // 调试信息
  useEffect(() => {
    console.log('[GroupChatPanel] ========== DIAP群聊调试信息 ==========')
    console.log('[GroupChatPanel] activeGroup:', activeGroup)
    console.log('[GroupChatPanel] actualAgents 数量:', actualAgents.length)
    console.log('[GroupChatPanel] messages 数量:', messages.length)
    console.log('[GroupChatPanel] diapGroupChat 状态:', diapGroupChat?.status)
    console.log('[GroupChatPanel] =======================================')
  }, [activeGroup, actualAgents.length, messages.length, diapGroupChat?.status])

  const scrollToBottom = () => {
    if (messagesEndRef.current) {
      messagesEndRef.current.scrollIntoView({ behavior: 'smooth', block: 'end' })
    }
  }

  // 滚动到底部
  // 监听消息变化，自动滚动到底部
  useEffect(() => {
    if (isScrolledToBottom) {
      scrollToBottom()
    }
  }, [messages, isScrolledToBottom, isLoading])

  // 监听滚动事件，判断是否在底部
  const handleScroll = () => {
    if (!containerRef.current) return
    const { scrollTop, scrollHeight, clientHeight } = containerRef.current
    const isAtBottom = scrollHeight - scrollTop - clientHeight < 50
    setIsScrolledToBottom(isAtBottom)
  }

  // 处理面板点击事件，切换输入目标到群聊
  const handlePanelClick = (e) => {
    console.log('[GroupChatPanel] 面板点击事件触发:', e.target)
    
    // 避免点击按钮或其他交互元素时触发
    if (e.target.tagName === 'BUTTON' || e.target.closest('button')) {
      console.log('[GroupChatPanel] 点击了按钮，忽略面板点击')
      return
    }
    
    // 立即添加点击动画效果
    setIsClicked(true)
    const panel = e.currentTarget
    panel.classList.add('clicked')
    setTimeout(() => {
      panel.classList.remove('clicked')
      setIsClicked(false)
    }, 300)
    
    console.log('[GroupChatPanel] 调用 onPanelClick')
    onPanelClick?.()
  }

  // 处理消息发送
  const handleSendMessage = useCallback(async () => {
    if (!messageInput.trim() || !activeGroup) {
      return
    }

    try {
      // 优先使用DIAP群聊的发送方法
      if (diapSendMessage && typeof diapSendMessage === 'function') {
        await diapSendMessage(activeGroup.groupId, messageInput.trim())
      } else if (onSendMessage && typeof onSendMessage === 'function') {
        await onSendMessage(activeGroup.groupId, messageInput.trim())
      } else {
        console.warn('[GroupChatPanel] 没有可用的发送方法')
      }
      
      setMessageInput('')
      console.log('[GroupChatPanel] 消息发送成功:', messageInput.trim())
    } catch (error) {
      console.error('[GroupChatPanel] 发送消息失败:', error)
      // 可以在这里显示错误提示
    }
  }, [messageInput, activeGroup, onSendMessage, diapSendMessage])

  // 处理键盘事件
  const handleKeyPress = useCallback((e) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSendMessage()
    }
  }, [handleSendMessage])

  // 状态显示已移除

  return (
    <section 
      className={`group-chat-panel ${inputTargetMode === 'groupChat' ? 'input-target-active' : ''} ${isClicked ? 'clicked' : ''}`} 
      onClick={handlePanelClick}
    >
      {/* 头部 */}
      <header className="group-chat-header">
        <div className="header-left">
          {onClose && (
            <button type="button" className="close-btn" onClick={onClose} title={t('agent.groupChat.close')}>
              <img src={CloseIcon} alt="关闭" />
            </button>
          )}
          <div className="group-icon">
            <img src={GroupIcon} alt="群组" />
          </div>
          <div className="header-info">
            <div className="group-title">{activeGroup?.groupName || t('agent.groupChat.title')}</div>
            <div className="group-subtitle">
              {activeGroup?.description || `${activeGroup?.channelName || t('agent.groupChat.action')} #${activeGroup?.groupId?.slice(-8) || 'N/A'}`}
            </div>
          </div>
        </div>
        <div className="header-right">
          {/* 状态显示已移除 */}
        </div>
      </header>

      {/* 参与智能体列表 - 头像区域 */}
      {actualAgents.length > 0 && (
        <div className="group-chat-agents-avatars">
          <div className="agents-avatars-label">{t('agent.groupChat.participants')} (actualAgents: {actualAgents.length})</div>
          <div className="agents-avatars-list">
            {actualAgents.map((agent) => (
              <AgentAvatar 
                key={agent.id || agent.agent_id || agent.did}
                agent={agent}
                onAgentClick={onAgentClick}
                t={t}
              />
            ))}
          </div>
        </div>
      )}
      {actualAgents.length === 0 && (
        <div style={{ padding: '10px', color: '#999', fontSize: '12px' }}>
          等待智能体加入...
        </div>
      )}

      {/* 内容区域 */}
      <div className="group-chat-content">
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
                    <img src={LoadingIcon} alt="加载中" className="loading-icon" />
                    <span className="typing-text">{t('agent.groupChat.typing')}</span>
                  </div>
                </div>
              )}
              <div ref={messagesEndRef} />
            </>
          )}
        </div>

        {/* 存档列表 */}
        {groupChatList && groupChatList.length > 0 && (
          <GroupChatArchiveList
            groupChatList={groupChatList}
            activeActionId={actionId}
            activeChannelId={activeChannelId}
            onSwitchGroupChat={onSwitchGroupChat}
          />
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
          <img src={RefreshIcon} alt={t('agent.groupChat.refresh')} />
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

      {/* 消息输入框 */}
      {activeGroup && (
        <div className="message-input-container">
          <div className="message-input-wrapper">
            <input
              type="text"
              className="message-input"
              value={messageInput}
              onChange={(e) => setMessageInput(e.target.value)}
              onKeyPress={handleKeyPress}
              placeholder="输入消息..."
              disabled={diapGroupChat?.isLoading}
            />
            <button
              type="button"
              className="send-button"
              onClick={handleSendMessage}
              disabled={!messageInput.trim() || diapGroupChat?.isLoading}
              title="发送消息 (Enter)"
            >
              {diapGroupChat?.isLoading ? '...' : '→'}
            </button>
          </div>
        </div>
      )}
    </section>
  )
}

export default GroupChatPanel

