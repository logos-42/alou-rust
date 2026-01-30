/**
 * Workflow API - 工作流管理API客户端
 * 调用后端 /workflow 接口来管理工作流
 */

import apiClient from './api';
import type { AgentInfo } from '@shared/types/services';

interface WorkflowStep {
  id: string;
  name: string;
  tool: string;
  args?: Record<string, any>;
  depends_on?: string[];
}

interface WorkflowDefinition {
  name: string;
  description: string;
  steps: WorkflowStep[];
  metadata?: Record<string, any>;
}

interface WorkflowExecution {
  id: string;
  workflow_id: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';
  current_step?: string;
  results: Record<string, any>;
  created_at: string;
  updated_at: string;
  started_at?: string;
  completed_at?: string;
}

interface CreateWorkflowResult {
  success: boolean;
  workflowId?: string;
  message?: string;
  workflow?: any;
  error?: string;
}

interface ExecuteWorkflowResult {
  success: boolean;
  executionId?: string;
  results?: any;
  message?: string;
  execution?: WorkflowExecution;
  error?: string;
}

interface WorkflowListResult {
  success: boolean;
  workflows?: any[];
  total?: number;
  message?: string;
  error?: string;
}

interface WorkflowStatsResult {
  success: boolean;
  stats?: {
    total: number;
    pending: number;
    running: number;
    completed: number;
    failed: number;
    cancelled: number;
  };
  message?: string;
  error?: string;
}

/**
 * 创建新的工作流
 */
export async function createWorkflow(
  workflow: WorkflowDefinition, 
  agentInfo: Partial<AgentInfo> = {}
): Promise<CreateWorkflowResult> {
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
      error: (error as Error).message || '创建工作流失败',
    };
  }
}

/**
 * 执行工作流
 */
export async function executeWorkflow(
  workflowId: string, 
  context: Record<string, any> = {},
  options: {
    mode?: string;
    timeout?: number;
    retry_count?: number;
  } = {}
): Promise<ExecuteWorkflowResult> {
  try {
    const response = await apiClient.post('/workflow/execute', {
      workflow_id: workflowId,
      context,
      options,
    });
    
    return {
      success: true,
      executionId: response.data.execution_id,
      results: response.data.results,
      message: response.data.message || '工作流执行成功',
      execution: response.data.execution,
    };
  } catch (error) {
    console.error('[workflowApi] 执行工作流失败:', error);
    return {
      success: false,
      error: (error as Error).message || '执行工作流失败',
    };
  }
}

/**
 * 获取工作流列表
 */
export async function getWorkflows(
  options: {
    page?: number;
    limit?: number;
    status?: string;
    agent_id?: string;
  } = {}
): Promise<WorkflowListResult> {
  try {
    const params = new URLSearchParams();
    if (options.page) params.append('page', String(options.page));
    if (options.limit) params.append('limit', String(options.limit));
    if (options.status) params.append('status', options.status);
    if (options.agent_id) params.append('agent_id', options.agent_id);

    const response = await apiClient.get(`/workflow/list?${params.toString()}`);
    
    return {
      success: true,
      workflows: response.data.workflows,
      total: response.data.total,
      message: response.data.message || '获取工作流列表成功',
    };
  } catch (error) {
    console.error('[workflowApi] 获取工作流列表失败:', error);
    return {
      success: false,
      error: (error as Error).message || '获取工作流列表失败',
    };
  }
}

/**
 * 获取工作流详情
 */
export async function getWorkflow(workflowId: string): Promise<{
  success: boolean;
  workflow?: any;
  error?: string;
}> {
  try {
    const response = await apiClient.get(`/workflow/${workflowId}`);
    
    return {
      success: true,
      workflow: response.data,
    };
  } catch (error) {
    console.error('[workflowApi] 获取工作流详情失败:', error);
    return {
      success: false,
      error: (error as Error).message || '获取工作流详情失败',
    };
  }
}

/**
 * 更新工作流
 */
export async function updateWorkflow(
  workflowId: string, 
  updates: Partial<WorkflowDefinition>
): Promise<{
  success: boolean;
  workflow?: any;
  message?: string;
  error?: string;
}> {
  try {
    const response = await apiClient.put(`/workflow/${workflowId}`, updates);
    
    return {
      success: true,
      workflow: response.data,
      message: response.data.message || '工作流更新成功',
    };
  } catch (error) {
    console.error('[workflowApi] 更新工作流失败:', error);
    return {
      success: false,
      error: (error as Error).message || '更新工作流失败',
    };
  }
}

/**
 * 删除工作流
 */
export async function deleteWorkflow(workflowId: string): Promise<{
  success: boolean;
  message?: string;
  error?: string;
}> {
  try {
    const response = await apiClient.delete(`/workflow/${workflowId}`);
    
    return {
      success: true,
      message: response.data.message || '工作流删除成功',
    };
  } catch (error) {
    console.error('[workflowApi] 删除工作流失败:', error);
    return {
      success: false,
      error: (error as Error).message || '删除工作流失败',
    };
  }
}

/**
 * 获取工作流执行历史
 */
export async function getWorkflowExecutions(
  workflowId: string,
  options: {
    page?: number;
    limit?: number;
    status?: string;
  } = {}
): Promise<{
  success: boolean;
  executions?: WorkflowExecution[];
  total?: number;
  message?: string;
  error?: string;
}> {
  try {
    const params = new URLSearchParams();
    if (options.page) params.append('page', String(options.page));
    if (options.limit) params.append('limit', String(options.limit));
    if (options.status) params.append('status', options.status);

    const response = await apiClient.get(`/workflow/${workflowId}/executions?${params.toString()}`);
    
    return {
      success: true,
      executions: response.data.executions,
      total: response.data.total,
      message: response.data.message || '获取执行历史成功',
    };
  } catch (error) {
    console.error('[workflowApi] 获取工作流执行历史失败:', error);
    return {
      success: false,
      error: (error as Error).message || '获取工作流执行历史失败',
    };
  }
}

/**
 * 获取执行详情
 */
export async function getWorkflowExecution(
  executionId: string
): Promise<{
  success: boolean;
  execution?: WorkflowExecution;
  error?: string;
}> {
  try {
    const response = await apiClient.get(`/workflow/execution/${executionId}`);
    
    return {
      success: true,
      execution: response.data,
    };
  } catch (error) {
    console.error('[workflowApi] 获取执行详情失败:', error);
    return {
      success: false,
      error: (error as Error).message || '获取执行详情失败',
    };
  }
}

/**
 * 取消工作流执行
 */
export async function cancelWorkflowExecution(
  executionId: string
): Promise<{
  success: boolean;
  message?: string;
  error?: string;
}> {
  try {
    const response = await apiClient.post(`/workflow/execution/${executionId}/cancel`);
    
    return {
      success: true,
      message: response.data.message || '工作流执行已取消',
    };
  } catch (error) {
    console.error('[workflowApi] 取消工作流执行失败:', error);
    return {
      success: false,
      error: (error as Error).message || '取消工作流执行失败',
    };
  }
}

/**
 * 获取工作流统计信息
 */
export async function getWorkflowStats(
  agentId?: string
): Promise<WorkflowStatsResult> {
  try {
    const params = agentId ? `?agent_id=${agentId}` : '';
    const response = await apiClient.get(`/workflow/stats${params}`);
    
    return {
      success: true,
      stats: response.data,
      message: response.data.message || '获取统计信息成功',
    };
  } catch (error) {
    console.error('[workflowApi] 获取工作流统计失败:', error);
    return {
      success: false,
      error: (error as Error).message || '获取工作流统计失败',
    };
  }
}

/**
 * 验证工作流定义
 */
export async function validateWorkflow(
  workflow: WorkflowDefinition
): Promise<{
  success: boolean;
  isValid?: boolean;
  errors?: string[];
  warnings?: string[];
  message?: string;
  error?: string;
}> {
  try {
    const response = await apiClient.post('/workflow/validate', { workflow });
    
    return {
      success: true,
      isValid: response.data.is_valid,
      errors: response.data.errors,
      warnings: response.data.warnings,
      message: response.data.message || '工作流验证完成',
    };
  } catch (error) {
    console.error('[workflowApi] 验证工作流失败:', error);
    return {
      success: false,
      error: (error as Error).message || '验证工作流失败',
    };
  }
}

/**
 * 克隆工作流
 */
export async function cloneWorkflow(
  workflowId: string,
  newName?: string
): Promise<CreateWorkflowResult> {
  try {
    const response = await apiClient.post(`/workflow/${workflowId}/clone`, {
      new_name: newName,
    });
    
    return {
      success: true,
      workflowId: response.data.workflow_id,
      message: response.data.message || '工作流克隆成功',
      workflow: response.data.workflow,
    };
  } catch (error) {
    console.error('[workflowApi] 克隆工作流失败:', error);
    return {
      success: false,
      error: (error as Error).message || '克隆工作流失败',
    };
  }
}

/**
 * 导出工作流
 */
export async function exportWorkflow(workflowId: string): Promise<{
  success: boolean;
  data?: string;
  filename?: string;
  error?: string;
}> {
  try {
    const response = await apiClient.get(`/workflow/${workflowId}/export`);
    
    return {
      success: true,
      data: response.data.data,
      filename: response.data.filename,
    };
  } catch (error) {
    console.error('[workflowApi] 导出工作流失败:', error);
    return {
      success: false,
      error: (error as Error).message || '导出工作流失败',
    };
  }
}

/**
 * 导入工作流
 */
export async function importWorkflow(
  workflowData: any,
  options: {
    overwrite?: boolean;
    validate_before_import?: boolean;
  } = {}
): Promise<CreateWorkflowResult> {
  try {
    const response = await apiClient.post('/workflow/import', {
      workflow_data: workflowData,
      options,
    });
    
    return {
      success: true,
      workflowId: response.data.workflow_id,
      message: response.data.message || '工作流导入成功',
      workflow: response.data.workflow,
    };
  } catch (error) {
    console.error('[workflowApi] 导入工作流失败:', error);
    return {
      success: false,
      error: (error as Error).message || '导入工作流失败',
    };
  }
}

/**
 * 批量操作工作流
 */
export async function batchWorkflowOperation(
  workflowIds: string[],
  operation: 'delete' | 'execute' | 'cancel',
  context?: Record<string, any>
): Promise<{
  success: boolean;
  results?: any[];
  message?: string;
  error?: string;
}> {
  try {
    const response = await apiClient.post('/workflow/batch', {
      workflow_ids: workflowIds,
      operation,
      context,
    });
    
    return {
      success: true,
      results: response.data.results,
      message: response.data.message || '批量操作完成',
    };
  } catch (error) {
    console.error('[workflowApi] 批量操作工作流失败:', error);
    return {
      success: false,
      error: (error as Error).message || '批量操作工作流失败',
    };
  }
}

/**
 * 搜索工作流
 */
export async function searchWorkflows(
  query: string,
  options: {
    page?: number;
    limit?: number;
    filters?: {
      status?: string;
      agent_id?: string;
      created_after?: string;
      created_before?: string;
    };
  } = {}
): Promise<WorkflowListResult> {
  try {
    const response = await apiClient.post('/workflow/search', {
      query,
      options,
    });
    
    return {
      success: true,
      workflows: response.data.workflows,
      total: response.data.total,
      message: response.data.message || '搜索完成',
    };
  } catch (error) {
    console.error('[workflowApi] 搜索工作流失败:', error);
    return {
      success: false,
      error: (error as Error).message || '搜索工作流失败',
    };
  }
}

/**
 * 获取工作流模板
 */
export async function getWorkflowTemplates(): Promise<{
  success: boolean;
  templates?: any[];
  message?: string;
  error?: string;
}> {
  try {
    const response = await apiClient.get('/workflow/templates');
    
    return {
      success: true,
      templates: response.data.templates,
      message: response.data.message || '获取模板成功',
    };
  } catch (error) {
    console.error('[workflowApi] 获取工作流模板失败:', error);
    return {
      success: false,
      error: (error as Error).message || '获取工作流模板失败',
    };
  }
}

/**
 * 从模板创建工作流
 */
export async function createWorkflowFromTemplate(
  templateId: string,
  customizations: Partial<WorkflowDefinition> = {}
): Promise<CreateWorkflowResult> {
  try {
    const response = await apiClient.post('/workflow/create-from-template', {
      template_id: templateId,
      customizations,
    });
    
    return {
      success: true,
      workflowId: response.data.workflow_id,
      message: response.data.message || '从模板创建工作流成功',
      workflow: response.data.workflow,
    };
  } catch (error) {
    console.error('[workflowApi] 从模板创建工作流失败:', error);
    return {
      success: false,
      error: (error as Error).message || '从模板创建工作流失败',
    };
  }
}

export type {
  WorkflowStep,
  WorkflowDefinition,
  WorkflowExecution,
  CreateWorkflowResult,
  ExecuteWorkflowResult,
  WorkflowListResult,
  WorkflowStatsResult,
};
