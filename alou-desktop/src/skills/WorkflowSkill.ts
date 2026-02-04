/**
 * Workflow Skill - 工作流管理技能
 * 将工作流管理功能封装为Claude可调用的技能
 */
import workflowService from '../services/workflowService'

// 工作流步骤类型
interface WorkflowStep {
  id: string;
  name: string;
  tool: string;
  args?: Record<string, unknown>;
  depends_on?: string[];
}

// 智能体信息类型
interface AgentInfo {
  id?: string;
  name?: string;
  [key: string]: unknown;
}

// 技能操作参数类型
interface CreateWorkflowParams {
  name: string;
  description: string;
  steps: WorkflowStep[];
  agent_info?: AgentInfo;
}

interface ExecuteWorkflowParams {
  workflow_id: string;
  api_key?: string;
  agent_info?: AgentInfo;
}

interface GetWorkflowStatusParams {
  workflow_id: string;
}

interface DeleteWorkflowParams {
  workflow_id: string;
}

interface RetryStepParams {
  workflow_id: string;
  step_id: string;
  api_key?: string;
  agent_info?: AgentInfo;
}

interface PauseWorkflowParams {
  workflow_id: string;
}

interface ResumeWorkflowParams {
  workflow_id: string;
  api_key?: string;
  agent_info?: AgentInfo;
}

// 操作结果类型
interface OperationResult {
  success: boolean;
  [key: string]: unknown;
}

export class WorkflowSkill {
  name: string;
  description: string;
  version: string;
  category: string;

  constructor() {
    this.name = 'workflow_manager';
    this.description = '管理和执行多步骤工作流程，支持步骤依赖、状态跟踪和错误恢复';
    this.version = '1.0.0';
    this.category = 'automation';
  }

  /**
   * 获取技能定义
   */
  getDefinition() {
    return {
      name: this.name,
      description: this.description,
      version: this.version,
      category: this.category,
      actions: {
        create_workflow: {
          description: '创建新的工作流',
          parameters: {
            type: 'object',
            properties: {
              name: {
                type: 'string',
                description: '工作流名称'
              },
              description: {
                type: 'string',
                description: '工作流描述'
              },
              steps: {
                type: 'array',
                description: '工作流步骤列表',
                items: {
                  type: 'object',
                  properties: {
                    id: {
                      type: 'string',
                      description: '步骤ID'
                    },
                    name: {
                      type: 'string',
                      description: '步骤名称'
                    },
                    tool: {
                      type: 'string',
                      description: '要调用的工具名称'
                    },
                    args: {
                      type: 'object',
                      description: '工具参数'
                    },
                    depends_on: {
                      type: 'array',
                      description: '依赖的步骤ID列表',
                      items: {
                        type: 'string'
                      }
                    }
                  },
                  required: ['id', 'name', 'tool']
                }
              },
              agent_info: {
                type: 'object',
                description: '智能体信息（可选）'
              }
            },
            required: ['name', 'description', 'steps']
          }
        },
        execute_workflow: {
          description: '执行工作流',
          parameters: {
            type: 'object',
            properties: {
              workflow_id: {
                type: 'string',
                description: '工作流ID'
              },
              api_key: {
                type: 'string',
                description: 'Claude API密钥（可选，后端会自动处理）'
              },
              agent_info: {
                type: 'object',
                description: '智能体信息（可选）'
              }
            },
            required: ['workflow_id']
          }
        },
        get_workflow_status: {
          description: '获取工作流状态',
          parameters: {
            type: 'object',
            properties: {
              workflow_id: {
                type: 'string',
                description: '工作流ID'
              }
            },
            required: ['workflow_id']
          }
        },
        list_workflows: {
          description: '列出所有工作流',
          parameters: {
            type: 'object',
            properties: {}
          }
        },
        delete_workflow: {
          description: '删除工作流',
          parameters: {
            type: 'object',
            properties: {
              workflow_id: {
                type: 'string',
                description: '工作流ID'
              }
            },
            required: ['workflow_id']
          }
        },
        retry_step: {
          description: '重试失败的步骤',
          parameters: {
            type: 'object',
            properties: {
              workflow_id: {
                type: 'string',
                description: '工作流ID'
              },
              step_id: {
                type: 'string',
                description: '步骤ID'
              },
              api_key: {
                type: 'string',
                description: 'Claude API密钥（可选，后端会自动处理）'
              },
              agent_info: {
                type: 'object',
                description: '智能体信息（可选）'
              }
            },
            required: ['workflow_id', 'step_id']
          }
        },
        pause_workflow: {
          description: '暂停工作流',
          parameters: {
            type: 'object',
            properties: {
              workflow_id: {
                type: 'string',
                description: '工作流ID'
              }
            },
            required: ['workflow_id']
          }
        },
        resume_workflow: {
          description: '恢复工作流',
          parameters: {
            type: 'object',
            properties: {
              workflow_id: {
                type: 'string',
                description: '工作流ID'
              },
              api_key: {
                type: 'string',
                description: 'Claude API密钥（可选，后端会自动处理）'
              },
              agent_info: {
                type: 'object',
                description: '智能体信息（可选）'
              }
            },
            required: ['workflow_id']
          }
        }
      }
    };
  }

  /**
   * 执行技能操作
   */
  async execute(action: string, parameters: Record<string, unknown>): Promise<OperationResult> {
    try {
      switch (action) {
        case 'create_workflow':
          return await this.createWorkflow(parameters as CreateWorkflowParams);
        case 'execute_workflow':
          return await this.executeWorkflow(parameters as ExecuteWorkflowParams);
        case 'get_workflow_status':
          return await this.getWorkflowStatus(parameters as GetWorkflowStatusParams);
        case 'list_workflows':
          return await this.listWorkflows(parameters);
        case 'delete_workflow':
          return await this.deleteWorkflow(parameters as DeleteWorkflowParams);
        case 'retry_step':
          return await this.retryStep(parameters as RetryStepParams);
        case 'pause_workflow':
          return await this.pauseWorkflow(parameters as PauseWorkflowParams);
        case 'resume_workflow':
          return await this.resumeWorkflow(parameters as ResumeWorkflowParams);
        default:
          throw new Error(`未知的操作: ${action}`);
      }
    } catch (error) {
      console.error(`[WorkflowSkill] 执行操作 ${action} 失败:`, error);
      throw error;
    }
  }

  /**
   * 创建工作流
   */
  async createWorkflow(params: CreateWorkflowParams): Promise<OperationResult> {
    const { name, description, steps, agent_info = {} } = params;
    
    const workflow = {
      name,
      description,
      steps: steps.map(step => ({
        id: step.id,
        name: step.name,
        tool: step.tool,
        args: step.args || {},
        depends_on: step.depends_on || []
      }))
    };

    const result = await workflowService.createWorkflow(workflow, agent_info);
    
    if (!result.success) {
      throw new Error(result.error || '创建工作流失败');
    }

    return {
      success: true,
      workflow_id: result.workflowId,
      message: result.message,
      details: {
        name,
        description,
        step_count: steps.length,
        created_at: new Date().toISOString()
      }
    };
  }

  /**
   * 执行工作流
   */
  async executeWorkflow(params: ExecuteWorkflowParams): Promise<OperationResult> {
    const { workflow_id, api_key = '', agent_info = {} } = params;
    
    const result = await workflowService.executeWorkflow(
      workflow_id,
      api_key,
      agent_info
    );

    if (!result.success) {
      throw new Error(result.error || '执行工作流失败');
    }

    return {
      success: true,
      execution_result: (result as any).result || undefined,
      steps: (result as any).steps || undefined,
      message: result.message,
      details: {
        workflow_id,
        status: (result as Record<string, unknown>)?.result?.status || 'unknown',
        step_count: (result as any).steps?.length || 0
      }
    };
  }

  /**
   * 获取工作流状态
   */
  async getWorkflowStatus(params: GetWorkflowStatusParams): Promise<OperationResult> {
    const { workflow_id } = params;
    
    const result = await workflowService.getWorkflowStatus(workflow_id);

    if (!result.success) {
      throw new Error(result.error || '获取工作流状态失败');
    }

    return {
      success: true,
      workflow: result.workflow,
      status: result.status,
      steps: result.steps || [],
      details: {
        workflow_id,
        name: (result.workflow as Record<string, unknown>)?.name,
        description: (result.workflow as Record<string, unknown>)?.description,
        step_count: ((result.workflow as Record<string, unknown>)?.steps as unknown[])?.length || 0
      }
    };
  }

  /**
   * 列出所有工作流
   */
  async listWorkflows(_params: Record<string, unknown>): Promise<OperationResult> {
    const result = await workflowService.listWorkflows();

    if (!result.success) {
      throw new Error(result.error || '列出工作流失败');
    }

    return {
      success: true,
      workflows: result.workflows || [],
      count: result.workflows?.length || 0,
      details: {
        total_count: result.workflows?.length || 0,
        timestamp: new Date().toISOString()
      }
    };
  }

  /**
   * 删除工作流
   */
  async deleteWorkflow(params: Record<string, unknown>): Promise<OperationResult> {
    const workflow_id = (params as any).workflow_id || '';

    const result = await workflowService.deleteWorkflow(workflow_id);

    if (!result.success) {
      throw new Error(result.error || '删除工作流失败');
    }

    return {
      success: true,
      message: result.message,
      details: {
        workflow_id,
        deleted_at: new Date().toISOString()
      }
    };
  }

  /**
   * 重试失败的步骤
   */
  async retryStep(params: RetryStepParams): Promise<OperationResult> {
    const { workflow_id, step_id, api_key = '', agent_info = {} } = params;
    
    const result = await workflowService.retryStep(
      workflow_id,
      step_id,
      api_key,
      agent_info
    );

    if (!result.success) {
      throw new Error(result.error || '重试步骤失败');
    }

    return {
      success: true,
      result: result.result,
      message: result.message,
      details: {
        workflow_id,
        step_id,
        retried_at: new Date().toISOString()
      }
    };
  }

  /**
   * 暂停工作流
   */
  async pauseWorkflow(params: Record<string, unknown>): Promise<OperationResult> {
    const workflow_id = (params as any).workflow_id || '';

    const result = await workflowService.pauseWorkflow(workflow_id);

    if (!result.success) {
      throw new Error(result.error || '暂停工作流失败');
    }

    return {
      success: true,
      message: result.message,
      details: {
        workflow_id,
        paused_at: new Date().toISOString()
      }
    };
  }

  /**
   * 恢复工作流
   */
  async resumeWorkflow(params: ResumeWorkflowParams): Promise<OperationResult> {
    const { workflow_id, api_key = '', agent_info = {} } = params;

    const result = await workflowService.resumeWorkflow(
      workflow_id,
      api_key,
      agent_info
    );

    if (!result.success) {
      throw new Error(result.error || '恢复工作流失败');
    }

    return {
      success: true,
      result: result.result,
      message: result.message,
      details: {
        workflow_id,
        resumed_at: new Date().toISOString(),
        status: (result.result as Record<string, unknown>)?.status || 'unknown'
      }
    };
  }

  /**
   * 创建示例工作流
   */
  createSampleWorkflow(): unknown {
    return workflowService.createSampleWorkflow();
  }

  /**
   * 验证参数
   */
  validateParameters(action: string, parameters: Record<string, unknown>): { valid: boolean; error?: string } {
    const definition = this.getDefinition();
    const actionDef = definition.actions[action as keyof typeof definition.actions] as { parameters?: { required?: string[] } } | undefined;
    
    if (!actionDef) {
      return { valid: false, error: `未知的操作: ${action}` };
    }

    // 检查必需参数
    const requiredParams = actionDef.parameters?.required || [];
    for (const param of requiredParams) {
      if (parameters[param] === undefined || parameters[param] === null) {
        return { valid: false, error: `缺少必需参数: ${param}` };
      }
    }

    return { valid: true };
  }
}

export default new WorkflowSkill();
