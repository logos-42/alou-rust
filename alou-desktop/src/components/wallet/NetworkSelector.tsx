import React from 'react'
import { useI18n } from '@/hooks/useI18n'
import './NetworkSelector.css'

export interface Network {
  chainId: string
  name: string
  type: string
  icon: string
}

export interface NetworkSelectorProps {
  networks?: Network[]
  currentNetwork?: string
  onSwitchNetwork?: (network: Network) => void
}

const NetworkSelector: React.FC<NetworkSelectorProps> = ({ networks = [], currentNetwork, onSwitchNetwork }) => {
  const { t } = useI18n()

  return (
    <div className="network-section">
      <h3>{t('networkSettings')}</h3>
      <div className="network-grid">
        {networks.map((network) => (
          <button
            type="button"
            key={network.chainId}
            onClick={() => onSwitchNetwork?.(network)}
            className={`network-card${currentNetwork === network.chainId ? ' active' : ''}`}
          >
            <div className="network-icon">{network.icon}</div>
            <div className="network-info">
              <div className="network-name">{network.name}</div>
              <div className="network-type">{network.type}</div>
            </div>
            {currentNetwork === network.chainId && <div className="network-check">✓</div>}
          </button>
        ))}
      </div>
    </div>
  )
}

export default NetworkSelector
