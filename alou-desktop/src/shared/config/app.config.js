/**
 * 应用配置文件
 * 集中管理所有应用配置
 */

// 基础配置
const BASE_CONFIG = {
  appName: 'Alou Desktop',
  version: '0.1.7',
  environment: process.env.NODE_ENV || 'development',
  
  // 架构信息
  architecture: {
    version: '0.2.4',
    description: '模块化架构 (渐进式迁移中)',
    modules: {
      agent: 'pending',    // 待迁移
      wallet: 'pending',   // 待迁移
      ipfs: 'pending',     // 待迁移
      crypto: 'pending',   // 待迁移
    },
  },
};

// API配置
const API_CONFIG = {
  baseURL: import.meta.env.VITE_API_BASE_URL || 'http://127.0.0.1:8787',
  timeout: 30000,
  retryAttempts: 3,
  
  endpoints: {
    session: '/session',
    agentChat: '/agent/chat',
    agentCreate: '/agent/create',
    // 其他端点...
  },
};

// IPFS配置
const IPFS_CONFIG = {
  apiUrl: import.meta.env.VITE_IPFS_API_URL || 'http://127.0.0.1:5001',
  gatewayUrl: import.meta.env.VITE_IPFS_GATEWAY_URL || 'http://127.0.0.1:8080',
  autoStart: true,
  dataDir: 'ipfs-data',
};

// 钱包配置
const WALLET_CONFIG = {
  defaultChainId: parseInt(import.meta.env.VITE_DEFAULT_CHAIN_ID || '1'),
  networks: JSON.parse(import.meta.env.VITE_WALLET_NETWORKS || '[]'),
  autoConnect: true,
};

// 性能监控配置
const PERFORMANCE_CONFIG = {
  enabled: true,
  samplingRate: 0.1, // 10%的请求会被监控
  reportInterval: 60000, // 每分钟报告一次
  thresholds: {
    warning: 100, // 100ms警告
    critical: 500, // 500ms严重
  },
};

// 模块配置
const MODULE_CONFIG = {
  // 代理模块配置
  agent: {
    name: 'agent-core',
    version: '1.0.0',
    migrationStatus: 'pending',
    dependencies: [],
  },
  
  // 钱包模块配置
  wallet: {
    name: 'wallet-core',
    version: '1.0.0',
    migrationStatus: 'pending',
    dependencies: ['crypto'],
  },
  
  // IPFS模块配置
  ipfs: {
    name: 'ipfs-core',
    version: '1.0.0',
    migrationStatus: 'pending',
    dependencies: [],
  },
  
  // 加密模块配置
  crypto: {
    name: 'crypto-core',
    version: '1.0.0',
    migrationStatus: 'pending',
    dependencies: [],
  },
};

// 构建配置
const BUILD_CONFIG = {
  // 路径别名
  aliases: {
    '@': './src',
    '@modules': './src/modules',
    '@ui': './src/ui',
    '@shared': './src/shared',
    '@bridges': './src/bridges',
  },
  
  // 代码分割配置
  codeSplitting: {
    enabled: true,
    chunks: {
      vendor: ['react', 'react-dom', 'react-router-dom'],
      ui: ['@ui/*'],
      modules: ['@modules/*'],
    },
  },
  
  // 性能优化
  optimization: {
    minify: true,
    treeShaking: true,
    lazyLoading: true,
  },
};

// 导出配置
export const config = {
  ...BASE_CONFIG,
  api: API_CONFIG,
  ipfs: IPFS_CONFIG,
  wallet: WALLET_CONFIG,
  performance: PERFORMANCE_CONFIG,
  modules: MODULE_CONFIG,
  build: BUILD_CONFIG,
};

// 环境相关配置
export const isDevelopment = BASE_CONFIG.environment === 'development';
export const isProduction = BASE_CONFIG.environment === 'production';
export const isTest = BASE_CONFIG.environment === 'test';

// 配置验证
export function validateConfig() {
  const errors = [];
  
  if (!API_CONFIG.baseURL) {
    errors.push('API baseURL is required');
  }
  
  if (!IPFS_CONFIG.apiUrl) {
    errors.push('IPFS API URL is required');
  }
  
  return {
    valid: errors.length === 0,
    errors,
  };
}

// 配置工具函数
export function getModuleConfig(moduleName) {
  return MODULE_CONFIG[moduleName] || null;
}

export function updateModuleStatus(moduleName, status) {
  if (MODULE_CONFIG[moduleName]) {
    MODULE_CONFIG[moduleName].migrationStatus = status;
    return true;
  }
  return false;
}

export function getMigrationProgress() {
  const modules = Object.values(MODULE_CONFIG);
  const total = modules.length;
  const completed = modules.filter(m => m.migrationStatus === 'completed').length;
  const inProgress = modules.filter(m => m.migrationStatus === 'in_progress').length;
  
  return {
    total,
    completed,
    inProgress,
    pending: total - completed - inProgress,
    progress: total > 0 ? Math.round((completed / total) * 100) : 0,
  };
}

// 默认导出
export default config;
