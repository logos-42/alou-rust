import React, { forwardRef, useImperativeHandle, useRef, useEffect } from 'react'
import MessageList from '@/components/MessageList'
import AgentStreamPanel from '@/components/agent/AgentStreamPanel'
import CloseIcon from '@/assets/关闭0.3.png'
import EditIcon from '@/assets/修改.png'
import './AgentConversationOverlay.css'

const DEFAULT_AVATAR = 'https://avatars.githubusercontent.com/u/16309930?v=4'

const AgentConversationOverlay = forwardRef(
  (
    {
      style,
      connectionStatus,
      connectionStatusLabel,
      messages,
      isLoading,
      onClose,
      onInspectMessage,
      onEdit,
      streamEvents = [],
      streamStatus = 'idle',
      embedded = false,
      title = '会话',
      subtitle,
      actions,
      emptyState,
      avatar,
      backgroundImage,
    },
    ref,
  ) => {
    const messageListRef = useRef(null)
    const conversationBodyRef = useRef(null)

    useImperativeHandle(
      ref,
      () => ({
        scrollToBottom: () => {
          // 优先滚动 conversation-body 容器
          const conversationBody = conversationBodyRef.current
          if (conversationBody) {
            conversationBody.scrollTop = conversationBody.scrollHeight
            return
          }
          
          // 备用方案：滚动 messageList 容器
          messageListRef.current?.scrollToBottom?.()
        },
      }),
      [],
    )

    // 主要的自动滚动逻辑 - 直接控制 conversation-body
    useEffect(() => {
      const conversationBody = conversationBodyRef.current
      if (!conversationBody) return

      const scrollToBottom = () => {
        conversationBody.scrollTop = conversationBody.scrollHeight
      }

      // 立即滚动
      scrollToBottom()
      
      // 延迟滚动，确保内容渲染完成
      setTimeout(scrollToBottom, 50)
      
      // 更长延迟，确保动态内容加载完成
      setTimeout(scrollToBottom, 150)
      
      // 最长延迟，确保所有异步内容加载完成
      setTimeout(scrollToBottom, 300)
    }, [messages, isLoading])

    // 专门处理新消息的滚动
    useEffect(() => {
      const conversationBody = conversationBodyRef.current
      if (!conversationBody || messages.length === 0) return

      const lastMessage = messages[messages.length - 1]
      if (lastMessage && (lastMessage.type === 'assistant' || lastMessage.type === 'user')) {
        // 对于新的用户或助手消息，强制滚动到底部
        const forceScrollToBottom = () => {
          conversationBody.scrollTop = conversationBody.scrollHeight
        }
        
        // 立即执行
        forceScrollToBottom()
        
        // 延迟执行，确保内容完全渲染
        setTimeout(forceScrollToBottom, 100)
        setTimeout(forceScrollToBottom, 250)
        setTimeout(forceScrollToBottom, 400)
      }
    }, [messages.length]) // 只监听消息数量变化

    const avatarSrc = avatar || DEFAULT_AVATAR
    const avatarAlt = typeof title === 'string' ? title : '智能体'

    const header = (
      <header>
        <div className="title">
          <div className="avatar-container">
            <span className="agent-avatar">
              <img src={avatarSrc} alt={avatarAlt} />
            </span>
            {onEdit && (
              <button 
                type="button" 
                className="edit-btn" 
                onClick={onEdit} 
                title="编辑智能体"
              >
                <img src={EditIcon} alt="编辑" />
              </button>
            )}
          </div>
          <div className="title-text">
            <span>{title}</span>
            {subtitle && <small>{subtitle}</small>}
          </div>
        </div>
        <div className="header-actions">
          {actions}
          <button type="button" className="close-btn" onClick={onClose} title="关闭对话">
            <img src={CloseIcon} alt="关闭" />
          </button>
        </div>
      </header>
    )

    const body = (
      <div className="conversation-body" ref={conversationBodyRef}>
        {messages.length === 0 && !isLoading && emptyState ? (
          emptyState
        ) : (
          <MessageList
            ref={messageListRef}
            messages={messages}
            isLoading={isLoading}
            loadingContent={
              <div className="stream-panel-wrapper">
                <AgentStreamPanel events={streamEvents} status={streamStatus} />
              </div>
            }
            onMessageSelect={onInspectMessage}
          />
        )}
      </div>
    )

    const panelClassName = `conversation-panel${embedded ? ' embedded' : ''}${backgroundImage ? ' has-background' : ''}`
    
    const panelStyle = backgroundImage
      ? {
          ...style,
          backgroundImage: `url(${backgroundImage})`,
          backgroundSize: 'cover',
          backgroundPosition: 'center',
          backgroundRepeat: 'no-repeat',
        }
      : style

    if (embedded) {
      return (
        <section className={panelClassName} style={panelStyle}>
          {header}
          {body}
        </section>
      )
    }

    return (
      <div className="conversation-overlay" style={style}>
        <section className={panelClassName} style={panelStyle}>
          {header}
          {body}
        </section>
      </div>
    )
  },
)

AgentConversationOverlay.displayName = 'AgentConversationOverlay'

export default AgentConversationOverlay
