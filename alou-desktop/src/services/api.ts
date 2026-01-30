/**
 * API 服务
 * 提供通用的 HTTP 请求封装和 API 调用功能
 */

// 本地类型定义
interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
  timestamp: string;
}

interface ServicePaginationParams {
  page?: number;
  limit?: number;
  offset?: number;
  sort?: string;
  order?: 'asc' | 'desc';
}

interface ServicePaginatedResult<T> {
  items: T[];
  total: number;
  page: number;
  limit: number;
  hasMore: boolean;
}

interface RequestOptions {
  method?: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';
  headers?: Record<string, string>;
  body?: any;
  params?: Record<string, any>;
  timeout?: number;
}

class ApiService {
  // @ts-ignore - Vite 环境变量
  private readonly API_BASE_URL = (import.meta as any).env?.VITE_API_BASE_URL || '/api';
  private readonly DEFAULT_TIMEOUT = 30000; // 30秒

  /**
   * 发送 HTTP 请求
   * @param endpoint API 端点
   * @param options 请求选项
   * @returns API 响应
   */
  async request<T = any>(
    endpoint: string, 
    options: RequestOptions = {}
  ): Promise<ApiResponse<T>> {
    const {
      method = 'GET',
      headers = {},
      body,
      params,
      timeout = this.DEFAULT_TIMEOUT
    } = options;

    try {
      // 构建 URL
      let url = `${this.API_BASE_URL}${endpoint}`;
      
      // 添加查询参数
      if (params && Object.keys(params).length > 0) {
        const searchParams = new URLSearchParams();
        Object.entries(params).forEach(([key, value]) => {
          if (value !== undefined && value !== null) {
            searchParams.append(key, String(value));
          }
        });
        url += `?${searchParams.toString()}`;
      }

      // 准备请求头
      const requestHeaders: Record<string, string> = {
        'Content-Type': 'application/json',
        ...headers,
      };

      // 添加认证头
      const authHeader = this.getAuthHeader();
      if (authHeader) {
        requestHeaders.Authorization = authHeader;
      }

      // 准备请求体
      let requestBody: string | undefined;
      if (body) {
        if (typeof body === 'string') {
          requestBody = body;
        } else {
          requestBody = JSON.stringify(body);
        }
      }

      // 创建 AbortController 用于超时控制
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), timeout);

      // 发送请求
      const response = await fetch(url, {
        method,
        headers: requestHeaders,
        body: requestBody,
        signal: controller.signal,
      });

      // 清除超时定时器
      clearTimeout(timeoutId);

      // 处理响应
      let responseData: any;
      const contentType = response.headers.get('content-type');
      
      if (contentType?.includes('application/json')) {
        responseData = await response.json();
      } else {
        responseData = await response.text();
      }

      if (!response.ok) {
        throw new Error(responseData?.error || responseData?.message || `HTTP ${response.status}`);
      }

      return {
        success: true,
        data: responseData,
        message: responseData?.message || '请求成功',
        timestamp: new Date().toISOString(),
      };
    } catch (error) {
      console.error(`[ApiService] ${method} ${endpoint} 请求失败:`, error);
      
      let errorMessage = '请求失败';
      if (error instanceof Error) {
        if (error.name === 'AbortError') {
          errorMessage = '请求超时';
        } else {
          errorMessage = error.message;
        }
      }

      return {
        success: false,
        error: errorMessage,
        timestamp: new Date().toISOString(),
      };
    }
  }

  /**
   * GET 请求
   * @param endpoint API 端点
   * @param params 查询参数
   * @param headers 请求头
   * @returns API 响应
   */
  async get<T = any>(
    endpoint: string, 
    params?: Record<string, any>, 
    headers?: Record<string, string>
  ): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, {
      method: 'GET',
      params,
      headers,
    });
  }

  /**
   * POST 请求
   * @param endpoint API 端点
   * @param body 请求体
   * @param headers 请求头
   * @returns API 响应
   */
  async post<T = any>(
    endpoint: string, 
    body?: any, 
    headers?: Record<string, string>
  ): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, {
      method: 'POST',
      body,
      headers,
    });
  }

  /**
   * PUT 请求
   * @param endpoint API 端点
   * @param body 请求体
   * @param headers 请求头
   * @returns API 响应
   */
  async put<T = any>(
    endpoint: string, 
    body?: any, 
    headers?: Record<string, string>
  ): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, {
      method: 'PUT',
      body,
      headers,
    });
  }

  /**
   * DELETE 请求
   * @param endpoint API 端点
   * @param headers 请求头
   * @returns API 响应
   */
  async delete<T = any>(
    endpoint: string, 
    headers?: Record<string, string>
  ): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, {
      method: 'DELETE',
      headers,
    });
  }

  /**
   * PATCH 请求
   * @param endpoint API 端点
   * @param body 请求体
   * @param headers 请求头
   * @returns API 响应
   */
  async patch<T = any>(
    endpoint: string, 
    body?: any, 
    headers?: Record<string, string>
  ): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, {
      method: 'PATCH',
      body,
      headers,
    });
  }

  /**
   * 分页 GET 请求
   * @param endpoint API 端点
   * @param paginationParams 分页参数
   * @param additionalParams 额外参数
   * @returns 分页结果
   */
  async getPaginated<T = any>(
    endpoint: string,
    paginationParams?: ServicePaginationParams,
    additionalParams?: Record<string, any>
  ): Promise<ApiResponse<ServicePaginatedResult<T>>> {
    const params = {
      ...paginationParams,
      ...additionalParams,
    };

    return this.get<ServicePaginatedResult<T>>(endpoint, params);
  }

  /**
   * 上传文件
   * @param endpoint API 端点
   * @param file 文件对象
   * @param additionalData 额外数据
   * @param onProgress 进度回调
   * @returns API 响应
   */
  async uploadFile<T = any>(
    endpoint: string,
    file: File,
    additionalData?: Record<string, any>,
    onProgress?: (progress: number) => void
  ): Promise<ApiResponse<T>> {
    try {
      const formData = new FormData();
      formData.append('file', file);

      // 添加额外数据
      if (additionalData) {
        Object.entries(additionalData).forEach(([key, value]) => {
          formData.append(key, String(value));
        });
      }

      const headers: Record<string, string> = {};
      const authHeader = this.getAuthHeader();
      if (authHeader) {
        headers.Authorization = authHeader;
      }

      let response: Response;
      
      if (onProgress) {
        // 使用 XMLHttpRequest 支持进度回调
        response = await new Promise((resolve, reject) => {
          const xhr = new XMLHttpRequest();
          
          // 监听进度
          xhr.upload.addEventListener('progress', (event) => {
            if (event.lengthComputable) {
              const progress = (event.loaded / event.total) * 100;
              onProgress(progress);
            }
          });

          // 监听完成
          xhr.addEventListener('load', () => {
            resolve(xhr.response as any);
          });

          // 监听错误
          xhr.addEventListener('error', () => {
            reject(new Error('文件上传失败'));
          });

          // 监听超时
          xhr.addEventListener('timeout', () => {
            reject(new Error('文件上传超时'));
          });

          // 配置请求
          xhr.timeout = this.DEFAULT_TIMEOUT;
          xhr.open('POST', `${this.API_BASE_URL}${endpoint}`);

          // 设置请求头
          Object.entries(headers).forEach(([key, value]) => {
            xhr.setRequestHeader(key, value);
          });

          // 发送请求
          xhr.responseType = 'json';
          xhr.send(formData);
        });
      } else {
        // 使用 fetch API
        response = await fetch(`${this.API_BASE_URL}${endpoint}`, {
          method: 'POST',
          body: formData,
          headers,
        });
      }

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData?.error || errorData?.message || `HTTP ${response.status}`);
      }

      const data = await response.json();

      return {
        success: true,
        data,
        message: data?.message || '文件上传成功',
        timestamp: new Date().toISOString(),
      };
    } catch (error) {
      console.error('[ApiService] 文件上传失败:', error);
      
      let errorMessage = '文件上传失败';
      if (error instanceof Error) {
        errorMessage = error.message;
      }

      return {
        success: false,
        error: errorMessage,
        timestamp: new Date().toISOString(),
      };
    }
  }

  /**
   * 下载文件
   * @param endpoint API 端点
   * @param filename 文件名
   * @param params 查询参数
   * @returns API 响应
   */
  async downloadFile(
    endpoint: string,
    filename?: string,
    params?: Record<string, any>
  ): Promise<ApiResponse<Blob>> {
    try {
      let url = `${this.API_BASE_URL}${endpoint}`;
      
      if (params && Object.keys(params).length > 0) {
        const searchParams = new URLSearchParams();
        Object.entries(params).forEach(([key, value]) => {
          if (value !== undefined && value !== null) {
            searchParams.append(key, String(value));
          }
        });
        url += `?${searchParams.toString()}`;
      }

      const headers: Record<string, string> = {};
      const authHeader = this.getAuthHeader();
      if (authHeader) {
        headers.Authorization = authHeader;
      }

      const response = await fetch(url, {
        method: 'GET',
        headers,
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData?.error || errorData?.message || `HTTP ${response.status}`);
      }

      const blob = await response.blob();

      // 自动下载文件
      if (filename) {
        const downloadUrl = window.URL.createObjectURL(blob);
        const link = document.createElement('a');
        link.href = downloadUrl;
        link.download = filename;
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
        window.URL.revokeObjectURL(downloadUrl);
      }

      return {
        success: true,
        data: blob,
        message: '文件下载成功',
        timestamp: new Date().toISOString(),
      };
    } catch (error) {
      console.error('[ApiService] 文件下载失败:', error);
      
      let errorMessage = '文件下载失败';
      if (error instanceof Error) {
        errorMessage = error.message;
      }

      return {
        success: false,
        error: errorMessage,
        timestamp: new Date().toISOString(),
      };
    }
  }

  /**
   * 获取认证头
   * @returns Authorization 头值
   */
  private getAuthHeader(): string | null {
    try {
      const tokens = localStorage.getItem('auth_tokens');
      if (tokens) {
        const { access_token } = JSON.parse(tokens);
        return access_token ? `Bearer ${access_token}` : null;
      }
    } catch (error) {
      console.error('[ApiService] 获取认证头失败:', error);
    }
    return null;
  }

  /**
   * 设置基础 URL
   * @param baseUrl 基础 URL
   */
  setBaseUrl(baseUrl: string): void {
    (this as any).API_BASE_URL = baseUrl;
  }

  /**
   * 获取当前基础 URL
   * @returns 基础 URL
   */
  getBaseUrl(): string {
    return (this as any).API_BASE_URL;
  }
}

// 创建单例实例
const apiService = new ApiService();

export { apiService };
export default apiService;
export type { RequestOptions };
