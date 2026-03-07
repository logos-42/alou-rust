/**
 * 心跳服务
 * Heartbeat Service
 * 
 * 负责管理 Alou 桌面端的心跳机制，包括：
 * - 定期发送心跳以保持系统活跃
 * - 健康检查和自愈功能
 * - 配置管理和状态监控
 */

import { invoke } from '@tauri-apps/api/core';
import type {
  HeartbeatConfig,
  HeartbeatState,
  HealthStatus,
  HeartbeatResponse,
} from '@shared/types/heartbeat';

/**
 * 心跳服务类
 * Heartbeat Service Class
 */
class HeartbeatService {
  private readonly COMMAND_PREFIX = 'heartbeat_';

  /**
   * 启动心跳
   * Start the heartbeat mechanism
   * @returns 操作结果
   */
  async startHeartbeat(): Promise<HeartbeatResponse<HeartbeatState>> {
    try {
      const result = await invoke<HeartbeatState>(`${this.COMMAND_PREFIX}start`);
      return {
        success: true,
        message: '心跳已启动',
        data: result,
      };
    } catch (error: any) {
      console.error('[HeartbeatService] 启动心跳失败:', error);
      return {
        success: false,
        error: this.formatError(error),
        message: '启动心跳失败',
      };
    }
  }

  /**
   * 停止心跳
   * Stop the heartbeat mechanism
   * @returns 操作结果
   */
  async stopHeartbeat(): Promise<HeartbeatResponse<HeartbeatState>> {
    try {
      const result = await invoke<HeartbeatState>(`${this.COMMAND_PREFIX}stop`);
      return {
        success: true,
        message: '心跳已停止',
        data: result,
      };
    } catch (error: any) {
      console.error('[HeartbeatService] 停止心跳失败:', error);
      return {
        success: false,
        error: this.formatError(error),
        message: '停止心跳失败',
      };
    }
  }

  /**
   * 立即触发心跳
   * Trigger a heartbeat immediately
   * @returns 操作结果
   */
  async triggerHeartbeatNow(): Promise<HeartbeatResponse<HeartbeatState>> {
    try {
      const result = await invoke<HeartbeatState>(`${this.COMMAND_PREFIX}trigger_now`);
      return {
        success: true,
        message: '心跳已触发',
        data: result,
      };
    } catch (error: any) {
      console.error('[HeartbeatService] 触发心跳失败:', error);
      return {
        success: false,
        error: this.formatError(error),
        message: '触发心跳失败',
      };
    }
  }

  /**
   * 获取心跳状态
   * Get current heartbeat state
   * @returns 心跳状态
   */
  async getState(): Promise<HeartbeatResponse<HeartbeatState>> {
    try {
      const result = await invoke<HeartbeatState>(`${this.COMMAND_PREFIX}get_state`);
      return {
        success: true,
        data: result,
      };
    } catch (error: any) {
      console.error('[HeartbeatService] 获取状态失败:', error);
      return {
        success: false,
        error: this.formatError(error),
        message: '获取状态失败',
      };
    }
  }

  /**
   * 获取心跳配置
   * Get heartbeat configuration
   * @returns 心跳配置
   */
  async getConfig(): Promise<HeartbeatResponse<HeartbeatConfig>> {
    try {
      const result = await invoke<HeartbeatConfig>(`${this.COMMAND_PREFIX}get_config`);
      return {
        success: true,
        data: result,
      };
    } catch (error: any) {
      console.error('[HeartbeatService] 获取配置失败:', error);
      return {
        success: false,
        error: this.formatError(error),
        message: '获取配置失败',
      };
    }
  }

  /**
   * 更新心跳配置
   * Update heartbeat configuration
   * @param config 配置更新（部分配置）
   * @returns 操作结果
   */
  async updateConfig(
    config: Partial<HeartbeatConfig>
  ): Promise<HeartbeatResponse<HeartbeatConfig>> {
    try {
      const result = await invoke<HeartbeatConfig>(`${this.COMMAND_PREFIX}update_config`, {
        config,
      });
      return {
        success: true,
        message: '配置已更新',
        data: result,
      };
    } catch (error: any) {
      console.error('[HeartbeatService] 更新配置失败:', error);
      return {
        success: false,
        error: this.formatError(error),
        message: '更新配置失败',
      };
    }
  }

  /**
   * 运行健康检查
   * Run health check
   * @returns 健康状态
   */
  async runHealthCheck(): Promise<HeartbeatResponse<HealthStatus>> {
    try {
      const result = await invoke<HealthStatus>(`${this.COMMAND_PREFIX}health_check`);
      return {
        success: true,
        data: result,
      };
    } catch (error: any) {
      console.error('[HeartbeatService] 健康检查失败:', error);
      return {
        success: false,
        error: this.formatError(error),
        message: '健康检查失败',
      };
    }
  }

  /**
   * 获取健康状态
   * Get health status
   * @returns 健康状态
   */
  async getHealthStatus(): Promise<HeartbeatResponse<HealthStatus>> {
    try {
      const result = await invoke<HealthStatus>(`${this.COMMAND_PREFIX}get_health_status`);
      return {
        success: true,
        data: result,
      };
    } catch (error: any) {
      console.error('[HeartbeatService] 获取健康状态失败:', error);
      return {
        success: false,
        error: this.formatError(error),
        message: '获取健康状态失败',
      };
    }
  }

  /**
   * 执行自愈
   * Execute self-healing
   * @returns 操作结果
   */
  async executeSelfHeal(): Promise<HeartbeatResponse<{ healed_issues: string[] }>> {
    try {
      const result = await invoke<{ healed_issues: string[] }>(
        `${this.COMMAND_PREFIX}self_heal`
      );
      return {
        success: true,
        message: `已修复 ${result.healed_issues.length} 个问题`,
        data: result,
      };
    } catch (error: any) {
      console.error('[HeartbeatService] 自愈执行失败:', error);
      return {
        success: false,
        error: this.formatError(error),
        message: '自愈执行失败',
      };
    }
  }

  /**
   * 格式化错误消息
   * Format error message
   * @param error 错误对象
   * @returns 格式化后的错误消息
   */
  private formatError(error: any): string {
    if (typeof error === 'string') {
      return error;
    }
    if (error instanceof Error) {
      return error.message;
    }
    if (typeof error === 'object' && error !== null) {
      return error.message || error.error || JSON.stringify(error);
    }
    return String(error);
  }
}

// 创建单例实例
const heartbeatService = new HeartbeatService();

export { heartbeatService };
export default heartbeatService;
export type { HeartbeatService };
