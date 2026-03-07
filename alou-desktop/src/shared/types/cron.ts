/**
 * Cron 定时任务相关类型定义
 * Cron Task Related Type Definitions
 */

/**
 * Cron 任务状态
 * Cron job state
 */
export type CronJobState = 'pending' | 'running' | 'completed' | 'failed';

/**
 * Cron 任务定义
 * Cron job definition
 */
export interface CronJob {
  /** 任务名称 */
  name: string;
  /** Cron 表达式（格式：分 时 日 月 星期） */
  schedule: string;
  /** 执行提示词（发送给 AI Agent） */
  prompt: string;
  /** 是否使用独立会话 */
  session_isolation: boolean;
  /** 结果文件路径（可选） */
  result_file?: string;
  /** 是否启用此任务 */
  enabled: boolean;
}

/**
 * Cron 任务执行结果
 * Cron job execution result
 */
export interface CronJobResult {
  /** 任务名称 */
  job_name: string;
  /** 执行状态 */
  status: CronJobState;
  /** 输出内容 */
  output?: string;
  /** 错误信息 */
  error?: string;
  /** 执行时间 */
  executed_at: string;
  /** 执行时长（毫秒） */
  execution_time_ms?: number;
  /** Session ID（用于会话隔离） */
  session_id?: string;
}

/**
 * Cron 配置
 * Cron configuration
 */
export interface CronConfig {
  /** Cron 任务列表 */
  jobs: CronJob[];
  /** 是否启用 Cron 调度器 */
  enabled: boolean;
  /** 检查间隔（秒） */
  check_interval_seconds: number;
}

/**
 * Cron 调度器状态
 * Cron scheduler state
 */
export interface CronSchedulerState {
  /** 调度器状态：stopped | running | paused */
  state: 'stopped' | 'running' | 'paused';
  /** 历史记录数量 */
  history_count: number;
}

/**
 * Cron 服务响应类型
 * Cron service response type
 */
export interface CronResponse<T = any> {
  /** 是否成功 */
  success: boolean;
  /** 响应消息 */
  message?: string;
  /** 错误信息 */
  error?: string;
  /** 响应数据 */
  data?: T;
}

/**
 * Cron 任务列表响应
 * Cron job list response
 */
export interface CronJobListResponse {
  /** 任务列表 */
  jobs: CronJob[];
  /** 任务数量 */
  count: number;
}

/**
 * Cron 执行历史响应
 * Cron execution history response
 */
export interface CronJobHistoryResponse {
  /** 执行历史列表 */
  history: CronJobResult[];
  /** 历史记录数量 */
  count: number;
}

/**
 * Cron 配置响应
 * Cron config response
 */
export interface CronConfigResponse {
  /** Cron 配置 */
  config: CronConfig;
}

/**
 * Cron 表达式解析结果
 * Cron expression parse result
 */
export interface ParsedCronExpression {
  /** 分钟 (0-59) */
  minute: number | '*';
  /** 小时 (0-23) */
  hour: number | '*';
  /** 日期 (1-31) */
  dayOfMonth: number | '*';
  /** 月份 (1-12) */
  month: number | '*';
  /** 星期 (0-6, 0=Sunday) */
  dayOfWeek: number | '*';
  /** 原始表达式 */
  original: string;
  /** 是否有效 */
  isValid: boolean;
  /** 错误信息（如果无效） */
  error?: string;
}

/**
 * Cron 表达式人类可读描述
 * Cron expression human-readable description
 */
export interface HumanReadableCron {
  /** 简短描述 */
  short: string;
  /** 详细描述 */
  long: string;
  /** 下次运行时间 */
  nextRun: string;
}
