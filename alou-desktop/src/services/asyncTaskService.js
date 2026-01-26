/**
 * 异步任务服务 - 简化的异步任务处理
 * 主要用于与后端AI任务接口通信
 */

import apiClient from './api'

export class AsyncTaskService {
  /**
   * 创建异步AI任务
   * @param {Object} options - 任务选项
   * @param {string} options.sessionId - 会话ID
   * @param {string} options.message - 用户消息
   * @param {string} options.walletAddress - 钱包地址
   * @param {string} options.chain - 链名称
   * @param {Array} options.contextEvents - 上下文事件
   * @param {string} options.eventSummary - 事件摘要
   * @returns {Promise<Object>} 任务创建结果
   */
  async createAITask({
    sessionId,
    message,
    walletAddress,
    chain,
    contextEvents,
    eventSummary,
  }) {
    try {
      console.log('[AsyncTaskService] 创建AI任务:', {
        sessionId,
        messageLength: message?.length,
        walletAddress,
        chain,
      })

      const response = await apiClient.post('/ai-task/init-and-start', {
        session_id: sessionId,
        prompt: message,  // 修复字段映射：后端期望 prompt 字段
        wallet_address: walletAddress,
        chain,
        context_events: contextEvents || [],
        event_summary: eventSummary || '',
      })

      return response.data
    } catch (error) {
      console.error('[AsyncTaskService] 创建任务失败:', error)
      throw error
    }
  }

  /**
   * 获取任务状态
   * @param {string} taskId - 任务ID
   * @returns {Promise<Object>} 任务状态
   */
  async getTaskStatus(taskId) {
    try {
      const response = await apiClient.get(`/ai-task/${taskId}/status`)
      return response.data
    } catch (error) {
      console.error('[AsyncTaskService] 获取任务状态失败:', error)
      throw error
    }
  }

  /**
   * 获取任务结果
   * @param {string} taskId - 任务ID
   * @returns {Promise<Object>} 任务结果
   */
  async getTaskResult(taskId) {
    try {
      const response = await apiClient.get(`/ai-task/${taskId}/result`)
      return response.data
    } catch (error) {
      console.error('[AsyncTaskService] 获取任务结果失败:', error)
      throw error
    }
  }
}

export default new AsyncTaskService()
