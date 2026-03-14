import React from 'react'
import { useI18n } from '@/hooks/useI18n'
import CloseIcon from '@/assets/关闭0.3.png'
import './SignatureModal.css'

const formatAddress = (address) => {
  if (!address || address.length <= 10) return address
  return `${address.slice(0, 6)}...${address.slice(-4)}`
}

const SignatureModal = ({ show, request, isSigning, onConfirm, onCancel }) => {
  const { t } = useI18n()

  if (!show) {
    return null
  }

  return (
    <div className="modal-overlay" onClick={onCancel}>
      <div className="modal-content" onClick={(event) => event.stopPropagation()}>
        <div className="modal-header">
          <h3>{t('signatureRequest')}</h3>
          <button type="button" onClick={onCancel} className="close-btn">
            <img src={CloseIcon} alt="关闭" />
          </button>
        </div>
        <div className="modal-body">
          <div className="signature-info">
            <div className="info-item">
              <span className="label">{t('from')}:</span>
              <span className="value">{formatAddress(request.from)}</span>
            </div>
            <div className="info-item">
              <span className="label">{t('to')}:</span>
              <span className="value">{formatAddress(request.to)}</span>
            </div>
            <div className="info-item">
              <span className="label">{t('amount')}:</span>
              <span className="value highlight">
                {request.value} {request.token}
              </span>
            </div>
            <div className="info-item">
              <span className="label">{t('gasFee')}:</span>
              <span className="value">{request.gasFee} ETH</span>
            </div>
          </div>
          <div className="warning-box">
            <span className="warning-icon">⚠️</span>
            <span>{t('signatureWarning')}</span>
          </div>
        </div>
        <div className="modal-footer">
          <button type="button" onClick={onCancel} className="btn-secondary">
            {t('cancel')}
          </button>
          <button type="button" onClick={onConfirm} className="btn-primary" disabled={isSigning}>
            {isSigning ? `${t('signing')}...` : t('confirm')}
          </button>
        </div>
      </div>
    </div>
  )
}

export default SignatureModal
