/**
 * Alou 插件 SDK
 * 
 * 用户可以通过这个 SDK 创建自定义插件
 * 
 * 使用方法：
 * 1. 在 ~/.alou/plugins/ 目录下创建插件文件夹
 * 2. 创建 skill.ts 文件
 * 3. 继承 Skill 类并实现必要方法
 */

export interface SkillContext {
  /** 当前用户 ID */
  userId: string;
  
  /** 当前对话 ID */
  conversationId: string;
  
  /** 访问配置 */
  config: PluginConfig;
  
  /** 日志记录 */
  logger: Logger;
  
  /** 工具调用 */
  tools: ToolRegistry;
}

/** 插件配置 */
export interface PluginConfig {
  /** API 基础 URL */
  baseUrl: string;
  
  /** API 密钥 */
  apiKey: string;
  
  /** 其他配置 */
  [key: string]: any;
}

/** 日志记录器 */
export interface Logger {
  info(message: string, data?: any): void;
  warn(message: string, data?: any): void;
  error(message: string, data?: any): void;
  debug(message: string, data?: any): void;
}

/** 工具注册表 */
export interface ToolRegistry {
  /** 调用工具 */
  call(toolName: string, args: any): Promise<any>;
  
  /** 获取工具列表 */
  list(): string[];
}

/** 技能执行结果 */
export interface SkillResult {
  /** 是否成功 */
  success: boolean;
  
  /** 输出内容 */
  output: string;
  
  /** 错误信息（如果失败） */
  error?: string;
  
  /** 元数据 */
  metadata?: Record<string, any>;
}

/**
 * 技能基类
 * 
 * 用户自定义技能需要继承这个类
 */
export abstract class Skill {
  /** 技能名称 */
  abstract name: string;
  
  /** 技能描述 */
  abstract description: string;
  
  /** 技能版本 */
  version: string = '1.0.0';
  
  /** 技能分类 */
  category: string = 'utility';
  
  /** 是否启用 */
  enabled: boolean = true;
  
  /** 所需参数 */
  parameters?: SkillParameters;
  
  /** 执行技能 */
  abstract execute(params: Record<string, any>, context: PluginContext): Promise<SkillResult>;
  
  /** 初始化技能 */
  async initialize(context: PluginContext): Promise<void> {}
  
  /** 清理资源 */
  async dispose(): Promise<void> {}
  
  /** 获取技能定义 */
  getDefinition(): SkillDefinition {
    return {
      name: this.name,
      description: this.description,
      version: this.version,
      category: this.category,
      enabled: this.enabled,
      parameters: this.parameters,
    };
  }
}

/** 技能参数定义 */
export interface SkillParameters {
  type: 'object';
  properties: Record<string, SkillParameter>;
  required?: string[];
}

/** 单个参数定义 */
export interface SkillParameter {
  type: string;
  description: string;
  default?: any;
  enum?: any[];
}

/** 技能定义（用于注册） */
export interface SkillDefinition {
  name: string;
  description: string;
  version: string;
  category: string;
  enabled: boolean;
  parameters?: SkillParameters;
}

/** 插件信息 */
export interface PluginInfo {
  id: string;
  name: string;
  version: string;
  description: string;
  author?: string;
  skills: SkillDefinition[];
  dependencies?: Record<string, string>;
}

/** 插件上下文 */
export interface PluginContext {
  config: PluginConfig;
  logger: Logger;
  storage: StorageService;
  http: HttpClient;
}

/** 存储服务 */
export interface StorageService {
  get(key: string): Promise<any>;
  set(key: string, value: any): Promise<void>;
  delete(key: string): Promise<void>;
  list(prefix?: string): Promise<string[]>;
}

/** HTTP 客户端 */
export interface HttpClient {
  get(url: string, options?: any): Promise<any>;
  post(url: string, data: any, options?: any): Promise<any>;
  put(url: string, data: any, options?: any): Promise<any>;
  delete(url: string, options?: any): Promise<any>;
}
