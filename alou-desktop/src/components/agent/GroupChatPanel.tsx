import React, { useEffect, useRef, useState, useMemo, useCallback } from 'react'
import GroupChatMessage from './GroupChatMessage'
import GroupChatArchiveList from './GroupChatArchiveList'
import { useI18n } from '@/hooks/useI18n'
import { useLocalIpfsGroupChat } from '@/hooks/useLocalIpfsGroupChat'
import { resolveAgentAvatar } from '../AgentChat/agentUtils'
import { getAgentName } from '@/utils/agentNameUtils'
import {
  getPartialMentionAtCursor,
  getMentionSuggestions,
  parseMentions
} from '@/utils/groupchat/mentionParser'
import GroupIcon from '@/assets/群组.png'
import RefreshIcon from '@/assets/刷新0.2.png'
import CloseIcon from '@/assets/关闭0.3.png'
import LoadingIcon from '@/assets/加载0.2.png'
import SendIcon from '@/assets/向上·发送 2.png'
import './GroupChatPanel.css'

/**
 * AgentAvatar - 智能体头像组件
 * 使用 resolveAgentAvatar 函数来处理头像显示
 */
const AgentAvatar = ({ agent, onAgentClick, t }) => {
  const agentId = agent.id || agent.agent_id || agent.did
  const avatar = resolveAgentAvatar(agent)
  
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
 * GroupChatPanel - 基于本地IPFS PubSub的群聊面板组件
 * 使用本地IPFS节点创建pubsub，消息存储在本地内存和KV中
 */
const GroupChatPanel = ({
  onClose,
  onAgentClick,
  onPanelClick,
  // 兼容性参数（保持与原有接口的兼容）
  externalActiveGroup,
  externalMessages = [],
  agents = [],
  status,
  onRefresh,
  externalIsLoading = false,
  groupChatList = [],
  activeChannelId,
  onSwitchGroupChat,
  onSelectAgent,
  externalOnSendMessage,
  onStop,
  inputTargetMode = 'agent'  // 默认值
}) => {
  const { t } = useI18n()
  
  // 使用本地IPFS群聊Hook
  const {
    isInitialized,
    isIpfsAvailable,
    localIdentity,
    groups,
    activeGroup: localActiveGroup,
    messages: localMessages,
    isLoading: localIsLoading,
    error,
    createGroup,
    joinGroup,
    switchToGroup,
    sendMessage,
    leaveGroup,
    refreshGroups,
    clearError
  } = useLocalIpfsGroupChat()

  // 优先使用外部传入的数据，否则使用本地数据
  const activeGroup = externalActiveGroup || localActiveGroup
  const isLoading = externalIsLoading || localIsLoading

  // 使用外部传入的消息或本地消息
  const messages = useMemo(() => {
    // 优先使用外部传入的消息
    if (externalMessages && externalMessages.length > 0) {
      return externalMessages
    }
    // 降级使用本地消息
    return localMessages
  }, [externalMessages, localMessages])
  const messagesEndRef = useRef(null)
  const containerRef = useRef(null)
  const [isScrolledToBottom, setIsScrolledToBottom] = useState(true)
  const [isClicked, setIsClicked] = useState(false)
  const [messageInput, setMessageInput] = useState('')
  const [sendError, setSendError] = useState(null) // 添加错误提示状态
  const [showCreateModal, setShowCreateModal] = useState(false) // 创建群聊模态框
  const [showJoinModal, setShowJoinModal] = useState(false) // 加入群聊模态框
  const [newGroupName, setNewGroupName] = useState('') // 新群聊名称
  const [joinGroupId, setJoinGroupId] = useState('') // 加入群聊 ID
  
  // @ 提令相关状态
  const [showMentionSuggestions, setShowMentionSuggestions] = useState(false)
  const [mentionSuggestions, setMentionSuggestions] = useState([])
  const [selectedSuggestionIndex, setSelectedSuggestionIndex] = useState(0)
  const [cursorPosition, setCursorPosition] = useState(0)
  const inputRef = useRef(null)

  // 使用 activeGroup 的 agents，而不是从外部传入
  const actualAgents = useMemo(() => {
    return activeGroup?.agents || agents || []
  }, [activeGroup, agents])

  // 错误处理
  useEffect(() => {
    if (error) {
      console.error('[GroupChatPanel] 群聊服务错误:', error)
    }
  }, [error])

  // 刷新群聊列表
  const handleRefresh = useCallback(async () => {
    try {
      clearError()
      await refreshGroups()
      onRefresh?.() // 调用外部刷新回调
    } catch (error) {
      console.error('[GroupChatPanel] 刷新失败:', error)
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [refreshGroups, clearError])

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
  const handlePanelClick = useCallback((e) => {
    // 避免点击按钮或其他交互元素时触发
    if (e.target.tagName === 'BUTTON' || e.target.closest('button')) {
      return
    }

    // 添加点击动画效果
    setIsClicked(true)
    const panel = e.currentTarget
    panel.classList.add('clicked')
    setTimeout(() => {
      panel.classList.remove('clicked')
      setIsClicked(false)
    }, 300)

    // 调用 onPanelClick
    onPanelClick?.()
  }, [onPanelClick])

  // 处理消息发送
  const handleSendMessage = useCallback(async () => {
    if (!messageInput.trim()) {
      console.warn('[GroupChatPanel] 消息内容为空')
      return
    }

    if (!activeGroup) {
      console.warn('[GroupChatPanel] 没有活跃的群聊')
      setSendError('服务不可用或没有活跃群聊，请先创建或加入群聊')
      return
    }

    const groupId = activeGroup.groupId || activeGroup.action_id
    if (!groupId) {
      console.error('[GroupChatPanel] 群聊ID缺失:', activeGroup)
      setSendError('群聊ID缺失，无法发送消息')
      return
    }

    // 解析消息中的 @ 提令
    const parsedMessage = parseMentions(messageInput.trim(), actualAgents)
    const messageContent = parsedMessage.cleanContent || messageInput.trim()
    
    console.log('[GroupChatPanel] 准备发送消息:', {
      groupId,
      messageLength: messageContent.length,
      hasMentions: parsedMessage.hasMentions,
      mentions: parsedMessage.mentions,
      isPrivateMention: parsedMessage.isPrivateMention,
      hasExternalCallback: !!externalOnSendMessage,
      isInitialized,
      isIpfsAvailable
    })

    try {
      clearError()
      setSendError(null) // 清除之前的错误
      
      // 构建消息选项，包含提令信息
      const messageOptions = {
        content: messageContent,
        mentions: parsedMessage.mentions,
        hasMentions: parsedMessage.hasMentions,
        isPrivateMention: parsedMessage.isPrivateMention,
        targetAgentId: parsedMessage.isPrivateMention ? parsedMessage.mentions[0]?.agentId : null
      }
      
      // 优先使用外部回调（包含智能体通知逻辑）
      if (externalOnSendMessage && typeof externalOnSendMessage === 'function') {
        // 参数顺序：(groupId, content, options)
        console.log('[GroupChatPanel] 调用外部回调:', groupId, messageContent.slice(0, 50), messageOptions)
        await externalOnSendMessage(groupId, messageContent, messageOptions)
        console.log('[GroupChatPanel] 使用外部回调发送消息成功')
      } else if (isInitialized && isIpfsAvailable) {
        // 降级到本地IPFS群聊服务
        console.log('[GroupChatPanel] 使用本地IPFS服务发送')
        await sendMessage(messageContent, messageOptions)
        console.log('[GroupChatPanel] 使用本地IPFS服务发送消息成功')
      } else if (isInitialized) {
        // 内存模式：直接发送消息
        console.log('[GroupChatPanel] 使用内存模式发送')
        await sendMessage(messageContent, messageOptions)
        console.log('[GroupChatPanel] 使用内存模式发送消息成功')
      } else {
        console.warn('[GroupChatPanel] 没有可用的发送方法')
        setSendError('群聊服务未初始化，请稍后重试')
        return
      }
      
      setMessageInput('')
      console.log('[GroupChatPanel] 消息发送成功')
    } catch (error) {
      console.error('[GroupChatPanel] 发送消息失败:', error)
      setSendError(error.message || '消息发送失败，请重试')
    }
  }, [messageInput, activeGroup, isInitialized, isIpfsAvailable, sendMessage, externalOnSendMessage, clearError, actualAgents])

  // 处理键盘事件
  const handleKeyPress = useCallback((e) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      // 如果显示提令建议，按 Enter 选择建议
      if (showMentionSuggestions && mentionSuggestions.length > 0) {
        e.preventDefault()
        handleSelectMention(mentionSuggestions[selectedSuggestionIndex])
        return
      }
      e.preventDefault()
      handleSendMessage()
    } else if (e.key === 'ArrowDown' && showMentionSuggestions) {
      e.preventDefault()
      setSelectedSuggestionIndex(prev => 
        Math.min(prev + 1, mentionSuggestions.length - 1)
      )
    } else if (e.key === 'ArrowUp' && showMentionSuggestions) {
      e.preventDefault()
      setSelectedSuggestionIndex(prev => Math.max(prev - 1, 0))
    } else if (e.key === 'Escape' && showMentionSuggestions) {
      setShowMentionSuggestions(false)
    } else if (e.key === 'Tab' && showMentionSuggestions && mentionSuggestions.length > 0) {
      e.preventDefault()
      handleSelectMention(mentionSuggestions[selectedSuggestionIndex])
    }
  }, [handleSendMessage, showMentionSuggestions, mentionSuggestions, selectedSuggestionIndex])

  // 处理输入变化 - 检测 @ 提令
  const handleInputChange = useCallback((e) => {
    const value = e.target.value
    const position = e.target.selectionStart || 0
    setCursorPosition(position)
    setMessageInput(value)
    
    // 检测是否在输入 @ 提令
    const partialMention = getPartialMentionAtCursor(value, position)
    
    if (partialMention !== null) {
      // 用户正在输入 @ 提令
      const suggestions = getMentionSuggestions(partialMention, actualAgents)
      setMentionSuggestions(suggestions)
      setSelectedSuggestionIndex(0)
      setShowMentionSuggestions(suggestions.length > 0)
    } else {
      setShowMentionSuggestions(false)
    }
  }, [actualAgents])

  // 选择提令建议
  const handleSelectMention = useCallback((agent) => {
    const partialMention = getPartialMentionAtCursor(messageInput, cursorPosition)
    if (partialMention === null) return
    
    // 找到 @ 的位置并替换
    const atIndex = messageInput.lastIndexOf('@', cursorPosition - 1)
    if (atIndex === -1) return
    
    // 构建新消息：@名称 + 空格
    const newMessage = 
      messageInput.substring(0, atIndex) + 
      '@' + agent.name + ' ' + 
      messageInput.substring(cursorPosition)
    
    setMessageInput(newMessage)
    setShowMentionSuggestions(false)
    
    // 聚焦输入框
    if (inputRef.current) {
      inputRef.current.focus()
    }
  }, [messageInput, cursorPosition])

  // 处理创建群聊
  const handleCreateGroup = useCallback(async () => {
    if (!newGroupName.trim()) {
      setSendError('请输入群聊名称')
      return
    }

    try {
      if (createGroup) {
        const newGroup = await createGroup({
          groupName: newGroupName.trim(),
          description: '新创建的群聊',
          isPublic: true
        })
        
        // 创建成功后，切换到新群聊
        if (newGroup && switchToGroup) {
          console.log('[GroupChatPanel] 群聊创建成功，切换到新群聊:', newGroup.groupId)
          await switchToGroup(newGroup.groupId)
        }
        
        setNewGroupName('')
        setShowCreateModal(false)
        console.log('[GroupChatPanel] 群聊创建成功:', newGroupName)
      } else {
        setSendError('创建群聊功能不可用')
      }
    } catch (error) {
      console.error('[GroupChatPanel] 创建群聊失败:', error)
      setSendError(error.message || '创建群聊失败，请重试')
    }
  }, [newGroupName, createGroup, switchToGroup])

  // 处理加入群聊
  const handleJoinGroup = useCallback(async () => {
    if (!joinGroupId.trim()) {
      setSendError('请输入群聊 ID')
      return
    }

    try {
      if (joinGroup) {
        await joinGroup(joinGroupId.trim())
        setJoinGroupId('')
        setShowJoinModal(false)
        console.log('[GroupChatPanel] 群聊加入成功:', joinGroupId)
      } else {
        setSendError('加入群聊功能不可用')
      }
    } catch (error) {
      console.error('[GroupChatPanel] 加入群聊失败:', error)
      setSendError(error.message || '加入群聊失败，请重试')
    }
  }, [joinGroupId, joinGroup])

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
          {/* 加载状态指示器 */}
          {isLoading && (
            <div className="header-loading-indicator">
              <div className="loading-spinner"></div>
              <span>加载中...</span>
            </div>
          )}
          {/* 停止按钮 - 当有任务运行时显示 */}
          {onStop && (
            <button
              type="button"
              className="stop-btn"
              onClick={(e) => {
                e.stopPropagation()
                onStop()
              }}
              title="停止任务"
            >
              <span className="stop-btn-text">⏹</span>
            </button>
          )}
          {/* 群聊操作按钮 */}
          <div className="group-chat-actions">
            <button
              type="button"
              className="action-btn"
              onClick={(e) => {
                e.stopPropagation()
                setShowCreateModal(true)
              }}
              title={t('agent.groupChat.create') || '创建群聊'}
            >
              ➕ 创建
            </button>
            <button
              type="button"
              className="action-btn"
              onClick={(e) => {
                e.stopPropagation()
                setShowJoinModal(true)
              }}
              title={t('agent.groupChat.join') || '加入群聊'}
            >
              🚀 加入
            </button>
          </div>
        </div>
        {/* 内存模式提示 */}
        {!isIpfsAvailable && isInitialized && (
          <div className="memory-mode-indicator" style={{
            fontSize: '11px',
            color: '#ff9800',
            padding: '4px 8px',
            background: 'rgba(255, 152, 0, 0.1)',
            borderRadius: '4px',
            marginRight: '8px'
          }}>
            📱 内存模式
          </div>
        )}
      </header>

      {/* 错误提示 */}
      {sendError && (
        <div className="send-error-toast">
          <span className="error-icon">⚠️</span>
          <span className="error-message">{sendError}</span>
          <button
            type="button"
            className="error-dismiss"
            onClick={() => setSendError(null)}
            title="关闭提示"
          >
            ×
          </button>
        </div>
      )}

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
            activeActionId={activeChannelId}
            activeChannelId={activeChannelId}
            onSwitchGroupChat={onSwitchGroupChat}
          />
        )}
      </div>

      {/* 刷新按钮 - 固定在右下角 */}
      {(onRefresh || isInitialized) && (
        <button
          type="button"
          className="refresh-btn-fixed"
          onClick={handleRefresh}
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
              ref={inputRef}
              type="text"
              className="message-input"
              value={messageInput}
              onChange={handleInputChange}
              onKeyDown={handleKeyPress}
              onBlur={() => setTimeout(() => setShowMentionSuggestions(false), 200)}
              placeholder={isIpfsAvailable ? "输入消息... (使用 @ 提令智能体)" : "内存模式 - 输入消息..."}
              disabled={!isInitialized || isLoading}
            />
            {/* @ 提令建议下拉框 */}
            {showMentionSuggestions && mentionSuggestions.length > 0 && (
              <div className="mention-suggestions">
                {mentionSuggestions.map((agent, index) => (
                  <div
                    key={agent.id}
                    className={`mention-suggestion-item ${index === selectedSuggestionIndex ? 'selected' : ''}`}
                    onClick={() => handleSelectMention(agent)}
                  >
                    <span className="mention-agent-name">@{agent.name}</span>
                  </div>
                ))}
              </div>
            )}
            <button
              type="button"
              className="send-button"
              onClick={handleSendMessage}
              disabled={!messageInput.trim() || !isInitialized || isLoading}
              title="发送消息 (Enter)"
            >
              <img src={SendIcon} alt="发送" />
            </button>
          </div>
        </div>
      )}

      {/* 创建群聊模态框 */}
      {showCreateModal && (
        <div className="modal-overlay" onClick={() => setShowCreateModal(false)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <h3>创建群聊</h3>
            <input
              type="text"
              className="modal-input"
              value={newGroupName}
              onChange={(e) => setNewGroupName(e.target.value)}
              placeholder="请输入群聊名称"
              autoFocus
            />
            <div className="modal-actions">
              <button
                type="button"
                className="modal-btn cancel"
                onClick={() => setShowCreateModal(false)}
              >
                取消
              </button>
              <button
                type="button"
                className="modal-btn confirm"
                onClick={handleCreateGroup}
              >
                创建
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 加入群聊模态框 */}
      {showJoinModal && (
        <div className="modal-overlay" onClick={() => setShowJoinModal(false)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <h3>加入群聊</h3>
            <input
              type="text"
              className="modal-input"
              value={joinGroupId}
              onChange={(e) => setJoinGroupId(e.target.value)}
              placeholder="请输入群聊 ID"
              autoFocus
            />
            <div className="modal-actions">
              <button
                type="button"
                className="modal-btn cancel"
                onClick={() => setShowJoinModal(false)}
              >
                取消
              </button>
              <button
                type="button"
                className="modal-btn confirm"
                onClick={handleJoinGroup}
              >
                加入
              </button>
            </div>
          </div>
        </div>
      )}
    </section>
  )
}

export default GroupChatPanel

