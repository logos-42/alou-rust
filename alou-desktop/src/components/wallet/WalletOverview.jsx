import React, { useMemo } from 'react'
import { useI18n } from '@/hooks/useI18n'
import './WalletOverview.css'

const formatAddress = (address) => {
  if (!address || address.length <= 10) return address
  return `${address.slice(0, 6)}...${address.slice(-4)}`
}

const calculateUSD = (ethAmount, price) => {
  const amount = parseFloat(ethAmount || '0')
  return (amount * price).toFixed(2)
}

const formatStableUsd = (value) => {
  if (value === undefined || value === null) {
    return '≈ $0.00'
  }
  const numeric = Number(value)
  if (Number.isFinite(numeric)) {
    return `≈ $${numeric.toFixed(2)}`
  }
  const coerced = typeof value === 'string' ? value : String(value)
  return `≈ $${coerced}`
}

const WalletOverview = ({ wallet, currentNetwork, networkName, ethPrice, supportedTokens = [], onSwitchWallet, onDisconnect }) => {
  const { t } = useI18n()

  const formattedEthUSD = useMemo(
    () => calculateUSD(wallet?.ethBalance, ethPrice),
    [ethPrice, wallet?.ethBalance],
  )

  const tokenCards = useMemo(() => {
    if (!Array.isArray(supportedTokens) || supportedTokens.length === 0) {
      return []
    }

    const balances = wallet?.tokenBalances || {}

    return supportedTokens.map((token) => {
      const balanceInfo = balances[token.symbol] || {}
      const displayBalance =
        balanceInfo.normalizedBalance ??
        balanceInfo.balance ??
        '0'

      return {
        symbol: token.symbol,
        name: token.name,
        balance: displayBalance,
        rawBalance: balanceInfo.rawBalance ?? '0',
        decimals: balanceInfo.decimals ?? token.decimals,
        usdDisplay: formatStableUsd(
          balanceInfo.normalizedBalance ?? displayBalance,
        ),
      }
    })
  }, [supportedTokens, wallet?.tokenBalances])

  return (
    <div className="wallet-overview-card">
      <div className="wallet-header">
        <div className="wallet-avatar">
          <div className="avatar-icon">👤</div>
        </div>
        <div className="wallet-details">
          <div className="wallet-address-full">{formatAddress(wallet?.address)}</div>
          <div className="wallet-network">
            <span className={`network-dot ${currentNetwork}`} />
            {networkName}
          </div>
        </div>
        <div className="wallet-actions">
          {typeof onSwitchWallet === 'function' && (
            <button type="button" onClick={onSwitchWallet} className="switch-btn">
              {t('switchWallet')}
            </button>
          )}
          <button type="button" onClick={onDisconnect} className="disconnect-btn">
            {t('disconnect')}
          </button>
        </div>
      </div>

      <div className="wallet-balances">
        <div className="balance-card">
          <div className="balance-label">ETH {t('balance')}</div>
          <div className="balance-amount">{wallet?.ethBalance || '0.0'}</div>
          <div className="balance-usd">≈ ${formattedEthUSD}</div>
        </div>
        {tokenCards.map((token) => (
          <div className="balance-card" key={token.symbol}>
            <div className="balance-label">
              {token.symbol} {t('balance')}
            </div>
            <div className="balance-amount">{token.balance}</div>
            <div className="balance-usd">{token.usdDisplay}</div>
          </div>
        ))}
      </div>
    </div>
  )
}

export default WalletOverview
