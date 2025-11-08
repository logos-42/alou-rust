// ============================================
// User Service - User API calls
// ============================================

import apiClient from './api'
import type { User, UpdateProfileRequest } from '@/types/auth'

export const userService = {
  /**
   * Get current user information
   */
  async getCurrentUser(): Promise<{ user: User }> {
    const response = await apiClient.get('/user/me')
    return response.data
  },

  /**
   * Update user profile
   */
  async updateProfile(data: UpdateProfileRequest): Promise<{ user: User }> {
    const response = await apiClient.put('/user/profile', data)
    return response.data
  },
}

