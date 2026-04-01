import React, { useState, useEffect, useCallback } from 'react'
import { desktopWalletService } from '@/services/desktopWalletService'
import {
  getDefaultPrivateKey,
  hasDefaultPrivateKey,
  maskPrivateKey,
  saveDefaultPrivateKey,
  saveDefaultWalletAddress,
  getMnemonic,
  getWalletList,
  removeWalletFromList,
} from '@/utils/secureStorage'
import './LocalWalletForm.css'

const LocalWalletForm = ({ onConnected, onError, onCancel, onCreateNew }) => {
  const [inputType, setInputType] = useState('privateKey')
  const [inputValue, setInputValue] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [hasDefaultKey, setHasDefaultKey] = useState(false)
  const [showPrivateKey, setShowPrivateKey] = useState(false)
  const [maskedKey, setMaskedKey] = useState('')
  const [storedMnemonic, setStoredMnemonic] = useState(null)
  const [walletList, setWalletList] = useState([])
  const [showWalletList, setShowWalletList] = useState(false)

  useEffect(() => {
    const loadData = async () => {
      try {
        const hasDefault = await hasDefaultPrivateKey()
        setHasDefaultKey(hasDefault)

        if (hasDefault) {
          const defaultKey = await getDefaultPrivateKey()
          if (defaultKey) {
            setMaskedKey(maskPrivateKey(defaultKey))
            setInputValue(defaultKey)
          }
        }

        const mnemonic = await getMnemonic()
        if (mnemonic) setStoredMnemonic(mnemonic)

        const wallets = await getWalletList()
        if (wallets.length > 0) setWalletList(wallets)
      } catch (error) {
        console.error('[LocalWalletForm] Failed to load data:', error)
      }
    }

    loadData()
  }, [])

  const handleWalletClick = useCallback(async (wallet) => {
    try {
      setIsLoading(true)

      if (wallet.chain === 'ethereum') {
        const result = await desktopWalletService.connectLocalWallet(
          inputValue.trim(),
          false
        )
        const chainId = await desktopWalletService.getCurrentChainId()
        await desktopWalletService.saveWalletConnection(result.address, 'local')
        onConnected({ address: result.address, chainId, walletType: 'local' })
      } else if (wallet.chain === 'solana') {
        // Solana login - just pass address
        onConnected({
          address: wallet.address,
          chainId: 'solana',
          walletType: 'local',
        })
      }
    } catch (error) {
      onError(error.message || '连接钱包失败')
    } finally {
      setIsLoading(false)
    }
  }, [inputValue, onConnected, onError])

  const handleSubmit = async (e) => {
    e.preventDefault()

    if (!inputValue.trim()) {
      onError('请输入私钥或助记词')
      return
    }

    try {
      setIsLoading(true)

      const result = await desktopWalletService.connectLocalWallet(
        inputValue.trim(),
        inputType === 'mnemonic'
      )

      const address = result.address
      const message = desktopWalletService.generateVerificationMessage(address)
      const signature = await desktopWalletService.signMessage(message)
      const isValid = await desktopWalletService.verifySignature(address, message, signature)

      if (!isValid) {
        throw new Error('签名验证失败')
      }

      const chainId = await desktopWalletService.getCurrentChainId()
      await desktopWalletService.saveWalletConnection(address, 'local')

      if (inputType === 'privateKey' && !hasDefaultKey) {
        await saveDefaultPrivateKey(inputValue.trim())
        await saveDefaultWalletAddress(address)
        setHasDefaultKey(true)
        setMaskedKey(maskPrivateKey(inputValue.trim()))
      }

      onConnected({ address, chainId, walletType: 'local' })
    } catch (error) {
      console.error('Local wallet connection error:', error)
      onError(error.message || '连接钱包失败')
    } finally {
      setIsLoading(false)
    }
  }

  const handleCreateNew = async () => {
    try {
      setIsLoading(true)

      const result = await desktopWalletService.agentCreateMultiChainWallet()

      if (!result.success) {
        throw new Error(result.error || '创建钱包失败')
      }

      const message = desktopWalletService.generateVerificationMessage(result.ethereum.address)
      const signature = await desktopWalletService.signMessage(message)
      const isValid = await desktopWalletService.verifySignature(
        result.ethereum.address,
        message,
        signature
      )

      if (!isValid) {
        throw new Error('签名验证失败')
      }

      const chainId = await desktopWalletService.getCurrentChainId()
      await desktopWalletService.saveWalletConnection(result.ethereum.address, 'local')

      onCreateNew({
        address: result.ethereum.address,
        chainId: chainId || '0x1',
        walletType: 'local',
        mnemonic: result.mnemonic,
        privateKey: result.ethereum.privateKey,
        solanaAddress: result.solana.address,
      })

      setStoredMnemonic(result.mnemonic)
      const wallets = await getWalletList()
      setWalletList(wallets)
    } catch (error) {
      console.error('Create wallet error:', error)
      onError(error.message || '创建钱包失败')
    } finally {
      setIsLoading(false)
    }
  }

  const handleRemoveWallet = async (address) => {
    await removeWalletFromList(address)
    const wallets = await getWalletList()
    setWalletList(wallets)
  }

  return (
    <div className="local-wallet-form">
      <div className="form-header">
        <h3>导入本地钱包</h3>
        <p className="form-subtitle">
          {hasDefaultKey
            ? `已保存默认钱包：${maskedKey}`
            : storedMnemonic
            ? '已保存助记词，可创建新链钱包或导入现有钱包'
            : '输入您的私钥或助记词以连接钱包，或创建新的 Agent 钱包'}
        </p>
      </div>

      {walletList.length > 0 && (
        <div className="wallet-list-section">
          <button
            type="button"
            className="wallet-list-toggle"
            onClick={() => setShowWalletList(!showWalletList)}
          >
            <span>已保存的钱包 ({walletList.length})</span>
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              style={{ transform: showWalletList ? 'rotate(180deg)' : 'rotate(0deg)', transition: 'transform 0.2s' }}
            >
              <polyline points="6 9 12 15 18 9" />
            </svg>
          </button>

          {showWalletList && (
            <div className="wallet-list">
              {walletList.map((wallet) => (
                <div key={wallet.id} className="wallet-list-item">
                  <div className="wallet-item-info">
                    <span className={`wallet-chain-badge ${wallet.chain}`}>
                      {wallet.chain === 'ethereum' ? 'ETH' : 'SOL'}
                    </span>
                    <span className="wallet-item-name">{wallet.name}</span>
                    <span className="wallet-item-address">
                      {wallet.address.slice(0, 8)}...{wallet.address.slice(-6)}
                    </span>
                  </div>
                  <div className="wallet-item-actions">
                    <button
                      type="button"
                      className="wallet-item-login-btn"
                      onClick={() => handleWalletClick(wallet)}
                      disabled={isLoading}
                    >
                      登录
                    </button>
                    <button
                      type="button"
                      className="wallet-item-remove-btn"
                      onClick={() => handleRemoveWallet(wallet.address)}
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
            </div>
          )}
        </div>
      )}

      <form onSubmit={handleSubmit} className="wallet-form">
        <div className="input-type-selector">
          <button
            type="button"
            className={`type-btn ${inputType === 'privateKey' ? 'active' : ''}`}
            onClick={() => setInputType('privateKey')}
          >
            私钥
          </button>
          <button
            type="button"
            className={`type-btn ${inputType === 'mnemonic' ? 'active' : ''}`}
            onClick={() => setInputType('mnemonic')}
          >
            助记词
          </button>
        </div>

        <div className="form-group">
          <label htmlFor="wallet-input">
            {inputType === 'privateKey' ? '私钥' : '助记词'}
          </label>
          <div className="password-input-wrapper">
            <textarea
              id="wallet-input"
              value={inputValue}
              onChange={(e) => setInputValue(e.target.value)}
              placeholder={
                inputType === 'privateKey'
                  ? '请输入您的私钥（0x 开头的 64 位十六进制）'
                  : '请输入您的助记词（12 或 24 个单词，用空格分隔）'
              }
              rows={inputType === 'mnemonic' ? 3 : 2}
              className="wallet-input"
              disabled={isLoading}
              type={inputType === 'privateKey' && !showPrivateKey ? 'password' : 'text'}
              style={inputType === 'privateKey' ? {
                fontFamily: 'monospace',
                letterSpacing: '0.5px',
                WebkitTextSecurity: showPrivateKey ? 'none' : 'disc'
              } : {}}
            />
            {inputType === 'privateKey' && (
              <button
                type="button"
                className="toggle-visibility-btn"
                onClick={() => setShowPrivateKey(!showPrivateKey)}
                title={showPrivateKey ? '隐藏私钥' : '显示私钥'}
              >
                {showPrivateKey ? (
                  <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                    <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24" />
                    <line x1="1" y1="1" x2="23" y2="23" />
                  </svg>
                ) : (
                  <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                    <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
                    <circle cx="12" cy="12" r="3" />
                  </svg>
                )}
              </button>
            )}
          </div>
          {hasDefaultKey && inputType === 'privateKey' && (
            <p className="security-notice-text">
              🔒 私钥已安全保存，下次登录将自动填充
            </p>
          )}
          {storedMnemonic && inputType === 'mnemonic' && (
            <p className="security-notice-text">
              🔒 助记词已加密保存
            </p>
          )}
        </div>

        <div className="form-actions">
          <button type="button" onClick={onCancel} className="cancel-btn" disabled={isLoading}>
            取消
          </button>
          <button type="submit" className="submit-btn" disabled={isLoading || !inputValue.trim()}>
            {isLoading ? '连接中...' : hasDefaultKey ? '快速登录' : '连接钱包'}
          </button>
        </div>
      </form>

      {hasDefaultKey && (
        <div className="clear-default-section">
          <button
            type="button"
            onClick={async () => {
              const { clearDefaultPrivateKey, clearMnemonic } = await import('@/utils/secureStorage')
              await clearDefaultPrivateKey()
              await clearMnemonic()
              setHasDefaultKey(false)
              setMaskedKey('')
              setStoredMnemonic(null)
              setInputValue('')
              onError('')
            }}
            className="clear-default-btn"
          >
            清除保存的私钥和助记词
          </button>
        </div>
      )}

      <div className="create-wallet-section">
        <div className="divider">
          <span>或</span>
        </div>
        <button
          type="button"
          onClick={handleCreateNew}
          className="create-btn"
          disabled={isLoading}
        >
          🤖 Agent 创建 ETH + SOL 钱包
        </button>
        <p className="create-warning">
          ⚠️ Agent 将自动创建 Ethereum 和 Solana 钱包，请保存好您的助记词和私钥，丢失后将无法恢复！
        </p>
      </div>
    </div>
  )
}

export default LocalWalletForm
