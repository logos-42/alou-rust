import React from 'react'
import { useI18n } from '@/hooks/useI18n'
import './WalletConnect.css'

const WalletConnect = ({ onConnectMetaMask, onConnectWalletConnect }) => {
  const { t } = useI18n()

  return (
    <div className="connect-section">
      <div className="connect-card">
        <div className="connect-icon">🔐</div>
        <h2>{t('connectWallet')}</h2>
        <p>{t('connectWalletDesc')}</p>
        <div className="wallet-options">
          <button type="button" onClick={onConnectMetaMask} className="wallet-btn metamask">
            <img
              src="https://upload.wikimedia.org/wikipedia/commons/3/36/MetaMask_Fox.svg"
              alt="MetaMask"
            />
            <span>MetaMask</span>
          </button>
          <button
            type="button"
            onClick={onConnectWalletConnect}
            className="wallet-btn walletconnect"
          >
            <span>🔗</span>
            <span>WalletConnect</span>
          </button>
        </div>
      </div>
    </div>
  )
}

export default WalletConnect

