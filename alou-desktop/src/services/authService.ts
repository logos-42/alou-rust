/**
 * 认证服务
 * 提供用户认证、授权和会话管理功能
 */

// 本地类型定义
interface ServiceResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
  timestamp: string;
}

interface LoginCredentials {
  email: string;
  password: string;
}

interface RegisterData {
  email: string;
  password: string;
  name: string;
}

interface AuthUser {
  id: string;
  email: string;
  name: string;
  avatar_url?: string;
  created_at: string;
  updated_at?: string;
}

interface AuthTokens {
  access_token: string;
  refresh_token: string;
  expires_in: number;
  token_type: string;
}

interface AuthResponse {
  user: AuthUser;
  tokens: AuthTokens;
}

class AuthService {
  // @ts-ignore - Vite 环境变量定义在 vite-env.d.ts 中
  private readonly API_BASE_URL = (import.meta as any).env.VITE_API_BASE_URL || '/api';
  private readonly TOKEN_KEY = 'auth_tokens';
  private readonly USER_KEY = 'auth_user';

  /**
   * 用户登录
   * @param credentials 登录凭据
   * @returns 认证响应
   */
  async login(credentials: LoginCredentials): Promise<ServiceResponse<AuthResponse>> {
    try {
      const response = await fetch(`${this.API_BASE_URL}/auth/login`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(credentials),
      });

      if (!response.ok) {
        throw new Error(`登录失败: ${response.status}`);
      }

      const data = await response.json();
      
      // 保存令牌和用户信息
      this.setAuthData(data);
      
      return {
        success: true,
        data,
        message: '登录成功',
        timestamp: new Date().toISOString(),
      };
    } catch (error) {
      console.error('[AuthService] 登录失败:', error);
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date().toISOString(),
      };
    }
  }

  /**
   * 用户注册
   * @param userData 注册数据
   * @returns 认证响应
   */
  async register(userData: RegisterData): Promise<ServiceResponse<AuthResponse>> {
    try {
      const response = await fetch(`${this.API_BASE_URL}/auth/register`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(userData),
      });

      if (!response.ok) {
        throw new Error(`注册失败: ${response.status}`);
      }

      const data = await response.json();
      
      // 保存令牌和用户信息
      this.setAuthData(data);
      
      return {
        success: true,
        data,
        message: '注册成功',
        timestamp: new Date().toISOString(),
      };
    } catch (error) {
      console.error('[AuthService] 注册失败:', error);
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date().toISOString(),
      };
    }
  }

  /**
   * 用户登出
   * @returns 登出结果
   */
  async logout(): Promise<ServiceResponse<void>> {
    try {
      const tokens = this.getTokens();
      
      if (tokens?.access_token) {
        // 调用后端登出接口
        await fetch(`${this.API_BASE_URL}/auth/logout`, {
          method: 'POST',
          headers: {
            'Authorization': `Bearer ${tokens.access_token}`,
            'Content-Type': 'application/json',
          },
        });
      }

      // 清除本地存储
      this.clearAuthData();
      
      return {
        success: true,
        message: '登出成功',
        timestamp: new Date().toISOString(),
      };
    } catch (error) {
      console.error('[AuthService] 登出失败:', error);
      // 即使后端登出失败，也清除本地数据
      this.clearAuthData();
      
      return {
        success: true, // 登出总是成功的，因为本地数据已清除
        message: '已清除本地登录状态',
        timestamp: new Date().toISOString(),
      };
    }
  }

  /**
   * 刷新令牌
   * @returns 新的认证响应
   */
  async refreshToken(): Promise<ServiceResponse<AuthTokens>> {
    try {
      const tokens = this.getTokens();
      
      if (!tokens?.refresh_token) {
        throw new Error('没有刷新令牌');
      }

      const response = await fetch(`${this.API_BASE_URL}/auth/refresh`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          refresh_token: tokens.refresh_token,
        }),
      });

      if (!response.ok) {
        throw new Error(`刷新令牌失败: ${response.status}`);
      }

      const newTokens = await response.json();
      
      // 更新令牌
      this.setTokens(newTokens);
      
      return {
        success: true,
        data: newTokens,
        message: '令牌刷新成功',
        timestamp: new Date().toISOString(),
      };
    } catch (error) {
      console.error('[AuthService] 刷新令牌失败:', error);
      // 刷新失败，清除认证数据
      this.clearAuthData();
      
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date().toISOString(),
      };
    }
  }

  /**
   * 获取当前用户信息
   * @returns 用户信息
   */
  async getCurrentUser(): Promise<ServiceResponse<AuthUser>> {
    try {
      const tokens = this.getTokens();
      
      if (!tokens?.access_token) {
        throw new Error('未登录');
      }

      const response = await fetch(`${this.API_BASE_URL}/auth/me`, {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${tokens.access_token}`,
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        throw new Error(`获取用户信息失败: ${response.status}`);
      }

      const user = await response.json();
      
      return {
        success: true,
        data: user,
        timestamp: new Date().toISOString(),
      };
    } catch (error) {
      console.error('[AuthService] 获取用户信息失败:', error);
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date().toISOString(),
      };
    }
  }

  /**
   * 检查是否已登录
   * @returns 是否已登录
   */
  isAuthenticated(): boolean {
    const tokens = this.getTokens();
    if (!tokens?.access_token) {
      return false;
    }

    // 检查令牌是否过期
    const now = Date.now();
    const expiresAt = tokens.expires_in * 1000; // 转换为毫秒
    
    return now < expiresAt;
  }

  /**
   * 获取认证头
   * @returns Authorization 头值
   */
  getAuthHeader(): string | null {
    const tokens = this.getTokens();
    return tokens?.access_token ? `Bearer ${tokens.access_token}` : null;
  }

  /**
   * 获取存储的令牌
   * @returns 认证令牌
   */
  getTokens(): AuthTokens | null {
    try {
      const stored = localStorage.getItem(this.TOKEN_KEY);
      return stored ? JSON.parse(stored) : null;
    } catch (error) {
      console.error('[AuthService] 获取令牌失败:', error);
      return null;
    }
  }

  /**
   * 获取存储的用户信息
   * @returns 用户信息
   */
  getUser(): AuthUser | null {
    try {
      const stored = localStorage.getItem(this.USER_KEY);
      return stored ? JSON.parse(stored) : null;
    } catch (error) {
      console.error('[AuthService] 获取用户信息失败:', error);
      return null;
    }
  }

  /**
   * 设置认证数据
   * @param authData 认证响应数据
   */
  private setAuthData(authData: AuthResponse): void {
    this.setTokens(authData.tokens);
    this.setUser(authData.user);
  }

  /**
   * 设置令牌
   * @param tokens 认证令牌
   */
  private setTokens(tokens: AuthTokens): void {
    try {
      localStorage.setItem(this.TOKEN_KEY, JSON.stringify(tokens));
    } catch (error) {
      console.error('[AuthService] 保存令牌失败:', error);
    }
  }

  /**
   * 设置用户信息
   * @param user 用户信息
   */
  private setUser(user: AuthUser): void {
    try {
      localStorage.setItem(this.USER_KEY, JSON.stringify(user));
    } catch (error) {
      console.error('[AuthService] 保存用户信息失败:', error);
    }
  }

  /**
   * 清除认证数据
   */
  private clearAuthData(): void {
    try {
      localStorage.removeItem(this.TOKEN_KEY);
      localStorage.removeItem(this.USER_KEY);
    } catch (error) {
      console.error('[AuthService] 清除认证数据失败:', error);
    }
  }

  /**
   * 钱包连接认证
   * @param address 钱包地址
   * @param signature 签名
   * @param message 签名消息
   * @returns 认证响应
   */
  async authenticateWithWallet(
    address: string, 
    signature: string, 
    message: string
  ): Promise<ServiceResponse<AuthResponse>> {
    try {
      const response = await fetch(`${this.API_BASE_URL}/auth/wallet`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          address,
          signature,
          message,
        }),
      });

      if (!response.ok) {
        throw new Error(`钱包认证失败: ${response.status}`);
      }

      const data = await response.json();
      
      // 保存令牌和用户信息
      this.setAuthData(data);
      
      return {
        success: true,
        data,
        message: '钱包认证成功',
        timestamp: new Date().toISOString(),
      };
    } catch (error) {
      console.error('[AuthService] 钱包认证失败:', error);
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date().toISOString(),
      };
    }
  }

  /**
   * 生成钱包认证消息
   * @param address 钱包地址
   * @returns 签名消息
   */
  generateWalletMessage(address: string): string {
    const timestamp = Date.now();
    const nonce = Math.random().toString(36).substring(2);
    return `请签名以登录 Alou Pay\n地址: ${address}\n时间戳: ${timestamp}\n随机数: ${nonce}`;
  }

  /**
   * 获取 Google 登录 URL
   * @returns Google 登录信息
   */
  async getGoogleLoginUrl(): Promise<{ auth_url: string; state: string }> {
    // 模拟实现
    const state = Math.random().toString(36).substring(2);
    const auth_url = `https://accounts.google.com/oauth/authorize?client_id=mock&redirect_uri=${encodeURIComponent(window.location.origin)}&response_type=code&scope=email%20profile&state=${state}`;
    
    return { auth_url, state };
  }

  /**
   * 处理 Google 回调
   * @param _code 授权码（未使用）
   * @param _state 状态参数（未使用）
   * @returns 认证响应
   */
  async handleGoogleCallback(_code: string, _state: string): Promise<AuthResponse> {
    // 模拟实现
    const mockUser: AuthUser = {
      id: 'google_user_' + Date.now(),
      email: 'user@gmail.com',
      name: 'Google User',
      avatar_url: 'https://lh3.googleusercontent.com/a/default-user',
      created_at: new Date().toISOString(),
    };

    const mockTokens: AuthTokens = {
      access_token: 'google_access_' + Math.random().toString(36).substring(2),
      refresh_token: 'google_refresh_' + Math.random().toString(36).substring(2),
      expires_in: Date.now() + 3600 * 1000, // 1小时后过期
      token_type: 'Bearer',
    };

    return {
      user: mockUser,
      tokens: mockTokens,
    };
  }

  /**
   * 验证令牌
   * @returns 验证结果
   */
  async verifyToken(): Promise<{ valid: boolean; user?: AuthUser }> {
    // 模拟实现
    const token = localStorage.getItem('access_token');
    if (!token) {
      return { valid: false };
    }

    // 这里应该调用后端验证令牌
    return { valid: true };
  }

}

// 创建单例实例
const authService = new AuthService();

export { authService };
export type {
  LoginCredentials,
  RegisterData,
  AuthUser,
  AuthTokens,
  AuthResponse,
};
