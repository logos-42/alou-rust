// ============================================
// Authentication Store - Pinia
// ============================================

import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import Cookies from 'js-cookie'
import { authService } from '@/services/auth.service'
import { userService } from '@/services/user.service'
import type { User } from '@/types/auth'

export const useAuthStore = defineStore('auth', () => {
  // State
  const user = ref<User | null>(null)
  const isAuthenticated = ref(false)
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  // Getters
  const currentUser = computed(() => user.value)
  const userName = computed(() => user.value?.name || user.value?.email || 'User')
  const userAvatar = computed(() => user.value?.avatar_url)

  // Actions

  /**
   * Initialize auth state from cookies
   */
  async function init() {
    const token = Cookies.get('access_token')
    if (token) {
      // 检查是否是钱包登录
      const walletAddress = localStorage.getItem('wallet_address')
      if (walletAddress && (token.startsWith('wallet_') || token.startsWith('web3_'))) {
        // 恢复钱包登录状态
        user.value = {
          id: walletAddress,
          email: walletAddress + '@wallet.local',
          name: walletAddress.slice(0, 6) + '...' + walletAddress.slice(-4),
          avatar_url: `https://api.dicebear.com/7.x/identicon/svg?seed=${walletAddress}`,
          created_at: new Date().toISOString()
        }
        isAuthenticated.value = true
      } else {
        await checkAuth()
      }
    }
  }

  /**
   * Login with Web3 wallet (MetaMask, WalletConnect, etc.)
   */
  async function loginWithWeb3Wallet(walletInfo: { 
    address: string
    chainId: string
    walletType: string
  }) {
    try {
      isLoading.value = true
      error.value = null

      const { address, chainId, walletType } = walletInfo

      // 创建用户信息
      const mockToken = 'web3_' + Date.now() + '_' + Math.random().toString(36).substr(2, 9)
      const mockUser: User = {
        id: address.toLowerCase(),
        email: address.toLowerCase() + '@wallet.local',
        name: address.slice(0, 6) + '...' + address.slice(-4),
        avatar_url: `https://api.dicebear.com/7.x/identicon/svg?seed=${address}`,
        created_at: new Date().toISOString()
      }

      // 保存token和用户信息
      Cookies.set('access_token', mockToken, { expires: 7 })
      localStorage.setItem('wallet_address', address)
      localStorage.setItem('wallet_type', walletType)
      localStorage.setItem('wallet_chain_id', chainId)
      
      user.value = mockUser
      isAuthenticated.value = true
      
      // 触发钱包切换事件
      window.dispatchEvent(new CustomEvent('wallet-changed', { 
        detail: { address: address.toLowerCase() } 
      }))

      return { user: mockUser, token: mockToken }
    } catch (err: any) {
      error.value = err.message || 'Web3钱包登录失败'
      throw err
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Login with crypto wallet (private key or mnemonic) - Legacy method
   */
  async function loginWithWallet(credentials: { privateKey?: string; mnemonic?: string }) {
    try {
      isLoading.value = true
      error.value = null

      // 在实际应用中，这里应该使用Web3库（如ethers.js）来验证钱包凭证
      // 并生成钱包地址作为用户标识
      
      // 暂时模拟登录成功，将凭证存储在localStorage中
      // 注意：在生产环境中不应该直接存储私钥或助记词
      
      let address = ''
      if (credentials.privateKey) {
        // 这里应该使用ethers.js从私钥派生地址
        // import { ethers } from 'ethers'
        // const wallet = new ethers.Wallet(credentials.privateKey)
        // address = wallet.address
        
        // 临时模拟地址生成
        address = '0x' + credentials.privateKey.slice(-40)
        localStorage.setItem('wallet_type', 'privateKey')
      } else if (credentials.mnemonic) {
        // 这里应该使用ethers.js从助记词派生地址
        // const wallet = ethers.Wallet.fromMnemonic(credentials.mnemonic)
        // address = wallet.address
        
        // 临时模拟地址生成
        const words = credentials.mnemonic.split(/\s+/)
        address = '0x' + words.join('').slice(0, 40).padEnd(40, '0')
        localStorage.setItem('wallet_type', 'mnemonic')
      } else {
        throw new Error('需要提供私钥或助记词')
      }

      // 创建模拟的token和用户信息
      const mockToken = 'wallet_' + Date.now() + '_' + Math.random().toString(36).substr(2, 9)
      const mockUser: User = {
        id: address,
        email: address + '@wallet.local',
        name: address.slice(0, 6) + '...' + address.slice(-4),
        avatar_url: `https://api.dicebear.com/7.x/identicon/svg?seed=${address}`,
        created_at: new Date().toISOString()
      }

      // 保存token和用户信息
      Cookies.set('access_token', mockToken, { expires: 7 })
      localStorage.setItem('wallet_address', address)
      
      user.value = mockUser
      isAuthenticated.value = true
      
      // 触发钱包切换事件
      window.dispatchEvent(new CustomEvent('wallet-changed', { 
        detail: { address: address.toLowerCase() } 
      }))

      return { user: mockUser, token: mockToken }
    } catch (err: any) {
      error.value = err.message || '钱包登录失败'
      throw err
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Start Google OAuth login flow
   */
  async function loginWithGoogle() {
    try {
      isLoading.value = true
      error.value = null

      const { auth_url, state } = await authService.getGoogleLoginUrl()

      // Store state in sessionStorage for verification
      sessionStorage.setItem('oauth_state', state)

      // Redirect to Google
      window.location.href = auth_url
    } catch (err: any) {
      error.value = err.response?.data?.message || 'Failed to initiate login'
      throw err
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Handle Google OAuth callback
   */
  async function handleGoogleCallback(code: string, state: string) {
    try {
      isLoading.value = true
      error.value = null

      // Verify state to prevent CSRF
      const savedState = sessionStorage.getItem('oauth_state')
      if (savedState !== state) {
        throw new Error('Invalid state parameter')
      }

      // Exchange code for tokens
      const authResponse = await authService.handleGoogleCallback(code, state)

      // Save tokens
      Cookies.set('access_token', authResponse.access_token, { expires: 1 })
      Cookies.set('refresh_token', authResponse.refresh_token, { expires: 30 })

      // Set user
      user.value = authResponse.user
      isAuthenticated.value = true

      // Clean up
      sessionStorage.removeItem('oauth_state')

      return authResponse
    } catch (err: any) {
      error.value = err.response?.data?.message || 'Login failed'
      throw err
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Check authentication status
   */
  async function checkAuth(): Promise<boolean> {
    try {
      const token = Cookies.get('access_token')
      if (!token) {
        isAuthenticated.value = false
        user.value = null
        return false
      }

      const result = await authService.verifyToken()
      if (result.valid && result.user) {
        user.value = result.user
        isAuthenticated.value = true
        return true
      } else {
        isAuthenticated.value = false
        user.value = null
        return false
      }
    } catch (err) {
      isAuthenticated.value = false
      user.value = null
      return false
    }
  }

  /**
   * Fetch current user info
   */
  async function fetchUser() {
    try {
      isLoading.value = true
      const { user: userData } = await userService.getCurrentUser()
      user.value = userData
      isAuthenticated.value = true
    } catch (err: any) {
      error.value = err.response?.data?.message || 'Failed to fetch user'
      throw err
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Update user profile
   */
  async function updateProfile(data: { name?: string; avatar_url?: string }) {
    try {
      isLoading.value = true
      error.value = null

      const { user: updatedUser } = await userService.updateProfile(data)
      user.value = updatedUser

      return updatedUser
    } catch (err: any) {
      error.value = err.response?.data?.message || 'Failed to update profile'
      throw err
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Logout
   */
  async function logout() {
    try {
      const refreshToken = Cookies.get('refresh_token')
      if (refreshToken) {
        await authService.logout(refreshToken)
      }
    } catch (err) {
      console.error('Logout error:', err)
    } finally {
      // Clear state regardless of API call result
      user.value = null
      isAuthenticated.value = false
      Cookies.remove('access_token')
      Cookies.remove('refresh_token')
      localStorage.removeItem('wallet_address')
      localStorage.removeItem('wallet_type')
      localStorage.removeItem('wallet_chain_id')
    }
  }

  /**
   * Clear error
   */
  function clearError() {
    error.value = null
  }

  return {
    // State
    user,
    isAuthenticated,
    isLoading,
    error,
    // Getters
    currentUser,
    userName,
    userAvatar,
    // Actions
    init,
    loginWithWeb3Wallet,
    loginWithWallet,
    loginWithGoogle,
    handleGoogleCallback,
    checkAuth,
    fetchUser,
    updateProfile,
    logout,
    clearError,
  }
})

