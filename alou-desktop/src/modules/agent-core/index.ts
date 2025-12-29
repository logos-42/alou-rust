/**
 * agent-core 模块入口文件
 * 统一导出模块的所有公共API
 */

// 导出实体类
export * from './entities/Agent';
export * from './entities/Channel';
export * from './entities/Message';
export * from './entities/Session';

// 导出端口（接口）
export * from './ports/IAgentService';
export * from './ports/IChannelService';
export * from './ports/IMessageService';
export * from './ports/ISessionService';

// 导出服务实现
export * from './services/AgentService';
export * from './services/ChannelService';
export * from './services/MessageService';
export * from './services/SessionService';

// 导出适配器
export * from './adapters/LegacyAdapter';

// 导出Hooks
export * from './hooks/useAgentManagement';
export * from './hooks/useChannelManagement';
export * from './hooks/useMessageManagement';
export * from './hooks/useSessionManagement';

// 导出工具函数
export * from './utils/agentUtils';
export * from './utils/channelUtils';

// 导出配置
export { config } from './config/module.config';

// 模块信息
export const MODULE_INFO = {
  name: 'agent-core',
  version: '1.0.0',
  description: '智能代理核心业务模块',
  dependencies: [],
  migrationStatus: 'pending' as const,
};

// 模块初始化函数
export function initializeModule(options?: ModuleInitOptions): Promise<void> {
  console.log(`[agent-core] 初始化模块 v${MODULE_INFO.version}`);
  
  // 这里可以添加模块初始化逻辑
  // 例如：注册服务、建立连接等
  
  return Promise.resolve();
}

// 模块清理函数
export function cleanupModule(): Promise<void> {
  console.log('[agent-core] 清理模块');
  
  // 这里可以添加模块清理逻辑
  // 例如：关闭连接、释放资源等
  
  return Promise.resolve();
}

// 类型定义
export interface ModuleInitOptions {
  debug?: boolean;
  configOverrides?: Partial<ModuleConfig>;
}

export interface ModuleConfig {
  apiBaseUrl: string;
  enableCaching: boolean;
  maxRetries: number;
}

// 默认配置
export const defaultConfig: ModuleConfig = {
  apiBaseUrl: import.meta.env.VITE_API_BASE_URL || 'http://127.0.0.1:8787',
  enableCaching: true,
  maxRetries: 3,
};
