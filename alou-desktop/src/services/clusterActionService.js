import apiClient from './api'

/**
 * 集群行动服务
 * 用于管理多个智能体协作执行复杂任务
 */
class ClusterActionService {
  /**
   * 创建集群行动
   * @param {string} description - 行动描述
   * @param {string} createdBy - 创建者（用户ID或"system"）
   * @param {object} metadata - 可选的元数据
   * @returns {Promise<object>} 创建的集群行动
   */
  async createClusterAction(description, createdBy, metadata = null) {
    try {
      const response = await apiClient.post('/api/cluster-action/create', {
        description,
        created_by: createdBy,
        metadata,
      })
      return response.data
    } catch (error) {
      console.error('[ClusterActionService] 创建集群行动失败:', error)
      throw error
    }
  }

  /**
   * 执行集群行动
   * @param {string} actionId - 集群行动ID
   * @param {string} walletAddress - 可选的钱包地址
   * @param {string} chain - 可选的链名称
   * @returns {Promise<object>} 执行结果
   */
  async executeClusterAction(actionId, walletAddress = null, chain = null) {
    try {
      const response = await apiClient.post('/api/cluster-action/execute', {
        action_id: actionId,
        wallet_address: walletAddress,
        chain,
      })
      return response.data
    } catch (error) {
      console.error('[ClusterActionService] 执行集群行动失败:', error)
      throw error
    }
  }

  /**
   * 获取集群行动状态
   * @param {string} actionId - 集群行动ID
   * @returns {Promise<object>} 状态信息
   */
  async getActionStatus(actionId) {
    try {
      const response = await apiClient.get(`/api/cluster-action/${actionId}/status`)
      return response.data
    } catch (error) {
      console.error('[ClusterActionService] 获取状态失败:', error)
      throw error
    }
  }

  /**
   * 获取集群行动结果
   * @param {string} actionId - 集群行动ID
   * @returns {Promise<object>} 执行结果
   */
  async getActionResults(actionId) {
    try {
      const response = await apiClient.get(`/api/cluster-action/${actionId}/results`)
      return response.data
    } catch (error) {
      console.error('[ClusterActionService] 获取结果失败:', error)
      throw error
    }
  }

  /**
   * 取消集群行动
   * @param {string} actionId - 集群行动ID
   * @returns {Promise<object>} 取消结果
   */
  async cancelAction(actionId) {
    try {
      const response = await apiClient.post(`/api/cluster-action/${actionId}/cancel`)
      return response.data
    } catch (error) {
      console.error('[ClusterActionService] 取消行动失败:', error)
      throw error
    }
  }

  /**
   * 分析任务是否需要集群行动
   * @param {string} taskDescription - 任务描述
   * @param {string[]} availableAgents - 可用智能体ID列表
   * @returns {Promise<object>} 分析结果
   */
  async analyzeTask(taskDescription, availableAgents) {
    try {
      const response = await apiClient.post('/api/cluster-action/analyze', {
        task_description: taskDescription,
        available_agents: availableAgents,
      })
      return response.data
    } catch (error) {
      console.error('[ClusterActionService] 分析任务失败:', error)
      throw error
    }
  }

  /**
   * 轮询获取集群行动状态（直到完成或失败）
   * @param {string} actionId - 集群行动ID
   * @param {number} interval - 轮询间隔（毫秒），默认1000ms
   * @param {number} timeout - 超时时间（毫秒），默认60000ms
   * @returns {Promise<object>} 最终状态
   */
  async pollActionStatus(actionId, interval = 1000, timeout = 60000) {
    const startTime = Date.now()
    
    return new Promise((resolve, reject) => {
      const poll = async () => {
        try {
          // 检查超时
          if (Date.now() - startTime > timeout) {
            reject(new Error('轮询超时'))
            return
          }

          const status = await this.getActionStatus(actionId)
          
          // 检查是否完成
          if (
            status.status === 'Completed' ||
            status.status === 'Failed' ||
            status.status === 'Cancelled'
          ) {
            resolve(status)
            return
          }

          // 继续轮询
          setTimeout(poll, interval)
        } catch (error) {
          reject(error)
        }
      }

      poll()
    })
  }
}

export default new ClusterActionService()

