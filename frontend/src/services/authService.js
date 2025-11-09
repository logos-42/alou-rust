// ============================================
// Authentication Service - Auth API calls
// ============================================

import apiClient from './api'

export const authService = {
  /**
   * Get Google OAuth login URL
   */
  async getGoogleLoginUrl() {
    const response = await apiClient.get('/auth/google/login')
    return response.data
  },

  /**
   * Handle Google OAuth callback
   */
  async handleGoogleCallback(code, state) {
    const response = await apiClient.get('/auth/google/callback', {
      params: { code, state },
    })
    return response.data
  },

  /**
   * Verify current token
   */
  async verifyToken() {
    const response = await apiClient.post('/auth/verify')
    return response.data
  },

  /**
   * Refresh access token
   */
  async refreshToken(refreshToken) {
    const response = await apiClient.post('/auth/refresh', {
      refresh_token: refreshToken,
    })
    return response.data
  },

  /**
   * Logout
   */
  async logout(refreshToken) {
    await apiClient.post('/auth/logout', {
      refresh_token: refreshToken,
    })
  },

  /**
   * Logout from all devices
   */
  async logoutAll() {
    await apiClient.post('/auth/logout-all')
  },
}
