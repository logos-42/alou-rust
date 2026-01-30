/**
 * 异步任务服务 - 简化的异步任务处理
 * 主要用于与后端AI任务接口通信
 */

import apiClient from './api';

// 通用API响应类型
interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
  timestamp: string;
}

interface CreateAITaskOptions {
  sessionId: string;
  message: string;
  walletAddress?: string;
  chain?: string;
  contextEvents?: any[];
  eventSummary?: string;
}

interface AITaskResult {
  taskId: string;
  status: 'pending' | 'running' | 'completed' | 'failed';
  message?: string;
  result?: any;
  error?: string;
  createdAt: string;
  updatedAt: string;
}

interface TaskStatus {
  taskId: string;
  status: 'pending' | 'running' | 'completed' | 'failed';
  progress?: number;
  message?: string;
  result?: any;
  error?: string;
  createdAt: string;
  updatedAt: string;
}

class AsyncTaskService {
  /**
   * 创建异步AI任务
   * @param options - 任务选项
   * @returns 任务创建结果
   */
  async createAITask(options: CreateAITaskOptions): Promise<ApiResponse<AITaskResult>> {
    try {
      console.log('[AsyncTaskService] 创建AI任务:', {
        sessionId: options.sessionId,
        messageLength: options.message?.length,
        chain: options.chain,
      });

      const payload = {
        session_id: options.sessionId,
        message: options.message,
        wallet_address: options.walletAddress,
        chain: options.chain,
        context_events: options.contextEvents,
        event_summary: options.eventSummary,
      };

      const response = await apiClient.post('/agent/async-task', payload);
      
      if (response.success && response.data) {
        console.log('[AsyncTaskService] AI任务创建成功:', response.data.taskId);
      }
      
      return response;
    } catch (error) {
      console.error('[AsyncTaskService] 创建AI任务失败:', error);
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date().toISOString(),
      };
    }
  }

  /**
   * 查询任务状态
   * @param taskId - 任务ID
   * @returns 任务状态
   */
  async getTaskStatus(taskId: string): Promise<ApiResponse<TaskStatus>> {
    try {
      console.log('[AsyncTaskService] 查询任务状态:', taskId);

      const response = await apiClient.get(`/agent/async-task/${taskId}/status`);
      
      return response;
    } catch (error) {
      console.error('[AsyncTaskService] 查询任务状态失败:', error);
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date().toISOString(),
      };
    }
  }

  /**
   * 获取任务结果
   * @param taskId - 任务ID
   * @returns 任务结果
   */
  async getTaskResult(taskId: string): Promise<ApiResponse<any>> {
    try {
      console.log('[AsyncTaskService] 获取任务结果:', taskId);

      const response = await apiClient.get(`/agent/async-task/${taskId}/result`);
      
      if (response.success) {
        console.log('[AsyncTaskService] 任务结果获取成功:', taskId);
      }
      
      return response;
    } catch (error) {
      console.error('[AsyncTaskService] 获取任务结果失败:', error);
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date().toISOString(),
      };
    }
  }

  /**
   * 取消任务
   * @param taskId - 任务ID
   * @returns 取消结果
   */
  async cancelTask(taskId: string): Promise<ApiResponse<void>> {
    try {
      console.log('[AsyncTaskService] 取消任务:', taskId);

      const response = await apiClient.post(`/agent/async-task/${taskId}/cancel`);
      
      if (response.success) {
        console.log('[AsyncTaskService] 任务取消成功:', taskId);
      }
      
      return response;
    } catch (error) {
      console.error('[AsyncTaskService] 取消任务失败:', error);
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date().toISOString(),
      };
    }
  }

  /**
   * 列出用户的任务
   * @param sessionId - 会话ID
   * @param options - 查询选项
   * @returns 任务列表
   */
  async listTasks(
    sessionId: string,
    options: {
      status?: 'pending' | 'running' | 'completed' | 'failed';
      limit?: number;
      offset?: number;
    } = {}
  ): Promise<ApiResponse<TaskStatus[]>> {
    try {
      console.log('[AsyncTaskService] 列出任务:', sessionId);

      const params = new URLSearchParams();
      params.append('session_id', sessionId);
      
      if (options.status) params.append('status', options.status);
      if (options.limit) params.append('limit', String(options.limit));
      if (options.offset) params.append('offset', String(options.offset));

      const response = await apiClient.get(`/agent/async-task/list?${params.toString()}`);
      
      console.log('[AsyncTaskService] 任务列表获取成功:', response.data?.length);
      return response;
    } catch (error) {
      console.error('[AsyncTaskService] 列出任务失败:', error);
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date().toISOString(),
      };
    }
  }

  /**
   * 删除任务
   * @param taskId - 任务ID
   * @returns 删除结果
   */
  async deleteTask(taskId: string): Promise<ApiResponse<void>> {
    try {
      console.log('[AsyncTaskService] 删除任务:', taskId);

      const response = await apiClient.delete(`/agent/async-task/${taskId}`);
      
      if (response.success) {
        console.log('[AsyncTaskService] 任务删除成功:', taskId);
      }
      
      return response;
    } catch (error) {
      console.error('[AsyncTaskService] 删除任务失败:', error);
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date().toISOString(),
      };
    }
  }

  /**
   * 等待任务完成
   * @param taskId - 任务ID
   * @param options - 等待选项
   * @returns 任务结果
   */
  async waitForTask(
    taskId: string,
    options: {
      timeout?: number; // 超时时间（毫秒）
      interval?: number; // 轮询间隔（毫秒）
      onProgress?: (status: TaskStatus) => void; // 进度回调
    } = {}
  ): Promise<ApiResponse<TaskStatus>> {
    const {
      timeout = 60000, // 默认60秒超时
      interval = 1000, // 默认1秒轮询
      onProgress,
    } = options;

    const startTime = Date.now();
    
    try {
      while (Date.now() - startTime < timeout) {
        const statusResponse = await this.getTaskStatus(taskId);
        
        if (!statusResponse.success) {
          return statusResponse;
        }

        const status = statusResponse.data!;
        
        // 调用进度回调
        if (onProgress) {
          onProgress(status);
        }

        // 检查任务是否完成
        if (status.status === 'completed' || status.status === 'failed') {
          console.log('[AsyncTaskService] 任务完成:', taskId, status.status);
          return statusResponse;
        }

        // 等待下次轮询
        await new Promise(resolve => setTimeout(resolve, interval));
      }

      // 超时
      throw new Error(`任务等待超时: ${taskId}`);
    } catch (error) {
      console.error('[AsyncTaskService] 等待任务失败:', error);
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date().toISOString(),
      };
    }
  }

  /**
   * 批量创建任务
   * @param tasks - 任务数组
   * @returns 创建结果
   */
  async createBatchTasks(tasks: CreateAITaskOptions[]): Promise<ApiResponse<AITaskResult[]>> {
    try {
      console.log('[AsyncTaskService] 批量创建任务:', tasks.length);

      const payload = {
        tasks: tasks.map(task => ({
          session_id: task.sessionId,
          message: task.message,
          wallet_address: task.walletAddress,
          chain: task.chain,
          context_events: task.contextEvents,
          event_summary: task.eventSummary,
        })),
      };

      const response = await apiClient.post('/agent/async-task/batch', payload);
      
      if (response.success && response.data) {
        console.log('[AsyncTaskService] 批量任务创建成功:', response.data.length);
      }
      
      return response;
    } catch (error) {
      console.error('[AsyncTaskService] 批量创建任务失败:', error);
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date().toISOString(),
      };
    }
  }

  /**
   * 获取任务统计信息
   * @param sessionId - 会话ID
   * @returns 统计信息
   */
  async getTaskStats(sessionId: string): Promise<ApiResponse<{
    total: number;
    pending: number;
    running: number;
    completed: number;
    failed: number;
  }>> {
    try {
      console.log('[AsyncTaskService] 获取任务统计:', sessionId);

      const response = await apiClient.get(`/agent/async-task/stats?session_id=${sessionId}`);
      
      return response;
    } catch (error) {
      console.error('[AsyncTaskService] 获取任务统计失败:', error);
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date().toISOString(),
      };
    }
  }
}

// 创建单例实例
const asyncTaskService = new AsyncTaskService();

export default asyncTaskService;
export { AsyncTaskService };
export type { 
  CreateAITaskOptions, 
  AITaskResult, 
  TaskStatus 
};
