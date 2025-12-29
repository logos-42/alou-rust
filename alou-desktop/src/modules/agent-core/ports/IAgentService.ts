/**
 * IAgentService 接口
 * 代理服务接口定义
 */

import { 
  Agent, 
  AgentConfig, 
  AgentFilter, 
  AgentSortOption, 
  AgentPaginationResult 
} from '../entities/Agent';

/**
 * 代理服务接口
 * 提供代理的创建、读取、更新、删除等操作
 */
export interface IAgentService {
  /**
   * 创建代理
   * @param config 代理配置
   * @returns 创建的代理
   */
  createAgent(config: AgentConfig): Promise<Agent>;
  
  /**
   * 获取代理
   * @param id 代理ID
   * @returns 代理或null（如果不存在）
   */
  getAgent(id: string): Promise<Agent | null>;
  
  /**
   * 更新代理
   * @param id 代理ID
   * @param updates 更新内容
   * @returns 更新后的代理
   */
  updateAgent(id: string, updates: Partial<Omit<Agent, 'id' | 'createdAt'>>): Promise<Agent>;
  
  /**
   * 删除代理
   * @param id 代理ID
   */
  deleteAgent(id: string): Promise<void>;
  
  /**
   * 获取代理列表
   * @param filter 过滤器
   * @param sort 排序方式
   * @param page 页码
   * @param pageSize 每页大小
   * @returns 分页结果
   */
  listAgents(
    filter?: AgentFilter,
    sort?: AgentSortOption,
    page?: number,
    pageSize?: number
  ): Promise<AgentPaginationResult>;
  
  /**
   * 搜索代理
   * @param query 搜索关键词
   * @param limit 返回数量限制
   * @returns 代理列表
   */
  searchAgents(query: string, limit?: number): Promise<Agent[]>;
  
  /**
   * 获取代理统计信息
   * @returns 统计信息
   */
  getAgentStats(): Promise<{
    total: number;
    active: number;
    byStatus: Record<string, number>;
    byCategory: Record<string, number>;
  }>;
  
  /**
   * 验证代理配置
   * @param config 代理配置
   * @returns 验证结果
   */
  validateAgentConfig(config: AgentConfig): Promise<{
    valid: boolean;
    errors: string[];
    warnings: string[];
  }>;
  
  /**
   * 导入代理
   * @param data 代理数据
   * @returns 导入的代理
   */
  importAgent(data: Record<string, any>): Promise<Agent>;
  
  /**
   * 导出代理
   * @param id 代理ID
   * @returns 代理数据
   */
  exportAgent(id: string): Promise<Record<string, any>>;
  
  /**
   * 复制代理
   * @param id 代理ID
   * @param newName 新代理名称
   * @returns 复制的代理
   */
  duplicateAgent(id: string, newName?: string): Promise<Agent>;
  
  /**
   * 批量操作
   * @param agentIds 代理ID列表
   * @param operation 操作类型
   * @param data 操作数据
   * @returns 操作结果
   */
  batchOperation(
    agentIds: string[],
    operation: 'archive' | 'unarchive' | 'delete' | 'update',
    data?: Record<string, any>
  ): Promise<{
    success: number;
    failed: number;
    errors: Array<{ agentId: string; error: string }>;
  }>;
  
  /**
   * 订阅代理事件
   * @param event 事件类型
   * @param callback 回调函数
   * @returns 取消订阅函数
   */
  subscribe(
    event: 'agentCreated' | 'agentUpdated' | 'agentDeleted' | 'agentStatusChanged',
    callback: (agent: Agent) => void
  ): () => void;
  
  /**
   * 检查代理名称是否可用
   * @param name 代理名称
   * @param excludeId 排除的代理ID
   * @returns 是否可用
   */
  isAgentNameAvailable(name: string, excludeId?: string): Promise<boolean>;
  
  /**
   * 获取推荐代理
   * @param userId 用户ID
   * @param limit 返回数量限制
   * @returns 推荐代理列表
   */
  getRecommendedAgents(userId: string, limit?: number): Promise<Agent[]>;
  
  /**
   * 更新代理状态
   * @param id 代理ID
   * @param status 新状态
   * @param reason 状态变更原因
   * @returns 更新后的代理
   */
  updateAgentStatus(
    id: string, 
    status: Agent['status'], 
    reason?: string
  ): Promise<Agent>;
  
  /**
   * 获取代理使用统计
   * @param id 代理ID
   * @param period 统计周期（天）
   * @returns 使用统计
   */
  getAgentUsageStats(
    id: string, 
    period?: number
  ): Promise<{
    totalMessages: number;
    totalTokens: number;
    averageResponseTime: number;
    successRate: number;
    usageByDay: Array<{ date: string; count: number }>;
  }>;
}

/**
 * 代理服务配置
 */
export interface AgentServiceConfig {
  apiBaseUrl: string;
  enableCaching: boolean;
  cacheTTL: number;
  maxRetries: number;
  requestTimeout: number;
  validationStrict: boolean;
}

/**
 * 代理服务事件
 */
export interface AgentServiceEvents {
  onAgentCreated?: (agent: Agent) => void;
  onAgentUpdated?: (agent: Agent, oldAgent: Agent) => void;
  onAgentDeleted?: (agentId: string) => void;
  onAgentStatusChanged?: (agent: Agent, oldStatus: Agent['status']) => void;
  onError?: (error: Error, context: string) => void;
}

/**
 * 代理服务选项
 */
export interface AgentServiceOptions {
  config?: Partial<AgentServiceConfig>;
  events?: AgentServiceEvents;
}

/**
 * 代理服务工厂函数
 */
export type AgentServiceFactory = (options?: AgentServiceOptions) => IAgentService;
