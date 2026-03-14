import React, { useMemo } from 'react'
import { useI18n } from '@/hooks/useI18n'
import './WalletOverview.css'

export interface TokenBalance {
  symbol: string
  name: string
  balance: string
  rawBalance: string
  decimals: number
  usdDisplay: string
}

export interface Wallet {
  address: string
  ethBalance: string
  tokenBalances: Record<string, {
    normalizedBalance?: string
    balance?: string
    rawBalance?: string
    decimals?: number
  }>
}

export interface WalletOverviewProps {
  wallet?: Wallet
  currentNetwork?: string
  networkName?: string
  ethPrice?: number
  supportedTokens?: Array<{
    symbol: string
    name: string
    decimals: number
  }>
  onSwitchWallet?: () => void
  onDisconnect?: () => void
}

const formatAddress = (address?: string): string => {
  if (!address || address.length <= 10) return address || ''
  return `${address.slice(0, 6)}...${address.slice(-4)}`
}

const calculateUSD = (ethAmount: string | undefined, price: number | undefined): string => {
  const amount = parseFloat(ethAmount || '0')
  return (amount * (price || 0)).toFixed(2)
}

const formatStableUsd = (value: string | number | null | undefined): string => {
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

const WalletOverview: React.FC<WalletOverviewProps> = ({
  wallet,
  currentNetwork,
  networkName,
  ethPrice,
  supportedTokens = [],
  onSwitchWallet,
  onDisconnect,
}) => {
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
          {typeof onDisconnect === 'function' && (
            <button type="button" onClick={onDisconnect} className="disconnect-btn">
              {t('disconnect')}
            </button>
          )}
        </div>
      </div>

      <div className="wallet-balance">
        <div className="balance-label">{t('ethBalance')}</div>
        <div className="balance-amount">
          {wallet?.ethBalance || '0'} ETH
        </div>
        <div className="balance-usd">{formattedEthUSD} USD</div>
      </div>

      {tokenCards.length > 0 && (
        <div className="wallet-tokens">
          <div className="tokens-header">{t('tokens')}</div>
          {tokenCards.map((token) => (
            <div key={token.symbol} className="token-item">
              <div className="token-info">
                <span className="token-symbol">{token.symbol}</span>
                <span className="token-name">{token.name}</span>
              </div>
              <div className="token-balance">
                <span className="balance">{token.balance}</span>
                <span className="usd">{token.usdDisplay}</span>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}

export default WalletOverview
