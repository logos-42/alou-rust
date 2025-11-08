// ============================================
// Authentication Service - Auth API calls
// ============================================

import apiClient from './api'
import type { User, AuthResponse, LoginResponse } from '@/types/auth'

export const authService = {
  /**
   * Get Google OAuth login URL
   */
  async getGoogleLoginUrl(): Promise<LoginResponse> {
    const response = await apiClient.get<LoginResponse>('/auth/google/login')
    return response.data
  },

  /**
   * Handle Google OAuth callback
   */
  async handleGoogleCallback(code: string, state: string): Promise<AuthResponse> {
    const response = await apiClient.get<AuthResponse>('/auth/google/callback', {
      params: { code, state },
    })
    return response.data
  },

  /**
   * Verify current token
   */
  async verifyToken(): Promise<{ valid: boolean; user?: User }> {
    const response = await apiClient.post('/auth/verify')
    return response.data
  },

  /**
   * Refresh access token
   */
  async refreshToken(refreshToken: string): Promise<AuthResponse> {
    const response = await apiClient.post<AuthResponse>('/auth/refresh', {
      refresh_token: refreshToken,
    })
    return response.data
  },

  /**
   * Logout
   */
  async logout(refreshToken: string): Promise<void> {
    await apiClient.post('/auth/logout', {
      refresh_token: refreshToken,
    })
  },

  /**
   * Logout from all devices
   */
  async logoutAll(): Promise<void> {
    await apiClient.post('/auth/logout-all')
  },
}

