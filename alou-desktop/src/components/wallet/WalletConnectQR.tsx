import React, { useEffect, useState } from 'react'
import { QRCodeSVG } from 'qrcode.react'
import { desktopWalletService } from '@/services/desktopWalletService'
import { useI18n } from '@/hooks/useI18n'
import './WalletConnectQR.css'

const WalletConnectQR = ({ onConnected, onError, onCancel }) => {
  const { t } = useI18n()
  const [qrUri, setQrUri] = useState('')
  const [isConnecting, setIsConnecting] = useState(false)

  useEffect(() => {
    let isMounted = true
    let provider = null
    // 保存监听器回调函数引用，用于清理
    const listeners = {
      display_uri: null,
      connect: null,
      disconnect: null,
      session_event: null,
      checkInterval: null,
    }

    const initWalletConnect = async () => {
      try {
        setIsConnecting(true)

        // 初始化WalletConnect（会检查是否已经初始化，避免重复）
        provider = await desktopWalletService.initWalletConnect()

        if (!isMounted) return

        // 先设置监听器，再获取QR URI（确保能捕获display_uri事件）
        // 监听QR URI更新（如果URI变化）
        listeners.display_uri = (uri) => {
          console.log('[WalletConnectQR] display_uri event received:', uri)
          if (isMounted && uri) {
            setQrUri(uri)
            setIsConnecting(false)
          }
        }
        provider.on('display_uri', listeners.display_uri)

        // 处理连接成功的函数
        const handleConnectionSuccess = async () => {
          if (!isMounted) return
          
          // 等待一小段时间确保 provider 已初始化
          await new Promise(resolve => setTimeout(resolve, 500))
          
          try {
            console.log('[WalletConnectQR] Connection success, getting accounts...')
            
            // 确保 provider 已创建（通过 connectWalletConnect）
            if (!desktopWalletService.provider) {
              console.log('[WalletConnectQR] Provider not found, calling connectWalletConnect...')
              await desktopWalletService.connectWalletConnect()
            }
            
            const accounts = await desktopWalletService.getAccounts()
            console.log('[WalletConnectQR] Accounts:', accounts)
            
            if (accounts && accounts.length > 0) {
              const address = accounts[0]
              console.log('[WalletConnectQR] Connected address:', address)
              
              const chainId = await desktopWalletService.getCurrentChainId()
              console.log('[WalletConnectQR] Chain ID:', chainId)

              // 生成验证消息并签名
              const message = desktopWalletService.generateVerificationMessage(address)
              console.log('[WalletConnectQR] Verification message generated')
              
              const signature = await desktopWalletService.signMessage(message)
              console.log('[WalletConnectQR] Message signed')

              // 验证签名
              const isValid = await desktopWalletService.verifySignature(
                address,
                message,
                signature
              )
              console.log('[WalletConnectQR] Signature valid:', isValid)

              if (isValid) {
                await desktopWalletService.saveWalletConnection(address, 'walletconnect')
                console.log('[WalletConnectQR] Wallet connection saved, calling onConnected')
                onConnected({ address, chainId, walletType: 'walletconnect' })
              } else {
                console.error('[WalletConnectQR] Signature verification failed')
                onError(t('login.error.signatureVerifyFailed'))
              }
            } else {
              console.warn('[WalletConnectQR] No accounts found, will retry via polling')
              // 不在这里处理，让轮询机制来处理
            }
          } catch (error) {
            console.error('[WalletConnectQR] Connection error:', error)
            if (isMounted) {
              onError(error.message || t('login.error.connectionFailed.prefix') + (error.message || t('common.unknownError')))
            }
          } finally {
            if (isMounted) {
              setIsConnecting(false)
            }
          }
        }

        // 监听连接成功事件（多种事件名称以兼容不同版本）
        listeners.connect = handleConnectionSuccess
        provider.on('connect', listeners.connect)
        
        // 监听 session 事件（WalletConnect v2）
        const sessionProposalHandler = (event) => {
          console.log('[WalletConnectQR] Session proposal event:', event)
        }
        provider.on?.('session_proposal', sessionProposalHandler)
        
        // 监听 session 更新事件
        const sessionRequestHandler = (event) => {
          console.log('[WalletConnectQR] Session request event:', event)
        }
        provider.on?.('session_request', sessionRequestHandler)
        
        // 监听会话建立事件
        const sessionEstablishHandler = async () => {
          console.log('[WalletConnectQR] Session established event received')
          await handleConnectionSuccess()
        }
        provider.on?.('session_establish', sessionEstablishHandler)
        
        // 监听 session_approve 事件（WalletConnect v2）
        const sessionApproveHandler = async (event) => {
          console.log('[WalletConnectQR] Session approved event:', event)
          // 等待一小段时间后处理连接
          setTimeout(() => {
            handleConnectionSuccess()
          }, 1000)
        }
        provider.on?.('session_approve', sessionApproveHandler)
        
        // 监听 accountsChanged 事件（如果存在）
        const accountsChangedHandler = async (accounts) => {
          console.log('[WalletConnectQR] Accounts changed event:', accounts)
          if (accounts && accounts.length > 0) {
            await handleConnectionSuccess()
          }
        }
        provider.on?.('accountsChanged', accountsChangedHandler)
        
        // 添加轮询检查连接状态（作为备用方案）
        let pollAttempts = 0
        const maxPollAttempts = 120 // 最多轮询 120 次（2分钟）
        let connectionHandled = false // 防止重复处理
        
        const checkConnectionInterval = setInterval(async () => {
          if (!isMounted || connectionHandled) {
            if (connectionHandled) {
              clearInterval(checkConnectionInterval)
            }
            return
          }
          
          pollAttempts++
          
          try {
            // 检查是否有活动的 session（多种方式检查）
            const hasSession = provider.session || 
                              (provider.client && provider.client.session) ||
                              (provider.signer && provider.signer.session)
            
            if (hasSession) {
              console.log('[WalletConnectQR] Session detected via polling after', pollAttempts, 'attempts', {
                hasProviderSession: !!provider.session,
                hasClientSession: !!(provider.client && provider.client.session),
                hasSignerSession: !!(provider.signer && provider.signer.session)
              })
              
              connectionHandled = true
              clearInterval(checkConnectionInterval)
              
              // 确保 provider 已创建
              if (!desktopWalletService.provider) {
                console.log('[WalletConnectQR] Creating provider from session...')
                const { BrowserProvider } = await import('ethers')
                desktopWalletService.provider = new BrowserProvider(provider)
                desktopWalletService.walletType = 'walletconnect'
              }
              
              await handleConnectionSuccess()
            }
            
            // 即使没有检测到 session，也尝试直接获取账户（作为备用检查）
            // 从第 3 次轮询开始，每 2 次尝试一次（更频繁）
            if (!connectionHandled && pollAttempts >= 3 && pollAttempts % 2 === 0) {
              try {
                // 先尝试创建 provider（如果还没有）
                if (!desktopWalletService.provider) {
                  console.log('[WalletConnectQR] Creating provider for account check (attempt', pollAttempts, ')')
                  const { BrowserProvider } = await import('ethers')
                  desktopWalletService.provider = new BrowserProvider(provider)
                  desktopWalletService.walletType = 'walletconnect'
                  // 等待一小段时间让 provider 初始化
                  await new Promise(resolve => setTimeout(resolve, 500))
                }
                
                // 尝试获取账户
                const testAccounts = await desktopWalletService.getAccounts()
                if (testAccounts && testAccounts.length > 0) {
                  console.log('[WalletConnectQR] ✅ Accounts found via direct check after', pollAttempts, 'attempts')
                  connectionHandled = true
                  clearInterval(checkConnectionInterval)
                  await handleConnectionSuccess()
                  return
                } else {
                  // 每 10 次尝试打印一次调试信息
                  if (pollAttempts % 10 === 0) {
                    console.log('[WalletConnectQR] No accounts yet, provider state:', {
                      hasProvider: !!desktopWalletService.provider,
                      providerType: desktopWalletService.provider?.constructor?.name,
                      walletType: desktopWalletService.walletType
                    })
                  }
                }
              } catch (accountError) {
                // 账户获取失败是正常的，继续轮询
                // 只在特定次数打印日志，避免日志过多
                if (pollAttempts === 5 || pollAttempts === 15 || pollAttempts === 30 || pollAttempts === 60) {
                  console.log('[WalletConnectQR] Account check error (attempt', pollAttempts, '):', accountError.message)
                }
              }
            }
            
            if (pollAttempts >= maxPollAttempts) {
              console.warn('[WalletConnectQR] Polling timeout after', maxPollAttempts, 'attempts')
              clearInterval(checkConnectionInterval)
              if (isMounted) {
                onError(t('login.error.connectionTimeout'))
              }
            } else if (pollAttempts % 10 === 0) {
              // 每 10 次尝试打印一次日志
              console.log('[WalletConnectQR] Polling for connection... attempt', pollAttempts)
            }
          } catch (error) {
            console.error('[WalletConnectQR] Polling check error:', error)
          }
        }, 1000) // 每秒检查一次
        
        // 保存 interval ID 以便清理
        listeners.checkInterval = checkConnectionInterval

        // 监听断开连接
        listeners.disconnect = () => {
          if (isMounted) {
            setIsConnecting(false)
            setQrUri('')
          }
        }
        provider.on('disconnect', listeners.disconnect)

        // 监听错误
        listeners.session_event = (event) => {
          console.log('Session event:', event)
        }
        provider.on('session_event', listeners.session_event)

        // 获取QR URI（这会触发 enable 并生成 URI）
        try {
          console.log('[WalletConnectQR] Requesting QR URI...')
          const uri = await desktopWalletService.getWalletConnectQrUri()
          console.log('[WalletConnectQR] QR URI received:', uri ? 'URI received' : 'null')
          if (isMounted && uri) {
            setQrUri(uri)
            setIsConnecting(false)
          } else if (isMounted && !uri) {
            // URI 为 null 可能意味着已经连接了，或者还在生成中
            console.warn('[WalletConnectQR] QR URI is null, waiting for display_uri event...')
          }
        } catch (uriError) {
          console.error('Failed to get QR URI:', uriError)
          if (isMounted) {
            setIsConnecting(false)
            if (uriError.code === 'WALLETCONNECT_PROJECT_ID_MISSING') {
              onError(t('login.error.walletConnectNotConfigured'))
            } else if (uriError.message?.includes('timeout')) {
              onError(t('login.error.qrCodeTimeout'))
            } else if (uriError.message?.includes('WebSocket') || uriError.message?.includes('connection')) {
              onError(t('login.error.wsConnectionFailed'))
            } else {
              onError(t('login.error.qrCodeGenerationFailed', { error: uriError.message || t('common.unknownError') }))
            }
          }
        }
      } catch (error) {
        console.error('WalletConnect init error:', error)
        if (isMounted) {
          if (error.code === 'WALLETCONNECT_PROJECT_ID_MISSING') {
            onError(t('login.error.walletConnectProjectIdMissing'))
          } else {
            onError(error.message || t('login.error.initWalletConnectFailed'))
          }
          setIsConnecting(false)
        }
      }
    }

    initWalletConnect()

    // 清理函数
    return () => {
      isMounted = false
      
      // 清除轮询检查
      if (listeners.checkInterval) {
        clearInterval(listeners.checkInterval)
      }
      
      if (provider) {
        try {
          // 移除监听器，需要提供回调函数引用
          if (typeof provider.off === 'function') {
            // WalletConnect v2 使用 off(event, callback) 移除监听器
            if (listeners.display_uri) provider.off('display_uri', listeners.display_uri)
            if (listeners.connect) provider.off('connect', listeners.connect)
            if (listeners.disconnect) provider.off('disconnect', listeners.disconnect)
            if (listeners.session_event) provider.off('session_event', listeners.session_event)
            provider.off?.('session_proposal', () => {})
            provider.off?.('session_request', () => {})
            provider.off?.('session_establish', () => {})
          } else if (typeof provider.removeAllListeners === 'function') {
            // 兼容旧版本 API
            provider.removeAllListeners('display_uri')
            provider.removeAllListeners('connect')
            provider.removeAllListeners('disconnect')
            provider.removeAllListeners('session_event')
            provider.removeAllListeners('session_proposal')
            provider.removeAllListeners('session_request')
            provider.removeAllListeners('session_establish')
          } else if (typeof provider.removeListener === 'function') {
            // 如果只有 removeListener，需要提供回调函数
            if (listeners.display_uri) provider.removeListener('display_uri', listeners.display_uri)
            if (listeners.connect) provider.removeListener('connect', listeners.connect)
            if (listeners.disconnect) provider.removeListener('disconnect', listeners.disconnect)
            if (listeners.session_event) provider.removeListener('session_event', listeners.session_event)
          }
        } catch (error) {
          console.warn('Error removing WalletConnect listeners:', error)
        }
      }
    }
  }, [onConnected, onError])

  const handleCancel = () => {
    desktopWalletService.disconnect().catch(console.error)
    onCancel()
  }

  const handleOpenBrowser = async () => {
    try {
      // 获取登录页面 URL
      // 优先使用环境变量配置的生产 URL，然后是开发服务器地址
      const productionUrl = import.meta.env.VITE_APP_URL || import.meta.env.VITE_BASE_URL || 'https://alou.onl'
      const devUrl = import.meta.env.DEV ? 'http://localhost:1420' : productionUrl
      
      // 打开浏览器到登录页面
      const loginUrl = `${devUrl}/login`
      
      console.log('[WalletConnectQR] Opening browser with URL:', loginUrl, {
        isDev: import.meta.env.DEV,
        productionUrl,
        windowOrigin: window.location.origin,
        envAppUrl: import.meta.env.VITE_APP_URL,
        envBaseUrl: import.meta.env.VITE_BASE_URL
      })
      
      console.log('[WalletConnectQR] Opening browser:', loginUrl, {
        isDesktop: desktopWalletService.isDesktop(),
        hasTauri: typeof window !== 'undefined' && (window.__TAURI__ || window.__TAURI_IPC__),
      })
      
      // 首先尝试使用 Tauri 命令（如果在 Tauri 环境中）
      if (typeof window !== 'undefined') {
        // 检查是否在 Tauri 环境中（即使 window.__TAURI__ 未加载，也尝试调用）
        try {
          const { invoke } = await import('@tauri-apps/api/core')
          await invoke('open_browser', { url: loginUrl })
          console.log('[WalletConnectQR] Browser opened via Tauri invoke')
          return
        } catch (tauriError) {
          console.warn('[WalletConnectQR] Tauri invoke failed, trying other methods:', tauriError)
          // 继续尝试其他方法
        }
      }
      
      // 备用方案 1：尝试使用 window.open
      if (typeof window !== 'undefined' && window.open) {
        try {
          const newWindow = window.open(loginUrl, '_blank', 'noopener,noreferrer')
          if (newWindow) {
            console.log('[WalletConnectQR] Browser opened via window.open')
            return
          } else {
            console.warn('[WalletConnectQR] window.open returned null, may be blocked by Tauri')
          }
        } catch (openError) {
          console.error('[WalletConnectQR] window.open error:', openError)
        }
      }
      
      // 备用方案 2：创建链接元素并点击（更可靠的方式）
      if (typeof document !== 'undefined') {
        try {
          const link = document.createElement('a')
          link.href = loginUrl
          link.target = '_blank'
          link.rel = 'noopener noreferrer'
          document.body.appendChild(link)
          link.click()
          document.body.removeChild(link)
          console.log('[WalletConnectQR] Browser opened via link click')
          return
        } catch (linkError) {
          console.error('[WalletConnectQR] Link click failed:', linkError)
        }
      }
      
      // 如果所有方法都失败，显示错误和链接
      onError(t('login.error.cannotOpenBrowser') + loginUrl)
    } catch (error) {
      console.error('[WalletConnectQR] Failed to open browser:', error)
      const productionUrl = import.meta.env.VITE_APP_URL || import.meta.env.VITE_BASE_URL || 'https://alou.onl'
      const devUrl = import.meta.env.DEV ? 'http://localhost:1420' : productionUrl
      const loginUrl = `${devUrl}/login`
      onError(t('login.error.cannotOpenBrowserPrefix') + (error.message || t('common.unknownError')) + '. ' + t('login.error.cannotOpenBrowser') + loginUrl)
    }
  }

  // 移除加载状态，直接显示二维码容器（即使二维码还没生成）

  return (
    <div className="wallet-connect-qr">
      <div className="qr-container">
        <h3>{t('login.walletconnect.scanTitle')}</h3>
        <p className="qr-instructions">
          {t('login.walletconnect.step1')}<br />
          {t('login.walletconnect.step2')}<br />
          {t('login.walletconnect.step3')}
        </p>

        {qrUri ? (
          <div className="qr-code-wrapper">
            <QRCodeSVG value={qrUri} size={256} level="M" />
          </div>
        ) : (
          <div className="qr-code-wrapper qr-placeholder">
            <div className="qr-placeholder-content">
              <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
                <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
                <rect x="7" y="7" width="10" height="10" />
                <path d="M7 3v4M17 3v4M3 7h4M3 17h4M21 7h-4M21 17h-4M7 21v-4M17 21v-4" />
              </svg>
              <p>{t('login.walletconnect.generating')}</p>
            </div>
          </div>
        )}

        <div className="qr-browser-option">
          <div className="qr-browser-hint">
            <div className="qr-browser-icon">🌐</div>
            <div className="qr-browser-text">
              <p className="qr-browser-title">{t('login.walletconnect.browserOption.title')}</p>
              <p className="qr-browser-desc">{t('login.walletconnect.browserOption.desc')}</p>
            </div>
          </div>
        </div>

        <div className="qr-actions">
          <button 
            type="button" 
            onClick={handleOpenBrowser} 
            className="browser-btn"
            title={t('login.walletconnect.openInBrowserTitle')}
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
              <polyline points="15 3 21 3 21 9" />
              <line x1="10" y1="14" x2="21" y2="3" />
            </svg>
            {t('login.walletconnect.openInBrowser')}
          </button>
          <button type="button" onClick={handleCancel} className="cancel-btn">
            {t('common.cancel')}
          </button>
        </div>
      </div>
    </div>
  )
}

export default WalletConnectQR

