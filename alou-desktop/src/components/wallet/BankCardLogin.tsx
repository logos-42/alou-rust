import React, { useState, useEffect, useCallback } from 'react'
import { useI18n } from '@/hooks/useI18n'
import {
  saveDefaultPrivateKey,
  saveDefaultWalletAddress,
  saveMnemonic,
  addWalletToList,
  getWalletList,
  removeWalletFromList,
} from '@/utils/secureStorage'
import { ethers } from 'ethers'
import './BankCardLogin.css'

const BankCardLogin = ({ onConnected, onError, onCancel }) => {
  const { t } = useI18n()
  const [loginMethod, setLoginMethod] = useState('bankCard')
  const [cardNumber, setCardNumber] = useState('')
  const [phoneNumber, setPhoneNumber] = useState('')
  const [smsCode, setSmsCode] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [smsSent, setSmsSent] = useState(false)
  const [countdown, setCountdown] = useState(0)
  const [savedCards, setSavedCards] = useState([])
  const [showSavedCards, setShowSavedCards] = useState(false)
  const [isAddingCard, setIsAddingCard] = useState(false)

  useEffect(() => {
    const loadCards = async () => {
      try {
        const wallets = await getWalletList()
        const bankCards = wallets.filter((w) => w.chain === 'bankCard' || w.chain === 'digitalRmb')
        setSavedCards(bankCards)
      } catch (error) {
        console.error('[BankCardLogin] Failed to load saved cards:', error)
      }
    }
    loadCards()
  }, [])

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

      // TODO: Call bank API
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

  const handleCardLogin = useCallback(async (card) => {
    try {
      setIsLoading(true)
      onConnected({
        address: card.address,
        chainId: card.chain === 'digitalRmb' ? '0xdcep' : '0x1',
        walletType: 'bankCard',
        metadata: {
          cardNumber: card.address.slice(-4),
          method: card.chain,
          name: card.name,
        },
      })
    } catch (error) {
      onError(error.message || t('login.bankCard.error.loginFailed'))
    } finally {
      setIsLoading(false)
    }
  }, [onConnected, onError, t])

  const handleRemoveCard = async (address) => {
    await removeWalletFromList(address)
    const wallets = await getWalletList()
    const bankCards = wallets.filter((w) => w.chain === 'bankCard' || w.chain === 'digitalRmb')
    setSavedCards(bankCards)
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

      // TODO: Call bank API to verify
      // const response = await fetch('/api/auth/bank-card/verify', { ... })

      // Generate wallet from bank verification
      const wallet = ethers.Wallet.createRandom()
      const address = wallet.address
      const privateKey = wallet.privateKey
      const mnemonic = wallet.mnemonic?.phrase || ''

      // Save credentials securely
      await saveDefaultPrivateKey(privateKey)
      await saveDefaultWalletAddress(address)
      if (mnemonic) await saveMnemonic(mnemonic)

      // Save card to list
      const chainType = loginMethod === 'digitalRmb' ? 'digitalRmb' : 'bankCard'
      const cardName = loginMethod === 'digitalRmb'
        ? `数字人民币 ${cleanCardNumber.slice(-4)}`
        : `银行卡 ${cleanCardNumber.slice(-4)}`

      await addWalletToList({
        id: `${chainType}_${cleanCardNumber.slice(-4)}`,
        name: cardName,
        chain: chainType,
        address: cleanCardNumber,
        createdAt: new Date().toISOString(),
      })

      // Update saved cards list
      const wallets = await getWalletList()
      const bankCards = wallets.filter((w) => w.chain === 'bankCard' || w.chain === 'digitalRmb')
      setSavedCards(bankCards)

      onConnected({
        address,
        chainId: loginMethod === 'digitalRmb' ? '0xdcep' : '0x1',
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
          onClick={() => { setLoginMethod('bankCard'); setIsAddingCard(false) }}
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
          onClick={() => { setLoginMethod('digitalRmb'); setIsAddingCard(false) }}
        >
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <circle cx="12" cy="12" r="10" />
            <path d="M12 6v12M8 10h8M8 14h8" />
          </svg>
          {t('login.bankCard.method.digitalRmb')}
        </button>
      </div>

      {savedCards.length > 0 && (
        <div className="saved-cards-section">
          <button
            type="button"
            className="saved-cards-toggle"
            onClick={() => setShowSavedCards(!showSavedCards)}
          >
            <span>已保存的{loginMethod === 'digitalRmb' ? '数字钱包' : '银行卡'} ({savedCards.length})</span>
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              style={{ transform: showSavedCards ? 'rotate(180deg)' : 'rotate(0deg)', transition: 'transform 0.2s' }}
            >
              <polyline points="6 9 12 15 18 9" />
            </svg>
          </button>

          {showSavedCards && (
            <div className="saved-cards-list">
              {savedCards
                .filter((card) => card.chain === loginMethod || (loginMethod === 'bankCard' && card.chain === 'bankCard'))
                .map((card) => (
                  <div key={card.id} className="saved-card-item">
                    <div className="saved-card-info">
                      <span className={`card-type-badge ${card.chain}`}>
                        {card.chain === 'digitalRmb' ? '数字人民币' : '银行卡'}
                      </span>
                      <span className="saved-card-name">{card.name}</span>
                      <span className="saved-card-number">****{card.address.slice(-4)}</span>
                    </div>
                    <div className="saved-card-actions">
                      <button
                        type="button"
                        className="saved-card-login-btn"
                        onClick={() => handleCardLogin(card)}
                        disabled={isLoading}
                      >
                        登录
                      </button>
                      <button
                        type="button"
                        className="saved-card-remove-btn"
                        onClick={() => handleRemoveCard(card.address)}
                        title="移除"
                      >
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                          <line x1="18" y1="6" x2="6" y2="18" />
                          <line x1="6" y1="6" x2="18" y2="18" />
                        </svg>
                      </button>
                    </div>
                  </div>
                ))}

              <button
                type="button"
                className="add-new-card-btn"
                onClick={() => setIsAddingCard(true)}
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <line x1="12" y1="5" x2="12" y2="19" />
                  <line x1="5" y1="12" x2="19" y2="12" />
                </svg>
                添加新的{loginMethod === 'digitalRmb' ? '数字钱包' : '银行卡'}
              </button>
            </div>
          )}
        </div>
      )}

      {(!savedCards.length || isAddingCard) && (
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
            {isAddingCard && (
              <button type="button" onClick={() => setIsAddingCard(false)} className="cancel-btn">
                返回
              </button>
            )}
            <button type="button" onClick={onCancel} className="cancel-btn" disabled={isLoading}>
              {t('login.bankCard.cancel')}
            </button>
            <button type="submit" className="submit-btn" disabled={isLoading}>
              {isLoading ? t('login.bankCard.loggingIn') : t('login.bankCard.login')}
            </button>
          </div>
        </form>
      )}

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
