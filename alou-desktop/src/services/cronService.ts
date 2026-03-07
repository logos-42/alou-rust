/**
 * Cron 定时任务服务
 * Cron Task Service
 *
 * 负责管理 Alou 桌面端的 Cron 定时任务，包括：
 * - 启动/停止/暂停/恢复调度器
 * - 添加/删除/启用/禁用任务
 * - 立即运行任务
 * - 获取执行历史
 * - 配置管理
 */

import { invoke } from '@tauri-apps/api/core';
import type {
  CronJob,
  CronJobResult,
  CronConfig,
  CronSchedulerState,
  CronResponse,
  CronJobListResponse,
  CronJobHistoryResponse,
  CronConfigResponse,
} from '@shared/types/cron';

/**
 * Cron 服务类
 * Cron Service Class
 */
class CronService {
  private readonly COMMAND_PREFIX = 'cron_';

  /**
   * 启动 Cron 调度器
   * Start the Cron scheduler
   * @returns 操作结果
   */
  async startScheduler(): Promise<CronResponse<CronSchedulerState>> {
    try {
      const result = await invoke<serde_json_Value>(`${this.COMMAND_PREFIX}start_scheduler`);
      return {
        success: true,
        message: 'Cron 调度器已启动',
        data: result as any,
      };
    } catch (error: any) {
      console.error('[CronService] 启动调度器失败:', error);
      return {
        success: false,
        error: this.formatError(error),
        message: '启动调度器失败',
      };
    }
  }

  /**
   * 停止 Cron 调度器
   * Stop the Cron scheduler
   * @returns 操作结果
   */
  async stopScheduler(): Promise<CronResponse<CronSchedulerState>> {
    try {
      const result = await invoke<serde_json_Value>(`${this.COMMAND_PREFIX}stop_scheduler`);
      return {
        success: true,
        message: 'Cron 调度器已停止',
        data: result as any,
      };
    } catch (error: any) {
      console.error('[CronService] 停止调度器失败:', error);
      return {
        success: false,
        error: this.formatError(error),
        message: '停止调度器失败',
      };
    }
  }

  /**
   * 暂停 Cron 调度器
   * Pause the Cron scheduler
   * @returns 操作结果
   */
  async pauseScheduler(): Promise<CronResponse<CronSchedulerState>> {
    try {
      const result = await invoke<serde_json_Value>(`${this.COMMAND_PREFIX}pause_scheduler`);
      return {
        success: true,
        message: 'Cron 调度器已暂停',
        data: result as any,
      };
    } catch (error: any) {
      console.error('[CronService] 暂停调度器失败:', error);
      return {
        success: false,
        error: this.formatError(error),
        message: '暂停调度器失败',
      };
    }
  }

  /**
   * 恢复 Cron 调度器
   * Resume the Cron scheduler
   * @returns 操作结果
   */
  async resumeScheduler(): Promise<CronResponse<CronSchedulerState>> {
    try {
      const result = await invoke<serde_json_Value>(`${this.COMMAND_PREFIX}resume_scheduler`);
      return {
        success: true,
        message: 'Cron 调度器已恢复',
        data: result as any,
      };
    } catch (error: any) {
      console.error('[CronService] 恢复调度器失败:', error);
      return {
        success: false,
        error: this.formatError(error),
        message: '恢复调度器失败',
      };
    }
  }

  /**
   * 获取 Cron 调度器状态
   * Get current Cron scheduler state
   * @returns Cron 调度器状态
   */
  async getSchedulerState(): Promise<CronResponse<CronSchedulerState>> {
    try {
      const result = await invoke<serde_json_Value>(`${this.COMMAND_PREFIX}get_scheduler_state`);
      const data = result as any;
      return {
        success: true,
        message: '获取状态成功',
        data: {
          state: data.state,
          history_count: data.history_count,
        },
      };
    } catch (error: any) {
      console.error('[CronService] 获取调度器状态失败:', error);
      return {
        success: false,
        error: this.formatError(error),
        message: '获取调度器状态失败',
      };
    }
  }

  /**
   * 列出所有 Cron 任务
   * List all Cron jobs
   * @returns Cron 任务列表
   */
  async listJobs(): Promise<CronResponse<CronJob[]>> {
    try {
      const result = await invoke<serde_json_Value>(`${this.COMMAND_PREFIX}list_jobs`);
      const data = result as CronJobListResponse;
      return {
        success: true,
        message: '获取任务列表成功',
        data: data.jobs,
      };
    } catch (error: any) {
      console.error('[CronService] 获取任务列表失败:', error);
      return {
        success: false,
        error: this.formatError(error),
        message: '获取任务列表失败',
      };
    }
  }

  /**
   * 添加 Cron 任务
   * Add a Cron job
   * @param job Cron 任务定义
   * @returns 操作结果
   */
  async addJob(job: CronJob): Promise<CronResponse<void>> {
    try {
      await invoke<serde_json_Value>(`${this.COMMAND_PREFIX}add_job`, { job });
      return {
        success: true,
        message: '任务已添加',
      };
    } catch (error: any) {
      console.error('[CronService] 添加任务失败:', error);
      return {
        success: false,
        error: this.formatError(error),
        message: '添加任务失败',
      };
    }
  }

  /**
   * 删除 Cron 任务
   * Remove a Cron job
   * @param name 任务名称
   * @returns 操作结果
   */
  async removeJob(name: string): Promise<CronResponse<void>> {
    try {
      await invoke<serde_json_Value>(`${this.COMMAND_PREFIX}remove_job`, { name });
      return {
        success: true,
        message: `任务 '${name}' 已删除`,
      };
    } catch (error: any) {
      console.error('[CronService] 删除任务失败:', error);
      return {
        success: false,
        error: this.formatError(error),
        message: '删除任务失败',
      };
    }
  }

  /**
   * 立即运行 Cron 任务
   * Run a Cron job immediately
   * @param name 任务名称
   * @returns 执行结果
   */
  async runJobNow(name: string): Promise<CronResponse<CronJobResult>> {
    try {
      const result = await invoke<serde_json_Value>(`${this.COMMAND_PREFIX}run_job_now`, { name });
      const data = result as any;
      return {
        success: data.success,
        message: data.success ? '任务执行成功' : '任务执行失败',
        data: data.result as CronJobResult,
      };
    } catch (error: any) {
      console.error('[CronService] 立即运行任务失败:', error);
      return {
        success: false,
        error: this.formatError(error),
        message: '立即运行任务失败',
      };
    }
  }

  /**
   * 启用/禁用 Cron 任务
   * Enable/Disable a Cron job
   * @param name 任务名称
   * @param enabled 是否启用
   * @returns 操作结果
   */
  async toggleJob(name: string, enabled: boolean): Promise<CronResponse<void>> {
    try {
      await invoke<serde_json_Value>(`${this.COMMAND_PREFIX}toggle_job`, { name, enabled });
      return {
        success: true,
        message: `任务 '${name}' 已${enabled ? '启用' : '禁用'}`,
      };
    } catch (error: any) {
      console.error('[CronService] 切换任务状态失败:', error);
      return {
        success: false,
        error: this.formatError(error),
        message: '切换任务状态失败',
      };
    }
  }

  /**
   * 获取 Cron 任务执行历史
   * Get Cron job execution history
   * @param limit 限制返回数量（可选）
   * @returns 执行历史列表
   */
  async getJobHistory(limit?: number): Promise<CronResponse<CronJobResult[]>> {
    try {
      const result = await invoke<serde_json_Value>(`${this.COMMAND_PREFIX}get_job_history`, { limit });
      const data = result as CronJobHistoryResponse;
      return {
        success: true,
        message: '获取执行历史成功',
        data: data.history,
      };
    } catch (error: any) {
      console.error('[CronService] 获取执行历史失败:', error);
      return {
        success: false,
        error: this.formatError(error),
        message: '获取执行历史失败',
      };
    }
  }

  /**
   * 获取 Cron 配置
   * Get Cron configuration
   * @returns Cron 配置
   */
  async getConfig(): Promise<CronResponse<CronConfig>> {
    try {
      const result = await invoke<serde_json_Value>(`${this.COMMAND_PREFIX}get_config`);
      const data = result as CronConfigResponse;
      return {
        success: true,
        message: '获取配置成功',
        data: data.config,
      };
    } catch (error: any) {
      console.error('[CronService] 获取配置失败:', error);
      return {
        success: false,
        error: this.formatError(error),
        message: '获取配置失败',
      };
    }
  }

  /**
   * 更新 Cron 配置
   * Update Cron configuration
   * @param config Cron 配置
   * @returns 操作结果
   */
  async updateConfig(config: CronConfig): Promise<CronResponse<void>> {
    try {
      await invoke<serde_json_Value>(`${this.COMMAND_PREFIX}update_config`, { config });
      return {
        success: true,
        message: '配置已更新',
      };
    } catch (error: any) {
      console.error('[CronService] 更新配置失败:', error);
      return {
        success: false,
        error: this.formatError(error),
        message: '更新配置失败',
      };
    }
  }

  /**
   * 创建默认 Cron 配置
   * Create default Cron configuration
   * @returns 操作结果和配置
   */
  async createDefaultConfig(): Promise<CronResponse<CronConfig>> {
    try {
      const result = await invoke<serde_json_Value>(`${this.COMMAND_PREFIX}create_default_config`);
      const data = result as any;
      return {
        success: true,
        message: '默认配置已创建',
        data: data.config as CronConfig,
      };
    } catch (error: any) {
      console.error('[CronService] 创建默认配置失败:', error);
      return {
        success: false,
        error: this.formatError(error),
        message: '创建默认配置失败',
      };
    }
  }

  /**
   * 清除 Cron 执行历史
   * Clear Cron execution history
   * @returns 操作结果
   */
  async clearJobHistory(): Promise<CronResponse<void>> {
    try {
      await invoke<serde_json_Value>(`${this.COMMAND_PREFIX}clear_job_history`);
      return {
        success: true,
        message: '历史记录已清除',
      };
    } catch (error: any) {
      console.error('[CronService] 清除历史记录失败:', error);
      return {
        success: false,
        error: this.formatError(error),
        message: '清除历史记录失败',
      };
    }
  }

  /**
   * 获取配置文件路径
   * Get configuration file path
   * @returns 配置文件路径
   */
  async getConfigPath(): Promise<CronResponse<string>> {
    try {
      const result = await invoke<serde_json_Value>(`${this.COMMAND_PREFIX}get_config_path`);
      const data = result as any;
      return {
        success: true,
        message: '获取配置文件路径成功',
        data: data.path as string,
      };
    } catch (error: any) {
      console.error('[CronService] 获取配置文件路径失败:', error);
      return {
        success: false,
        error: this.formatError(error),
        message: '获取配置文件路径失败',
      };
    }
  }

  /**
   * 格式化错误信息
   * Format error message
   * @param error 错误对象
   * @returns 格式化后的错误信息
   */
  private formatError(error: any): string {
    if (typeof error === 'string') {
      return error;
    }
    if (error?.message) {
      return error.message;
    }
    return String(error);
  }
}

// 导出单例
export default new CronService();
