// ============================================
// User Service - User API calls
// ============================================

import apiClient from './api'

export const userService = {
  /**
   * Get current user information
   */
  async getCurrentUser() {
    const response = await apiClient.get('/user/me')
    return response.data
  },

  /**
   * Update user profile
   */
  async updateProfile(data) {
    const response = await apiClient.put('/user/profile', data)
    return response.data
  },
}

