import { useMemo } from 'react'
import React from 'react'

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
  // 按钮配置
  const buttonConfig = useMemo(
    () => ({
      className: `group-chat-toggle-btn-fixed ${showGroupChat ? 'active' : ''}`,
      onClick: showGroupChat ? closeGroupChat : openGroupChat,
      title: showGroupChat ? '关闭群聊' : '打开群聊',
      'aria-label': showGroupChat ? '关闭群聊' : '打开群聊',
      disabled: !canOpenGroupChat && !showGroupChat,
    }),
    [showGroupChat, canOpenGroupChat, openGroupChat, closeGroupChat],
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
                👥
              </button>
            )}
          </div>
        )
      },
    [showConversationPanel, buttonConfig],
  )

  return {
    buttonConfig,
    ConversationPanelWrapper,
  }
}

export default useGroupChatButton

