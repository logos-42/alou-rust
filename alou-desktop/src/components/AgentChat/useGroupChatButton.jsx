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
  canOpenGroupChat,
  openGroupChat,
  closeGroupChat,
}) => {
  const { t } = useI18n()
  
  // 按钮配置
  const buttonConfig = useMemo(
    () => ({
      className: `group-chat-toggle-btn-fixed ${showGroupChat ? 'active' : ''}`,
      onClick: showGroupChat ? closeGroupChat : openGroupChat,
      title: showGroupChat ? t('agent.groupChat.close') : t('agent.groupChat.open'),
      'aria-label': showGroupChat ? t('agent.groupChat.close') : t('agent.groupChat.open'),
      disabled: !canOpenGroupChat && !showGroupChat,
    }),
    [showGroupChat, canOpenGroupChat, openGroupChat, closeGroupChat, t],
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

  // 包装对话面板的包装器组件
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
              {...buttonConfig}
              onClick={(e) => {
                // 阻止事件冒泡到父容器
                e.stopPropagation();
                buttonConfig.onClick();
              }}
            >
              <img src={GroupIcon} alt={showGroupChat ? t('agent.groupChat.close') : t('agent.groupChat.open')} />
            </button>
          </div>
        )
      },
    [buttonConfig, showGroupChat, t, handleConversationPanelClick],
  )

  return {
    buttonConfig,
    ConversationPanelWrapper,
  }
}

export default useGroupChatButton

