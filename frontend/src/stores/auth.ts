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
      await checkAuth()
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
    loginWithGoogle,
    handleGoogleCallback,
    checkAuth,
    fetchUser,
    updateProfile,
    logout,
    clearError,
  }
})

