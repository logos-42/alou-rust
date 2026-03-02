/**
 * Alou Skills SDK - 类型定义和接口
 * 
 * 提供技能开发所需的核心类型和接口
 * 
 * @packageDocumentation
 */

/**
 * 技能执行上下文
 * 
 * 提供给技能执行时的环境和工具
 */
export interface SkillContext {
  /** 当前用户 ID */
  userId: string;

  /** 当前对话 ID */
  conversationId: string;

  /** 当前群聊 ID（如果在群聊中） */
  groupId?: string;

  /** 访问配置 */
  config: SkillConfig;

  /** 日志记录器 */
  logger: Logger;

  /** 工具注册表 */
  tools: ToolRegistry;

  /** 存储服务 */
  storage: StorageService;

  /** HTTP 客户端 */
  http: HttpClient;

  /** 当前智能体信息 */
  agent?: AgentInfo;
}

/**
 * 技能配置
 */
export interface SkillConfig {
  /** API 基础 URL */
  baseUrl: string;

  /** API 密钥 */
  apiKey?: string;

  /** 技能特定配置 */
  skillSettings?: Record<string, any>;

  /** 其他配置 */
  [key: string]: any;
}

/**
 * 日志记录器接口
 */
export interface Logger {
  /** 信息日志 */
  info(message: string, data?: any): void;
  
  /** 警告日志 */
  warn(message: string, data?: any): void;
  
  /** 错误日志 */
  error(message: string, data?: any): void;
  
  /** 调试日志 */
  debug(message: string, data?: any): void;
  
  /** 追踪日志 */
  trace(message: string, data?: any): void;
}

/**
 * 工具注册表接口
 */
export interface ToolRegistry {
  /**
   * 调用工具
   * @param toolName 工具名称
   * @param args 工具参数
   * @returns 工具执行结果
   */
  call(toolName: string, args: any): Promise<any>;

  /**
   * 获取可用工具列表
   * @returns 工具名称数组
   */
  list(): string[];

  /**
   * 检查工具是否可用
   * @param toolName 工具名称
   * @returns 是否可用
   */
  has(toolName: string): boolean;

  /**
   * 获取工具描述
   * @param toolName 工具名称
   * @returns 工具描述
   */
  describe(toolName: string): ToolDescription | undefined;
}

/**
 * 工具描述
 */
export interface ToolDescription {
  name: string;
  description: string;
  parameters?: ParameterSchema;
  returns?: string;
  examples?: string[];
}

/**
 * 参数 Schema
 */
export interface ParameterSchema {
  type: 'object';
  properties: Record<string, ParameterProperty>;
  required?: string[];
}

/**
 * 参数属性
 */
export interface ParameterProperty {
  type: string;
  description: string;
  default?: any;
  enum?: any[];
  items?: ParameterProperty;
  properties?: Record<string, ParameterProperty>;
}

/**
 * 存储服务接口
 */
export interface StorageService {
  /**
   * 获取值
   * @param key 键
   * @returns 值
   */
  get<T = any>(key: string): Promise<T | undefined>;
  
  /**
   * 设置值
   * @param key 键
   * @param value 值
   */
  set<T = any>(key: string, value: T): Promise<void>;
  
  /**
   * 删除值
   * @param key 键
   */
  delete(key: string): Promise<void>;
  
  /**
   * 列出键
   * @param prefix 前缀（可选）
   * @returns 键数组
   */
  list(prefix?: string): Promise<string[]>;
  
  /**
   * 检查键是否存在
   * @param key 键
   * @returns 是否存在
   */
  has(key: string): Promise<boolean>;
}

/**
 * HTTP 客户端接口
 */
export interface HttpClient {
  /**
   * GET 请求
   */
  get<T = any>(url: string, options?: RequestOptions): Promise<HttpResponse<T>>;
  
  /**
   * POST 请求
   */
  post<T = any>(url: string, data?: any, options?: RequestOptions): Promise<HttpResponse<T>>;
  
  /**
   * PUT 请求
   */
  put<T = any>(url: string, data?: any, options?: RequestOptions): Promise<HttpResponse<T>>;
  
  /**
   * PATCH 请求
   */
  patch<T = any>(url: string, data?: any, options?: RequestOptions): Promise<HttpResponse<T>>;
  
  /**
   * DELETE 请求
   */
  delete<T = any>(url: string, options?: RequestOptions): Promise<HttpResponse<T>>;
}

/**
 * HTTP 请求选项
 */
export interface RequestOptions {
  headers?: Record<string, string>;
  params?: Record<string, string>;
  timeout?: number;
  credentials?: 'include' | 'omit' | 'same-origin';
}

/**
 * HTTP 响应
 */
export interface HttpResponse<T> {
  status: number;
  statusText: string;
  headers: Record<string, string>;
  data: T;
  ok: boolean;
}

/**
 * 智能体信息
 */
export interface AgentInfo {
  id: string;
  name: string;
  displayName?: string;
  description?: string;
  capabilities?: string[];
  status: 'online' | 'offline' | 'busy' | 'idle';
  avatar?: string;
}

/**
 * 技能执行结果
 */
export interface SkillResult {
  /** 是否成功 */
  success: boolean;

  /** 输出内容 */
  output: string | any;

  /** 错误信息（如果失败） */
  error?: string;

  /** 错误详情 */
  errorDetails?: {
    code: string;
    message: string;
    stack?: string;
  };

  /** 元数据 */
  metadata?: {
    /** 执行时长（毫秒） */
    duration?: number;
    /** 技能版本 */
    version?: string;
    /** 其他元数据 */
    [key: string]: any;
  };

  /** 中间结果（用于流式执行） */
  intermediateResults?: any[];

  /** 后续动作建议 */
  nextActions?: string[];
}

/**
 * 技能参数定义
 */
export interface SkillParameters {
  /** 参数类型 */
  type: 'object';
  
  /** 参数属性 */
  properties: Record<string, SkillParameter>;
  
  /** 必需参数 */
  required?: string[];
  
  /** 附加属性 */
  additionalProperties?: boolean;
}

/**
 * 单个参数定义
 */
export interface SkillParameter {
  /** 参数类型 */
  type: string;
  
  /** 参数描述 */
  description: string;
  
  /** 默认值 */
  default?: any;
  
  /** 枚举值 */
  enum?: any[];
  
  /** 最小值（数字类型） */
  minimum?: number;
  
  /** 最大值（数字类型） */
  maximum?: number;
  
  /** 最小长度（字符串类型） */
  minLength?: number;
  
  /** 最大长度（字符串类型） */
  maxLength?: number;
  
  /** 数组项定义 */
  items?: SkillParameter;
  
  /** 对象属性 */
  properties?: Record<string, SkillParameter>;
}

/**
 * 技能定义（用于注册和发现）
 */
export interface SkillDefinition {
  /** 技能名称 */
  name: string;
  
  /** 技能描述 */
  description: string;
  
  /** 版本号 */
  version: string;
  
  /** 分类 */
  category: string;
  
  /** 是否启用 */
  enabled: boolean;
  
  /** 参数定义 */
  parameters?: SkillParameters;
  
  /** 作者 */
  author?: string;
  
  /** 许可证 */
  license?: string;
  
  /** 所需工具 */
  requiredTools?: string[];
  
  /** 标签 */
  tags?: string[];
  
  /** 技能图标 */
  icon?: string;
  
  /** 文档 URL */
  documentationUrl?: string;
  
  /** 示例 */
  examples?: SkillExample[];
}

/**
 * 技能示例
 */
export interface SkillExample {
  name: string;
  description: string;
  input: Record<string, any>;
  output: string;
}

/**
 * 技能类别
 */
export type SkillCategory = 
  | 'utility'       // 通用工具
  | 'automation'    // 自动化
  | 'data'          // 数据处理
  | 'analysis'      // 分析
  | 'blockchain'    // 区块链
  | 'web'           // 网络
  | 'file'          // 文件
  | 'communication' // 通信
  | 'development'   // 开发
  | 'research'      // 研究
  | 'custom';       // 自定义

/**
 * 技能元数据（SKILL.md 解析结果）
 */
export interface SkillMetadata {
  name: string;
  description: string;
  version: string;
  license: string;
  author: string;
  category: SkillCategory;
  allowedTools: string[];
  parameters: SkillParameters;
  instructions: string;
  dependencies: string[];
  changelog: string;
  examples: SkillExample[];
}

/**
 * 技能信息（运行时）
 */
export interface SkillInfo {
  /** 技能 ID（唯一标识） */
  id: string;
  
  /** 技能定义 */
  definition: SkillDefinition;
  
  /** 技能实例 */
  instance?: Skill;
  
  /** 技能路径 */
  path: string;
  
  /** 加载时间 */
  loadedAt: number;
  
  /** 最后执行时间 */
  lastExecutedAt?: number;
  
  /** 执行次数 */
  executionCount: number;
  
  /** 是否已加载 */
  loaded: boolean;
  
  /** 加载错误 */
  loadError?: Error;
}

/**
 * 技能发现结果
 */
export interface SkillDiscoveryResult {
  /** 发现的技能列表 */
  skills: SkillDefinition[];
  
  /** 扫描路径 */
  scannedPaths: string[];
  
  /** 错误信息 */
  errors: Array<{
    path: string;
    error: string;
  }>;
  
  /** 扫描时间 */
  scannedAt: number;
}

/**
 * 技能执行选项
 */
export interface SkillExecutionOptions {
  /** 超时时间（毫秒） */
  timeout?: number;
  
  /** 重试次数 */
  retryCount?: number;
  
  /** 重试间隔（毫秒） */
  retryDelay?: number;
  
  /** 是否流式返回 */
  stream?: boolean;
  
  /** 回调函数 */
  onProgress?: (progress: SkillProgress) => void;
  
  /** 自定义上下文 */
  context?: Partial<SkillContext>;
}

/**
 * 技能执行进度
 */
export interface SkillProgress {
  /** 当前步骤 */
  currentStep: number;
  
  /** 总步骤数 */
  totalSteps: number;
  
  /** 进度百分比 */
  percentage: number;
  
  /** 当前状态描述 */
  status: string;
  
  /** 中间结果 */
  intermediateResult?: any;
  
  /** 预计剩余时间（毫秒） */
  estimatedRemaining?: number;
}

/**
 * 技能加载选项
 */
export interface SkillLoadOptions {
  /** 是否立即初始化 */
  autoInitialize?: boolean;
  
  /** 初始化上下文 */
  context?: SkillContext;
  
  /** 加载超时（毫秒） */
  timeout?: number;
}
