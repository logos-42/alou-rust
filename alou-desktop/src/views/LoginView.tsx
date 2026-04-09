import React, { useCallback, useEffect, useMemo, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import useAuthStore from '@/stores/authStore'
import { walletService } from '@/services/walletService'
import WalletConnectQR from '@/components/wallet/WalletConnectQR'
import LocalWalletForm from '@/components/wallet/LocalWalletForm'
// import BankCardLogin from '@/components/wallet/BankCardLogin' // Hidden: Bank card and digital RMB login
import { useI18n } from '@/hooks/useI18n'
import { useTheme } from '@/hooks/useTheme'
import CloseIcon from '@/assets/关闭0.3.png'
import WalletIcon from '@/assets/钱包0.3.png'
import './LoginView.css'

const getWalletButtons = (isDesktop, t) => {
  if (isDesktop) {
    // 桌面版钱包选项
    return [
      {
        id: 'walletconnect',
        name: t('login.wallet.walletconnect'),
        description: () => t('login.wallet.walletconnect.desc'),
        icon: (
          <svg width="40" height="40" viewBox="0 0 300 185" fill="none">
            <path
              d="M61.439 36.256c48.91-47.888 128.212-47.888 177.123 0l5.886 5.764a6.041 6.041 0 010 8.67l-20.136 19.716a3.179 3.179 0 01-4.428 0l-8.101-7.931c-34.122-33.408-89.444-33.408-123.566 0l-8.675 8.494a3.179 3.179 0 01-4.428 0L54.978 51.253a6.041 6.041 0 010-8.67l6.461-6.327zm218.965 40.806l17.921 17.547a6.041 6.041 0 010 8.67l-80.81 79.122c-2.446 2.394-6.41 2.394-8.856 0l-57.354-56.155a1.59 1.59 0 00-2.214 0L91.737 182.4c-2.446 2.394-6.41 2.394-8.856 0L2.07 103.278a6.041 6.041 0 010-8.67l17.921-17.547c2.446-2.394 6.41-2.394 8.856 0l57.354 56.155a1.59 1.59 0 002.214 0l57.354-56.155c2.446-2.395 6.41-2.395 8.856 0l57.354 56.155a1.59 1.59 0 002.214 0l57.354-56.155c2.446-2.394 6.41-2.394 8.856 0z"
              fill="#FFFFFF"
            />
          </svg>
        ),
      },
      {
        id: 'local',
        name: t('login.wallet.local'),
        description: () => t('login.wallet.local.desc'),
        icon: (
          <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
            <path d="M7 11V7a5 5 0 0 1 10 0v4" />
          </svg>
        ),
      },
    ]
  }

  // 浏览器版钱包选项
  return [
    {
      id: 'metamask',
      name: t('login.wallet.metamask'),
      description: (hasMetaMask) => hasMetaMask ? t('login.wallet.metamask.installed') : t('login.wallet.metamask.notInstalled'),
      icon: (
        <img
          src="https://upload.wikimedia.org/wikipedia/commons/3/36/MetaMask_Fox.svg"
          alt="MetaMask"
        />
      ),
    },
    {
      id: 'walletconnect',
      name: t('login.wallet.walletconnect'),
      description: () => t('login.wallet.walletconnect.desc'),
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
      name: t('login.wallet.coinbase'),
      description: () => t('login.wallet.coinbase.desc'),
      icon: (
        <svg width="40" height="40" viewBox="0 0 1024 1024" fill="none">
          <rect width="1024" height="1024" rx="512" fill="#FFFFFF" />
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
}

const LoginView = () => {
  const { t } = useI18n()
  const { isDarkMode, toggleTheme } = useTheme()
  const navigate = useNavigate()
  const loginWithWeb3Wallet = useAuthStore((state) => state.loginWithWeb3Wallet)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState('')
  const [currentWallet, setCurrentWallet] = useState(null)
  const [hasMetaMask, setHasMetaMask] = useState(false)
  // 强制假设为桌面环境（在 Tauri 项目中，默认应该是桌面环境）
  // 只有在明确检测到浏览器环境时才设为 false
  const [isDesktop, setIsDesktop] = useState(() => {
    if (typeof window === 'undefined') return false
    
    // 首先检查是否是明确的浏览器环境
    const isExplicitBrowser = typeof window.chrome !== 'undefined' && 
                               typeof window.chrome.runtime !== 'undefined' &&
                               !window.__TAURI__ &&
                               !window.__TAURI_IPC__ &&
                               !import.meta.env?.TAURI_PLATFORM
    
    // 如果不是明确的浏览器环境，默认假设是桌面环境（因为这是 Tauri 项目）
    // 或者检测到任何 Tauri 标识，都认为是桌面环境
    const hasTauri = typeof window.__TAURI__ !== 'undefined' || 
                     typeof window.__TAURI_IPC__ !== 'undefined' ||
                     (typeof import.meta !== 'undefined' && Boolean(import.meta.env?.TAURI_PLATFORM)) ||
                     (typeof navigator !== 'undefined' && /tauri/i.test(navigator.userAgent || ''))
    
    // 默认假设是桌面环境（除非明确检测到是浏览器）
    // 由于这是 Tauri 项目，运行在桌面应用中时默认应该返回 true
    const isDesktopDefault = !isExplicitBrowser || hasTauri
    
    console.log('[LoginView] Initial desktop detection:', {
      isDesktopDefault,
      hasTauri,
      isExplicitBrowser,
      __TAURI__: typeof window.__TAURI__,
      __TAURI_IPC__: typeof window.__TAURI_IPC__,
      TAURI_PLATFORM: import.meta.env?.TAURI_PLATFORM,
      userAgent: typeof navigator !== 'undefined' ? navigator.userAgent : '',
    })
    
    return isDesktopDefault
  })
  // 桌面版默认显示本地钱包表单，浏览器版为 null
  const [connectionMode, setConnectionMode] = useState(() => {
    return isDesktop ? 'local' : null
  })
  const [newWalletData, setNewWalletData] = useState(null)
  // const [loginCategory, setLoginCategory] = useState('crypto') // 'crypto' or 'bankCard' - Hidden: Bank card login

  useEffect(() => {
    // 延迟检测，确保 Tauri API 已经加载
    const detectDesktop = () => {
      // 使用多种方式检测
      const hasTauri = typeof window.__TAURI__ !== 'undefined' || 
                       typeof window.__TAURI_IPC__ !== 'undefined' ||
                       (typeof import.meta !== 'undefined' && Boolean(import.meta.env?.TAURI_PLATFORM)) ||
                       (typeof navigator !== 'undefined' && /tauri/i.test(navigator.userAgent || ''))
      
      // 检查是否是明确的浏览器环境
      const isExplicitBrowser = typeof window.chrome !== 'undefined' && 
                                 typeof window.chrome.runtime !== 'undefined' &&
                                 !window.__TAURI__ &&
                                 !window.__TAURI_IPC__ &&
                                 !import.meta.env?.TAURI_PLATFORM
      
      // 默认假设是桌面环境（除非明确检测到是浏览器且没有 Tauri 标识）
      const desktop = !isExplicitBrowser || hasTauri
      
      console.log('[LoginView] Desktop detection (useEffect):', desktop, {
        hasTauri,
        isExplicitBrowser,
        __TAURI__: typeof window.__TAURI__,
        __TAURI_IPC__: typeof window.__TAURI_IPC__,
        TAURI_PLATFORM: import.meta.env?.TAURI_PLATFORM,
        userAgent: typeof navigator !== 'undefined' ? navigator.userAgent : '',
        chrome: typeof window.chrome,
      })
      
      setIsDesktop((prevDesktop) => {
        // 只有在检测结果发生变化时才更新
        if (desktop !== prevDesktop) {
          // 如果是桌面环境，确保默认连接模式为本地钱包
          if (desktop) {
            setConnectionMode((prevMode) => {
              if (prevMode !== 'local' && prevMode !== 'walletconnect') {
                console.log('[LoginView] Setting connection mode to local')
                return 'local'
              }
              return prevMode
            })
          }
          return desktop
        }
        return prevDesktop
      })
    }

    setHasMetaMask(typeof window !== 'undefined' && !!window.ethereum)
    
    // 立即检测一次
    detectDesktop()
    
    // 延迟再检测一次（等待 Tauri API 加载）
    const timer = setTimeout(() => {
      detectDesktop()
    }, 200)
    
    // 再延迟检测一次（更保险）
    const timer2 = setTimeout(() => {
      detectDesktop()
    }, 500)
    
    // 最后检测一次（确保 Tauri API 完全加载）
    const timer3 = setTimeout(() => {
      detectDesktop()
    }, 1000)
    
    return () => {
      clearTimeout(timer)
      clearTimeout(timer2)
      clearTimeout(timer3)
    }
  }, []) // 只在组件挂载时执行一次

  // 关闭窗口函数（桌面版关闭窗口，浏览器版返回首页）
  const closeWindow = useCallback(async () => {
    if (isDesktop) {
      // 桌面版：关闭窗口
      try {
        const { getCurrentWindow } = await import('@tauri-apps/api/window')
        const appWindow = getCurrentWindow()
        await appWindow.close()
      } catch (error) {
        console.error('Failed to close window:', error)
        // 如果关闭失败，尝试导航回首页
        navigate('/')
      }
    } else {
      // 浏览器版：返回首页
    navigate('/')
    }
  }, [isDesktop, navigate])

  const goBack = closeWindow

  const connectMetaMask = useCallback(async () => {
    try {
      setIsLoading(true)
      setCurrentWallet('metamask')
      setError('')

      if (!walletService.isWalletAvailable()) {
        throw new Error(t('login.error.metamaskNotInstalled'))
      }

      let accounts = []
      try {
        accounts = await walletService.requestAccounts({ forceSelect: true })
      } catch (requestError) {
        if (requestError?.code === 4001) {
          throw requestError
        }
        throw new Error(requestError?.message || t('login.error.getAccountFailed'))
      }

      if (!accounts || accounts.length === 0) {
        throw new Error(t('login.error.noAccountRetrieved'))
      }

      const chainId = await walletService.getCurrentChainId()

      const loginResult = await loginWithWeb3Wallet({
        address: accounts[0],
        chainId,
        walletType: 'metamask',
      })

      // 如果不是桌面环境，尝试同步钱包信息到桌面应用
      if (!isDesktop) {
        try {
          const { syncWalletToDesktop } = await import('@/services/walletSyncService')
          const token = document.cookie
            .split('; ')
            .find(row => row.startsWith('access_token='))
            ?.split('=')[1] || ''
          
          await syncWalletToDesktop({
            address: accounts[0],
            chainId,
            walletType: 'metamask',
            token: loginResult?.token || token,
          })
          
          console.log('[LoginView] Wallet info synced to desktop app')
        } catch (syncError) {
          console.warn('[LoginView] Failed to sync wallet to desktop:', syncError)
          // 同步失败不影响登录流程，继续
        }
      }

      navigate('/')
    } catch (err) {
      if (err?.code === 4001) {
        setError(t('login.error.userRejected'))
      } else if (err?.code === -32002) {
        setError(t('login.error.pendingRequest'))
      } else if (err?.code === -32603) {
        setError(t('login.error.internalError'))
      } else {
        setError(err?.message || t('login.error.connectionFailed'))
      }
    } finally {
      setIsLoading(false)
      setCurrentWallet(null)
    }
  }, [loginWithWeb3Wallet, navigate, t])

  const handleWalletConnect = useCallback((id) => {
    if (isDesktop) {
      // 桌面环境：显示WalletConnect QR码或本地钱包表单
      if (id === 'walletconnect') {
        setConnectionMode('walletconnect')
        setError('')
      } else if (id === 'local') {
        setConnectionMode('local')
        setError('')
      } else {
        setError(t('login.error.notAvailableOnDesktop'))
      }
    } else {
      // 浏览器环境：直接尝试连接 MetaMask（如果可用）
      if (id === 'metamask') {
        connectMetaMask()
      } else {
        setIsLoading(false)
        setCurrentWallet(null)
        if (id === 'walletconnect') {
          setError(t('login.error.walletConnectComingSoon'))
        } else if (id === 'coinbase') {
          setError(t('login.error.coinbaseComingSoon'))
        }
      }
    }
  }, [isDesktop, connectMetaMask, t])

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

  const handleWalletConnected = useCallback(
    async ({ address, chainId, walletType }) => {
      try {
        await loginWithWeb3Wallet({
          address,
          chainId,
          walletType,
        })
        // 连接成功后，不需要重置 connectionMode，直接跳转
        navigate('/')
      } catch (err) {
        setError(err.message || t('login.error.loginFailed'))
      }
    },
    [loginWithWeb3Wallet, navigate, t]
  )

  const handleWalletError = useCallback((err) => {
    setError(err || t('login.error.connectionFailed'))
  }, [t])

  const handleConnectionCancel = useCallback(async () => {
    // 取消时关闭窗口（和右上角关闭按钮共用功能）
    await closeWindow()
  }, [closeWindow])

  const handleNewWalletCreated = useCallback((walletData) => {
    setNewWalletData(walletData)
    // 显示助记词和私钥给用户保存
    // 然后继续登录流程
    handleWalletConnected({
      address: walletData.address,
      chainId: walletData.chainId || '0x1',
      walletType: 'local',
    })
  }, [handleWalletConnected])

  const wallets = useMemo(
    () => {
      const buttons = getWalletButtons(isDesktop, t)
      return buttons.map((button) => ({
        ...button,
        description: typeof button.description === 'function'
          ? button.description(hasMetaMask)
          : button.description,
      }))
    },
    [isDesktop, hasMetaMask, t],
  )

  return (
    <div className={`login-container ${isDarkMode ? 'dark-mode' : 'light-mode'}`}>
      <div className="login-box">
        <button type="button" onClick={goBack} className="close-btn" title={t('common.back')}>
          <img src={CloseIcon} alt="关闭" />
        </button>

        <button
          type="button"
          onClick={toggleTheme}
          className="theme-toggle-btn"
          title={isDarkMode ? t('common.theme.day') : t('common.theme.night')}
        >
          {isDarkMode ? (
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <circle cx="12" cy="12" r="5" />
              <line x1="12" y1="1" x2="12" y2="3" />
              <line x1="12" y1="21" x2="12" y2="23" />
              <line x1="4.22" y1="4.22" x2="5.64" y2="5.64" />
              <line x1="18.36" y1="18.36" x2="19.78" y2="19.78" />
              <line x1="1" y1="12" x2="3" y2="12" />
              <line x1="21" y1="12" x2="23" y2="12" />
              <line x1="4.22" y1="19.78" x2="5.64" y2="18.36" />
              <line x1="18.36" y1="5.64" x2="19.78" y2="4.22" />
            </svg>
          ) : (
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
            </svg>
          )}
        </button>

        <div className="logo">
          <img src={WalletIcon} alt="钱包" className="logo-icon" />
        </div>

        <h1 className="title">{t('login.title')}</h1>
        <p className="subtitle">
          {isDesktop
            ? t('login.subtitle.desktop.new')
            : t('login.subtitle.browser')}
        </p>

        {error && (
          <div className="error-message">
            <span className="error-icon">⚠️</span>
            {error}
          </div>
        )}

        {/* 桌面版：默认显示 WalletConnect 二维码或本地钱包表单 */}
        {/* DEBUG: isDesktop={String(isDesktop)}, connectionMode={connectionMode} */}
        {isDesktop ? (
          <>
            {/* 登录类别切换：加密钱包 / 银行卡 - 银行卡已隐藏 */}
            {/* <div className="login-category-switcher">
              <button
                type="button"
                className={`category-btn ${loginCategory === 'crypto' ? 'active' : ''}`}
                onClick={() => setLoginCategory('crypto')}
              >
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
                  <path d="M7 11V7a5 5 0 0 1 10 0v4" />
                </svg>
                {t('login.category.crypto')}
              </button>
              <button
                type="button"
                className={`category-btn ${loginCategory === 'bankCard' ? 'active' : ''}`}
                onClick={() => setLoginCategory('bankCard')}
              >
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <rect x="2" y="5" width="20" height="14" rx="2" />
                  <line x1="2" y1="10" x2="22" y2="10" />
                </svg>
                {t('login.category.bankCard')}
              </button>
            </div> */}

            {/* 默认显示加密钱包登录 */}
            {/* loginCategory === 'crypto' */}
            <>
              <>
                {connectionMode === 'walletconnect' ? (
                  <WalletConnectQR
                    onConnected={handleWalletConnected}
                    onError={handleWalletError}
                    onCancel={handleConnectionCancel}
                  />
                ) : connectionMode === 'local' ? (
                  <LocalWalletForm
                    onConnected={handleWalletConnected}
                    onError={handleWalletError}
                    onCancel={handleConnectionCancel}
                    onCreateNew={handleNewWalletCreated}
                  />
                ) : (
                  <div className="desktop-notice">
                    <p>{t('login.mode.selectHint.new')}</p>
                  </div>
                )}

                {/* 加密钱包模式切换器 */}
                <div className="desktop-mode-switcher">
                  <button
                    type="button"
                    className={`mode-btn ${connectionMode === 'local' ? 'active' : ''}`}
                    onClick={() => setConnectionMode('local')}
                  >
                    {t('login.mode.localWallet')}
                  </button>
                  <button
                    type="button"
                    className={`mode-btn ${connectionMode === 'walletconnect' ? 'active' : ''}`}
                    onClick={() => setConnectionMode('walletconnect')}
                  >
                    {t('login.mode.phoneScan')}
                  </button>
                </div>
              </>
            ) : (
              /* <BankCardLogin
                onConnected={handleWalletConnected}
                onError={handleWalletError}
                onCancel={handleConnectionCancel}
              /> */
              null
            )}
          </>
        ) : (
          /* 浏览器版：显示钱包选项列表 */
          <>
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
                <p>{t('login.status.metamaskNotDetected')}</p>
                <a
                  href="https://metamask.io/download/"
                  target="_blank"
                  rel="noreferrer"
                  className="install-link"
                >
                  {t('login.help.downloadMetaMask')}
                </a>
              </div>
            )}
            {hasMetaMask && (
              <div className="browser-success">
                <p>{t('login.status.metamaskDetected')}</p>
              </div>
            )}
          </>
        )}

        <div className="security-notice">
          <div className="notice-icon">🔒</div>
          <div className="notice-content">
            <h3>{t('login.security.title')}</h3>
            <ul>
              <li>{t('login.security.tip1')}</li>
              <li>{t('login.security.tip2')}</li>
              <li>{t('login.security.tip3')}</li>
            </ul>
          </div>
        </div>

        {!isDesktop && (
          <div className="help-section">
            <p className="help-text">{t('login.help.noWallet')}</p>
            <a
              href="https://metamask.io/download/"
              target="_blank"
              rel="noreferrer"
              className="help-link"
            >
              {t('login.help.downloadMetaMask')}
            </a>
          </div>
        )}

        <p className="terms">
          {t('login.terms.prefix')}
          <a href="/terms" target="_blank" rel="noreferrer">
            {' '}
            {t('login.terms.termsOfService')}
          </a>
          {t('login.terms.and')}
          <a href="/privacy" target="_blank" rel="noreferrer">
            {' '}
            {t('login.terms.privacyPolicy')}
          </a>
        </p>
      </div>
    </div>
  )
}

export default LoginView
