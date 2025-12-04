import React, { useState, useRef, useEffect, useMemo } from 'react'
import CollapseIcon from '@/assets/侧边栏收缩.png'
import SearchIcon from '@/assets/搜索.png'
import CreateIcon from '@/assets/创建.png'
import './AgentSidebarLeft.css'

const formatDate = (timestamp) => {
  // 如果时间戳是秒级（小于 10000000000），转换为毫秒级
  const msTimestamp = timestamp < 10000000000 ? timestamp * 1000 : timestamp
  return new Date(msTimestamp).toLocaleDateString('zh-CN', {
    month: '2-digit',
    day: '2-digit',
  })
}

const MODEL_TYPES = [
  { id: 'claude', name: 'Claude', available: true },
  { id: 'gemini', name: 'Gemini', available: false },
  { id: 'gpt', name: 'GPT', available: false },
  { id: 'qwen', name: 'Qwen', available: false },
  { id: 'kimi', name: 'Kimi', available: false },
  { id: 'deepseek', name: 'DeepSeek', available: false },
  { id: 'alou', name: 'Alou', available: true },
  { id: 'grok', name: 'Grok', available: false },
]

const clamp = (value, min, max) => Math.min(Math.max(value, min), max)
const MIN_MENU_WIDTH = 176

const AgentSidebarLeft = ({
  channels = [],
  activeChannelId,
  keyword,
  isLoading = false,
  errorMessage,
  onKeywordChange,
  onSelectChannel,
  onCreateChannel,
  onDeleteChannel,
  onInviteToChannel,
  onRefresh,
  isCollapsed = false,
  onToggleCollapse,
  selectedModelType,
  onModelTypeChange,
  onShowIdentityPanel,
}) => {
  const sidebarClassName = `sidebar-left${isCollapsed ? ' collapsed' : ''}`
  const [modelMenuState, setModelMenuState] = useState({ visible: false, top: 0, left: 0, width: 0 })
  const menuRef = useRef(null)
  const badgeAnchorRef = useRef(null)

  const availableModels = useMemo(() => MODEL_TYPES.filter((model) => model.available), [])
  const upcomingModels = useMemo(() => MODEL_TYPES.filter((model) => !model.available), [])
  const selectedModelLabel = useMemo(() => {
    if (!selectedModelType) {
      return ''
    }
    const matched = MODEL_TYPES.find((item) => item.id === selectedModelType)
    return matched?.name || ''
  }, [selectedModelType])

  const closeModelMenu = () => {
    setModelMenuState({ visible: false, top: 0, left: 0, width: 0 })
    badgeAnchorRef.current = null
  }

  // 点击外部关闭菜单
  useEffect(() => {
    if (!modelMenuState.visible) {
      return undefined
    }

    const handleClickOutside = (event) => {
      const isMenu = menuRef.current?.contains(event.target)
      const isBadge = badgeAnchorRef.current?.contains(event.target)
      if (!isMenu && !isBadge) {
        closeModelMenu()
      }
    }

    const handleViewportChange = () => {
      closeModelMenu()
    }

    document.addEventListener('mousedown', handleClickOutside)
    window.addEventListener('scroll', handleViewportChange, true)
    window.addEventListener('resize', handleViewportChange)

    return () => {
      document.removeEventListener('mousedown', handleClickOutside)
      window.removeEventListener('scroll', handleViewportChange, true)
      window.removeEventListener('resize', handleViewportChange)
    }
  }, [modelMenuState.visible])

  useEffect(() => {
    if (isCollapsed) {
      closeModelMenu()
    }
  }, [isCollapsed])

  const openModelMenu = (event) => {
    if (isCollapsed) {
      return
    }
    const rect = event.currentTarget.getBoundingClientRect()
    const scrollY = typeof window !== 'undefined' ? window.scrollY : 0
    const scrollX = typeof window !== 'undefined' ? window.scrollX : 0
    const viewportWidth = typeof window !== 'undefined' ? window.innerWidth : rect.width + 40
    const desiredWidth = Math.max(MIN_MENU_WIDTH, rect.width + 96)
    const leftOffset = rect.left + scrollX - (desiredWidth - rect.width) / 2
    const clampedLeft = clamp(leftOffset, 12, viewportWidth - desiredWidth - 12)
    const topOffset = Math.max(rect.top + scrollY - 4, 12)
    badgeAnchorRef.current = event.currentTarget
    setModelMenuState({
      visible: true,
      top: topOffset,
      left: clampedLeft,
      width: desiredWidth,
    })
  }

  const handleModelClick = (model) => {
    if (!model.available) {
      return
    }
    const nextValue = selectedModelType === model.id ? null : model.id
    onModelTypeChange?.(nextValue)
    closeModelMenu()
  }

  return (
    <aside className={sidebarClassName}>
      <div className="sidebar-header">
        <button
          type="button"
          className="brand-toggle"
          onClick={() => onToggleCollapse?.()}
          aria-label={isCollapsed ? '展开智能体列表' : '折叠智能体列表'}
        >
          <img src={CollapseIcon} alt="折叠" className="brand-icon" />
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

      {!isCollapsed && selectedModelLabel && (
        <div className="model-filter-chip">
          <span>已筛选：{selectedModelLabel}</span>
          <button
            type="button"
            onClick={() => {
              onModelTypeChange?.(null)
              closeModelMenu()
              onShowIdentityPanel?.()
            }}
            aria-label="清除模型筛选"
          >
            ×
          </button>
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
                      <span
                        className={`channel-badge badge-claude ${selectedModelType === 'claude' ? 'selected' : ''}`}
                        onClick={(e) => {
                          e.stopPropagation()
                          openModelMenu(e)
                        }}
                        style={{ cursor: 'pointer' }}
                        title="点击筛选模型类型"
                      >
                        Agent
                      </span>
                    )}
                    {channel.meta?.ipns && (
                      <span
                        className={`channel-badge status-badge ${
                          channel.meta?.diap_identity?.is_registered ? 'badge-success' : 'badge-warning'
                        }`}
                        aria-label={
                          channel.meta?.diap_identity?.is_registered ? 'DIAP 已注册' : 'DIAP 未注册'
                        }
                        role="button"
                        tabIndex={0}
                        onClick={(event) => {
                          event.stopPropagation()
                          onShowIdentityPanel?.()
                        }}
                        onKeyDown={(event) => {
                          if (event.key === 'Enter' || event.key === ' ') {
                            event.preventDefault()
                            onShowIdentityPanel?.()
                          }
                        }}
                        style={{ cursor: 'pointer' }}
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
                <div className="channel-actions">
                  <button
                    type="button"
                    className="channel-action-btn invite-btn"
                    onClick={(e) => {
                      e.stopPropagation()
                      onInviteToChannel?.(channel)
                    }}
                    title="邀请其他智能体"
                    aria-label="邀请其他智能体加入群组"
                  >
                    <svg viewBox="0 0 20 20" fill="none">
                      <path
                        d="M10 4v12M4 10h12"
                        stroke="currentColor"
                        strokeWidth="2"
                        strokeLinecap="round"
                      />
                      <circle
                        cx="10"
                        cy="10"
                        r="8"
                        stroke="currentColor"
                        strokeWidth="1.5"
                        opacity="0.5"
                      />
                    </svg>
                  </button>
                  <button
                    type="button"
                    className="channel-action-btn delete-btn"
                    onClick={(e) => {
                      e.stopPropagation()
                      if (window.confirm(`确定要删除智能体 "${channel.name}" 吗？`)) {
                        onDeleteChannel?.(channel)
                      }
                    }}
                    title="删除智能体"
                    aria-label="删除智能体"
                  >
                    <svg viewBox="0 0 20 20" fill="none">
                      <path
                        d="M6 6l8 8M14 6l-8 8"
                        stroke="currentColor"
                        strokeWidth="2"
                        strokeLinecap="round"
                      />
                    </svg>
                  </button>
                </div>
              </>
            )}
          </div>
        ))}
      </div>

      {modelMenuState.visible && (
        <div
          ref={menuRef}
          className="model-filter-menu floating"
          style={{
            top: modelMenuState.top,
            left: modelMenuState.left,
            width: Math.max(modelMenuState.width, MIN_MENU_WIDTH),
          }}
          role="dialog"
          aria-label="模型筛选"
        >
          <div className="model-filter-header">
            <span>筛选模型</span>
            <button
              type="button"
              className="reset-btn"
              onClick={() => {
                onModelTypeChange?.(null)
                closeModelMenu()
              }}
              disabled={!selectedModelType}
            >
              重置
            </button>
          </div>

          <div className="model-filter-section">
            <div className="section-label">已接入</div>
            {availableModels.map((model) => (
              <div
                key={model.id}
                className={`model-filter-item ${selectedModelType === model.id ? 'active' : ''}`}
                onClick={() => handleModelClick(model)}
                role="button"
                tabIndex={0}
              >
                <span className="model-name">{model.name}</span>
              </div>
            ))}
          </div>

          <div className="model-filter-section">
            <div className="section-label">即将上线</div>
            {upcomingModels.map((model) => (
              <div key={model.id} className="model-filter-item coming-soon">
                <span className="model-name">{model.name}</span>
                <span className="coming-soon-label">后续接入</span>
              </div>
            ))}
          </div>
        </div>
      )}
    </aside>
  )
}

export default AgentSidebarLeft
