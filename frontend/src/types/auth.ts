// ============================================
// Authentication Types
// ============================================

export interface User {
  id: string
  email: string
  name?: string
  avatar_url?: string
  did?: string
  created_at: string
}

export interface AuthResponse {
  access_token: string
  refresh_token: string
  expires_in: number
  user: User
}

export interface LoginResponse {
  auth_url: string
  state: string
}

export interface UpdateProfileRequest {
  name?: string
  avatar_url?: string
}

