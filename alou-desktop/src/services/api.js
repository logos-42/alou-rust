// ============================================
// API Service - Axios instance configuration
// ============================================

import axios from 'axios'
import Cookies from 'js-cookie'

// API base URL - use local dev server in development
// 在开发环境下，使用Vite代理（当前开发服务器地址）来解决CORS问题
// 强制在开发模式下使用空字符串，确保通过Vite代理
const API_BASE_URL = import.meta.env.DEV ? '' : 'https://alou-edge.yuanjieliu65.workers.dev'

// Debug logging
console.log('[API] Environment:', {
  VITE_API_BASE_URL: import.meta.env.VITE_API_BASE_URL,
  DEV: import.meta.env.DEV,
  MODE: import.meta.env.MODE,
  API_BASE_URL: API_BASE_URL,
  baseURL: API_BASE_URL ? `${API_BASE_URL}/api` : '/api'
})

// Create axios instance
const apiClient = axios.create({
  baseURL: API_BASE_URL ? `${API_BASE_URL}/api` : '/api',
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
})

// Request interceptor - add auth token
apiClient.interceptors.request.use(
  (config) => {
    const token = Cookies.get('access_token')
    if (token && config.headers) {
      config.headers.Authorization = `Bearer ${token}`
    }
    return config
  },
  (error) => {
    return Promise.reject(error)
  },
)

// Response interceptor - handle errors
apiClient.interceptors.response.use(
  (response) => {
    return response
  },
  async (error) => {
    const originalRequest = error.config

    // 静默处理 404 错误（后端可能未实现某些端点）
    if (error.response?.status === 404) {
      // 对于未实现的端点，静默处理，不输出错误
      // 让调用方决定如何处理（通常会回退到本地模拟）
      // 注意：浏览器控制台仍会显示 404，这是网络层面的，无法完全阻止
      // 但我们可以通过端点检查机制来减少不必要的请求
      return Promise.reject(error)
    }

    // Check if it's a connection refused error
    const isConnectionError = 
      error.code === 'ECONNREFUSED' || 
      error.code === 'ERR_NETWORK' ||
      error.code === 'ETIMEDOUT' ||
      error.message?.includes('ERR_CONNECTION_REFUSED') ||
      error.message?.includes('Failed to fetch') ||
      error.message?.includes('ETIMEDOUT') ||
      !error.response

    // Only log connection errors once per endpoint to avoid spam
    if (isConnectionError) {
      const errorKey = `${error.config?.method || 'unknown'}_${error.config?.url || 'unknown'}`
      const lastErrorTime = window.__lastApiError?.[errorKey] || 0
      const now = Date.now()
      
      // Only log if it's been more than 5 seconds since last error for this endpoint
      if (now - lastErrorTime > 5000) {
        if (!window.__lastApiError) {
          window.__lastApiError = {}
        }
        window.__lastApiError[errorKey] = now
        
        console.warn(`[API] Connection to backend server failed (${API_BASE_URL}). Make sure the backend server is running or set VITE_API_BASE_URL environment variable.`)
        console.warn(`[API] Error details:`, {
          code: error.code,
          message: error.message,
          url: error.config?.url,
          method: error.config?.method,
        })
      }
      
      // For connection errors, don't throw detailed errors for health checks
      if (error.config?.url?.includes('/health')) {
        return Promise.reject(new Error('Backend server unavailable'))
      }
      
      // 对于连接错误，返回一个特殊的错误对象，让调用方知道是网络问题
      const networkError = new Error('Network error: Cannot connect to backend server')
      networkError.isNetworkError = true
      networkError.originalError = error
      networkError.endpoint = error.config?.url
      return Promise.reject(networkError)
    }

    // If 401 and not already retried, try to refresh token
    if (error.response?.status === 401 && originalRequest && !originalRequest._retry) {
      originalRequest._retry = true

      try {
        const refreshToken = Cookies.get('refresh_token')
        if (refreshToken) {
        const response = await axios.post(API_BASE_URL ? `${API_BASE_URL}/api/auth/refresh` : '/api/auth/refresh', {
          refresh_token: refreshToken,
        })

          const { access_token } = response.data
          Cookies.set('access_token', access_token, { expires: 1 })

          // Retry original request with new token
          if (!originalRequest.headers) {
            originalRequest.headers = {}
          }
          originalRequest.headers.Authorization = `Bearer ${access_token}`
          return apiClient(originalRequest)
        }
      } catch (refreshError) {
        // Refresh failed, clear tokens and redirect to login
        Cookies.remove('access_token')
        Cookies.remove('refresh_token')
        window.location.href = '/login'
        return Promise.reject(refreshError)
      }
    }

    return Promise.reject(error)
  },
)

// API test functions
export const apiService = {
  // Test API connection
  async testApiConnection(apiKey, provider, model) {
    try {
      const response = await apiClient.post('/user/config/verify', {
        api_key: apiKey,
        provider,
        model,
      })
      return {
        success: true,
        data: response.data,
      }
    } catch (error) {
      console.error('[API Service] Test connection failed:', error)
      
      // Handle specific error cases
      if (error.response?.status === 404) {
        return {
          success: false,
          error: '验证端点未实现，请确保后端服务正在运行',
          details: error.message,
        }
      }
      
      if (error.code === 'ECONNREFUSED' || error.code === 'ERR_NETWORK') {
        return {
          success: false,
          error: '无法连接到后端服务器，请检查网络连接',
          details: error.message,
        }
      }
      
      return {
        success: false,
        error: error.response?.data?.error || error.message || '未知错误',
        details: error.response?.data,
      }
    }
  },

  // Save API configuration
  async saveApiConfig(apiKey, provider, model) {
    try {
      const response = await apiClient.post('/user/config', {
        api_key: apiKey,
        provider,
        model,
      })
      return {
        success: true,
        data: response.data,
      }
    } catch (error) {
      console.error('[API Service] Save config failed:', error)
      return {
        success: false,
        error: error.response?.data?.error || error.message || '保存配置失败',
        details: error.response?.data,
      }
    }
  },

  // Get API configuration
  async getApiConfig() {
    try {
      const response = await apiClient.get('/user/config')
      return {
        success: true,
        data: response.data,
      }
    } catch (error) {
      // 如果是 401 错误，用户未认证，返回默认配置
      if (error.response?.status === 401) {
        console.log('[API Service] User not authenticated, returning default config')
        return {
          success: true,
          data: {
            provider: 'deepseek',
            model: 'deepseek-chat',
            has_api_key: false,
            updated_at: 0,
          },
        }
      }
      
      console.error('[API Service] Get config failed:', error)
      return {
        success: false,
        error: error.response?.data?.error || error.message || '获取配置失败',
        details: error.response?.data,
      }
    }
  },
}

export default apiClient
