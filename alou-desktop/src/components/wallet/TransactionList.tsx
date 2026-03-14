import React from 'react'
import { useI18n } from '@/hooks/useI18n'
import './TransactionList.css'

export interface Transaction {
  hash: string
  type: 'send' | 'receive' | 'contract'
  to?: string
  from?: string
  value: string
  token: string
  timestamp: number
  status: 'confirmed' | 'pending' | 'failed'
}

export interface TransactionListProps {
  transactions?: Transaction[]
  isRefreshing?: boolean
  onRefresh?: () => void
  onViewTransaction?: (tx: Transaction) => void
}

const formatAddress = (address?: string): string => {
  if (!address || address.length <= 10) return address || ''
  return `${address.slice(0, 6)}...${address.slice(-4)}`
}

const TransactionList: React.FC<TransactionListProps> = ({
  transactions = [],
  isRefreshing,
  onRefresh,
  onViewTransaction,
}) => {
  const { t } = useI18n()

  const formatTime = (timestamp: number): string => {
    const now = Date.now()
    const diff = now - timestamp
    const minutes = Math.floor(diff / 60000)
    const hours = Math.floor(diff / 3600000)
    const days = Math.floor(diff / 86400000)

    if (minutes < 1) return t('justNow')
    if (minutes < 60) return `${minutes}${t('minutesAgo')}`
    if (hours < 24) return `${hours}${t('hoursAgo')}`
    return `${days}${t('daysAgo')}`
  }

  return (
    <div className="transactions-section">
      <div className="section-header">
        <h3>{t('recentTransactions')}</h3>
        <button type="button" onClick={onRefresh} className="refresh-btn">
          <span className={isRefreshing ? 'spinning' : ''}>🔄</span>
        </button>
      </div>

      {transactions.length === 0 ? (
        <div className="empty-state">
          <div className="empty-icon">📭</div>
          <p>{t('noTransactions')}</p>
        </div>
      ) : (
        <div className="transaction-list">
          {transactions.map((tx) => (
            <div
              key={tx.hash}
              className="transaction-item"
              onClick={() => onViewTransaction?.(tx)}
            >
              <div className={`tx-icon ${tx.type}`}>
                {tx.type === 'send' ? '📤' : tx.type === 'receive' ? '📥' : '🔄'}
              </div>
              <div className="tx-details">
                <div className="tx-title">
                  {tx.type === 'send'
                    ? t('sent')
                    : tx.type === 'receive'
                      ? t('received')
                      : 'Contract'}
                </div>
                <div className="tx-address">{formatAddress(tx.to || tx.from)}</div>
              </div>
              <div className="tx-amount">
                <div className={`amount-value ${tx.type}`}>
                  {tx.type === 'send' ? '-' : '+'}
                  {tx.value} {tx.token}
                </div>
                <div className="tx-time">{formatTime(tx.timestamp)}</div>
              </div>
              <div className={`tx-status ${tx.status}`}>
                {tx.status === 'confirmed' ? '✓' : tx.status === 'pending' ? '⏳' : '❌'}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}

export default TransactionList
