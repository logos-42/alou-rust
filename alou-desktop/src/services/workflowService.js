/**
 * Workflow Service - 桌面版工作流管理服务
 * 直接使用 Claude Agent SDK 执行工作流
 */
import { invoke } from '@tauri-apps/api/core'

export class WorkflowService {
  /**
   * 创建工作流
   * @param {Object} workflow - 工作流定义
   * @param {string} workflow.name - 工作流名称
   * @param {string} workflow.description - 工作流描述
   * @param {Array} workflow.steps - 工作流步骤
   * @param {Object} agentInfo - 智能体信息
   * @returns {Promise<Object>} 创建结果
   */
  async createWorkflow(workflow, agentInfo = {}) {
    try {
      const result = await invoke('create_workflow', {
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

      return {
        success: true,
        workflowId: result.workflow_id,
        message: '工作流创建成功'
      }
    } catch (error) {
      console.error('[WorkflowService] 创建工作流失败:', error)
      return {
        success: false,
        error: error.message || '创建工作流失败'
      }
    }
  }

  /**
   * 执行工作流
   * @param {string} workflowId - 工作流ID
   * @param {string} apiKey - Claude API密钥
   * @param {Object} agentInfo - 智能体信息
   * @param {Function} onProgress - 进度回调函数
   * @returns {Promise<Object>} 执行结果
   */
  async executeWorkflow(workflowId, apiKey, agentInfo = {}, onProgress = null) {
    try {
      const result = await invoke('execute_workflow', {
        workflowId,
        apiKey,
        agentInfo,
        onProgress: !!onProgress
      })

      // 处理实时进度更新
      if (onProgress && result.progress_stream) {
        // 这里可以设置事件监听器来接收实时进度
        this._setupProgressListener(onProgress)
      }

      return {
        success: true,
        result: result.execution_result,
        steps: result.step_results || [],
        message: '工作流执行完成'
      }
    } catch (error) {
      console.error('[WorkflowService] 执行工作流失败:', error)
      return {
        success: false,
        error: error.message || '执行工作流失败'
      }
    }
  }

  /**
   * 获取工作流状态
   * @param {string} workflowId - 工作流ID
   * @returns {Promise<Object>} 工作流状态
   */
  async getWorkflowStatus(workflowId) {
    try {
      const result = await invoke('get_workflow_status', { workflowId })

      return {
        success: true,
        workflow: result.workflow,
        status: result.status,
        steps: result.steps || []
      }
    } catch (error) {
      console.error('[WorkflowService] 获取工作流状态失败:', error)
      return {
        success: false,
        error: error.message || '获取工作流状态失败'
      }
    }
  }

  /**
   * 列出所有工作流
   * @returns {Promise<Object>} 工作流列表
   */
  async listWorkflows() {
    try {
      const result = await invoke('list_workflows')

      return {
        success: true,
        workflows: result.workflows || []
      }
    } catch (error) {
      console.error('[WorkflowService] 列出工作流失败:', error)
      return {
        success: false,
        error: error.message || '列出工作流失败'
      }
    }
  }

  /**
   * 删除工作流
   * @param {string} workflowId - 工作流ID
   * @returns {Promise<Object>} 删除结果
   */
  async deleteWorkflow(workflowId) {
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
        error: error.message || '删除工作流失败'
      }
    }
  }

  /**
   * 重试失败的步骤
   * @param {string} workflowId - 工作流ID
   * @param {string} stepId - 步骤ID
   * @param {string} apiKey - Claude API密钥
   * @param {Object} agentInfo - 智能体信息
   * @returns {Promise<Object>} 重试结果
   */
  async retryStep(workflowId, stepId, apiKey, agentInfo = {}) {
    try {
      const result = await invoke('retry_workflow_step', {
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
        error: error.message || '重试步骤失败'
      }
    }
  }

  /**
   * 暂停工作流
   * @param {string} workflowId - 工作流ID
   * @returns {Promise<Object>} 暂停结果
   */
  async pauseWorkflow(workflowId) {
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
        error: error.message || '暂停工作流失败'
      }
    }
  }

  /**
   * 恢复工作流
   * @param {string} workflowId - 工作流ID
   * @param {string} apiKey - Claude API密钥
   * @param {Object} agentInfo - 智能体信息
   * @returns {Promise<Object>} 恢复结果
   */
  async resumeWorkflow(workflowId, apiKey, agentInfo = {}) {
    try {
      const result = await invoke('resume_workflow', {
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
        error: error.message || '恢复工作流失败'
      }
    }
  }

  /**
   * 创建示例工作流
   * @returns {Object} 示例工作流定义
   */
  createSampleWorkflow() {
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

  /**
   * 设置进度监听器
   * @private
   */
  _setupProgressListener(onProgress) {
    // 这里可以设置事件监听器来接收来自Tauri后端的实时进度更新
    // 由于Tauri的事件系统，这里需要根据实际实现调整
    console.log('[WorkflowService] 设置进度监听器')
  }
}

export default new WorkflowService()