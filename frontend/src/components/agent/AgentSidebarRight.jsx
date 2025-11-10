import React, { useCallback } from 'react'
import { useI18n } from '@/hooks/useI18n'
import CollapseIcon from '@/assets/收缩.png'
import DotsIcon from '@/assets/三点.png'
import './AgentSidebarRight.css'

const formatDate = (timestamp) =>
  new Date(timestamp).toLocaleDateString('zh-CN', {
    month: '2-digit',
    day: '2-digit',
  })

const formatTime = (timestamp) =>
  new Date(timestamp).toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  })

const formatLogDetail = (log) => {
  const detail = log.detail
  if (!detail) return ''

  if (log.action === 'channel_selected') {
    const name = detail.name || '频道'
    const status = detail.statusLabel || ''
    return status ? `${name} · ${status}` : name
  }

  if (log.action === 'user_message') {
    return detail.content || ''
  }

  if (log.action === 'wallet_refresh' && detail.balance) {
    const token = detail.token || 'ETH'
    return `${detail.balance} ${token}`
  }

  try {
    return JSON.stringify(detail)
  } catch (error) {
    console.error('Failed to stringify log detail', error)
    return ''
  }
}

const AgentSidebarRight = ({
  walletSnapshot,
  transactions = [],
  interactionLogs = [],
  isInteractionCollapsed,
  isCollapsed,
  onRefreshWallet,
  onToggleInteraction,
  onToggleCollapse,
  onInspectWallet,
  onInspectTransaction,
  connectActionSlot,
}) => {
  const { t } = useI18n()

  const handleCopy = useCallback((address) => {
    if (!address || typeof navigator === 'undefined') return
    navigator.clipboard.writeText(address).catch((error) => {
      console.error('Failed to copy address:', error)
    })
  }, [])

  return (
    <aside className={`sidebar-right${isCollapsed ? ' collapsed' : ''}`}>
      <button
        type="button"
        className="collapse-toggle"
        onClick={onToggleCollapse}
        aria-label={t('interactionLogs')}
      >
        <img
          src={isCollapsed ? DotsIcon : CollapseIcon}
          alt={isCollapsed ? '展开' : '折叠'}
          className="toggle-icon"
        />
      </button>

      {!isCollapsed && (
        <div className="sidebar-content">
          <section className="wallet-card">
            <header>
              <div className="title">
                <span className="emoji">💰</span>
                <span>{t('agentAssets')}</span>
              </div>
              <button
                type="button"
                className="refresh-btn"
                onClick={(event) => {
                  event.stopPropagation()
                  onRefreshWallet?.()
                }}
              >
                {t('refresh')}
              </button>
            </header>
            {walletSnapshot ? (
              <div
                className="wallet-body"
                role="button"
                tabIndex={0}
                onClick={() => onInspectWallet?.(walletSnapshot)}
                onKeyDown={(event) => {
                  if (event.key === 'Enter' || event.key === ' ') {
                    onInspectWallet?.(walletSnapshot)
                  }
                }}
              >
                <div className="balance">
                  <div className="amount">
                    {walletSnapshot.balance} {walletSnapshot.token || 'ETH'}
                  </div>
                  <div className="fiat">≈ {walletSnapshot.balanceFiat ?? '--'} USD</div>
                </div>
                <div
                  className="address"
                  onClick={() => handleCopy(walletSnapshot.address)}
                  onKeyDown={(event) => {
                    if (event.key === 'Enter') {
                      handleCopy(walletSnapshot.address)
                    }
                  }}
                  role="button"
                  tabIndex={0}
                >
                  {walletSnapshot.address.slice(0, 6)}...
                  {walletSnapshot.address.slice(-4)}
                </div>
                <div className="network">
                  {t('currentNetwork')}：{walletSnapshot.networkLabel}
                </div>
              </div>
            ) : (
              <div className="wallet-empty">
                <p>{t('noWalletConnected')}</p>
                {typeof connectActionSlot === 'function' ? connectActionSlot() : connectActionSlot}
              </div>
            )}
          </section>

          <section className="history-card">
            <header>
              <div className="title">
                <span className="emoji">📜</span>
                <span>{t('recentTransactions')}</span>
              </div>
            </header>
            <ul>
              {transactions.map((tx) => (
                <li
                  key={tx.id}
                  className={tx.status}
                  role="button"
                  tabIndex={0}
                  onClick={() => onInspectTransaction?.(tx)}
                  onKeyDown={(event) => {
                    if (event.key === 'Enter' || event.key === ' ') {
                      onInspectTransaction?.(tx)
                    }
                  }}
                >
                  <div className="tx-main">
                    <div className="tx-amount">
                      {tx.direction === 'out' ? '-' : '+'}
                      {tx.amount} {tx.token}
                    </div>
                    <div className="tx-status">{tx.statusLabel}</div>
                  </div>
                  <div className="tx-sub">
                    <span className="tx-address">
                      {tx.counterparty.slice(0, 6)}...{tx.counterparty.slice(-4)}
                    </span>
                    <span className="tx-time">{formatDate(tx.timestamp)}</span>
                  </div>
                </li>
              ))}
            </ul>
          </section>

          <section className={`context-card${isInteractionCollapsed ? ' collapsed' : ''}`}>
            <header>
              <div className="title">
                <span className="emoji">🧠</span>
                <span>{t('interactionLogs')}</span>
              </div>
              <button type="button" className="collapse-btn" onClick={onToggleInteraction}>
                <img
                  src={isInteractionCollapsed ? DotsIcon : CollapseIcon}
                  alt={isInteractionCollapsed ? '展开' : '折叠'}
                  className="collapse-icon"
                />
              </button>
            </header>
            {!isInteractionCollapsed && (
              <div className="context-body">
                <ul>
                  {interactionLogs.map((log) => (
                    <li key={log.id}>
                      <div className="log-main">
                        <span className="log-label">{log.label}</span>
                        <span className="log-time">{formatTime(log.timestamp)}</span>
                      </div>
                      {log.detail && <div className="log-detail">{formatLogDetail(log)}</div>}
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </section>
        </div>
      )}
    </aside>
  )
}

export default AgentSidebarRight
