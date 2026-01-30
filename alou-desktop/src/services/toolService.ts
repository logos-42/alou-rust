/**
 * 工具服务 - 统一管理本地和远程工具执行
 * 支持智能路由：本地优先，远程回退，异步任务处理
 */

import apiClient from './api'
import type { AxiosError } from 'axios'

/**
 * 工具执行选项
 */
export interface ToolExecutionOptions {
  /** 优先本地执行 */
  preferLocal?: boolean
  /** 超时时间（毫秒） */
  timeout?: number
  /** 允许异步执行 */
  allowAsync?: boolean
  /** 会话ID */
  sessionId?: string | null
}

/**
 * 远程执行选项
 */
export interface RemoteExecutionOptions {
  timeout?: number
  allowAsync?: boolean
  sessionId?: string | null
}

/**
 * 本地工具执行结果
 */
export interface LocalToolResult {
  success: boolean
  data?: any
  execution_time_ms?: number
  output?: string
  warnings?: string[]
  error?: string
}

/**
 * 远程工具执行结果
 */
export interface RemoteToolResult {
  success: boolean
  data?: any
  result?: any
  execution_time_ms?: number
  output?: string
  message?: string
  warnings?: string[]
  task_id?: string
}

/**
 * 工具任务状态响应
 */
export interface ToolTaskStatus {
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled'
  result?: any
  execution_time_ms?: number
  output?: string
  warnings?: string[]
  error?: string
}

/**
 * 工具执行结果
 */
export interface ToolExecutionResult {
  success: boolean
  data?: any
  executionTimeMs: number
  output?: string
  warnings?: string[]
  executionMode: 'local' | 'remote'
  toolId: string
  timestamp: number
  taskId?: string
}

/**
 * 工具信息
 */
export interface ToolInfo {
  id: string
  name: string
  category: string
  executionMode?: 'local' | 'remote'
  availableModes?: string[]
  [key: string]: any
}

/**
 * 工具调用
 */
export interface ToolCall {
  tool: string
  args?: Record<string, any>
  id?: string
}

/**
 * 批量执行结果
 */
export interface BatchExecutionResult {
  tool: string
  success: boolean
  result?: ToolExecutionResult
  error?: string
  toolCallId?: string
}

/**
 * 执行历史记录
 */
export interface ExecutionHistoryItem {
  id: string
  toolId: string
  timestamp: number
  success: boolean
  executionMode: 'local' | 'remote'
  [key: string]: any
}

class ToolService {
  private localTools: Set<string>
  private complexTools: Set<string>

  constructor() {
    this.localTools = new Set([
      'filesystem', 'search', 'bash', 'plan', 'skills'
    ])
    this.complexTools = new Set([
      'network_scan', 'large_file_process', 'long_running_task'
    ])
  }

  /**
   * 执行工具（智能路由）
   * @param toolId - 工具ID
   * @param args - 工具参数
   * @param options - 执行选项
   * @returns 执行结果
   */
  async executeTool(
    toolId: string,
    args: Record<string, any>,
    options: ToolExecutionOptions = {}
  ): Promise<ToolExecutionResult> {
    const {
      preferLocal = true,
      timeout = 30000,
      allowAsync = true,
      sessionId = null
    } = options

    console.log(`[ToolService] 执行工具: ${toolId}`, { preferLocal, timeout, allowAsync })

    // 1. 优先尝试本地执行
    if (preferLocal && this.canExecuteLocally(toolId)) {
      try {
        console.log(`[ToolService] 尝试本地执行: ${toolId}`)
        const result = await this.executeLocalTool(toolId, args, timeout)
        return {
          ...result,
          executionMode: 'local',
          toolId,
          timestamp: Date.now()
        }
      } catch (error) {
        console.warn(`[ToolService] 本地执行失败，回退到远程: ${(error as Error).message}`)
        // 继续到远程执行
      }
    }

    // 2. 远程执行（同步或异步）
    console.log(`[ToolService] 使用远程执行: ${toolId}`)
    const result = await this.executeRemoteTool(toolId, args, {
      timeout,
      allowAsync,
      sessionId
    })

    return {
      ...result,
      executionMode: 'remote',
      toolId,
      timestamp: Date.now()
    }
  }

  /**
   * 检查工具是否可以本地执行
   * @param toolId - 工具ID
   * @returns 是否可以本地执行
   */
  canExecuteLocally(toolId: string): boolean {
    return this.localTools.has(toolId)
  }

  /**
   * 检查工具是否需要异步执行
   * @param toolId - 工具ID
   * @param args - 工具参数
   * @returns 是否需要异步执行
   */
  shouldExecuteAsync(toolId: string, args: Record<string, any>): boolean {
    // 复杂工具强制异步
    if (this.complexTools.has(toolId)) {
      return true
    }

    // 大文件操作异步
    if (toolId === 'filesystem' && args.operation === 'copy' && args.recursive) {
      return true
    }

    // 长时间搜索异步
    if (toolId === 'search' && args.type === 'code') {
      return true
    }

    return false
  }

  /**
   * 本地工具执行（通过Tauri）
   * @param toolId - 工具ID
   * @param args - 工具参数
   * @param timeout - 超时时间
   * @returns 执行结果
   */
  private async executeLocalTool(
    toolId: string,
    args: Record<string, any>,
    timeout: number
  ): Promise<Omit<ToolExecutionResult, 'executionMode' | 'toolId' | 'timestamp'>> {
    const { invoke } = await import('@tauri-apps/api/core')

    try {
      const result = await invoke<LocalToolResult>('execute_tool', {
        toolId,
        args: JSON.stringify(args),
        timeout
      })

      if (!result.success) {
        throw new Error(result.error || 'Tool execution failed')
      }

      return {
        success: true,
        data: result.data,
        executionTimeMs: result.execution_time_ms || 0,
        output: result.output,
        warnings: result.warnings || []
      }
    } catch (error) {
      console.error(`[ToolService] 本地工具执行失败: ${toolId}`, error)
      throw new Error(`Local execution failed: ${(error as Error).message}`)
    }
  }

  /**
   * 远程工具执行（通过Workers）
   * @param toolId - 工具ID
   * @param args - 工具参数
   * @param options - 执行选项
   * @returns 执行结果
   */
  private async executeRemoteTool(
    toolId: string,
    args: Record<string, any>,
    options: RemoteExecutionOptions = {}
  ): Promise<Omit<ToolExecutionResult, 'executionMode' | 'toolId' | 'timestamp'>> {
    const { timeout = 30000, allowAsync = true, sessionId = null } = options

    const requestData = {
      tool_id: toolId,
      args,
      session_id: sessionId,
      async: allowAsync && this.shouldExecuteAsync(toolId, args)
    }

    try {
      const response = await apiClient.post<RemoteToolResult>('/tools/execute', requestData, {
        timeout
      })

      const result = response.data

      // 如果是异步任务，等待完成
      if (result.task_id && allowAsync) {
        console.log(`[ToolService] 异步任务创建: ${result.task_id}`)
        return await this.waitForToolCompletion(result.task_id, timeout)
      }

      return {
        success: result.success !== false,
        data: result.data || result.result || {},
        executionTimeMs: result.execution_time_ms || 0,
        output: result.output || result.message,
        warnings: result.warnings || [],
        taskId: result.task_id
      }
    } catch (error) {
      console.error(`[ToolService] 远程工具执行失败: ${toolId}`, error)

      const axiosError = error as AxiosError
      if (axiosError.code === 'ECONNABORTED' || axiosError.message?.includes('timeout')) {
        throw new Error(`远程工具执行超时 (${timeout}ms)`)
      }

      throw new Error(`Remote execution failed: ${(error as Error).message}`)
    }
  }

  /**
   * 等待工具任务完成
   * @param taskId - 任务ID
   * @param timeout - 总超时时间
   * @returns 任务结果
   */
  private async waitForToolCompletion(
    taskId: string,
    timeout: number = 60000
  ): Promise<Omit<ToolExecutionResult, 'executionMode' | 'toolId' | 'timestamp'>> {
    const startTime = Date.now()
    const pollInterval = 2000 // 2秒轮询一次

    return new Promise((resolve, reject) => {
      const checkStatus = async () => {
        try {
          // 检查是否超时
          if (Date.now() - startTime > timeout) {
            reject(new Error(`Tool execution timeout (${timeout}ms)`))
            return
          }

          // 查询任务状态
          const response = await apiClient.get<ToolTaskStatus>(`/api/tools/status/${taskId}`)
          const status = response.data

          console.log(`[ToolService] 任务状态: ${taskId} -> ${status.status}`)

          switch (status.status) {
            case 'completed':
              resolve({
                success: true,
                data: status.result || {},
                executionTimeMs: status.execution_time_ms || 0,
                output: status.output || 'Task completed',
                warnings: status.warnings || [],
                taskId
              })
              break

            case 'failed':
              reject(new Error(status.error || 'Task execution failed'))
              break

            case 'cancelled':
              reject(new Error('Task was cancelled'))
              break

            case 'running':
            case 'pending':
              // 继续轮询
              setTimeout(checkStatus, pollInterval)
              break

            default:
              reject(new Error(`Unknown task status: ${status.status}`))
              break
          }
        } catch (error) {
          reject(new Error(`Status check failed: ${(error as Error).message}`))
        }
      }

      // 开始轮询
      checkStatus()
    })
  }

  /**
   * 获取工具列表
   * @returns 工具列表
   */
  async getToolList(): Promise<ToolInfo[]> {
    try {
      // 优先从本地获取
      const localTools = await this.getLocalToolList()

      // 从远程获取（用于比较和补充）
      const remoteTools = await this.getRemoteToolList()

      // 合并结果
      return this.mergeToolLists(localTools, remoteTools)
    } catch (error) {
      console.warn('[ToolService] 获取工具列表失败:', error)
      // 回退到本地工具列表
      return this.getLocalToolList()
    }
  }

  /**
   * 获取本地工具列表
   * @returns 本地工具列表
   */
  private async getLocalToolList(): Promise<ToolInfo[]> {
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      const result = await invoke<{ tools: Array<Record<string, any>> }>('get_tool_list')

      return result.tools.map(tool => ({
        ...tool,
        id: tool.id || 'unknown',
        name: tool.name || tool.id || 'Unknown Tool',
        category: tool.category || 'General',
        executionMode: 'local' as const
      }))
    } catch (error) {
      console.warn('[ToolService] 获取本地工具列表失败:', error)
      // 返回静态列表作为回退
      return [
        { id: 'filesystem', name: 'File System', category: 'FileSystem', executionMode: 'local' },
        { id: 'search', name: 'Search', category: 'Search', executionMode: 'local' },
        { id: 'bash', name: 'Bash Shell', category: 'Terminal', executionMode: 'local' },
        { id: 'plan', name: 'Task Planning', category: 'Planning', executionMode: 'local' },
        { id: 'skills', name: 'Skills', category: 'Skills', executionMode: 'local' }
      ]
    }
  }

  /**
   * 获取远程工具列表
   * @returns 远程工具列表
   */
  private async getRemoteToolList(): Promise<ToolInfo[]> {
    try {
      const response = await apiClient.get<{ tools: Array<Record<string, any>> }>('/tools/list')
      return response.data.tools.map(tool => ({
        ...tool,
        id: tool.id || 'unknown',
        name: tool.name || tool.id || 'Unknown Tool',
        category: tool.category || 'General',
        executionMode: 'remote' as const
      }))
    } catch (error) {
      console.warn('[ToolService] 获取远程工具列表失败:', error)
      return []
    }
  }

  /**
   * 合并工具列表
   * @param localTools - 本地工具
   * @param remoteTools - 远程工具
   * @returns 合并后的工具列表
   */
  private mergeToolLists(localTools: ToolInfo[], remoteTools: ToolInfo[]): ToolInfo[] {
    const merged = new Map<string, ToolInfo>()

    // 添加本地工具
    localTools.forEach(tool => {
      merged.set(tool.id, { ...tool, availableModes: ['local'] })
    })

    // 添加或更新远程工具
    remoteTools.forEach(tool => {
      if (merged.has(tool.id)) {
        const existing = merged.get(tool.id)!
        const modes = existing.availableModes || []
        if (!modes.includes('remote')) {
          existing.availableModes = [...modes, 'remote']
        }
        // 优先使用本地版本的元信息
      } else {
        merged.set(tool.id, { ...tool, availableModes: ['remote'] })
      }
    })

    return Array.from(merged.values())
  }

  /**
   * 批量执行工具
   * @param toolCalls - 工具调用数组
   * @param options - 执行选项
   * @returns 执行结果数组
   */
  async executeBatch(
    toolCalls: ToolCall[],
    options: ToolExecutionOptions = {}
  ): Promise<BatchExecutionResult[]> {
    const results: BatchExecutionResult[] = []

    for (const toolCall of toolCalls) {
      try {
        const result = await this.executeTool(
          toolCall.tool,
          toolCall.args || {},
          options
        )
        results.push({
          tool: toolCall.tool,
          success: true,
          result,
          toolCallId: toolCall.id
        })
      } catch (error) {
        results.push({
          tool: toolCall.tool,
          success: false,
          error: (error as Error).message,
          toolCallId: toolCall.id
        })
      }
    }

    return results
  }

  /**
   * 取消工具执行
   * @param executionId - 执行ID
   * @returns 是否成功取消
   */
  async cancelExecution(executionId: string): Promise<boolean> {
    try {
      // 尝试本地取消
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('cancel_tool_execution', { executionId })
      return true
    } catch (error) {
      // 尝试远程取消
      try {
        await apiClient.post(`/api/tools/cancel/${executionId}`)
        return true
      } catch (remoteError) {
        console.error('[ToolService] 取消执行失败:', error, remoteError)
        return false
      }
    }
  }

  /**
   * 获取执行历史
   * @param limit - 限制数量
   * @returns 执行历史
   */
  async getExecutionHistory(limit: number = 50): Promise<ExecutionHistoryItem[]> {
    try {
      // 从本地存储获取历史
      const localHistory = await this.getLocalExecutionHistory(limit)

      // 从远程获取历史
      const remoteHistory = await this.getRemoteExecutionHistory(limit)

      // 合并并排序
      const combined = [...localHistory, ...remoteHistory]
      combined.sort((a, b) => b.timestamp - a.timestamp)

      return combined.slice(0, limit)
    } catch (error) {
      console.warn('[ToolService] 获取执行历史失败:', error)
      return []
    }
  }

  /**
   * 获取本地执行历史
   * @param limit - 限制数量
   * @returns 本地执行历史
   */
  private async getLocalExecutionHistory(limit: number = 50): Promise<ExecutionHistoryItem[]> {
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      const result = await invoke<{ history: ExecutionHistoryItem[] }>('get_execution_history', { limit })
      return result.history || []
    } catch (error) {
      return []
    }
  }

  /**
   * 获取远程执行历史
   * @param limit - 限制数量
   * @returns 远程执行历史
   */
  private async getRemoteExecutionHistory(limit: number = 50): Promise<ExecutionHistoryItem[]> {
    try {
      const response = await apiClient.get<{ history: ExecutionHistoryItem[] }>('/tools/history', {
        params: { limit }
      })
      return response.data.history || []
    } catch (error) {
      return []
    }
  }
}

// 创建单例实例
const toolService = new ToolService()

export default toolService

// 导出类和实例
export { ToolService, toolService }
