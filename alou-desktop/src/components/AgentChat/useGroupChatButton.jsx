import { useMemo } from 'react'
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

  // 包装对话面板的包装器组件
  const ConversationPanelWrapper = useMemo(
    () =>
      ({ children }) => {
        return (
          <div className="conversation-panel-wrapper">
            {children}
            {showConversationPanel && (
              <button type="button" {...buttonConfig}>
                <img src={GroupIcon} alt={showGroupChat ? t('agent.groupChat.close') : t('agent.groupChat.open')} />
              </button>
            )}
          </div>
        )
      },
    [showConversationPanel, buttonConfig, showGroupChat, t],
  )

  return {
    buttonConfig,
    ConversationPanelWrapper,
  }
}

export default useGroupChatButton

