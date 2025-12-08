import React, { useCallback, useEffect, useMemo, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import useAuthStore from '@/stores/authStore'
import { walletService } from '@/services/walletService'
import AlouLogo from '@/assets/AlouLogo.png'
import './LoginView.css'

const walletButtons = [
  {
    id: 'metamask',
    name: 'MetaMask',
    description: (hasMetaMask) => (hasMetaMask ? '已安装' : '需要安装浏览器插件'),
    icon: (
      <img
        src="https://upload.wikimedia.org/wikipedia/commons/3/36/MetaMask_Fox.svg"
        alt="MetaMask"
      />
    ),
  },
  {
    id: 'walletconnect',
    name: 'WalletConnect',
    description: () => '扫码连接移动钱包',
    icon: (
      <svg width="40" height="40" viewBox="0 0 300 185" fill="none">
        <path
          d="M61.439 36.256c48.91-47.888 128.212-47.888 177.123 0l5.886 5.764a6.041 6.041 0 010 8.67l-20.136 19.716a3.179 3.179 0 01-4.428 0l-8.101-7.931c-34.122-33.408-89.444-33.408-123.566 0l-8.675 8.494a3.179 3.179 0 01-4.428 0L54.978 51.253a6.041 6.041 0 010-8.67l6.461-6.327zm218.965 40.806l17.921 17.547a6.041 6.041 0 010 8.67l-80.81 79.122c-2.446 2.394-6.41 2.394-8.856 0l-57.354-56.155a1.59 1.59 0 00-2.214 0L91.737 182.4c-2.446 2.394-6.41 2.394-8.856 0L2.07 103.278a6.041 6.041 0 010-8.67l17.921-17.547c2.446-2.394 6.41-2.394 8.856 0l57.354 56.155a1.59 1.59 0 002.214 0l57.354-56.155c2.446-2.395 6.41-2.395 8.856 0l57.354 56.155a1.59 1.59 0 002.214 0l57.354-56.155c2.446-2.394 6.41-2.394 8.856 0z"
          fill="#3B99FC"
        />
      </svg>
    ),
  },
  {
    id: 'coinbase',
    name: 'Coinbase Wallet',
    description: () => '安全易用的加密钱包',
    icon: (
      <svg width="40" height="40" viewBox="0 0 1024 1024" fill="none">
        <rect width="1024" height="1024" rx="512" fill="#0052FF" />
        <path
          fillRule="evenodd"
          clipRule="evenodd"
          d="M512 768c141.385 0 256-114.615 256-256S653.385 256 512 256 256 370.615 256 512s114.615 256 256 256zm-40-384h80c13.255 0 24 10.745 24 24v80h80c13.255 0 24 10.745 24 24v80c0 13.255-10.745 24-24 24h-80v80c0 13.255-10.745 24-24 24h-80c-13.255 0-24-10.745-24-24v-80h-80c-13.255 0-24-10.745-24-24v-80c0-13.255 10.745-24 24-24h80v-80c0-13.255 10.745-24 24-24z"
          fill="white"
        />
      </svg>
    ),
  },
]

const LoginView = () => {
  const navigate = useNavigate()
  const loginWithWeb3Wallet = useAuthStore((state) => state.loginWithWeb3Wallet)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState('')
  const [currentWallet, setCurrentWallet] = useState(null)
  const [hasMetaMask, setHasMetaMask] = useState(false)

  useEffect(() => {
    setHasMetaMask(typeof window !== 'undefined' && !!window.ethereum)
  }, [])

  const goBack = useCallback(() => {
    navigate('/')
  }, [navigate])

  const handleWalletConnect = useCallback((id) => {
    setIsLoading(false)
    setCurrentWallet(null)
    if (id === 'walletconnect') {
      setError('WalletConnect 功能即将推出')
    } else if (id === 'coinbase') {
      setError('Coinbase Wallet 功能即将推出')
    }
  }, [])

  const connectMetaMask = useCallback(async () => {
    try {
      setIsLoading(true)
      setCurrentWallet('metamask')
      setError('')

      if (!walletService.isWalletAvailable()) {
        throw new Error('请先安装 MetaMask 浏览器插件')
      }

      let accounts = []
      try {
        accounts = await walletService.requestAccounts({ forceSelect: true })
      } catch (requestError) {
        if (requestError?.code === 4001) {
          throw requestError
        }
        throw new Error(requestError?.message || '请求钱包账户失败')
      }

      if (!accounts || accounts.length === 0) {
        throw new Error('未能获取钱包地址')
      }

      const chainId = await walletService.getCurrentChainId()

      await loginWithWeb3Wallet({
        address: accounts[0],
        chainId,
        walletType: 'metamask',
      })

      navigate('/')
    } catch (err) {
      if (err?.code === 4001) {
        setError('您拒绝了连接请求，请在 MetaMask 中选择要连接的钱包')
      } else if (err?.code === -32002) {
        setError('请在 MetaMask 中确认连接请求（可能已有待处理的请求）')
      } else if (err?.code === -32603) {
        setError('MetaMask 内部错误，请刷新页面重试')
      } else {
        setError(err?.message || '连接 MetaMask 失败，请重试')
      }
    } finally {
      setIsLoading(false)
      setCurrentWallet(null)
    }
  }, [loginWithWeb3Wallet, navigate])

  const handleWalletClick = useCallback(
    (id) => {
      if (isLoading) return
      if (id === 'metamask') {
        connectMetaMask()
      } else {
        handleWalletConnect(id)
      }
    },
    [connectMetaMask, handleWalletConnect, isLoading],
  )

  const wallets = useMemo(
    () =>
      walletButtons.map((button) => ({
        ...button,
        description: button.description(hasMetaMask),
      })),
    [hasMetaMask],
  )

  return (
    <div className="login-container">
      <div className="login-box">
        <button type="button" onClick={goBack} className="close-btn" title="返回">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19,6.41L17.59,5L12,10.59L6.41,5L5,6.41L10.59,12L5,17.59L6.41,19L12,13.41L17.59,19L19,17.59L13.41,12L19,6.41Z" />
          </svg>
        </button>

        <div className="logo">
          <img src={AlouLogo} alt="Alou" className="logo-icon" />
        </div>

        <h1 className="title">连接钱包</h1>
        <p className="subtitle">选择您的加密钱包以安全登录（浏览器版）</p>

        {error && (
          <div className="error-message">
            <span className="error-icon">⚠️</span>
            {error}
          </div>
        )}

        <div className="wallet-options">
          {wallets.map((wallet) => (
            <button
              key={wallet.id}
              type="button"
              onClick={() => handleWalletClick(wallet.id)}
              disabled={isLoading}
              className={`wallet-btn${isLoading && currentWallet === wallet.id ? ' loading' : ''}`}
            >
              <div
                className={`wallet-icon${wallet.id === 'walletconnect' ? ' wallet-icon-walletconnect' : ''}`}
              >
                {wallet.icon}
              </div>
              <div className="wallet-info">
                <div className="wallet-name">{wallet.name}</div>
                <div className="wallet-desc">{wallet.description}</div>
              </div>
              <div className="wallet-arrow">
                {isLoading && currentWallet === wallet.id ? (
                  <div className="spinner" />
                ) : (
                  <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M8.59,16.58L13.17,12L8.59,7.41L10,6L16,12L10,18L8.59,16.58Z" />
                  </svg>
                )}
              </div>
            </button>
          ))}
        </div>
        {!hasMetaMask && (
          <div className="browser-notice">
            <p>⚠️ 未检测到 MetaMask 插件，请先安装 MetaMask 浏览器扩展</p>
            <a
              href="https://metamask.io/download/"
              target="_blank"
              rel="noreferrer"
              className="install-link"
            >
              下载 MetaMask
            </a>
          </div>
        )}
        {hasMetaMask && (
          <div className="browser-success">
            <p>✅ 已检测到 MetaMask，点击上方按钮即可连接</p>
          </div>
        )}

        <div className="security-notice">
          <div className="notice-icon">🔒</div>
          <div className="notice-content">
            <h3>安全提示</h3>
            <ul>
              <li>我们不会存储您的私钥或助记词</li>
              <li>请确认您访问的是正确的网站</li>
              <li>不要与他人分享您的钱包信息</li>
            </ul>
          </div>
        </div>

        <div className="help-section">
          <p className="help-text">没有钱包？</p>
          <a
            href="https://metamask.io/download/"
            target="_blank"
            rel="noreferrer"
            className="help-link"
          >
            下载 MetaMask
          </a>
        </div>

        <p className="terms">
          连接钱包即表示您同意我们的
          <a href="/terms" target="_blank" rel="noreferrer">
            {' '}
            服务条款
          </a>
          和
          <a href="/privacy" target="_blank" rel="noreferrer">
            {' '}
            隐私政策
          </a>
        </p>
      </div>
    </div>
  )
}

export default LoginView
