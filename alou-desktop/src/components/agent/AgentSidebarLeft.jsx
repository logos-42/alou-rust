import React from 'react'
import CollapseIcon from '@/assets/侧边栏收缩.png'
import SearchIcon from '@/assets/搜索.png'
import CreateIcon from '@/assets/创建.png'
import './AgentSidebarLeft.css'

const formatDate = (timestamp) =>
  new Date(timestamp).toLocaleDateString('zh-CN', {
    month: '2-digit',
    day: '2-digit',
  })

const AgentSidebarLeft = ({
  channels = [],
  activeChannelId,
  keyword,
  isLoading = false,
  errorMessage,
  onKeywordChange,
  onSelectChannel,
  onCreateChannel,
  onRefresh,
  isCollapsed = false,
  onToggleCollapse,
}) => {
  const sidebarClassName = `sidebar-left${isCollapsed ? ' collapsed' : ''}`

  return (
    <aside className={sidebarClassName}>
      <div className="sidebar-header">
        <button
          type="button"
          className="brand-toggle"
          onClick={() => onToggleCollapse?.()}
          aria-label={isCollapsed ? '展开智能体列表' : '折叠智能体列表'}
        >
          <img src={CollapseIcon} alt="Alou" className="brand-icon" />
        </button>
        {!isCollapsed && (
          <button type="button" className="new-channel-btn" onClick={onCreateChannel}>
            <img src={CreateIcon} alt="创建频道" />
          </button>
        )}
      </div>

      {!isCollapsed && (
        <div className="channel-search">
          <input
            type="text"
            value={keyword}
            placeholder="搜索智能体"
            onChange={(event) => onKeywordChange?.(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === 'Enter') {
                onRefresh?.()
              }
            }}
          />
          <img src={SearchIcon} alt="搜索" className="search-icon" />
        </div>
      )}

      <div className="channel-list">
        {isLoading && <div className="channel-placeholder">正在加载智能体...</div>}
        {!isLoading && errorMessage && (
          <div className="channel-placeholder channel-error">
            <div>{errorMessage}</div>
            {onRefresh && (
              <button type="button" onClick={onRefresh}>
                重试
              </button>
            )}
          </div>
        )}
        {!isLoading && !errorMessage && channels.length === 0 && (
          <div className="channel-placeholder">
            暂无智能体。请在上方输入 IPNS / CID / DID 搜索。
          </div>
        )}
        {channels.map((channel) => (
          <div
            key={channel.id}
            className={`channel-item${channel.id === activeChannelId ? ' active' : ''}`}
            onClick={() => onSelectChannel?.(channel)}
            role="button"
            title={channel.name}
            tabIndex={0}
            onKeyDown={(event) => {
              if (event.key === 'Enter' || event.key === ' ') {
                onSelectChannel?.(channel)
              }
            }}
          >
            <div className="channel-icon" style={{ background: channel.color }}>
              {channel.avatar ? (
                <img src={channel.avatar} alt={channel.name} />
              ) : (
                channel.icon
              )}
              <span className={`status-indicator ${channel.status}`} />
            </div>
            {!isCollapsed && (
              <>
                <div className="channel-info">
                  <div className="channel-name">{channel.name}</div>
                  <div className="channel-meta">
                    <span className={`status-dot ${channel.status}`} />
                    {channel.meta?.agent_type === 'claude_agent_sdk' && (
                      <span className="channel-badge badge-claude">Claude</span>
                    )}
                    {channel.meta?.ipns && (
                      <span
                        className={`channel-badge status-badge ${
                          channel.meta?.diap_identity?.is_registered ? 'badge-success' : 'badge-warning'
                        }`}
                        aria-label={
                          channel.meta?.diap_identity?.is_registered ? 'DIAP 已注册' : 'DIAP 未注册'
                        }
                      >
                        <span className="status-icon">
                          {channel.meta?.diap_identity?.is_registered ? (
                            <svg viewBox="0 0 16 16">
                              <path
                                d="M3 8.5l3 3 7-7.5"
                                fill="none"
                                stroke="currentColor"
                                strokeWidth="2"
                                strokeLinecap="round"
                                strokeLinejoin="round"
                              />
                            </svg>
                          ) : (
                            <svg viewBox="0 0 16 16">
                              <path
                                d="M4 4l8 8M12 4l-8 8"
                                fill="none"
                                stroke="currentColor"
                                strokeWidth="2"
                                strokeLinecap="round"
                              />
                            </svg>
                          )}
                        </span>
                      </span>
                    )}
                  </div>
                </div>
                <div className="channel-date">{formatDate(channel.updatedAt)}</div>
              </>
            )}
          </div>
        ))}
      </div>
    </aside>
  )
}

export default AgentSidebarLeft
