/**
 * Workflow Service - 桌面版工作流管理服务
 * 直接使用 Claude Agent SDK 执行工作流
 */
import { invoke } from '@tauri-apps/api/core'

/**
 * 工作流模式枚举
 */
export enum WorkflowMode {
  INTERACTIVE = 'interactive',
  AUTO = 'auto',
  PARALLEL = 'parallel',
  SMART = 'smart'
}

/**
 * 工作流步骤定义
 */
export interface WorkflowStepDefinition {
  id: string
  name: string
  tool: string
  args?: Record<string, any>
  depends_on?: string[]
}

/**
 * 工作流定义
 */
export interface WorkflowDefinition {
  name: string
  description?: string
  steps: WorkflowStepDefinition[]
}

/**
 * 智能体信息
 */
export interface AgentInfo {
  name?: string
  description?: string
  [key: string]: any
}

/**
 * 创建工作流结果
 */
export interface CreateWorkflowResult {
  success: boolean
  workflowId?: string
  message?: string
  error?: string
}

/**
 * 执行工作流结果
 */
export interface ExecuteWorkflowResult {
  success: boolean
  executionId?: string
  workflowId?: string
  status?: string
  message?: string
  error?: string
}

/**
 * 执行状态结果
 */
export interface ExecutionStatusResult {
  success: boolean
  execution?: any
  status?: string
  progress?: number
  currentStep?: string
  error?: string
}

/**
 * 暂停/恢复/取消执行结果
 */
export interface ExecutionControlResult {
  success: boolean
  executionId?: string
  status?: string
  message?: string
  error?: string
}

/**
 * 工作流状态结果
 */
export interface WorkflowStatusResult {
  success: boolean
  data?: {
    status: string
    progress: number
    current_step?: string
    result?: any
    error?: string
  }
  workflow?: any
  status?: string
  steps?: any[]
  error?: string
}

/**
 * 工作流列表结果
 */
export interface WorkflowListResult {
  success: boolean
  workflows?: any[]
  error?: string
}

/**
 * 删除工作流结果
 */
export interface DeleteWorkflowResult {
  success: boolean
  message?: string
  error?: string
}

/**
 * 重试步骤结果
 */
export interface RetryStepResult {
  success: boolean
  result?: any
  message?: string
  error?: string
}

/**
 * 暂停/恢复工作流结果
 */
export interface WorkflowControlResult {
  success: boolean
  result?: any
  message?: string
  error?: string
}

/**
 * 进度回调函数
 */
export type ProgressCallback = (progress: number, message?: string) => void

export class WorkflowService {
  private currentMode: WorkflowMode = WorkflowMode.SMART
  private executionCount: number = 0
  private successCount: number = 0
  private failureCount: number = 0
  private lastExecutionTime: number = 0

  /**
   * 切换工作流模式
   * @param mode - 新的工作流模式
   * @returns 切换结果
   */
  async switchMode(mode: WorkflowMode): Promise<{ success: boolean; message?: string }> {
    try {
      this.currentMode = mode
      return {
        success: true,
        message: `已切换到 ${mode} 模式`
      }
    } catch (error) {
      console.error('[WorkflowService] 切换模式失败:', error)
      return {
        success: false,
        message: (error as Error).message || '切换模式失败'
      }
    }
  }

  /**
   * 获取工作流统计信息
   * @returns 统计信息
   */
  getStats(): {
    totalExecutions: number
    successCount: number
    failureCount: number
    currentMode: WorkflowMode
    lastExecutionTime: number
  } {
    return {
      totalExecutions: this.executionCount,
      successCount: this.successCount,
      failureCount: this.failureCount,
      currentMode: this.currentMode,
      lastExecutionTime: this.lastExecutionTime
    }
  }

  /**
   * 重置工作流状态
   */
  reset(): void {
    this.executionCount = 0
    this.successCount = 0
    this.failureCount = 0
    this.lastExecutionTime = 0
  }

  /**
   * 创建工作流
   * @param workflow - 工作流定义
   * @param agentInfo - 智能体信息
   * @returns 创建结果
   */
  async createWorkflow(
    workflow: WorkflowDefinition,
    agentInfo: AgentInfo = {}
  ): Promise<CreateWorkflowResult> {
    try {
      this.executionCount++
      const result = await invoke<{ workflow_id: string }>('create_workflow', {
        workflow: {
          name: workflow.name,
          description: workflow.description,
          steps: workflow.steps.map(step => ({
            id: step.id,
            name: step.name,
            tool: step.tool,
            args: step.args || {},
            depends_on: step.depends_on || []
          }))
        },
        agentInfo
      })

      this.successCount++
      this.lastExecutionTime = Date.now()
      return {
        success: true,
        workflowId: result.workflow_id,
        message: '工作流创建成功'
      }
    } catch (error) {
      console.error('[WorkflowService] 创建工作流失败:', error)
      this.failureCount++
      this.lastExecutionTime = Date.now()
      return {
        success: false,
        error: (error as Error).message || '创建工作流失败'
      }
    }
  }

  /**
   * 执行工作流（异步）
   * @param workflowId - 工作流ID
   * @param apiKey - Claude API密钥
   * @param agentInfo - 智能体信息
   * @param onProgress - 进度回调函数
   * @returns 执行结果
   */
  async executeWorkflow(
    workflowId: string,
    apiKey: string,
    agentInfo: AgentInfo = {}
  ): Promise<ExecuteWorkflowResult> {
    try {
      const result = await invoke<{
        execution_id: string
        workflow_id: string
        status: string
        message: string
      }>('execute_workflow', {
        workflowId,
        apiKey,
        agentInfo,
      })

      // 返回执行ID，表明异步执行已启动
      return {
        success: true,
        executionId: result.execution_id,
        workflowId: result.workflow_id,
        status: result.status,
        message: result.message
      }
    } catch (error) {
      console.error('[WorkflowService] 执行工作流失败:', error)
      return {
        success: false,
        error: (error as Error).message || '执行工作流失败'
      }
    }
  }

  /**
   * 获取执行状态
   * @param executionId - 执行ID
   * @returns 执行状态
   */
  async getExecutionStatus(executionId: string): Promise<ExecutionStatusResult> {
    try {
      const result = await invoke<{
        execution: any
        status: string
        progress: number
        current_step: string
      }>('get_execution_status', { executionId })

      return {
        success: true,
        execution: result.execution,
        status: result.status,
        progress: result.progress,
        currentStep: result.current_step
      }
    } catch (error) {
      console.error('[WorkflowService] 获取执行状态失败:', error)
      return {
        success: false,
        error: (error as Error).message || '获取执行状态失败'
      }
    }
  }

  /**
   * 暂停执行
   * @param executionId - 执行ID
   * @returns 暂停结果
   */
  async pauseExecution(executionId: string): Promise<ExecutionControlResult> {
    console.log('[WorkflowService] pauseExecution called with:', executionId)
    try {
      console.log('[WorkflowService] Calling pause_execution Tauri command');
        const result = await invoke<{
        execution_id: string
        status: string
        message: string
      }>('pause_execution', { executionId })

      return {
        success: true,
        executionId: result.execution_id,
        status: result.status,
        message: result.message
      }
    } catch (error) {
      console.error('[WorkflowService] 暂停执行失败:', error)
      return {
        success: false,
        error: (error as Error).message || '暂停执行失败'
      }
    }
  }

  /**
   * 恢复执行
   * @param executionId - 执行ID
   * @returns 恢复结果
   */
  async resumeExecution(executionId: string): Promise<ExecutionControlResult> {
    try {
      const result = await invoke<{
        execution_id: string
        status: string
        message: string
      }>('resume_execution', { executionId })

      return {
        success: true,
        executionId: result.execution_id,
        status: result.status,
        message: result.message
      }
    } catch (error) {
      console.error('[WorkflowService] 恢复执行失败:', error)
      return {
        success: false,
        error: (error as Error).message || '恢复执行失败'
      }
    }
  }

  /**
   * 取消执行
   * @param executionId - 执行ID
   * @returns 取消结果
   */
  async cancelExecution(executionId: string): Promise<ExecutionControlResult> {
    try {
      const result = await invoke<{
        execution_id: string
        status: string
        message: string
      }>('cancel_execution', { executionId })

      return {
        success: true,
        executionId: result.execution_id,
        status: result.status,
        message: result.message
      }
    } catch (error) {
      console.error('[WorkflowService] 取消执行失败:', error)
      return {
        success: false,
        error: (error as Error).message || '取消执行失败'
      }
    }
  }

  /**
   * 获取工作流状态
   * @param workflowId - 工作流ID
   * @returns 工作流状态
   */
  async getWorkflowStatus(workflowId: string): Promise<WorkflowStatusResult> {
    try {
      const result = await invoke<{
        data: {
          status: string
          progress: number
          current_step?: string
          result?: any
          error?: string
        }
      }>('get_workflow_status', { workflowId })

      return {
        success: true,
        data: result.data,
        status: result.data.status,
        workflow: result.data
      }
    } catch (error) {
      console.error('[WorkflowService] 获取工作流状态失败:', error)
      return {
        success: false,
        error: (error as Error).message || '获取工作流状态失败'
      }
    }
  }

  /**
   * 列出所有工作流
   * @returns 工作流列表
   */
  async listWorkflows(): Promise<WorkflowListResult> {
    try {
      const result = await invoke<{ workflows: any[] }>('list_workflows')

      return {
        success: true,
        workflows: result.workflows || []
      }
    } catch (error) {
      console.error('[WorkflowService] 列出工作流失败:', error)
      return {
        success: false,
        error: (error as Error).message || '列出工作流失败'
      }
    }
  }

  /**
   * 删除工作流
   * @param workflowId - 工作流ID
   * @returns 删除结果
   */
  async deleteWorkflow(workflowId: string): Promise<DeleteWorkflowResult> {
    try {
      await invoke('delete_workflow', { workflowId })

      return {
        success: true,
        message: '工作流删除成功'
      }
    } catch (error) {
      console.error('[WorkflowService] 删除工作流失败:', error)
      return {
        success: false,
        error: (error as Error).message || '删除工作流失败'
      }
    }
  }

  /**
   * 重试失败的步骤
   * @param workflowId - 工作流ID
   * @param stepId - 步骤ID
   * @param apiKey - Claude API密钥
   * @param agentInfo - 智能体信息
   * @returns 重试结果
   */
  async retryStep(
    workflowId: string,
    stepId: string,
    apiKey: string,
    agentInfo: AgentInfo = {}
  ): Promise<RetryStepResult> {
    try {
      const result = await invoke<{ step_result: any }>('retry_workflow_step', {
        workflowId,
        stepId,
        apiKey,
        agentInfo
      })

      return {
        success: true,
        result: result.step_result,
        message: '步骤重试完成'
      }
    } catch (error) {
      console.error('[WorkflowService] 重试步骤失败:', error)
      return {
        success: false,
        error: (error as Error).message || '重试步骤失败'
      }
    }
  }

  /**
   * 暂停工作流
   * @param workflowId - 工作流ID
   * @returns 暂停结果
   */
  async pauseWorkflow(workflowId: string): Promise<WorkflowControlResult> {
    try {
      await invoke('pause_workflow', { workflowId })

      return {
        success: true,
        message: '工作流已暂停'
      }
    } catch (error) {
      console.error('[WorkflowService] 暂停工作流失败:', error)
      return {
        success: false,
        error: (error as Error).message || '暂停工作流失败'
      }
    }
  }

  /**
   * 恢复工作流
   * @param workflowId - 工作流ID
   * @param apiKey - Claude API密钥
   * @param agentInfo - 智能体信息
   * @returns 恢复结果
   */
  async resumeWorkflow(
    workflowId: string,
    apiKey: string,
    agentInfo: AgentInfo = {}
  ): Promise<WorkflowControlResult> {
    try {
      const result = await invoke<{ execution_result: any }>('resume_workflow', {
        workflowId,
        apiKey,
        agentInfo
      })

      return {
        success: true,
        result: result.execution_result,
        message: '工作流已恢复'
      }
    } catch (error) {
      console.error('[WorkflowService] 恢复工作流失败:', error)
      return {
        success: false,
        error: (error as Error).message || '恢复工作流失败'
      }
    }
  }

  /**
   * 创建示例工作流
   * @returns 示例工作流定义
   */
  createSampleWorkflow(): WorkflowDefinition {
    return {
      name: "示例工作流",
      description: "演示桌面版Claude SDK工作流执行的示例",
      steps: [
        {
          id: "analyze_task",
          name: "分析任务需求",
          tool: "claude_analysis",
          args: {
            task: "分析用户的需求并制定执行计划"
          },
          depends_on: []
        },
        {
          id: "design_solution",
          name: "设计解决方案",
          tool: "claude_design",
          args: {
            requirements: "基于分析结果设计具体的解决方案"
          },
          depends_on: ["analyze_task"]
        },
        {
          id: "implement_solution",
          name: "实施解决方案",
          tool: "claude_implement",
          args: {
            plan: "根据设计方案实施具体步骤"
          },
          depends_on: ["design_solution"]
        },
        {
          id: "validate_result",
          name: "验证结果",
          tool: "claude_validate",
          args: {
            output: "验证实施结果是否符合预期"
          },
          depends_on: ["implement_solution"]
        }
      ]
    }
  }
}

// 创建单例实例
const workflowService = new WorkflowService();

export default workflowService;
