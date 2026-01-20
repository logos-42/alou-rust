import React, { useEffect, useRef } from 'react'
import { Routes, Route, Navigate, useLocation, useNavigate } from 'react-router-dom'
import useAuthStore from '@/stores/authStore'

import HomeView from '@/views/HomeView'
import LoginView from '@/views/LoginView'
import AuthCallbackView from '@/views/AuthCallbackView'
import WalletSyncCallbackView from '@/views/WalletSyncCallbackView'
import WalletView from '@/views/WalletView'
import AboutView from '@/views/AboutView'
import SubscriptionView from '@/views/SubscriptionView'
import SdkExampleView from '@/routes/SdkExample.jsx'
import AvatarTest from '@/components/AvatarTest'

// 钱包同步监听组件
const WalletSyncListener = () => {
  const navigate = useNavigate()
  const location = useLocation()
  const loginWithWeb3Wallet = useAuthStore((state) => state.loginWithWeb3Wallet)
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated)
  const pollingIntervalRef = useRef(null)

  useEffect(() => {
    // 只在桌面环境且未登录时监听钱包同步
    const isDesktop = typeof window !== 'undefined' && 
                      (window.__TAURI__ !== undefined || 
                       window.__TAURI_IPC__ !== undefined ||
                       (typeof import.meta !== 'undefined' && Boolean(import.meta.env?.TAURI_PLATFORM)))

    if (!isDesktop || isAuthenticated) {
      return
    }

    // 检查URL参数（如果通过回调URL打开）
    const params = new URLSearchParams(location.search)
    const address = params.get('address')
    const chainId = params.get('chainId')
    const walletType = params.get('walletType')
    const token = params.get('token')

    if (address && location.pathname.includes('wallet-sync')) {
      // 从URL参数获取钱包信息并登录
      loginWithWeb3Wallet({
        address,
        chainId: chainId || '0x1',
        walletType: walletType || 'metamask',
      })
        .then(() => {
          console.log('[WalletSyncListener] Wallet synced from browser URL')
          navigate('/', { replace: true })
        })
        .catch((error) => {
          console.error('[WalletSyncListener] Failed to sync wallet from URL:', error)
        })
      return
    }

    // 检查URL参数或文件系统（如果通过深度链接、回调URL或文件系统同步）
    // 如果没有URL参数，持续从文件系统读取
    const checkPendingSync = async () => {
      try {
        const { invoke } = await import('@tauri-apps/api/core')
        
        // 检查文件的函数
        const checkFile = async () => {
          try {
            const data = await invoke('read_wallet_sync_data')
            if (data) {
              const syncData = JSON.parse(data)
              
              // 检查数据是否过期（5分钟内有效）
              const now = Date.now()
              const fiveMinutes = 5 * 60 * 1000
              if (now - syncData.timestamp <= fiveMinutes) {
                console.log('[WalletSyncListener] Found wallet sync data in temp file:', syncData)
                
                try {
                  await loginWithWeb3Wallet({
                    address: syncData.address,
                    chainId: syncData.chainId,
                    walletType: syncData.walletType,
                  })
                  
                  console.log('[WalletSyncListener] Wallet synced successfully from browser')
                  
                  // 停止轮询
                  if (pollingIntervalRef.current) {
                    clearInterval(pollingIntervalRef.current)
                    pollingIntervalRef.current = null
                  }
                  
                  // 跳转到主页
                  navigate('/', { replace: true })
                  return true
                } catch (error) {
                  console.error('[WalletSyncListener] Failed to sync wallet:', error)
                }
              } else {
                console.log('[WalletSyncListener] Wallet sync data expired')
              }
            }
          } catch (error) {
            // 文件不存在，继续轮询
            if (error && typeof error === 'string' && !error.includes('Failed to read sync data')) {
              console.warn('[WalletSyncListener] Error reading sync file:', error)
            }
          }
          return false
        }
        
        // 立即检查一次
        if (await checkFile()) {
          return // 如果立即找到，就不需要设置轮询了
        }
        
        // 持续轮询，每500ms检查一次
        console.log('[WalletSyncListener] Starting continuous polling for wallet sync data (every 500ms)')
        pollingIntervalRef.current = setInterval(async () => {
          const found = await checkFile()
          if (found && pollingIntervalRef.current) {
            clearInterval(pollingIntervalRef.current)
            pollingIntervalRef.current = null
            console.log('[WalletSyncListener] Wallet sync completed, stopped polling')
          }
        }, 500) // 每500ms检查一次
        
        // 也调用 watchForWalletSync 作为备用（URL参数方式）
        try {
          const { watchForWalletSync } = await import('@/services/walletSyncService')
          watchForWalletSync(async (syncData) => {
            console.log('[WalletSyncListener] Received wallet sync from browser (watchForWalletSync):', syncData)
            
            try {
              await loginWithWeb3Wallet({
                address: syncData.address,
                chainId: syncData.chainId,
                walletType: syncData.walletType,
              })
              
              console.log('[WalletSyncListener] Wallet synced successfully from browser')
              
              // 停止轮询
              if (pollingIntervalRef.current) {
                clearInterval(pollingIntervalRef.current)
                pollingIntervalRef.current = null
              }
              
              // 跳转到主页
              navigate('/', { replace: true })
            } catch (error) {
              console.error('[WalletSyncListener] Failed to sync wallet:', error)
            }
          })
        } catch (watchError) {
          console.warn('[WalletSyncListener] watchForWalletSync failed:', watchError)
        }
      } catch (error) {
        console.warn('[WalletSyncListener] Failed to set up wallet sync listener:', error)
      }
    }

    // 如果没有从URL参数获取到地址，持续从文件系统读取
    if (!address) {
      checkPendingSync()
    }
    
    // 清理函数：停止轮询
    return () => {
      if (pollingIntervalRef.current) {
        clearInterval(pollingIntervalRef.current)
        pollingIntervalRef.current = null
        console.log('[WalletSyncListener] Stopped polling for wallet sync data')
      }
    }
  }, [navigate, location, loginWithWeb3Wallet, isAuthenticated])

  return null
}

const ProtectedRoute = ({ children }) => {
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated)
  const init = useAuthStore((state) => state.init)
  const location = useLocation()

  useEffect(() => {
    init()
  }, [init])

  if (!isAuthenticated) {
    return <Navigate to="/login" replace state={{ from: location }} />
  }

  return children
}

const AuthHiddenRoute = ({ children }) => {
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated)
  const init = useAuthStore((state) => state.init)

  useEffect(() => {
    init()
  }, [init])

  if (isAuthenticated) {
    return <Navigate to="/" replace />
  }

  return children
}

const AppRoutes = () => (
  <>
    <WalletSyncListener />
    <Routes>
      <Route path="/" element={<HomeView />} />
    <Route
      path="/wallet"
      element={
        <ProtectedRoute>
          <WalletView />
        </ProtectedRoute>
      }
    />
    <Route path="/about" element={<AboutView />} />
    <Route
      path="/subscription"
      element={
        <ProtectedRoute>
          <SubscriptionView />
        </ProtectedRoute>
      }
    />
    <Route
      path="/login"
      element={
        <AuthHiddenRoute>
          <LoginView />
        </AuthHiddenRoute>
      }
    />
    <Route path="/auth/callback" element={<AuthCallbackView />} />
    <Route path="/wallet-sync" element={<WalletSyncCallbackView />} />
    <Route
      path="/sdk"
      element={
        <ProtectedRoute>
          <SdkExampleView />
        </ProtectedRoute>
      }
    />
    <Route path="/avatar-test" element={<AvatarTest />} />
    <Route path="*" element={<Navigate to="/" replace />} />
    </Routes>
  </>
)

export default AppRoutes
