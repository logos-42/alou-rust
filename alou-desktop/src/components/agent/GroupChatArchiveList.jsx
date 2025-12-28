import React, { useState, useEffect } from 'react'
import { useI18n } from '@/hooks/useI18n'
import CollapseIcon from '@/assets/收缩4.png'
import './GroupChatArchiveList.css'

/**
 * GroupChatArchiveList - 群聊存档列表组件
 * 显示当前频道的所有群聊记录，支持切换查看不同的群聊
 * 支持折叠/展开功能
 */
const GroupChatArchiveList = ({
  groupChatList = [],
  activeActionId,
  activeChannelId,
  onSwitchGroupChat,
}) => {
  const { t } = useI18n()

  // 折叠状态（按频道存储，默认展开）
  const [isCollapsed, setIsCollapsed] = useState(() => {
    if (typeof window !== 'undefined' && activeChannelId) {
      const saved = localStorage.getItem(`group-chat-archive-collapsed-${activeChannelId}`)
      return saved === 'true'
    }
    return false // 默认展开
  })

  // 当频道切换时，加载对应频道的折叠状态
  useEffect(() => {
    if (activeChannelId) {
      const saved = localStorage.getItem(`group-chat-archive-collapsed-${activeChannelId}`)
      setIsCollapsed(saved === 'true')
    } else {
      setIsCollapsed(false)
    }
  }, [activeChannelId])

  // 保存折叠状态到 localStorage
  useEffect(() => {
    if (typeof window !== 'undefined' && activeChannelId) {
      localStorage.setItem(`group-chat-archive-collapsed-${activeChannelId}`, isCollapsed.toString())
    }
  }, [isCollapsed, activeChannelId])

  // 如果没有群聊，不显示列表
  if (!groupChatList || groupChatList.length === 0) {
    return null
  }

  const toggleCollapse = () => {
    setIsCollapsed(!isCollapsed)
  }

  // 获取群聊显示名称
  const getGroupChatName = (action) => {
    return action.description || 
           action.metadata?.channel_name || 
           `${t('agent.groupChat.action')} #${action.action_id?.slice(-8) || 'N/A'}`
  }

  return (
    <div className={`group-chat-archive-list ${isCollapsed ? 'collapsed' : ''}`}>
      <div className="archive-list-header">
        {!isCollapsed && (
          <>
            <div className="archive-list-title">{t('agent.groupChat.archive')}</div>
            <div className="archive-list-count">({groupChatList.length})</div>
          </>
        )}
        <button
          type="button"
          className="archive-collapse-btn"
          onClick={toggleCollapse}
          title={isCollapsed ? t('agent.groupChat.expand') : t('agent.groupChat.collapse')}
        >
          <img src={CollapseIcon} alt={isCollapsed ? t('agent.groupChat.expand') : t('agent.groupChat.collapse')} />
        </button>
      </div>
      {!isCollapsed && (
        <div className="archive-list-content">
          {groupChatList.map((action) => {
            const isActive = action.action_id === activeActionId
            return (
              <button
                key={action.action_id}
                type="button"
                className={`archive-list-item ${isActive ? 'active' : ''}`}
                onClick={() => {
                  if (onSwitchGroupChat && action.action_id !== activeActionId) {
                    onSwitchGroupChat(action.action_id)
                  }
                }}
                title={getGroupChatName(action)}
              >
                <div className="archive-item-content">
                  <div className="archive-item-name">{getGroupChatName(action)}</div>
                  {action.metadata?.channel_name && action.description && (
                    <div className="archive-item-subtitle">{action.metadata.channel_name}</div>
                  )}
                </div>
              </button>
            )
          })}
        </div>
      )}
    </div>
  )
}

export default GroupChatArchiveList

