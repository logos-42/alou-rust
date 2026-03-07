/**
 * 心跳服务相关类型定义
 * Heartbeat Service Type Definitions
 */

/**
 * 心跳配置接口
 * Heartbeat configuration interface
 */
export interface HeartbeatConfig {
  /** 是否启用心跳 */
  enabled: boolean;
  /** 心跳间隔（分钟） */
  interval_minutes: number;
  /** 使用的模型 */
  model: string;
  /** 心跳文件路径 */
  heartbeat_file_path: string;
  /** 便宜模型（用于常规心跳） */
  cheap_model: string;
  /** 昂贵模型（用于重要操作） */
  expensive_model: string;
}

/**
 * 心跳状态接口
 * Heartbeat state interface
 */
export interface HeartbeatState {
  /** 是否正在运行 */
  is_running: boolean;
  /** 上次心跳时间戳（毫秒） */
  last_heartbeat: number | null;
  /** 下次心跳时间戳（毫秒） */
  next_heartbeat: number | null;
  /** 总心跳次数 */
  total_beats: number;
}

/**
 * 检查结果接口
 * Check result interface
 */
export interface CheckResult {
  /** 检查项名称 */
  name: string;
  /** 检查是否通过 */
  passed: boolean;
  /** 检查详情 */
  details?: string;
  /** 建议的修复措施 */
  recommendation?: string;
}

/**
 * 问题接口
 * Issue interface
 */
export interface Issue {
  /** 问题代码 */
  code: string;
  /** 问题描述 */
  description: string;
  /** 严重程度 */
  severity: 'low' | 'medium' | 'high' | 'critical';
  /** 建议的修复措施 */
  fix?: string;
}

/**
 * 健康状态接口
 * Health status interface
 */
export interface HealthStatus {
  /** 整体健康状态 */
  overall: 'Good' | 'Warning' | 'Critical';
  /** 检查结果列表 */
  checks: CheckResult[];
  /** 问题列表 */
  issues: Issue[];
}

/**
 * 心跳服务响应类型
 * Heartbeat service response type
 */
export interface HeartbeatResponse<T = any> {
  /** 是否成功 */
  success: boolean;
  /** 响应消息 */
  message?: string;
  /** 错误信息 */
  error?: string;
  /** 响应数据 */
  data?: T;
}
