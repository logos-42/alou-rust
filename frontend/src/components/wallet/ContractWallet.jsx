import React from 'react'
import { useI18n } from '@/hooks/useI18n'
import './ContractWallet.css'

const formatAddress = (address) => {
  if (!address || address.length <= 10) return address
  return `${address.slice(0, 6)}...${address.slice(-4)}`
}

const ContractWallet = ({
  contractWallet,
  onCreate,
  onDeposit,
  onWithdraw,
  onManage,
}) => {
  const { t } = useI18n()

  return (
    <div className="contract-wallet-section">
      <div className="section-header">
        <h3>{t('agentContractWallet')}</h3>
        {!contractWallet && (
          <button type="button" onClick={onCreate} className="create-btn">
            {t('createContractWallet')}
          </button>
        )}
      </div>

      {!contractWallet ? (
        <div className="empty-state">
          <div className="empty-icon">🤖</div>
          <p>{t('noContractWallet')}</p>
          <p className="hint">{t('contractWalletHint')}</p>
        </div>
      ) : (
        <div className="contract-wallet-card">
          <div className="contract-header">
            <div className="contract-icon">🤖</div>
            <div className="contract-info">
              <div className="contract-label">{t('agentWallet')}</div>
              <div className="contract-address">
                {formatAddress(contractWallet.address)}
              </div>
            </div>
          </div>

          <div className="contract-balance">
            <div className="balance-label">{t('totalBalance')}</div>
            <div className="balance-amount">{contractWallet.balance} ETH</div>
          </div>

          <div className="contract-actions">
            <button type="button" onClick={onDeposit} className="action-btn primary">
              {t('deposit')}
            </button>
            <button type="button" onClick={onWithdraw} className="action-btn">
              {t('withdraw')}
            </button>
            <button type="button" onClick={onManage} className="action-btn">
              {t('manage')}
            </button>
          </div>
        </div>
      )}
    </div>
  )
}

export default ContractWallet

