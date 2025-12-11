import React, { useState, useRef, useEffect, useMemo } from 'react'
import CollapseIcon from '@/assets/侧边栏收缩.png'
import SearchIcon from '@/assets/搜索.png'
import CreateIcon from '@/assets/创建.png'
import { useI18n } from '@/hooks/useI18n'
import './AgentSidebarLeft.css'

const formatDate = (timestamp) => {
  // 如果时间戳是秒级（小于 10000000000），转换为毫秒级
  const msTimestamp = timestamp < 10000000000 ? timestamp * 1000 : timestamp
  return new Date(msTimestamp).toLocaleDateString('zh-CN', {
    month: '2-digit',
    day: '2-digit',
  })
}

// 模式类型：Agent 模式（自定义智能体）和 Alou 模式（平台模式）
const MODE_TYPES = [
  { id: 'agent', name: 'Agent', available: true },
  { id: 'alou', name: 'Alou', available: true },
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
  currentMode = 'agent', // 当前模式：'agent' 或 'alou'
  onModeChange, // 切换模式的回调
  onShowIdentityPanel,
}) => {
  const { t } = useI18n()
  const sidebarClassName = `sidebar-left${isCollapsed ? ' collapsed' : ''}`
  const [modelMenuState, setModelMenuState] = useState({ visible: false, top: 0, left: 0, width: 0 })
  const menuRef = useRef(null)
  const badgeAnchorRef = useRef(null)

  const availableModes = useMemo(() => MODE_TYPES.filter((mode) => mode.available), [])
  const selectedModeLabel = useMemo(() => {
    if (!selectedModelType) {
      return ''
    }
    const matched = MODE_TYPES.find((item) => item.id === selectedModelType)
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

  const handleModelClick = (mode) => {
    if (!mode.available) {
      return
    }
    const nextValue = selectedModelType === mode.id ? null : mode.id
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
          aria-label={isCollapsed ? t('agent.sidebar.expandList') : t('agent.sidebar.collapseList')}
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
            placeholder={t('agent.list.search.placeholder')}
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

      {!isCollapsed && selectedModeLabel && (
        <div className="model-filter-chip">
          <span>{t('agent.sidebar.filtered')}{selectedModeLabel}</span>
          <button
            type="button"
            onClick={() => {
              onModelTypeChange?.(null)
              closeModelMenu()
              onShowIdentityPanel?.()
            }}
            aria-label={t('agent.sidebar.clearFilter')}
          >
            ×
          </button>
        </div>
      )}

      <div className="channel-list">
        {isLoading && <div className="channel-placeholder">{t('agent.sidebar.loading')}</div>}
        {!isLoading && errorMessage && (
          <div className="channel-placeholder channel-error">
            <div>{errorMessage}</div>
            {onRefresh && (
              <button type="button" onClick={onRefresh}>
                {t('common.retry')}
              </button>
            )}
          </div>
        )}
        {!isLoading && !errorMessage && channels.length === 0 && (
          <div className="channel-placeholder">
            {t('agent.sidebar.empty')}
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
                <img 
                  src={channel.avatar} 
                  alt={channel.name}
                  onError={(e) => {
                    // 图片加载失败时隐藏图片，显示默认图标
                    e.target.style.display = 'none'
                    const parent = e.target.parentElement
                    if (parent && !parent.querySelector('.fallback-icon')) {
                      const fallback = document.createElement('span')
                      fallback.className = 'fallback-icon'
                      fallback.textContent = channel.icon || channel.name?.charAt(0) || '🤖'
                      parent.insertBefore(fallback, e.target)
                    }
                  }}
                />
              ) : (
                channel.icon || channel.name?.charAt(0) || '🤖'
              )}
              <span className={`status-indicator ${channel.status}`} />
            </div>
            {!isCollapsed && (
              <>
                <div className="channel-info">
                  <div className="channel-name">
                    {channel.name}
                    {/* 显示创建中状态 */}
                    {(channel.meta?.status === 'creating' || channel.tempId || channel.id?.startsWith('temp_')) && (
                      <span className="creating-indicator" title="正在创建 DIAP 身份...">
                        <span className="creating-spinner" />
                      </span>
                    )}
                  </div>
                  <div className="channel-meta">
                    <span className={`status-dot ${channel.status}`} />
                    {/* 模式切换按钮：显示当前模式，点击切换 */}
                    <span
                      className={`channel-badge badge-claude ${currentMode === 'agent' ? 'badge-agent' : 'badge-alou'}`}
                      onClick={(e) => {
                        e.stopPropagation()
                        // 切换模式：agent <-> alou
                        const nextMode = currentMode === 'agent' ? 'alou' : 'agent'
                        onModeChange?.(nextMode)
                      }}
                      style={{ cursor: 'pointer' }}
                      title={`当前模式：${currentMode === 'agent' ? 'Agent' : 'Alou'}，点击切换`}
                    >
                      {currentMode === 'agent' ? 'Agent' : 'Alou'}
                    </span>
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
            <div className="section-label">选择模式</div>
            {availableModes.map((mode) => (
              <div
                key={mode.id}
                className={`model-filter-item ${selectedModelType === mode.id ? 'active' : ''}`}
                onClick={() => handleModelClick(mode)}
                role="button"
                tabIndex={0}
              >
                <span className="model-name">{mode.name}</span>
              </div>
            ))}
          </div>
        </div>
      )}
    </aside>
  )
}

export default AgentSidebarLeft
