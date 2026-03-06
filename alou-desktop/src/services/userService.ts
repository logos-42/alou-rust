// ============================================
// User Service - User API calls
// ============================================

import apiClient from './api';
import type { ServiceResponse } from '@/shared/types';
import agentAssetsService from './agentAssetsService';

// 用户信息类型
export interface User {
  id: string;
  username: string;
  email?: string;
  avatar?: string;
  avatar_url?: string;
  createdAt: string;
  updatedAt: string;
  [key: string]: any;
}

// 用户资料更新数据
export interface UserProfileUpdate {
  username?: string;
  avatar?: string;
  avatar_url?: string;
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

  /**
   * Upload user avatar
   * @param file - Avatar file to upload
   * @returns The avatar URL/CID
   */
  async uploadAvatar(file: File): Promise<ServiceResponse<{ avatar_url: string; avatar_cid?: string }>> {
    try {
      // Try to upload to IPFS first
      const result = await agentAssetsService.uploadAvatar(file);
      
      if (result?.cid) {
        // Get the avatar URL from CID
        const avatarUrl = await agentAssetsService.getAvatar(result.cid);
        return {
          success: true,
          data: {
            avatar_url: avatarUrl,
            avatar_cid: result.cid,
          },
          message: 'Avatar uploaded successfully',
          timestamp: new Date().toISOString(),
        };
      }
      
      throw new Error('Failed to get CID from upload');
    } catch (error) {
      console.warn('[UserService] IPFS upload failed, falling back to base64:', error);
      
      // Fallback to base64 encoding
      return new Promise((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = () => {
          const base64 = reader.result as string;
          resolve({
            success: true,
            data: {
              avatar_url: base64,
            },
            message: 'Avatar uploaded successfully (base64)',
            timestamp: new Date().toISOString(),
          });
        };
        reader.onerror = () => {
          reject(new Error('Failed to read avatar file'));
        };
        reader.readAsDataURL(file);
      });
    }
  },

  /**
   * Update user avatar with file upload
   * @param file - Avatar file
   * @returns Updated user data
   */
  async updateAvatar(file: File): Promise<ServiceResponse<User>> {
    // First upload the avatar
    const uploadResult = await userService.uploadAvatar(file);
    
    if (!uploadResult.success) {
      throw new Error(uploadResult.message || 'Failed to upload avatar');
    }

    // Then update the profile with the new avatar URL
    const updateData: UserProfileUpdate = {
      avatar_url: uploadResult.data.avatar_url,
    };

    // If we have a CID, also include it
    if (uploadResult.data.avatar_cid) {
      updateData.avatar = uploadResult.data.avatar_cid;
    }

    return userService.updateProfile(updateData);
  },
};

export default userService;
