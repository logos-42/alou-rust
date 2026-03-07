/**
 * 类型定义索引文件
 * 统一导出所有类型定义
 */

// 从全局类型定义导入
import type {
  Nullable,
  Optional,
  Maybe,
  ApiResponse,
  PaginationParams,
  PaginatedResult,
  PerformanceMetric,
  PerformanceReport,
  MigrationStatus,
  ModuleMigrationInfo,
} from './global';

// 从各个模块导入
export * from './agent';
export * from './auth';
export * from './heartbeat';
export type {
  AgentSession,
  Channel,
  Message,
  ServiceResponse,
  WalletInfo,
  NetworkInfo,
  TransactionInfo,
  BlockchainWalletInfo,
  BlockchainNetworkInfo,
  BlockchainTransactionInfo
} from './services';

// 重新导出全局类型
export type {
  Nullable,
  Optional,
  Maybe,
  ApiResponse,
  PaginationParams,
  PaginatedResult,
  PerformanceMetric,
  PerformanceReport,
  MigrationStatus,
  ModuleMigrationInfo,
};

// 通用工具类型
export type DeepPartial<T> = T extends object ? {
  [P in keyof T]?: DeepPartial<T[P]>;
} : T;

export type RequireAtLeastOne<T, Keys extends keyof T = keyof T> = 
  Pick<T, Exclude<keyof T, Keys>> & {
    [K in Keys]-?: Required<Pick<T, K>> & Partial<Pick<T, Exclude<Keys, K>>>
  }[Keys];

export type RequireExactlyOne<T, Keys extends keyof T = keyof T> = {
  [K in Keys]: Required<Pick<T, K>> & Partial<Record<Exclude<Keys, K>, undefined>>
}[Keys] & Pick<T, Exclude<keyof T, Keys>>;

// 模块相关类型
export interface ModuleConfig {
  name: string;
  version: string;
  dependencies: string[];
  migrationStatus: MigrationStatus;
}

export interface ModuleRegistry {
  [moduleName: string]: ModuleConfig;
}

// 性能监控配置
export interface PerformanceConfig {
  enabled: boolean;
  samplingRate: number; // 0-1
  reportInterval: number; // milliseconds
  thresholds: {
    warning: number; // milliseconds
    critical: number; // milliseconds
  };
}