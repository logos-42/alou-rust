import { useMemo, useCallback } from 'react'
import React from 'react'
import { useI18n } from '@/hooks/useI18n'
import GroupIcon from '@/assets/群组.png'

/**
 * useGroupChatButton - 群聊按钮 Hook
 * 返回群聊切换按钮的渲染配置和包装器组件
 */
export const useGroupChatButton = ({
  showConversationPanel,
  showGroupChat,
  openGroupChat,
  closeGroupChat,
}) => {
  const { t } = useI18n()

  // 使用 useCallback 确保事件处理函数的稳定性
  const handleToggleGroupChat = useCallback(() => {
    console.log('[useGroupChatButton] 切换群聊状态:', { showGroupChat })
    if (showGroupChat) {
      closeGroupChat()
    } else {
      openGroupChat()
    }
  }, [showGroupChat, openGroupChat, closeGroupChat])

  // 按钮配置 - 使用稳定的函数引用
  const buttonConfig = useMemo(
    () => ({
      className: `group-chat-toggle-btn-fixed ${showGroupChat ? 'active' : ''}`,
      onClick: handleToggleGroupChat,
      title: showGroupChat ? t('agent.groupChat.close') : t('agent.groupChat.open'),
      'aria-label': showGroupChat ? t('agent.groupChat.close') : t('agent.groupChat.open'),
    }),
    [showGroupChat, handleToggleGroupChat, t],
  )

  // 处理对话面板点击，切换输入目标到智能体
  const handleConversationPanelClick = useCallback((e) => {
    console.log('[ConversationPanelWrapper] 对话面板点击事件触发:', e.target)
    
    // 检查是否点击了群聊按钮
    const isGroupChatButton = e.target.closest('.group-chat-toggle-btn-fixed')
    if (isGroupChatButton) {
      console.log('[ConversationPanelWrapper] 点击了群聊按钮，不阻止事件')
      // 不阻止群聊按钮的点击事件，让它正常处理
      return
    }
    
    // 避免点击其他按钮时触发
    if (e.target.tagName === 'BUTTON' || e.target.closest('button')) {
      console.log('[ConversationPanelWrapper] 点击了其他按钮，忽略面板点击')
      return
    }
    
    // 只有在群聊面板打开时才处理点击
    if (showGroupChat) {
      // 添加点击动画效果
      const panel = e.currentTarget
      panel.classList.add('clicked')
      setTimeout(() => {
        panel.classList.remove('clicked')
      }, 300)
      
      console.log('[ConversationPanelWrapper] 触发切换到智能体模式')
      
      // 触发切换到智能体模式
      window.dispatchEvent(new CustomEvent('switch-input-target', {
        detail: { target: 'agent' }
      }))
    } else {
      console.log('[ConversationPanelWrapper] 群聊面板未打开，忽略点击')
    }
  }, [showGroupChat])

  // 使用 useCallback 确保按钮点击处理器的稳定性
  const handleButtonClick = useCallback((e) => {
    console.log('[useGroupChatButton] 群聊按钮点击')
    // 阻止事件冒泡到父容器
    e.stopPropagation()
    // 阻止默认行为
    e.preventDefault()
    // 调用切换函数
    handleToggleGroupChat()
  }, [handleToggleGroupChat])
  
  // 包装对话面板的包装器组件 - 使用稳定的函数引用
  const ConversationPanelWrapper = useMemo(
    () =>
      ({ children }) => {
        return (
          <div
            className={`conversation-panel-wrapper ${showGroupChat ? 'group-chat-active' : ''}`}
            onClick={handleConversationPanelClick}
          >
            {children}
            {/* 群聊按钮独立于对话面板显示 */}
            <button 
              type="button" 
              className={buttonConfig.className}
              onClick={handleButtonClick}
              title={buttonConfig.title}
              aria-label={buttonConfig['aria-label']}
              disabled={buttonConfig.disabled}
            >
              <img src={GroupIcon} alt={showGroupChat ? t('agent.groupChat.close') : t('agent.groupChat.open')} />
            </button>
          </div>
        )
      },
    [buttonConfig, showGroupChat, t, handleConversationPanelClick, handleButtonClick],
  )

  return {
    buttonConfig,
    ConversationPanelWrapper,
  }
}

export default useGroupChatButton

