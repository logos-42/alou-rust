import React, { useEffect, useState } from 'react'
import { useNavigate, useLocation } from 'react-router-dom'
import useAuthStore from '@/stores/authStore'
import './AuthCallbackView.css'

/**
 * 解析 URL 查询参数的 Hook
 */
const useQuery = () => {
  const { search } = useLocation()
  return React.useMemo(() => new URLSearchParams(search), [search])
}

/**
 * AuthCallbackView 组件
 * 处理 OAuth 认证回调，展示登录状态
 */
const AuthCallbackView: React.FC = () => {
  const navigate = useNavigate()
  const query = useQuery()
  const handleGoogleCallback = useAuthStore((state: any) => state.handleGoogleCallback)

  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState('')

  useEffect(() => {
    const run = async () => {
      try {
        const code = query.get('code')
        const state = query.get('state')

        if (!code || !state) {
          throw new Error('缺少必要的认证参数')
        }

        await handleGoogleCallback(code, state)
        setTimeout(() => navigate('/', { replace: true }), 1000)
      } catch (err) {
        setError((err as Error)?.message || '认证失败，请重试')
        setIsLoading(false)
      }
    }

    run()
  }, [handleGoogleCallback, navigate, query])

  const goToLogin = () => {
    navigate('/login')
  }

  return (
    <div className="callback-container">
      <div className="callback-box">
        {isLoading && !error && (
          <div className="loading">
            <div className="spinner" />
            <p>正在登录...</p>
          </div>
        )}

        {!isLoading && error && (
          <div className="error">
            <div className="error-icon">❌</div>
            <h2>登录失败</h2>
            <p>{error}</p>
            <button type="button" onClick={goToLogin} className="btn-primary">
              返回登录
            </button>
          </div>
        )}

        {isLoading && !error && (
          <div className="success">
            <div className="success-icon">✓</div>
            <h2>登录成功</h2>
            <p>正在跳转...</p>
          </div>
        )}
      </div>
    </div>
  )
}

export default AuthCallbackView
