import { create } from 'zustand'
import Cookies from 'js-cookie'
import { authService } from '@/services/authService'
import { userService } from '@/services/userService'

/**
 * 用户信息接口
 */
export interface User {
  id: string
  email: string
  name?: string
  avatar_url?: string
  created_at?: string
  [key: string]: any
}

/**
 * 钱包信息接口
 */
export interface WalletInfo {
  address: string
  chainId?: number
  walletType?: string
}

/**
 * 认证响应接口
 */
export interface AuthResponse {
  user: User
  access_token: string
  refresh_token?: string
  token?: string // 兼容性字段
}

/**
 * 登录选项接口
 */
export interface Web3LoginOptions {
  address: string
  chainId?: number
  walletType?: string
}

export interface WalletLoginOptions {
  privateKey?: string
  mnemonic?: string
}

/**
 * AuthStore 状态接口
 */
interface AuthState {
  user: User | null
  isAuthenticated: boolean
  isLoading: boolean
  error: string | null
}

/**
 * AuthStore 动作接口
 */
interface AuthActions {
  currentUser: () => User | null
  userName: () => string
  userAvatar: () => string | undefined
  init: () => Promise<void>
  loginWithWeb3Wallet: (options: Web3LoginOptions) => Promise<AuthResponse>
  loginWithWallet: (options: WalletLoginOptions) => Promise<AuthResponse>
  loginWithGoogle: () => Promise<void>
  handleGoogleCallback: (code: string, state: string) => Promise<AuthResponse>
  checkAuth: () => Promise<boolean>
  fetchUser: () => Promise<void>
  updateProfile: (data: Partial<User>) => Promise<User>
  logout: () => Promise<void>
  clearError: () => void
}

type AuthStore = AuthState & AuthActions

const buildWalletUser = (address: string): User => {
  const normalized = address ? address.toLowerCase() : ''
  return {
    id: normalized,
    email: `${normalized}@wallet.local`,
    name: `${normalized.slice(0, 6)}...${normalized.slice(-4)}`,
    avatar_url: `https://api.dicebear.com/7.x/identicon/svg?seed=${normalized}`,
    created_at: new Date().toISOString(),
  }
}

const persistWalletInfo = ({ address, walletType, chainId }: Partial<WalletInfo>): void => {
  if (typeof window === 'undefined') return
  if (address) {
    localStorage.setItem('wallet_address', address)
  }
  if (walletType) {
    localStorage.setItem('wallet_type', walletType)
  }
  if (chainId) {
    localStorage.setItem('wallet_chain_id', String(chainId))
  }
}

const clearWalletInfo = (): void => {
  if (typeof window === 'undefined') return
  localStorage.removeItem('wallet_address')
  localStorage.removeItem('wallet_type')
  localStorage.removeItem('wallet_chain_id')
}

const dispatchWalletChanged = (address: string | undefined): void => {
  if (typeof window === 'undefined') return
  window.dispatchEvent(
    new CustomEvent('wallet-changed', {
      detail: { address: address ? address.toLowerCase() : undefined },
    }),
  )
}

const initialState: AuthState = {
  user: null,
  isAuthenticated: false,
  isLoading: false,
  error: null,
}

const useAuthStore = create<AuthStore>((set, get) => ({
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

  loginWithWeb3Wallet: async ({ address, chainId, walletType }: Web3LoginOptions): Promise<AuthResponse> => {
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
      return { user: mockUser, token: mockToken, access_token: mockToken }
    } catch (error: any) {
      const message = error?.message || (typeof error === 'string' ? error : 'Web3钱包登录失败')
      set({ error: message })
      throw error
    } finally {
      set({ isLoading: false })
    }
  },

  loginWithWallet: async ({ privateKey, mnemonic }: WalletLoginOptions): Promise<AuthResponse> => {
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
      return { user: mockUser, token: mockToken, access_token: mockToken }
    } catch (error: any) {
      const message = error?.message || (typeof error === 'string' ? error : '钱包登录失败')
      set({ error: message })
      throw error
    } finally {
      set({ isLoading: false })
    }
  },

  loginWithGoogle: async (): Promise<void> => {
    set({ isLoading: true, error: null })
    try {
      const { auth_url, state } = await authService.getGoogleLoginUrl()
      if (typeof window !== 'undefined') {
        sessionStorage.setItem('oauth_state', state)
        window.location.href = auth_url
      }
    } catch (error: any) {
      const message = error?.response?.data?.message || error?.message || 'Failed to initiate login'
      set({ error: message })
      throw error
    } finally {
      set({ isLoading: false })
    }
  },

  handleGoogleCallback: async (code: string, state: string): Promise<AuthResponse> => {
    set({ isLoading: true, error: null })
    try {
      if (typeof window !== 'undefined') {
        const savedState = sessionStorage.getItem('oauth_state')
        if (savedState !== state) {
          throw new Error('Invalid state parameter')
        }
      }

      const authResponse = await authService.handleGoogleCallback(code, state)

      // Handle different response formats
      const serviceResponse = authResponse as any
      const tokens = serviceResponse.tokens || serviceResponse
      const user = serviceResponse.user || serviceResponse.data?.user || serviceResponse

      Cookies.set('access_token', tokens.access_token, { expires: 1 })
      Cookies.set('refresh_token', tokens.refresh_token, { expires: 30 })

      set({
        user: user,
        isAuthenticated: true,
      })

      if (typeof window !== 'undefined') {
        sessionStorage.removeItem('oauth_state')
      }

      return authResponse
    } catch (error: any) {
      const message = error?.response?.data?.message || error?.message || 'Login failed'
      set({ error: message })
      throw error
    } finally {
      set({ isLoading: false })
    }
  },

  checkAuth: async (): Promise<boolean> => {
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

  fetchUser: async (): Promise<void> => {
    set({ isLoading: true })
    try {
      const response = await userService.getCurrentUser()
      const user = response.data || response
      set({ user, isAuthenticated: true })
    } catch (error: any) {
      const message = error?.response?.data?.message || error?.message || 'Failed to fetch user'
      set({ error: message })
      throw error
    } finally {
      set({ isLoading: false })
    }
  },

  updateProfile: async (data: Partial<User>): Promise<User> => {
    set({ isLoading: true, error: null })
    try {
      const response = await userService.updateProfile(data)
      const user = response.data || response
      set({ user })
      return user
    } catch (error: any) {
      const message = error?.response?.data?.message || error?.message || 'Failed to update profile'
      set({ error: message })
      throw error
    } finally {
      set({ isLoading: false })
    }
  },

  logout: async (): Promise<void> => {
    try {
      const refreshToken = Cookies.get('refresh_token')
      // authService.logout may not expect a parameter
      if (refreshToken) {
        await authService.logout(refreshToken as any)
      } else {
        await authService.logout(undefined as any)
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
