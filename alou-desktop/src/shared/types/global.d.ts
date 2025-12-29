/**
 * 全局类型定义文件
 * 为新架构提供类型支持
 */

// 模块路径别名声明
declare module '@modules/*';
declare module '@ui/*';
declare module '@shared/*';
declare module '@bridges/*';

// 环境变量类型定义
interface ImportMetaEnv {
  readonly VITE_API_BASE_URL: string;
  readonly VITE_IPFS_API_URL: string;
  readonly VITE_IPFS_GATEWAY_URL: string;
  // 添加其他环境变量...
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}

// 通用类型定义
type Nullable<T> = T | null;
type Optional<T> = T | undefined;
type Maybe<T> = T | null | undefined;

// 响应结果包装器
interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
}

// 分页参数
interface PaginationParams {
  page?: number;
  pageSize?: number;
  total?: number;
}

// 分页结果
interface PaginatedResult<T> {
  items: T[];
  pagination: PaginationParams;
}

// 性能监控相关类型
interface PerformanceMetric {
  name: string;
  duration: number;
  timestamp: number;
  metadata?: Record<string, any>;
}

interface PerformanceReport {
  module: string;
  operation: string;
  avgDuration: number;
  callCount: number;
  recommendations: string[];
}

// 模块迁移状态
type MigrationStatus = 'pending' | 'in_progress' | 'completed' | 'failed';

interface ModuleMigrationInfo {
  moduleName: string;
  currentStatus: MigrationStatus;
  jsLineCount: number;
  targetLineCount: number;
  progress: number; // 0-100
  dependencies: string[];
}

// 导出所有类型
export {
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
