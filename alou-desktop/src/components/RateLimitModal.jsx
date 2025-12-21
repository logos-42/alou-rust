import React from 'react'
import { useI18n } from '@/hooks/useI18n'
import { useNavigate } from 'react-router-dom'
import './RateLimitModal.css'

const RateLimitModal = ({ 
  isOpen, 
  onClose, 
  remainingRequests = 0, 
  resetTime = null,
  onSubscribe 
}) => {
  const { t } = useI18n()
  const navigate = useNavigate()

  if (!isOpen) return null

  const formatResetTime = (timestamp) => {
    if (!timestamp) return ''
    try {
      const date = new Date(timestamp * 1000)
      return date.toLocaleString()
    } catch {
      return ''
    }
  }

  const handleSubscribe = () => {
    if (onSubscribe) {
      onSubscribe()
    } else {
      navigate('/subscription')
    }
    onClose?.()
  }

  return (
    <div className={`rate-limit-modal-overlay ${isOpen ? 'active' : ''}`} onClick={onClose}>
      <div className="rate-limit-modal" onClick={(e) => e.stopPropagation()}>
        <div className="rate-limit-modal-header">
          <h3>{t('agent.chat.error.rateLimitExceeded')}</h3>
          <button className="rate-limit-modal-close" onClick={onClose}>×</button>
        </div>
        <div className="rate-limit-modal-content">
          <p className="rate-limit-message">{t('agent.chat.error.rateLimitMessage')}</p>
          {remainingRequests !== undefined && (
            <div className="rate-limit-info">
              <span className="rate-limit-label">{t('agent.chat.error.rateLimitRemaining')}:</span>
              <span className="rate-limit-value">{remainingRequests}</span>
            </div>
          )}
          {resetTime && (
            <div className="rate-limit-info">
              <span className="rate-limit-label">{t('agent.chat.error.rateLimitReset')}:</span>
              <span className="rate-limit-value">{formatResetTime(resetTime)}</span>
            </div>
          )}
        </div>
        <div className="rate-limit-modal-footer">
          <button className="rate-limit-subscribe-btn" onClick={handleSubscribe}>
            {t('agent.chat.error.subscribeNow')}
          </button>
        </div>
      </div>
    </div>
  )
}

export default RateLimitModal

