import React, { useEffect, useState } from 'react'
import { QRCodeSVG } from 'qrcode.react'
import { desktopWalletService } from '@/services/desktopWalletService'
import './WalletConnectQR.css'

const WalletConnectQR = ({ onConnected, onError, onCancel }) => {
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
    }

    const initWalletConnect = async () => {
      try {
        setIsConnecting(true)

        // 初始化WalletConnect（会检查是否已经初始化，避免重复）
        provider = await desktopWalletService.initWalletConnect()

        if (!isMounted) return

        // 获取QR URI
        try {
          const uri = await desktopWalletService.getWalletConnectQrUri()
          if (isMounted && uri) {
            setQrUri(uri)
          }
        } catch (uriError) {
          console.error('Failed to get QR URI:', uriError)
          if (isMounted) {
            if (uriError.code === 'WALLETCONNECT_PROJECT_ID_MISSING') {
              onError('WalletConnect 未配置：需要在 .env 文件中设置 VITE_WALLETCONNECT_PROJECT_ID')
            } else {
              onError('生成二维码失败：' + (uriError.message || '未知错误'))
            }
          }
          return
        }

        if (!isMounted || !provider) return

        // 监听QR URI更新（如果URI变化）
        listeners.display_uri = (uri) => {
          if (isMounted) {
            setQrUri(uri)
          }
        }
        provider.on('display_uri', listeners.display_uri)

        // 监听连接成功
        listeners.connect = async () => {
          if (!isMounted) return
          try {
            const accounts = await desktopWalletService.getAccounts()
            if (accounts.length > 0) {
              const address = accounts[0]
              const chainId = await desktopWalletService.getCurrentChainId()

              // 生成验证消息并签名
              const message = desktopWalletService.generateVerificationMessage(address)
              const signature = await desktopWalletService.signMessage(message)

              // 验证签名
              const isValid = await desktopWalletService.verifySignature(
                address,
                message,
                signature
              )

              if (isValid) {
                await desktopWalletService.saveWalletConnection(address, 'walletconnect')
                onConnected({ address, chainId, walletType: 'walletconnect' })
              } else {
                onError('签名验证失败')
              }
            }
          } catch (error) {
            console.error('Connection error:', error)
            if (isMounted) {
              onError(error.message || '连接失败')
            }
          } finally {
            if (isMounted) {
              setIsConnecting(false)
            }
          }
        }
        provider.on('connect', listeners.connect)

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

        // 开始连接流程（获取 QR URI）
        provider.enable().catch((err) => {
          console.error('WalletConnect enable error:', err)
          if (isMounted && err.message !== 'User closed modal') {
            if (err.message?.includes('Unauthorized') || err.message?.includes('invalid key')) {
              onError('WalletConnect Project ID 无效，请在 .env 文件中配置正确的 VITE_WALLETCONNECT_PROJECT_ID')
            } else {
              onError('连接失败：' + err.message)
            }
          }
          if (isMounted) {
            setIsConnecting(false)
          }
        })
      } catch (error) {
        console.error('WalletConnect init error:', error)
        if (isMounted) {
          if (error.code === 'WALLETCONNECT_PROJECT_ID_MISSING') {
            onError('WalletConnect 未配置：需要在 .env 文件中设置 VITE_WALLETCONNECT_PROJECT_ID，访问 https://cloud.walletconnect.com 获取 Project ID')
          } else {
            onError(error.message || '初始化WalletConnect失败')
          }
          setIsConnecting(false)
        }
      }
    }

    initWalletConnect()

    // 清理函数
    return () => {
      isMounted = false
      if (provider) {
        try {
          // 移除监听器，需要提供回调函数引用
          if (typeof provider.off === 'function') {
            // WalletConnect v2 使用 off(event, callback) 移除监听器
            if (listeners.display_uri) provider.off('display_uri', listeners.display_uri)
            if (listeners.connect) provider.off('connect', listeners.connect)
            if (listeners.disconnect) provider.off('disconnect', listeners.disconnect)
            if (listeners.session_event) provider.off('session_event', listeners.session_event)
          } else if (typeof provider.removeAllListeners === 'function') {
            // 兼容旧版本 API
            provider.removeAllListeners('display_uri')
            provider.removeAllListeners('connect')
            provider.removeAllListeners('disconnect')
            provider.removeAllListeners('session_event')
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
      // 获取当前开发服务器地址
      const devUrl = import.meta.env.DEV 
        ? 'http://localhost:1420' 
        : window.location.origin
      
      // 打开浏览器到登录页面
      const loginUrl = `${devUrl}/login`
      
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
      onError(`无法自动打开浏览器，请手动访问：${loginUrl}`)
    } catch (error) {
      console.error('[WalletConnectQR] Failed to open browser:', error)
      const devUrl = import.meta.env.DEV 
        ? 'http://localhost:1420' 
        : window.location.origin
      const loginUrl = `${devUrl}/login`
      onError(`无法打开浏览器：${error.message || '未知错误'}。请手动访问：${loginUrl}`)
    }
  }

  // 移除加载状态，直接显示二维码容器（即使二维码还没生成）

  return (
    <div className="wallet-connect-qr">
      <div className="qr-container">
        <h3>使用移动钱包扫码连接</h3>
        <p className="qr-instructions">
          1. 打开您的移动钱包应用（MetaMask、Trust Wallet等）<br />
          2. 扫描下方二维码<br />
          3. 在钱包中确认连接
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
              <p>正在生成二维码...</p>
            </div>
          </div>
        )}

        <div className="qr-browser-option">
          <div className="qr-browser-hint">
            <div className="qr-browser-icon">🌐</div>
            <div className="qr-browser-text">
              <p className="qr-browser-title">想使用浏览器中的 MetaMask？</p>
              <p className="qr-browser-desc">点击下方按钮在浏览器中打开，浏览器会自动与 MetaMask 插件互动</p>
            </div>
          </div>
        </div>

        <div className="qr-actions">
          <button 
            type="button" 
            onClick={handleOpenBrowser} 
            className="browser-btn"
            title="在浏览器中使用钱包插件登录"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
              <polyline points="15 3 21 3 21 9" />
              <line x1="10" y1="14" x2="21" y2="3" />
            </svg>
            在浏览器中打开
          </button>
          <button type="button" onClick={handleCancel} className="cancel-btn">
            取消
          </button>
        </div>
      </div>
    </div>
  )
}

export default WalletConnectQR

