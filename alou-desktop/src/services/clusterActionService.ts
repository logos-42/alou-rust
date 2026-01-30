import apiClient from './api'
import type { AxiosError } from 'axios'

/**
 * 集群行动创建参数
 */
export interface ClusterActionCreateParams {
  description: string
  created_by: string
  metadata?: Record<string, any> | null
}

/**
 * 集群行动执行参数
 */
export interface ClusterActionExecuteParams {
  action_id: string
  wallet_address?: string | null
  chain?: string | null
}

/**
 * 集群行动分析参数
 */
export interface ClusterActionAnalyzeParams {
  task_description: string
  available_agents: string[]
}

/**
 * 集群行动
 */
export interface ClusterAction {
  id: string
  description: string
  created_by: string
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled'
  metadata?: Record<string, any>
  created_at?: string
  updated_at?: string
  [key: string]: any
}

/**
 * 集群行动状态
 */
export interface ClusterActionStatus {
  actionId: string
  status: 'Pending' | 'Running' | 'Completed' | 'Failed' | 'Cancelled'
  progress?: number
  result?: any
  error?: string
  [key: string]: any
}

/**
 * 集群行动结果
 */
export interface ClusterActionResult {
  actionId: string
  status: string
  result?: any
  error?: string
  [key: string]: any
}

/**
 * 任务分析结果
 */
export interface TaskAnalysisResult {
  needsClusterAction: boolean
  recommendedAgents?: string[]
  reason?: string
  estimatedSteps?: number
  [key: string]: any
}

/**
 * 群聊消息
 */
export interface GroupChatMessage {
  id: string
  content: string
  sender: string
  timestamp: number
  type?: string
  [key: string]: any
}

/**
 * 群聊消息列表响应
 */
export interface GroupChatMessagesResponse {
  messages: GroupChatMessage[]
  topic?: string
  total?: number
}

/**
 * 消息回调函数类型
 */
export type MessageCallback = (message: GroupChatMessage) => void

/**
 * 取消订阅函数类型
 */
export type UnsubscribeFunction = () => void

/**
 * 集群行动服务
 * 用于管理多个智能体协作执行复杂任务
 */
class ClusterActionService {
  // 缓存端点可用性检查结果（避免重复检查）
  // null = 未检查, true = 可用, false = 不可用
  private _endpointAvailable: boolean | null = null
  private _checkingEndpoint = false

  /**
   * 检查端点是否可用（仅在第一次调用时检查）
   * 使用轻量级请求检查，避免产生实际的业务请求
   * @returns 端点是否可用
   */
  private async _checkEndpointOnce(): Promise<boolean> {
    // 如果已经检查过，直接返回结果
    if (this._endpointAvailable !== null) {
      return this._endpointAvailable
    }

    // 如果正在检查，等待检查完成
    if (this._checkingEndpoint) {
      // 等待最多 2 秒
      let waitCount = 0
      while (this._checkingEndpoint && waitCount < 20) {
        await new Promise(resolve => setTimeout(resolve, 100))
        waitCount++
      }
      return this._endpointAvailable === true
    }

    // 开始检查
    this._checkingEndpoint = true
    try {
      // 发送一个轻量级的健康检查请求
      // 使用最小的数据来检查端点是否存在
      const response = await apiClient.post(
        '/cluster-action/create',
        { description: '__health_check__', created_by: '__system__' },
        {
          validateStatus: (status: number) => {
            // 接受 200, 400, 404 状态码
            // 200 = 成功
            // 400 = 端点存在但参数错误（说明端点存在）
            // 404 = 端点不存在
            return status === 200 || status === 400 || status === 404
          },
          timeout: 2000, // 2 秒超时，快速失败
        }
      )

      // 如果返回 400（参数错误），说明端点存在
      this._endpointAvailable = response.status === 200 || response.status === 400
    } catch (error) {
      const axiosError = error as AxiosError
      // 404 或其他错误说明端点不可用
      if (axiosError.response?.status === 404) {
        this._endpointAvailable = false
      } else if (axiosError.code === 'ECONNABORTED' || axiosError.code === 'ERR_NETWORK') {
        // 网络错误，暂时标记为不可用
        this._endpointAvailable = false
      } else {
        // 其他错误，保守处理，标记为不可用
        this._endpointAvailable = false
      }
    } finally {
      this._checkingEndpoint = false
    }

    return this._endpointAvailable === true
  }

  /**
   * 创建集群行动
   * @param description - 行动描述
   * @param createdBy - 创建者（用户ID或"system"）
   * @param metadata - 可选的元数据
   * @returns 创建的集群行动
   */
  async createClusterAction(
    description: string,
    createdBy: string,
    metadata: Record<string, any> | null = null
  ): Promise<ClusterAction> {
    // 先检查端点是否可用（仅第一次调用时检查）
    const isAvailable = await this._checkEndpointOnce()
    
    // 如果端点不可用，直接抛出错误（避免发送实际请求）
    if (!isAvailable) {
      const error = new Error('Cluster action endpoint not available') as Error & { response?: { status: number } }
      error.response = { status: 404 }
      throw error
    }

    try {
      const response = await apiClient.post<ClusterAction>('/cluster-action/create', {
        description,
        created_by: createdBy,
        metadata,
      })
      // 成功则确认端点可用
      this._endpointAvailable = true
      return response.data
    } catch (error) {
      const axiosError = error as AxiosError
      // 如果返回 404，更新端点可用性缓存
      if (axiosError.response?.status === 404) {
        this._endpointAvailable = false
      } else if (axiosError.response?.status !== 404) {
        // 其他错误才输出日志
        console.error('[ClusterActionService] 创建集群行动失败:', error)
      }
      throw error
    }
  }

  /**
   * 执行集群行动
   * @param actionId - 集群行动ID
   * @param walletAddress - 可选的钱包地址
   * @param chain - 可选的链名称
   * @returns 执行结果
   */
  async executeClusterAction(
    actionId: string,
    walletAddress: string | null = null,
    chain: string | null = null
  ): Promise<ClusterActionResult> {
    try {
      const response = await apiClient.post<ClusterActionResult>('/cluster-action/execute', {
        action_id: actionId,
        wallet_address: walletAddress,
        chain,
      })
      return response.data
    } catch (error) {
      const axiosError = error as AxiosError
      if (axiosError.response?.status === 404) {
        console.warn('[ClusterActionService] 执行集群行动 API 未实现')
      } else {
        console.error('[ClusterActionService] 执行集群行动失败:', error)
      }
      throw error
    }
  }

  /**
   * 获取集群行动状态
   * @param actionId - 集群行动ID
   * @returns 状态信息
   */
  async getActionStatus(actionId: string): Promise<ClusterActionStatus> {
    try {
      const response = await apiClient.get<ClusterActionStatus>(`/cluster-action/${actionId}/status`)
      return response.data
    } catch (error) {
      const axiosError = error as AxiosError
      if (axiosError.response?.status === 404) {
        console.warn('[ClusterActionService] 获取状态 API 未实现')
      } else {
        console.error('[ClusterActionService] 获取状态失败:', error)
      }
      throw error
    }
  }

  /**
   * 获取集群行动结果
   * @param actionId - 集群行动ID
   * @returns 执行结果
   */
  async getActionResults(actionId: string): Promise<ClusterActionResult> {
    try {
      const response = await apiClient.get<ClusterActionResult>(`/cluster-action/${actionId}/results`)
      return response.data
    } catch (error) {
      const axiosError = error as AxiosError
      if (axiosError.response?.status === 404) {
        console.warn('[ClusterActionService] 获取结果 API 未实现')
      } else {
        console.error('[ClusterActionService] 获取结果失败:', error)
      }
      throw error
    }
  }

  /**
   * 取消集群行动
   * @param actionId - 集群行动ID
   * @returns 取消结果
   */
  async cancelAction(actionId: string): Promise<ClusterActionResult> {
    try {
      const response = await apiClient.post<ClusterActionResult>(`/cluster-action/${actionId}/cancel`)
      return response.data
    } catch (error) {
      const axiosError = error as AxiosError
      if (axiosError.response?.status === 404) {
        console.warn('[ClusterActionService] 取消行动 API 未实现')
      } else {
        console.error('[ClusterActionService] 取消行动失败:', error)
      }
      throw error
    }
  }

  /**
   * 分析任务是否需要集群行动
   * @param taskDescription - 任务描述
   * @param availableAgents - 可用智能体ID列表
   * @returns 分析结果
   */
  async analyzeTask(taskDescription: string, availableAgents: string[]): Promise<TaskAnalysisResult> {
    try {
      const response = await apiClient.post<TaskAnalysisResult>('/cluster-action/analyze', {
        task_description: taskDescription,
        available_agents: availableAgents,
      })
      return response.data
    } catch (error) {
      const axiosError = error as AxiosError
      if (axiosError.response?.status === 404) {
        console.warn('[ClusterActionService] 分析任务 API 未实现')
      } else {
        console.error('[ClusterActionService] 分析任务失败:', error)
      }
      throw error
    }
  }

  /**
   * 轮询获取集群行动状态（直到完成或失败）
   * @param actionId - 集群行动ID
   * @param interval - 轮询间隔（毫秒），默认1000ms
   * @param timeout - 超时时间（毫秒），默认60000ms
   * @returns 最终状态
   */
  async pollActionStatus(
    actionId: string,
    interval = 1000,
    timeout = 60000
  ): Promise<ClusterActionStatus> {
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

  /**
   * 获取群聊消息（PubSub）
   * @param actionId - 集群行动ID
   * @param since - 可选的时间戳，只获取此时间之后的消息
   * @returns 消息列表
   */
  async getGroupChatMessages(
    actionId: string,
    since: number | null = null
  ): Promise<GroupChatMessagesResponse> {
    try {
      const topic = `diap/cluster_action/${actionId}`
      const params: { topic: string; since?: number } = { topic }
      if (since) {
        params.since = since
      }
      const response = await apiClient.get<GroupChatMessagesResponse>('/pubsub/messages', { params })
      return response.data
    } catch (error) {
      console.error('[ClusterActionService] 获取群聊消息失败:', error)
      throw error
    }
  }

  /**
   * 订阅群聊消息（轮询方式）
   * @param actionId - 集群行动ID
   * @param onMessage - 消息回调函数
   * @param interval - 轮询间隔（毫秒），默认1000ms
   * @returns 取消订阅函数
   */
  subscribeToGroupChat(
    actionId: string,
    onMessage: MessageCallback,
    interval = 1000
  ): UnsubscribeFunction {
    if (!actionId || !onMessage) {
      console.warn('[ClusterActionService] subscribeToGroupChat 需要 actionId 和 onMessage')
      return () => {}
    }

    let lastTimestamp = Date.now()
    let isSubscribed = true

    const poll = async () => {
      if (!isSubscribed) return

      try {
        const response = await this.getGroupChatMessages(actionId, lastTimestamp)
        if (response?.messages && response.messages.length > 0) {
          response.messages.forEach((msg) => {
            onMessage(msg)
            if (msg.timestamp > lastTimestamp) {
              lastTimestamp = msg.timestamp
            }
          })
        }
      } catch (error) {
        console.error('[ClusterActionService] 订阅群聊消息失败:', error)
      }
    }

    // 立即执行一次
    poll()

    // 设置轮询
    const pollInterval = setInterval(() => {
      if (isSubscribed) {
        poll()
      }
    }, interval)

    // 返回取消订阅函数
    return () => {
      isSubscribed = false
      if (pollInterval) {
        clearInterval(pollInterval as unknown as number)
      }
    }
  }
}

export default new ClusterActionService()
