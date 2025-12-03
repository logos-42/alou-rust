import React, { useCallback, useEffect, useMemo, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import useAuthStore from '@/stores/authStore'
import { walletService } from '@/services/walletService'
import { desktopWalletService } from '@/services/desktopWalletService'
import WalletConnectQR from '@/components/wallet/WalletConnectQR'
import LocalWalletForm from '@/components/wallet/LocalWalletForm'
import './LoginView.css'

const getWalletButtons = (isDesktop) => {
  if (isDesktop) {
    // 桌面版钱包选项
    return [
      {
        id: 'walletconnect',
        name: 'WalletConnect',
        description: () => '扫码连接移动钱包',
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
        name: '本地钱包',
        description: () => '导入私钥或创建新钱包',
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
  // 桌面版默认显示 WalletConnect 二维码，浏览器版为 null
  const [connectionMode, setConnectionMode] = useState(() => {
    // 根据 isDesktop 的初始值设置
    return isDesktop ? 'walletconnect' : null
  })
  const [newWalletData, setNewWalletData] = useState(null) // 用于存储新创建的钱包数据

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
          // 如果是桌面环境，确保默认连接模式为 WalletConnect
          if (desktop) {
            setConnectionMode((prevMode) => {
              if (prevMode !== 'walletconnect' && prevMode !== 'local') {
                console.log('[LoginView] Setting connection mode to walletconnect')
                return 'walletconnect'
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
        setError('该连接方式在桌面版不可用，请使用 WalletConnect 或本地钱包')
      }
    } else {
      // 浏览器环境：直接尝试连接 MetaMask（如果可用）
      if (id === 'metamask') {
        connectMetaMask()
      } else {
        setIsLoading(false)
        setCurrentWallet(null)
        if (id === 'walletconnect') {
          setError('WalletConnect 功能即将推出，请使用 MetaMask 浏览器插件')
        } else if (id === 'coinbase') {
          setError('Coinbase Wallet 功能即将推出，请使用 MetaMask 浏览器插件')
        }
      }
    }
  }, [isDesktop, connectMetaMask])

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
        setError(err.message || '登录失败')
      }
    },
    [loginWithWeb3Wallet, navigate]
  )

  const handleWalletError = useCallback((err) => {
    setError(err || '连接钱包失败')
  }, [])

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
      const buttons = getWalletButtons(isDesktop)
      return buttons.map((button) => ({
        ...button,
        description: typeof button.description === 'function'
          ? button.description(hasMetaMask)
          : button.description,
      }))
    },
    [isDesktop, hasMetaMask],
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
          <div className="logo-icon">💰</div>
        </div>

        <h1 className="title">连接钱包</h1>
        <p className="subtitle">
          {isDesktop 
            ? '使用手机钱包扫码连接，或在浏览器中使用钱包插件（桌面版）' 
            : '选择您的加密钱包以安全登录（浏览器版）'}
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
              // 桌面版默认回退：显示切换器（理论上不应该走到这里，因为初始化时已经设置了 connectionMode）
              <>
                <div className="desktop-mode-switcher">
                  <button
                    type="button"
                    className={`mode-btn active`}
                    onClick={() => setConnectionMode('walletconnect')}
                  >
                    手机扫码
                  </button>
                  <button
                    type="button"
                    className="mode-btn"
                    onClick={() => setConnectionMode('local')}
                  >
                    本地钱包
                  </button>
                </div>

                <div className="desktop-notice">
                  <p>💡 请选择连接方式：WalletConnect（手机扫码）或本地钱包</p>
                </div>
              </>
            )}

            {/* 桌面版模式切换器：在所有模式下显示，方便切换 */}
            <div className="desktop-mode-switcher">
              <button
                type="button"
                className={`mode-btn ${connectionMode === 'walletconnect' ? 'active' : ''}`}
                onClick={() => setConnectionMode('walletconnect')}
              >
                手机扫码
              </button>
              <button
                type="button"
                className={`mode-btn ${connectionMode === 'local' ? 'active' : ''}`}
                onClick={() => setConnectionMode('local')}
              >
                本地钱包
              </button>
            </div>
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
          </>
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

        {!isDesktop && (
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
        )}

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
