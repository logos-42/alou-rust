import React, { useState, useEffect } from 'react'
import { desktopWalletService } from '@/services/desktopWalletService'
import { 
  getDefaultPrivateKey, 
  hasDefaultPrivateKey, 
  maskPrivateKey,
  saveDefaultPrivateKey,
  saveDefaultWalletAddress 
} from '@/utils/secureStorage'
import './LocalWalletForm.css'

const LocalWalletForm = ({ onConnected, onError, onCancel, onCreateNew }) => {
  const [inputType, setInputType] = useState('privateKey') // 'privateKey' or 'mnemonic'
  const [inputValue, setInputValue] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [hasDefaultKey, setHasDefaultKey] = useState(false)
  const [showPrivateKey, setShowPrivateKey] = useState(false)
  const [maskedKey, setMaskedKey] = useState('')

  // Load default private key on mount
  useEffect(() => {
    const loadDefaultKey = async () => {
      try {
        const hasDefault = await hasDefaultPrivateKey()
        setHasDefaultKey(hasDefault)
        
        if (hasDefault) {
          const defaultKey = await getDefaultPrivateKey()
          if (defaultKey) {
            setMaskedKey(maskPrivateKey(defaultKey))
            // Auto-fill for quick login (key is masked in UI)
            setInputValue(defaultKey)
          }
        }
      } catch (error) {
        console.error('[LocalWalletForm] Failed to load default key:', error)
      }
    }
    
    loadDefaultKey()
  }, [])

  const handleSubmit = async (e) => {
    e.preventDefault()

    if (!inputValue.trim()) {
      onError('请输入私钥或助记词')
      return
    }

    try {
      setIsLoading(true)

      // 连接本地钱包
      const result = await desktopWalletService.connectLocalWallet(
        inputValue.trim(),
        inputType === 'mnemonic'
      )

      const address = result.address

      // 生成验证消息并签名
      const message = desktopWalletService.generateVerificationMessage(address)
      const signature = await desktopWalletService.signMessage(message)

      // 验证签名
      const isValid = await desktopWalletService.verifySignature(address, message, signature)

      if (!isValid) {
        throw new Error('签名验证失败')
      }

      // 保存连接信息
      const chainId = await desktopWalletService.getCurrentChainId()
      await desktopWalletService.saveWalletConnection(address, 'local')

      // 如果是私钥且用户选择保存，保存为默认私钥
      if (inputType === 'privateKey' && !hasDefaultKey) {
        // 首次登录，询问是否保存
        // 为了用户体验，自动保存（可以在设置中清除）
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

      // 创建新钱包
      const walletData = await desktopWalletService.createNewWallet()

      // 生成验证消息并签名
      const message = desktopWalletService.generateVerificationMessage(walletData.address)
      const signature = await walletData.wallet.signMessage(message)

      // 验证签名
      const isValid = await desktopWalletService.verifySignature(
        walletData.address,
        message,
        signature
      )

      if (!isValid) {
        throw new Error('签名验证失败')
      }

      // 保存连接信息
      const chainId = await desktopWalletService.getCurrentChainId()
      await desktopWalletService.saveWalletConnection(walletData.address, 'local')

      // 将钱包数据传递给父组件（用于显示助记词和私钥）
      onCreateNew(walletData)
    } catch (error) {
      console.error('Create wallet error:', error)
      onError(error.message || '创建钱包失败')
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <div className="local-wallet-form">
      <div className="form-header">
        <h3>导入本地钱包</h3>
        <p className="form-subtitle">
          {hasDefaultKey 
            ? `已保存默认钱包：${maskedKey}` 
            : '输入您的私钥或助记词以连接钱包'}
        </p>
      </div>

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
              const { clearDefaultPrivateKey } = await import('@/utils/secureStorage')
              await clearDefaultPrivateKey()
              setHasDefaultKey(false)
              setMaskedKey('')
              setInputValue('')
              onError('')
            }}
            className="clear-default-btn"
          >
            清除保存的私钥
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
          创建新钱包
        </button>
        <p className="create-warning">
          ⚠️ 创建新钱包后，请务必保存好您的助记词和私钥，丢失后将无法恢复！
        </p>
      </div>
    </div>
  )
}

export default LocalWalletForm
