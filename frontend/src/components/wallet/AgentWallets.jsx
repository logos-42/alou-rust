import React from 'react'
import './AgentWallets.css'

const formatAddress = (address) => {
  if (!address) return ''
  return `${address.slice(0, 6)}...${address.slice(-4)}`
}

const getChainSymbol = (chain) => {
  const symbols = {
    ethereum: 'ETH',
    base: 'ETH',
    polygon: 'MATIC',
  }
  return symbols[chain] || 'ETH'
}

const formatDate = (timestamp) => {
  if (!timestamp) return ''
  const date = new Date(timestamp * 1000)
  return date.toLocaleDateString('zh-CN', { month: 'short', day: 'numeric' })
}

const AgentWallets = ({ wallets = [] }) => {
  if (!wallets.length) {
    return null
  }

  return (
    <div className="agent-wallets-section">
      <h3 className="section-title">🤖 智能体钱包</h3>
      <div className="agent-wallets-grid">
        {wallets.map((wallet) => (
          <div key={wallet.address} className="agent-wallet-card">
            <div className="wallet-header">
              <span className="wallet-chain">{wallet.chain}</span>
              <span className="wallet-badge">AI</span>
            </div>
            <div className="wallet-address">{formatAddress(wallet.address)}</div>
            <div className="wallet-balance">
              <span className="balance-label">余额:</span>
              <span className="balance-value">
                {wallet.balance || '0'} {getChainSymbol(wallet.chain)}
              </span>
            </div>
            <div className="wallet-info">
              <span className="info-item">
                交易: {wallet.transactions?.length || 0}
              </span>
              <span className="info-item">创建: {formatDate(wallet.created_at)}</span>
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}

export default AgentWallets

