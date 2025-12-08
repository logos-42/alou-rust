import React, { useState } from 'react'
import { desktopWalletService } from '@/services/desktopWalletService'
import './LocalWalletForm.css'

const LocalWalletForm = ({ onConnected, onError, onCancel, onCreateNew }) => {
  const [inputType, setInputType] = useState('privateKey') // 'privateKey' or 'mnemonic'
  const [inputValue, setInputValue] = useState('')
  const [isLoading, setIsLoading] = useState(false)

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
        <p className="form-subtitle">输入您的私钥或助记词以连接钱包</p>
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
          <textarea
            id="wallet-input"
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            placeholder={
              inputType === 'privateKey'
                ? '请输入您的私钥（0x开头的64位十六进制）'
                : '请输入您的助记词（12或24个单词，用空格分隔）'
            }
            rows={inputType === 'mnemonic' ? 3 : 2}
            className="wallet-input"
            disabled={isLoading}
          />
        </div>

        <div className="form-actions">
          <button type="button" onClick={onCancel} className="cancel-btn" disabled={isLoading}>
            取消
          </button>
          <button type="submit" className="submit-btn" disabled={isLoading || !inputValue.trim()}>
            {isLoading ? '连接中...' : '连接钱包'}
          </button>
        </div>
      </form>

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

