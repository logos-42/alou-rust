import React from 'react'
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
  onKeywordChange,
  onSelectChannel,
  onCreateChannel,
}) => {
  return (
    <aside className="sidebar-left">
      <div className="sidebar-header">
        <div className="brand">alou</div>
        <button type="button" className="new-channel-btn" onClick={onCreateChannel}>
          ＋
        </button>
      </div>

      <div className="channel-search">
        <input
          type="text"
          value={keyword}
          placeholder="搜索智能体"
          onChange={(event) => onKeywordChange?.(event.target.value)}
        />
      </div>

      <div className="channel-list">
        {channels.map((channel) => (
          <div
            key={channel.id}
            className={`channel-item${channel.id === activeChannelId ? ' active' : ''}`}
            onClick={() => onSelectChannel?.(channel)}
            role="button"
            tabIndex={0}
            onKeyDown={(event) => {
              if (event.key === 'Enter' || event.key === ' ') {
                onSelectChannel?.(channel)
              }
            }}
          >
            <div className="channel-icon" style={{ background: channel.color }}>
              {channel.icon}
            </div>
            <div className="channel-info">
              <div className="channel-name">{channel.name}</div>
              <div className="channel-meta">
                <span className={`status-dot ${channel.status}`} />
                <span className="channel-status">{channel.statusLabel}</span>
              </div>
            </div>
            <div className="channel-date">{formatDate(channel.updatedAt)}</div>
          </div>
        ))}
      </div>
    </aside>
  )
}

export default AgentSidebarLeft

