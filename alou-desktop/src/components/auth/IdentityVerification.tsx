import React, { useState, useCallback } from 'react'
import { useI18n } from '@/hooks/useI18n'
import './IdentityVerification.css'

interface IdentityVerificationProps {
  onVerified?: (data: { idCard: string; phone: string }) => void
  onError?: (error: string) => void
  onCancel?: () => void
}

const IdentityVerification: React.FC<IdentityVerificationProps> = ({
  onVerified,
  onError,
  onCancel,
}) => {
  const { t } = useI18n()
  const [idCard, setIdCard] = useState('')
  const [phone, setPhone] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState('')

  const validateIdCard = (value: string): boolean => {
    // 18位身份证号校验
    const reg = /^[1-9]\d{5}(19|20)\d{2}(0[1-9]|1[0-2])(0[1-9]|[12]\d|3[01])\d{3}[\dXx]$/
    return reg.test(value)
  }

  const validatePhone = (value: string): boolean => {
    const reg = /^1[3-9]\d{9}$/
    return reg.test(value)
  }

  const handleSubmit = useCallback(async () => {
    setError('')

    if (!idCard.trim()) {
      const msg = t('login.identity.error.idCardRequired')
      setError(msg)
      onError?.(msg)
      return
    }

    if (!validateIdCard(idCard.trim())) {
      const msg = t('login.identity.error.idCardInvalid')
      setError(msg)
      onError?.(msg)
      return
    }

    if (!phone.trim()) {
      const msg = t('login.identity.error.phoneRequired')
      setError(msg)
      onError?.(msg)
      return
    }

    if (!validatePhone(phone.trim())) {
      const msg = t('login.identity.error.phoneInvalid')
      setError(msg)
      onError?.(msg)
      return
    }

    setIsLoading(true)
    try {
      // DIAP 协议捕获身份证和手机号进行认证绑定
      // 当前为占位实现，待 DIAP 更新后接入
      console.log('[IdentityVerification] 提交实名认证:', {
        idCard: idCard.trim(),
        phone: phone.trim(),
      })

      // 模拟 DIAP 协议处理
      await new Promise((resolve) => setTimeout(resolve, 1000))

      // 存储认证状态到 localStorage（临时方案，后续由 DIAP 管理）
      localStorage.setItem('diap_identity_verified', 'true')
      localStorage.setItem('diap_identity_phone', phone.trim())
      // 不存储完整身份证号，仅标记已验证
      localStorage.setItem('diap_identity_verified_at', new Date().toISOString())

      onVerified?.({ idCard: idCard.trim(), phone: phone.trim() })
    } catch (err: any) {
      const msg = err?.message || t('login.identity.error.verifyFailed')
      setError(msg)
      onError?.(msg)
    } finally {
      setIsLoading(false)
    }
  }, [idCard, phone, onVerified, onError, t])

  const handleCancel = useCallback(() => {
    onCancel?.()
  }, [onCancel])

  return (
    <div className="identity-verification">
      <div className="identity-header">
        <svg
          width="24"
          height="24"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          className="identity-icon"
        >
          <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
          <circle cx="12" cy="7" r="4" />
        </svg>
        <div>
          <h3 className="identity-title">{t('login.identity.title')}</h3>
          <p className="identity-desc">{t('login.identity.desc')}</p>
        </div>
      </div>

      {error && (
        <div className="identity-error">
          <span className="identity-error-icon">⚠️</span>
          {error}
        </div>
      )}

      <div className="identity-form">
        <div className="identity-field">
          <label className="identity-label">{t('login.identity.idCard')}</label>
          <input
            type="text"
            className="identity-input"
            placeholder={t('login.identity.idCard.placeholder')}
            value={idCard}
            onChange={(e) => setIdCard(e.target.value)}
            maxLength={18}
            disabled={isLoading}
          />
        </div>

        <div className="identity-field">
          <label className="identity-label">{t('login.identity.phone')}</label>
          <input
            type="tel"
            className="identity-input"
            placeholder={t('login.identity.phone.placeholder')}
            value={phone}
            onChange={(e) => setPhone(e.target.value)}
            maxLength={11}
            disabled={isLoading}
          />
        </div>
      </div>

      <div className="identity-notice">
        <span className="identity-notice-icon">🔒</span>
        <span>{t('login.identity.notice')}</span>
      </div>

      <div className="identity-actions">
        <button
          type="button"
          className="identity-btn identity-btn-cancel"
          onClick={handleCancel}
          disabled={isLoading}
        >
          {t('login.identity.cancel')}
        </button>
        <button
          type="button"
          className="identity-btn identity-btn-submit"
          onClick={handleSubmit}
          disabled={isLoading}
        >
          {isLoading ? t('login.identity.verifying') : t('login.identity.submit')}
        </button>
      </div>
    </div>
  )
}

export default IdentityVerification
