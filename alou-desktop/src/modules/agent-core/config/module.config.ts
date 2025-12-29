/**
 * agent-core 模块配置文件
 * 管理模块级别的配置
 */

import { ModuleConfig, defaultConfig } from '../index';

// 模块配置
export const config: ModuleConfig = {
  ...defaultConfig,
  // 模块特定配置
  enableCaching: true,
  maxRetries: 3,
};

// 功能开关配置
export const featureFlags = {
  // 代理相关功能
  agent: {
    enableCreation: true,
    enableDeletion: true,
    enableImport: true,
    enableExport: true,
  },
  
  // 频道相关功能
  channel: {
    enableCreation: true,
    enableDeletion: true,
    enableSwitching: true,
    enableSearch: true,
  },
  
  // 消息相关功能
  message: {
    enableSending: true,
    enableDeletion: true,
    enableHistory: true,
    enableStreaming: true,
  },
  
  // 会话相关功能
  session: {
    enableCreation: true,
    enableClosing: true,
    enablePersistence: true,
  },
};

// 性能配置
export const performanceConfig = {
  // 缓存配置
  caching: {
    agentCacheTTL: 5 * 60 * 1000, // 5分钟
    channelCacheTTL: 2 * 60 * 1000, // 2分钟
    messageCacheTTL: 1 * 60 * 1000, // 1分钟
    maxCacheSize: 1000, // 最大缓存条目数
  },
  
  // 请求配置
  requests: {
    timeout: 30000, // 30秒
    retryDelay: 1000, // 1秒
    maxConcurrent: 5, // 最大并发请求数
  },
  
  // 消息配置
  messages: {
    batchSize: 50, // 消息批量大小
    debounceTime: 300, // 防抖时间(ms)
    throttleTime: 100, // 节流时间(ms)
  },
};

// 验证配置
export const validationConfig = {
  // 代理验证规则
  agent: {
    name: {
      minLength: 1,
      maxLength: 100,
      pattern: /^[a-zA-Z0-9_\-\s]+$/,
    },
    description: {
      maxLength: 500,
    },
  },
  
  // 频道验证规则
  channel: {
    name: {
      minLength: 1,
      maxLength: 50,
    },
  },
  
  // 消息验证规则
  message: {
    content: {
      minLength: 1,
      maxLength: 5000,
    },
  },
};

// 错误配置
export const errorConfig = {
  // 错误消息
  messages: {
    agentNotFound: '代理不存在',
    channelNotFound: '频道不存在',
    messageSendFailed: '消息发送失败',
    sessionExpired: '会话已过期',
    networkError: '网络连接错误',
    validationError: '数据验证失败',
  },
  
  // 错误代码
  codes: {
    AGENT_NOT_FOUND: 'AGENT_001',
    CHANNEL_NOT_FOUND: 'CHANNEL_001',
    MESSAGE_SEND_FAILED: 'MESSAGE_001',
    SESSION_EXPIRED: 'SESSION_001',
    NETWORK_ERROR: 'NETWORK_001',
    VALIDATION_ERROR: 'VALIDATION_001',
  },
};

// 导出所有配置
export default {
  config,
  featureFlags,
  performanceConfig,
  validationConfig,
  errorConfig,
};

// 配置工具函数
export function updateConfig(updates: Partial<ModuleConfig>): void {
  Object.assign(config, updates);
  console.log('[agent-core] 配置已更新', updates);
}

export function getFeatureFlag(module: keyof typeof featureFlags, flag: string): boolean {
  return featureFlags[module]?.[flag as keyof typeof featureFlags[typeof module]] ?? false;
}

export function validateAgentName(name: string): { valid: boolean; error?: string } {
  const rules = validationConfig.agent.name;
  
  if (name.length < rules.minLength) {
    return { valid: false, error: `名称至少需要${rules.minLength}个字符` };
  }
  
  if (name.length > rules.maxLength) {
    return { valid: false, error: `名称不能超过${rules.maxLength}个字符` };
  }
  
  if (!rules.pattern.test(name)) {
    return { valid: false, error: '名称只能包含字母、数字、下划线、连字符和空格' };
  }
  
  return { valid: true };
}

export function validateMessageContent(content: string): { valid: boolean; error?: string } {
  const rules = validationConfig.message.content;
  
  if (content.length < rules.minLength) {
    return { valid: false, error: `消息内容不能为空` };
  }
  
  if (content.length > rules.maxLength) {
    return { valid: false, error: `消息内容不能超过${rules.maxLength}个字符` };
  }
  
  return { valid: true };
}
