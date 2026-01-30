// ============================================
// User Service - User API calls
// ============================================

import apiClient from './api';
import type { ServiceResponse } from '@/shared/types';

// 用户信息类型
export interface User {
  id: string;
  username: string;
  email?: string;
  avatar?: string;
  createdAt: string;
  updatedAt: string;
  [key: string]: any;
}

// 用户资料更新数据
export interface UserProfileUpdate {
  username?: string;
  avatar?: string;
  bio?: string;
  [key: string]: any;
}

export const userService = {
  /**
   * Get current user information
   */
  async getCurrentUser(): Promise<ServiceResponse<User>> {
    const response = await apiClient.get<User>('/user/me');
    return {
      success: true,
      data: response.data,
      timestamp: new Date().toISOString(),
    };
  },

  /**
   * Update user profile
   */
  async updateProfile(data: UserProfileUpdate): Promise<ServiceResponse<User>> {
    const response = await apiClient.put<User>('/user/profile', data);
    return {
      success: true,
      data: response.data,
      message: 'Profile updated successfully',
      timestamp: new Date().toISOString(),
    };
  },
};

export default userService;
