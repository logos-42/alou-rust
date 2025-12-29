/**
 * 认证相关类型定义
 * 从原有src/types/auth.ts迁移而来
 */

export interface User {
  id: string;
  email: string;
  name?: string;
  avatar_url?: string;
  did?: string;
  created_at: string;
}

export interface AuthResponse {
  access_token: string;
  refresh_token: string;
  expires_in: number;
  user: User;
}

export interface LoginResponse {
  auth_url: string;
  state: string;
}

export interface UpdateProfileRequest {
  name?: string;
  avatar_url?: string;
}

// 新增类型：认证状态
export type AuthStatus = 'authenticated' | 'unauthenticated' | 'loading' | 'error';

// 新增类型：认证错误
export interface AuthError {
  code: string;
  message: string;
  details?: Record<string, any>;
}

// 新增类型：会话信息
export interface SessionInfo {
  userId: string;
  email: string;
  expiresAt: number;
  permissions: string[];
}

// 所有类型已经通过export语句导出
// 不需要额外的export type语句
