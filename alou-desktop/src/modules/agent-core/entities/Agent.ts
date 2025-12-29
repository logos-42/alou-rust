/**
 * Agent 实体类
 * 表示智能代理的核心数据模型
 */

// 简单的ID生成函数
function generateId(): string {
  return 'agent_' + Date.now().toString(36) + Math.random().toString(36).substr(2, 9);
}

// 代理配置接口
export interface AgentConfig {
  model: string;
  temperature: number;
  maxTokens: number;
  systemPrompt?: string;
  topP?: number;
  frequencyPenalty?: number;
  presencePenalty?: number;
  stopSequences?: string[];
}

// 代理状态类型
export type AgentStatus = 'idle' | 'active' | 'error' | 'offline' | 'initializing' | 'busy';

// 代理能力类型
export interface AgentCapabilities {
  canChat: boolean;
  canGenerateImages?: boolean;
  canProcessFiles?: boolean;
  canUseTools?: boolean;
  canAccessWeb?: boolean;
  supportedFileTypes?: string[];
  maxFileSize?: number; // bytes
}

// 代理元数据
export interface AgentMetadata {
  createdBy: string;
  tags?: string[];
  category?: string;
  rating?: number;
  usageCount?: number;
  lastUsed?: Date;
  isPublic?: boolean;
  isFeatured?: boolean;
}

// 代理实体类
export class Agent {
  // 核心属性
  readonly id: string;
  name: string;
  description?: string;
  config: AgentConfig;
  status: AgentStatus;
  avatar?: string;
  
  // 扩展属性
  capabilities: AgentCapabilities;
  metadata: AgentMetadata;
  
  // 时间戳
  readonly createdAt: Date;
  updatedAt: Date;
  
  // 构造函数
  constructor(data: Partial<Agent> & { name: string; config: AgentConfig }) {
    this.id = data.id || generateId();
    this.name = data.name;
    this.description = data.description;
    this.config = data.config;
    this.status = data.status || 'idle';
    this.avatar = data.avatar;
    
    // 默认能力配置
    this.capabilities = data.capabilities || {
      canChat: true,
      canGenerateImages: false,
      canProcessFiles: false,
      canUseTools: false,
      canAccessWeb: false,
    };
    
    // 默认元数据
    this.metadata = data.metadata || {
      createdBy: 'system',
      tags: [],
      category: 'general',
      rating: 0,
      usageCount: 0,
      isPublic: false,
      isFeatured: false,
    };
    
    // 时间戳
    const now = new Date();
    this.createdAt = data.createdAt || now;
    this.updatedAt = data.updatedAt || now;
  }
  
  // 更新代理信息
  update(updates: Partial<Omit<Agent, 'id' | 'createdAt'>>): this {
    if (updates.name !== undefined) this.name = updates.name;
    if (updates.description !== undefined) this.description = updates.description;
    if (updates.config !== undefined) this.config = updates.config;
    if (updates.status !== undefined) this.status = updates.status;
    if (updates.avatar !== undefined) this.avatar = updates.avatar;
    if (updates.capabilities !== undefined) this.capabilities = updates.capabilities;
    if (updates.metadata !== undefined) this.metadata = updates.metadata;
    
    this.updatedAt = new Date();
    return this;
  }
  
  // 验证代理配置
  validateConfig(): { valid: boolean; errors: string[] } {
    const errors: string[] = [];
    
    // 验证名称
    if (!this.name || this.name.trim().length === 0) {
      errors.push('代理名称不能为空');
    }
    
    if (this.name.length > 100) {
      errors.push('代理名称不能超过100个字符');
    }
    
    // 验证模型
    if (!this.config.model || this.config.model.trim().length === 0) {
      errors.push('模型名称不能为空');
    }
    
    // 验证温度
    if (this.config.temperature < 0 || this.config.temperature > 2) {
      errors.push('温度必须在0到2之间');
    }
    
    // 验证最大令牌数
    if (this.config.maxTokens < 1 || this.config.maxTokens > 100000) {
      errors.push('最大令牌数必须在1到100000之间');
    }
    
    // 验证系统提示
    if (this.config.systemPrompt && this.config.systemPrompt.length > 10000) {
      errors.push('系统提示不能超过10000个字符');
    }
    
    return {
      valid: errors.length === 0,
      errors,
    };
  }
  
  // 检查代理是否可用
  isAvailable(): boolean {
    return this.status === 'idle' || this.status === 'active';
  }
  
  // 检查代理是否支持特定能力
  supports(capability: keyof AgentCapabilities): boolean {
    return this.capabilities[capability] === true;
  }
  
  // 增加使用计数
  incrementUsageCount(): void {
    this.metadata.usageCount = (this.metadata.usageCount || 0) + 1;
    this.metadata.lastUsed = new Date();
    this.updatedAt = new Date();
  }
  
  // 转换为纯对象（用于序列化）
  toJSON(): Record<string, any> {
    return {
      id: this.id,
      name: this.name,
      description: this.description,
      config: this.config,
      status: this.status,
      avatar: this.avatar,
      capabilities: this.capabilities,
      metadata: {
        ...this.metadata,
        lastUsed: this.metadata.lastUsed?.toISOString(),
      },
      createdAt: this.createdAt.toISOString(),
      updatedAt: this.updatedAt.toISOString(),
    };
  }
  
  // 从纯对象创建实例（用于反序列化）
  static fromJSON(data: Record<string, any>): Agent {
    return new Agent({
      id: data.id,
      name: data.name,
      description: data.description,
      config: data.config,
      status: data.status,
      avatar: data.avatar,
      capabilities: data.capabilities,
      metadata: {
        ...data.metadata,
        lastUsed: data.metadata?.lastUsed ? new Date(data.metadata.lastUsed) : undefined,
      },
      createdAt: data.createdAt ? new Date(data.createdAt) : new Date(),
      updatedAt: data.updatedAt ? new Date(data.updatedAt) : new Date(),
    });
  }
  
  // 创建默认代理配置
  static createDefaultConfig(model: string = 'gpt-4'): AgentConfig {
    return {
      model,
      temperature: 0.7,
      maxTokens: 2000,
      systemPrompt: '你是一个有帮助的AI助手。',
      topP: 1.0,
      frequencyPenalty: 0,
      presencePenalty: 0,
    };
  }
  
  // 创建示例代理
  static createExample(name: string = '示例代理'): Agent {
    return new Agent({
      name,
      config: Agent.createDefaultConfig(),
      description: '这是一个示例代理',
      avatar: 'https://example.com/avatar.png',
      metadata: {
        createdBy: 'system',
        tags: ['example', 'demo'],
        category: 'general',
        rating: 4.5,
        usageCount: 0,
        isPublic: true,
        isFeatured: false,
      },
    });
  }
}

// 代理过滤器类型
export interface AgentFilter {
  status?: AgentStatus | AgentStatus[];
  category?: string;
  tags?: string[];
  capabilities?: Partial<AgentCapabilities>;
  isPublic?: boolean;
  isFeatured?: boolean;
  search?: string;
  minRating?: number;
  maxRating?: number;
}

// 代理排序选项
export type AgentSortOption = 'name' | 'createdAt' | 'updatedAt' | 'rating' | 'usageCount';

// 代理分页结果
export interface AgentPaginationResult {
  agents: Agent[];
  total: number;
  page: number;
  pageSize: number;
  totalPages: number;
}

// 所有类型已经通过export语句导出
// 不需要额外的export type语句
