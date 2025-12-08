import React, { useEffect, useState } from 'react'
import { useNavigate, useLocation } from 'react-router-dom'
import useAuthStore from '@/stores/authStore'
import './WalletSyncCallbackView.css'

const useQuery = () => {
  const { search } = useLocation()
  return React.useMemo(() => new URLSearchParams(search), [search])
}

const WalletSyncCallbackView = () => {
  const navigate = useNavigate()
  const query = useQuery()
  const loginWithWeb3Wallet = useAuthStore((state) => state.loginWithWeb3Wallet)

  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState('')
  const [success, setSuccess] = useState(false)

  useEffect(() => {
    const run = async () => {
      try {
        const address = query.get('address')
        const chainId = query.get('chainId')
        const walletType = query.get('walletType')
        const token = query.get('token')

        if (!address) {
          throw new Error('缺少钱包地址参数')
        }

        console.log('[WalletSyncCallback] Received wallet sync:', {
          address,
          chainId,
          walletType,
          hasToken: !!token,
        })

        // 使用钱包信息登录
        await loginWithWeb3Wallet({
          address,
          chainId: chainId || '0x1',
          walletType: walletType || 'metamask',
        })

        setSuccess(true)
        
        // 延迟跳转，让用户看到成功消息
        setTimeout(() => {
          navigate('/', { replace: true })
        }, 1500)
      } catch (err) {
        console.error('[WalletSyncCallback] Error:', err)
        setError(err?.message || '同步钱包信息失败')
        setIsLoading(false)
      }
    }

    run()
  }, [query, loginWithWeb3Wallet, navigate])

  return (
    <div className="wallet-sync-callback-container">
      <div className="wallet-sync-callback-box">
        {isLoading && !error && !success && (
          <div className="loading">
            <div className="spinner" />
            <p>正在同步钱包信息...</p>
          </div>
        )}

        {success && (
          <div className="success">
            <div className="success-icon">✓</div>
            <h2>钱包信息已同步</h2>
            <p>正在跳转到主页面...</p>
          </div>
        )}

        {!isLoading && error && (
          <div className="error">
            <div className="error-icon">❌</div>
            <h2>同步失败</h2>
            <p>{error}</p>
            <button 
              type="button" 
              onClick={() => navigate('/login')} 
              className="btn-primary"
            >
              返回登录
            </button>
          </div>
        )}
      </div>
    </div>
  )
}

export default WalletSyncCallbackView

