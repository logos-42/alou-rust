import React, { useState } from 'react'
import { useI18n } from '@/hooks/useI18n'
import { saveDefaultPrivateKey, saveDefaultWalletAddress } from '@/utils/secureStorage'
import { ethers } from 'ethers'
import './BankCardLogin.css'

const BankCardLogin = ({ onConnected, onError, onCancel }) => {
  const { t } = useI18n()
  const [loginMethod, setLoginMethod] = useState('bankCard') // 'bankCard' or 'digitalRmb'
  const [cardNumber, setCardNumber] = useState('')
  const [phoneNumber, setPhoneNumber] = useState('')
  const [smsCode, setSmsCode] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [smsSent, setSmsSent] = useState(false)
  const [countdown, setCountdown] = useState(0)

  const formatCardNumber = (value) => {
    const digits = value.replace(/\D/g, '').slice(0, 19)
    return digits.replace(/(\d{4})(?=\d)/g, '$1 ')
  }

  const formatPhoneNumber = (value) => {
    const digits = value.replace(/\D/g, '').slice(0, 11)
    return digits.replace(/(\d{3})(\d{0,4})(\d{0,4})/, (_, a, b, c) => {
      if (b && c) return `${a} ${b} ${c}`
      if (b) return `${a} ${b}`
      return a
    })
  }

  const handleSendSms = async () => {
    if (!phoneNumber || phoneNumber.replace(/\s/g, '').length < 11) {
      onError(t('login.bankCard.error.phoneRequired'))
      return
    }

    try {
      setIsLoading(true)

      // TODO: Call bank API to send SMS verification code
      // const response = await fetch('/api/auth/bank-card/send-sms', {
      //   method: 'POST',
      //   headers: { 'Content-Type': 'application/json' },
      //   body: JSON.stringify({
      //     cardNumber: cardNumber.replace(/\s/g, ''),
      //     phone: phoneNumber.replace(/\s/g, ''),
      //     method: loginMethod,
      //   }),
      // })

      // Simulate SMS sent
      setSmsSent(true)
      setCountdown(60)

      const timer = setInterval(() => {
        setCountdown((prev) => {
          if (prev <= 1) {
            clearInterval(timer)
            return 0
          }
          return prev - 1
        })
      }, 1000)
    } catch (error) {
      onError(error.message || t('login.bankCard.error.smsFailed'))
    } finally {
      setIsLoading(false)
    }
  }

  const handleSubmit = async (e) => {
    e.preventDefault()

    const cleanCardNumber = cardNumber.replace(/\s/g, '')
    const cleanPhone = phoneNumber.replace(/\s/g, '')
    const cleanSmsCode = smsCode.trim()

    if (!cleanCardNumber) {
      onError(t('login.bankCard.error.cardRequired'))
      return
    }
    if (!cleanPhone) {
      onError(t('login.bankCard.error.phoneRequired'))
      return
    }
    if (!cleanSmsCode) {
      onError(t('login.bankCard.error.smsRequired'))
      return
    }

    try {
      setIsLoading(true)

      // TODO: Call bank API to verify SMS code and get wallet credentials
      // const response = await fetch('/api/auth/bank-card/verify', {
      //   method: 'POST',
      //   headers: { 'Content-Type': 'application/json' },
      //   body: JSON.stringify({
      //     cardNumber: cleanCardNumber,
      //     phone: cleanPhone,
      //     smsCode: cleanSmsCode,
      //     method: loginMethod,
      //   }),
      // })
      // const data = await response.json()
      //
      // if (!response.ok) {
      //   throw new Error(data.message || t('login.bankCard.error.verifyFailed'))
      // }
      //
      // const { privateKey, address } = data

      // Simulate wallet creation from bank verification
      // In production, the bank API would return or derive a wallet
      const wallet = ethers.Wallet.createRandom()
      const address = wallet.address
      const privateKey = wallet.privateKey

      // Save credentials securely
      await saveDefaultPrivateKey(privateKey)
      await saveDefaultWalletAddress(address)

      onConnected({
        address,
        chainId: '0x1',
        walletType: 'bankCard',
        metadata: {
          cardNumber: cleanCardNumber.slice(-4),
          phone: cleanPhone.slice(-4),
          method: loginMethod,
        },
      })
    } catch (error) {
      onError(error.message || t('login.bankCard.error.loginFailed'))
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <div className="bank-card-login">
      <div className="form-header">
        <h3>{t('login.bankCard.title')}</h3>
        <p className="form-subtitle">
          {t('login.bankCard.subtitle')}
        </p>
      </div>

      <div className="method-selector">
        <button
          type="button"
          className={`method-btn ${loginMethod === 'bankCard' ? 'active' : ''}`}
          onClick={() => setLoginMethod('bankCard')}
        >
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <rect x="2" y="5" width="20" height="14" rx="2" />
            <line x1="2" y1="10" x2="22" y2="10" />
          </svg>
          {t('login.bankCard.method.bankCard')}
        </button>
        <button
          type="button"
          className={`method-btn ${loginMethod === 'digitalRmb' ? 'active' : ''}`}
          onClick={() => setLoginMethod('digitalRmb')}
        >
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <circle cx="12" cy="12" r="10" />
            <path d="M12 6v12M8 10h8M8 14h8" />
          </svg>
          {t('login.bankCard.method.digitalRmb')}
        </button>
      </div>

      <form onSubmit={handleSubmit} className="bank-card-form">
        <div className="form-group">
          <label htmlFor="card-number">
            {loginMethod === 'bankCard' ? t('login.bankCard.cardNumber') : t('login.bankCard.digitalRmbWallet')}
          </label>
          <input
            id="card-number"
            type="text"
            value={cardNumber}
            onChange={(e) => setCardNumber(formatCardNumber(e.target.value))}
            placeholder={
              loginMethod === 'bankCard'
                ? t('login.bankCard.cardNumber.placeholder')
                : t('login.bankCard.digitalRmbWallet.placeholder')
            }
            className="bank-card-input"
            disabled={isLoading}
            autoComplete="off"
          />
        </div>

        <div className="form-group">
          <label htmlFor="phone-number">{t('login.bankCard.phoneNumber')}</label>
          <input
            id="phone-number"
            type="tel"
            value={phoneNumber}
            onChange={(e) => setPhoneNumber(formatPhoneNumber(e.target.value))}
            placeholder={t('login.bankCard.phoneNumber.placeholder')}
            className="bank-card-input"
            disabled={isLoading}
            autoComplete="tel"
          />
        </div>

        <div className="form-group">
          <label htmlFor="sms-code">{t('login.bankCard.smsCode')}</label>
          <div className="sms-input-wrapper">
            <input
              id="sms-code"
              type="text"
              value={smsCode}
              onChange={(e) => setSmsCode(e.target.value.replace(/\D/g, '').slice(0, 6))}
              placeholder={t('login.bankCard.smsCode.placeholder')}
              className="bank-card-input sms-input"
              disabled={isLoading || !smsSent}
              autoComplete="one-time-code"
            />
            <button
              type="button"
              className="sms-send-btn"
              onClick={handleSendSms}
              disabled={isLoading || countdown > 0}
            >
              {countdown > 0 ? `${countdown}s` : t('login.bankCard.sendSms')}
            </button>
          </div>
        </div>

        <div className="form-actions">
          <button type="button" onClick={onCancel} className="cancel-btn" disabled={isLoading}>
            {t('login.bankCard.cancel')}
          </button>
          <button type="submit" className="submit-btn" disabled={isLoading}>
            {isLoading ? t('login.bankCard.loggingIn') : t('login.bankCard.login')}
          </button>
        </div>
      </form>

      <div className="security-notice">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
          <path d="M7 11V7a5 5 0 0 1 10 0v4" />
        </svg>
        <span>{t('login.bankCard.securityNotice')}</span>
      </div>
    </div>
  )
}

export default BankCardLogin
