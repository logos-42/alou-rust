/**
 * Alou 自主循环服务
 * 
 * 控制自主智能体主循环的启动、停止、暂停、恢复等
 */

import { invoke } from '@tauri-apps/api/core';

// ============ 类型定义 ============

// 自主循环配置
export interface AutonomousLoopConfig {
  heartbeat_interval_seconds: number;
  task_check_interval_seconds: number;
  memory_save_interval_seconds: number;
  progress_report_interval_seconds: number;
  auto_restart: boolean;
  enabled: boolean;
}

// 自主循环状态
export interface AutonomousLoopState {
  is_running: boolean;
  is_paused: boolean;
  current_task_id: string | null;
  last_heartbeat: number;
  tasks_completed: number;
  tasks_failed: number;
  total_iterations: number;
  config: AutonomousLoopConfig;
}

// 任务优先级
export type TaskPriority = 'critical' | 'high' | 'medium' | 'low';

// ============ 自主循环服务 ============

class AutonomousLoopService {
  private static instance: AutonomousLoopService;

  private constructor() {}

  static getInstance(): AutonomousLoopService {
    if (!AutonomousLoopService.instance) {
      AutonomousLoopService.instance = new AutonomousLoopService();
    }
    return AutonomousLoopService.instance;
  }

  // ============ 生命周期控制 ============

  /**
   * 启动自主循环
   */
  async start(): Promise<{ success: boolean; message: string }> {
    try {
      console.log('🚀 启动自主循环...');
      const result = await invoke<{
        success: boolean;
        message: string;
      }>('start_autonomous_loop');
      
      if (result.success) {
        console.log('✅ 自主循环已启动');
      } else {
        console.error('❌ 启动失败:', result);
      }
      
      return result;
    } catch (error) {
      console.error('启动自主循环失败:', error);
      return { 
        success: false, 
        message: error instanceof Error ? error.message : String(error) 
      };
    }
  }

  /**
   * 停止自主循环
   */
  async stop(): Promise<{ success: boolean; message: string }> {
    try {
      console.log('🛑 停止自主循环...');
      const result = await invoke<{
        success: boolean;
        message: string;
      }>('stop_autonomous_loop');
      
      if (result.success) {
        console.log('✅ 自主循环已停止');
      }
      
      return result;
    } catch (error) {
      console.error('停止自主循环失败:', error);
      return { 
        success: false, 
        message: error instanceof Error ? error.message : String(error) 
      };
    }
  }

  /**
   * 暂停自主循环
   */
  async pause(): Promise<{ success: boolean; message: string }> {
    try {
      console.log('⏸️ 暂停自主循环...');
      const result = await invoke<{
        success: boolean;
        message: string;
      }>('pause_autonomous_loop');
      
      if (result.success) {
        console.log('✅ 自主循环已暂停');
      }
      
      return result;
    } catch (error) {
      console.error('暂停自主循环失败:', error);
      return { 
        success: false, 
        message: error instanceof Error ? error.message : String(error) 
      };
    }
  }

  /**
   * 恢复自主循环
   */
  async resume(): Promise<{ success: boolean; message: string }> {
    try {
      console.log('▶️ 恢复自主循环...');
      const result = await invoke<{
        success: boolean;
        message: string;
      }>('resume_autonomous_loop');
      
      if (result.success) {
        console.log('✅ 自主循环已恢复');
      }
      
      return result;
    } catch (error) {
      console.error('恢复自主循环失败:', error);
      return { 
        success: false, 
        message: error instanceof Error ? error.message : String(error) 
      };
    }
  }

  // ============ 状态查询 ============

  /**
   * 获取自主循环状态
   */
  async getState(): Promise<AutonomousLoopState | null> {
    try {
      const result = await invoke<{
        is_running: boolean;
        is_paused: boolean;
        current_task_id: string | null;
        last_heartbeat: number;
        tasks_completed: number;
        tasks_failed: number;
        total_iterations: number;
        config: AutonomousLoopConfig;
      }>('get_autonomous_loop_state');
      
      return result;
    } catch (error) {
      console.error('获取自主循环状态失败:', error);
      return null;
    }
  }

  // ============ 任务管理 ============

  /**
   * 添加自主任务
   */
  async addTask(
    title: string, 
    description: string, 
    priority: TaskPriority = 'medium'
  ): Promise<{ success: boolean; message: string }> {
    try {
      console.log(`📝 添加自主任务: ${title}`);
      const result = await invoke<{
        success: boolean;
        message: string;
      }>('add_autonomous_task', {
        title,
        description,
        priority,
      });
      
      if (result.success) {
        console.log('✅ 任务已添加');
      }
      
      return result;
    } catch (error) {
      console.error('添加任务失败:', error);
      return { 
        success: false, 
        message: error instanceof Error ? error.message : String(error) 
      };
    }
  }

  // ============ 便捷方法 ============

  /**
   * 切换运行/停止状态
   */
  async toggle(): Promise<{ success: boolean; action: 'started' | 'stopped'; message: string }> {
    const state = await this.getState();
    
    if (!state) {
      return { success: false, action: 'stopped', message: '无法获取状态' };
    }
    
    if (state.is_running) {
      const result = await this.stop();
      return { 
        success: result.success, 
        action: 'stopped' as const, 
        message: result.message 
      };
    } else {
      const result = await this.start();
      return { 
        success: result.success, 
        action: 'started' as const, 
        message: result.message 
      };
    }
  }

  /**
   * 获取运行状态文本
   */
  getStatusText(state: AutonomousLoopState): string {
    if (!state.is_running) {
      return '已停止';
    }
    if (state.is_paused) {
      return '已暂停';
    }
    return '运行中';
  }

  /**
   * 获取状态颜色
   */
  getStatusColor(state: AutonomousLoopState): string {
    if (!state.is_running) {
      return '#ff4444'; // 红色 - 停止
    }
    if (state.is_paused) {
      return '#ffaa00'; // 黄色 - 暂停
    }
    return '#44ff44'; // 绿色 - 运行
  }

  /**
   * 格式化时间戳
   */
  formatTimestamp(timestamp: number): string {
    const date = new Date(timestamp * 1000);
    return date.toLocaleString('zh-CN');
  }

  /**
   * 计算运行时间
   */
  getUptime(state: AutonomousLoopState): string {
    if (!state.is_running) {
      return '0秒';
    }
    
    const now = Date.now();
    const lastHeartbeat = state.last_heartbeat * 1000;
    const uptimeMs = now - lastHeartbeat;
    
    const seconds = Math.floor(uptimeMs / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    
    if (hours > 0) {
      return `${hours}小时${minutes % 60}分钟`;
    }
    if (minutes > 0) {
      return `${minutes}分钟${seconds % 60}秒`;
    }
    return `${seconds}秒`;
  }
}

export default AutonomousLoopService.getInstance();
