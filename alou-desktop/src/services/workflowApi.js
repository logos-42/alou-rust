/**
 * Workflow API - 工作流管理API客户端
 * 调用后端 /workflow 接口来管理工作流
 */

import apiClient from './api';

/**
 * 创建新的工作流
 */
export async function createWorkflow(workflow, agentInfo = {}) {
  try {
    const response = await apiClient.post('/workflow/create', {
      workflow,
      agent_info: agentInfo,
    });
    
    return {
      success: true,
      workflowId: response.data.workflow_id,
      message: response.data.message || '工作流创建成功',
      ...response.data,
    };
  } catch (error) {
    console.error('[workflowApi] 创建工作流失败:', error);
    return {
      success: false,
      error: error.message || '创建工作流失败',
    };
  }
}

/**
 * 执行工作流
 */
export async function executeWorkflow(workflowId, options = {}, agentInfo = {}) {
  try {
    const response = await apiClient.post('/workflow/execute', {
      workflow_id: workflowId,
      options,
      agent_info: agentInfo,
    });
    
    return {
      success: true,
      executionId: response.data.execution_id,
      message: response.data.message || '工作流执行已开始',
      ...response.data,
    };
  } catch (error) {
    console.error('[workflowApi] 执行工作流失败:', error);
    return {
      success: false,
      error: error.message || '执行工作流失败',
    };
  }
}

/**
 * 获取工作流状态
 */
export async function getWorkflowStatus(workflowId, executionId = null) {
  try {
    const response = await apiClient.get('/workflow/status', {
      params: {
        workflow_id: workflowId,
        execution_id: executionId,
      },
    });
    
    return {
      success: true,
      status: response.data.status,
      progress: response.data.progress || 0,
      steps: response.data.steps || [],
      result: response.data.result,
      error: response.data.error,
      ...response.data,
    };
  } catch (error) {
    console.error('[workflowApi] 获取工作流状态失败:', error);
    return {
      success: false,
      error: error.message || '获取状态失败',
    };
  }
}

/**
 * 列出所有工作流
 */
export async function listWorkflows(options = {}) {
  try {
    const response = await apiClient.get('/workflow/list', {
      params: options,
    });
    
    return {
      success: true,
      workflows: response.data.workflows || [],
      total: response.data.total || 0,
      ...response.data,
    };
  } catch (error) {
    console.error('[workflowApi] 列出工作流失败:', error);
    return {
      success: false,
      error: error.message || '列出工作流失败',
      workflows: [],
    };
  }
}

/**
 * 删除工作流
 */
export async function deleteWorkflow(workflowId) {
  try {
    const response = await apiClient.post('/workflow/delete', {
      workflow_id: workflowId,
    });
    
    return {
      success: true,
      message: response.data.message || '工作流删除成功',
      ...response.data,
    };
  } catch (error) {
    console.error('[workflowApi] 删除工作流失败:', error);
    return {
      success: false,
      error: error.message || '删除工作流失败',
    };
  }
}

/**
 * 重试工作流步骤
 */
export async function retryStep(workflowId, stepId, executionId = null) {
  try {
    const response = await apiClient.post('/workflow/retry', {
      workflow_id: workflowId,
      step_id: stepId,
      execution_id: executionId,
    });
    
    return {
      success: true,
      message: response.data.message || '步骤重试已开始',
      ...response.data,
    };
  } catch (error) {
    console.error('[workflowApi] 重试步骤失败:', error);
    return {
      success: false,
      error: error.message || '重试步骤失败',
    };
  }
}

/**
 * 暂停工作流
 */
export async function pauseWorkflow(workflowId, executionId = null) {
  try {
    const response = await apiClient.post('/workflow/pause', {
      workflow_id: workflowId,
      execution_id: executionId,
    });
    
    return {
      success: true,
      message: response.data.message || '工作流已暂停',
      ...response.data,
    };
  } catch (error) {
    console.error('[workflowApi] 暂停工作流失败:', error);
    return {
      success: false,
      error: error.message || '暂停工作流失败',
    };
  }
}

/**
 * 恢复工作流
 */
export async function resumeWorkflow(workflowId, executionId = null, fromStep = null) {
  try {
    const response = await apiClient.post('/workflow/resume', {
      workflow_id: workflowId,
      execution_id: executionId,
      from_step: fromStep,
    });
    
    return {
      success: true,
      message: response.data.message || '工作流已恢复',
      ...response.data,
    };
  } catch (error) {
    console.error('[workflowApi] 恢复工作流失败:', error);
    return {
      success: false,
      error: error.message || '恢复工作流失败',
    };
  }
}

/**
 * 创建示例工作流（用于演示）
 */
export async function createSampleWorkflow() {
  const sampleWorkflow = {
    name: '示例工作流',
    description: '这是一个演示用的工作流',
    steps: [
      {
        id: 'step1',
        name: '分析任务',
        type: 'analysis',
        config: {
          prompt: '分析用户需求',
        },
      },
      {
        id: 'step2',
        name: '生成代码',
        type: 'generation',
        depends_on: ['step1'],
        config: {
          template: 'code_template',
        },
      },
      {
        id: 'step3',
        name: '测试验证',
        type: 'validation',
        depends_on: ['step2'],
        config: {
          test_type: 'unit',
        },
      },
    ],
  };

  return sampleWorkflow;
}

// 默认导出（为了兼容性）
const workflowApi = {
  createWorkflow,
  executeWorkflow,
  getWorkflowStatus,
  listWorkflows,
  deleteWorkflow,
  retryStep,
  pauseWorkflow,
  resumeWorkflow,
  createSampleWorkflow,
};

export default workflowApi;
