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

const WalletOverview = ({
  wallet,
  currentNetwork,
  networkName,
  ethPrice,
  onSwitchWallet,
  onDisconnect,
}) => {
  const { t } = useI18n()

  const formattedEthUSD = useMemo(
    () => calculateUSD(wallet?.ethBalance, ethPrice),
    [ethPrice, wallet?.ethBalance],
  )

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
        <div className="balance-card">
          <div className="balance-label">USDC {t('balance')}</div>
          <div className="balance-amount">{wallet?.usdcBalance || '0.0'}</div>
          <div className="balance-usd">≈ ${wallet?.usdcBalance || '0.0'}</div>
        </div>
      </div>
    </div>
  )
}

export default WalletOverview

