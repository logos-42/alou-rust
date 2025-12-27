import { create } from 'zustand'
import Cookies from 'js-cookie'
import { authService } from '@/services/authService'
import { userService } from '@/services/userService'

const buildWalletUser = (address) => {
  const normalized = address ? address.toLowerCase() : ''
  return {
    id: normalized,
    email: `${normalized}@wallet.local`,
    name: `${normalized.slice(0, 6)}...${normalized.slice(-4)}`,
    avatar_url: `https://api.dicebear.com/7.x/identicon/svg?seed=${normalized}`,
    created_at: new Date().toISOString(),
  }
}

const persistWalletInfo = ({ address, walletType, chainId }) => {
  if (typeof window === 'undefined') return
  if (address) {
    localStorage.setItem('wallet_address', address)
  }
  if (walletType) {
    localStorage.setItem('wallet_type', walletType)
  }
  if (chainId) {
    localStorage.setItem('wallet_chain_id', chainId)
  }
}

const clearWalletInfo = () => {
  if (typeof window === 'undefined') return
  localStorage.removeItem('wallet_address')
  localStorage.removeItem('wallet_type')
  localStorage.removeItem('wallet_chain_id')
}

const dispatchWalletChanged = (address) => {
  if (typeof window === 'undefined') return
  window.dispatchEvent(
    new CustomEvent('wallet-changed', {
      detail: { address: address ? address.toLowerCase() : undefined },
    }),
  )
}

const initialState = {
  user: null,
  isAuthenticated: false,
  isLoading: false,
  error: null,
}

const useAuthStore = create((set, get) => ({
  ...initialState,

  currentUser: () => get().user,
  userName: () => {
    const user = get().user
    return user?.name || user?.email || 'User'
  },
  userAvatar: () => get().user?.avatar_url,

  init: async () => {
    const token = Cookies.get('access_token')
    if (!token) {
      return
    }

    const walletAddress =
      typeof window !== 'undefined' ? localStorage.getItem('wallet_address') : null

    if (walletAddress && (token.startsWith('wallet_') || token.startsWith('web3_'))) {
      const walletUser = buildWalletUser(walletAddress)
      set({
        user: walletUser,
        isAuthenticated: true,
      })
    } else {
      await get().checkAuth()
    }
  },

  loginWithWeb3Wallet: async ({ address, chainId, walletType }) => {
    set({ isLoading: true, error: null })
    try {
      const mockToken = `web3_${Date.now()}_${Math.random().toString(36).slice(2, 11)}`
      const mockUser = buildWalletUser(address)

      Cookies.set('access_token', mockToken, { expires: 7 })
      persistWalletInfo({ address, walletType, chainId })

      set({
        user: mockUser,
        isAuthenticated: true,
      })

      dispatchWalletChanged(address)
      return { user: mockUser, token: mockToken }
    } catch (error) {
      const message = error?.message || (typeof error === 'string' ? error : 'Web3钱包登录失败')
      set({ error: message })
      throw error
    } finally {
      set({ isLoading: false })
    }
  },

  loginWithWallet: async ({ privateKey, mnemonic }) => {
    set({ isLoading: true, error: null })
    try {
      let address = ''
      if (privateKey) {
        address = `0x${privateKey.slice(-40)}`
        persistWalletInfo({ address, walletType: 'privateKey' })
      } else if (mnemonic) {
        const words = mnemonic.split(/\s+/)
        address = `0x${words.join('').slice(0, 40).padEnd(40, '0')}`
        persistWalletInfo({ address, walletType: 'mnemonic' })
      } else {
        throw new Error('需要提供私钥或助记词')
      }

      const mockToken = `wallet_${Date.now()}_${Math.random().toString(36).slice(2, 11)}`
      const mockUser = buildWalletUser(address)

      Cookies.set('access_token', mockToken, { expires: 7 })
      set({
        user: mockUser,
        isAuthenticated: true,
      })

      dispatchWalletChanged(address)
      return { user: mockUser, token: mockToken }
    } catch (error) {
      const message = error?.message || (typeof error === 'string' ? error : '钱包登录失败')
      set({ error: message })
      throw error
    } finally {
      set({ isLoading: false })
    }
  },

  loginWithGoogle: async () => {
    set({ isLoading: true, error: null })
    try {
      const { auth_url, state } = await authService.getGoogleLoginUrl()
      if (typeof window !== 'undefined') {
        sessionStorage.setItem('oauth_state', state)
        window.location.href = auth_url
      }
    } catch (error) {
      const message = error?.response?.data?.message || error?.message || 'Failed to initiate login'
      set({ error: message })
      throw error
    } finally {
      set({ isLoading: false })
    }
  },

  handleGoogleCallback: async (code, state) => {
    set({ isLoading: true, error: null })
    try {
      if (typeof window !== 'undefined') {
        const savedState = sessionStorage.getItem('oauth_state')
        if (savedState !== state) {
          throw new Error('Invalid state parameter')
        }
      }

      const authResponse = await authService.handleGoogleCallback(code, state)

      Cookies.set('access_token', authResponse.access_token, { expires: 1 })
      Cookies.set('refresh_token', authResponse.refresh_token, { expires: 30 })

      set({
        user: authResponse.user,
        isAuthenticated: true,
      })

      if (typeof window !== 'undefined') {
        sessionStorage.removeItem('oauth_state')
      }

      return authResponse
    } catch (error) {
      const message = error?.response?.data?.message || error?.message || 'Login failed'
      set({ error: message })
      throw error
    } finally {
      set({ isLoading: false })
    }
  },

  checkAuth: async () => {
    try {
      const token = Cookies.get('access_token')
      if (!token) {
        set({ isAuthenticated: false, user: null })
        return false
      }

      const result = await authService.verifyToken()
      if (result.valid && result.user) {
        set({
          isAuthenticated: true,
          user: result.user,
        })
        return true
      }

      set({ isAuthenticated: false, user: null })
      return false
    } catch (error) {
      set({ isAuthenticated: false, user: null })
      return false
    }
  },

  fetchUser: async () => {
    set({ isLoading: true })
    try {
      const { user } = await userService.getCurrentUser()
      set({ user, isAuthenticated: true })
    } catch (error) {
      const message = error?.response?.data?.message || error?.message || 'Failed to fetch user'
      set({ error: message })
      throw error
    } finally {
      set({ isLoading: false })
    }
  },

  updateProfile: async (data) => {
    set({ isLoading: true, error: null })
    try {
      const { user } = await userService.updateProfile(data)
      set({ user })
      return user
    } catch (error) {
      const message = error?.response?.data?.message || error?.message || 'Failed to update profile'
      set({ error: message })
      throw error
    } finally {
      set({ isLoading: false })
    }
  },

  logout: async () => {
    try {
      const refreshToken = Cookies.get('refresh_token')
      if (refreshToken) {
        await authService.logout(refreshToken)
      }
    } catch (error) {
      console.error('Logout error:', error)
    } finally {
      Cookies.remove('access_token')
      Cookies.remove('refresh_token')
      clearWalletInfo()
      set({ ...initialState })
    }
  },

  clearError: () => set({ error: null }),
}))

export { useAuthStore }
export default useAuthStore
