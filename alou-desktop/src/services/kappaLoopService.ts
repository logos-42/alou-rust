/**
 * Kappa Loop Service - 卡帕斯循环服务
 * 
 * 前端桥接服务，调用后端 Rust 实现的卡帕斯循环
 * 
 * 核心功能：
 * - Test → Detect → Repair → Verify → Learn → Repeat
 * - 健康评分监测
 * - 熔断器保护
 * - 自动自修复
 */

import { invoke } from '@tauri-apps/api/core';

// ============ 类型定义 ============

export interface KappaLoopConfig {
  project_root: string;
  target_files: string[];
  max_iterations: number;
  dry_run: boolean;
  strict: boolean;
  auto_push: boolean;
  cycle_interval_secs: number;
  enable_web: boolean;
  max_consecutive_failures: number;
  health_score_threshold: number;
}

export interface KappaLoopState {
  is_running: boolean;
  is_paused: boolean;
  current_iteration: number;
  total_cycles_completed: number;
  total_improvements: number;
  total_regressions: number;
  total_failures: number;
  health_score: number;
  circuit_breaker_open: boolean;
  last_error: string | null;
  experiments_log: KappaExperimentSummary[];
}

export interface KappaExperimentSummary {
  iteration: number;
  file: string;
  outcome: string;
  hypothesis: string;
  tests_before: [number, number];
  tests_after: [number, number];
  reflection: string;
  timestamp: string;
  health_score_delta: number;
}

// ============ 默认配置 ============

const DEFAULT_CONFIG: KappaLoopConfig = {
  project_root: '.',
  target_files: [
    'autonomous_loop.rs',
    'agent/ai_client.rs',
    'agent/executor.rs',
  ],
  max_iterations: 10,
  dry_run: true,
  strict: false,
  auto_push: false,
  cycle_interval_secs: 300,
  enable_web: true,
  max_consecutive_failures: 5,
  health_score_threshold: 60.0,
};

// ============ Kappa Loop 服务 ============

class KappaLoopService {
  private static instance: KappaLoopService;

  private constructor() {}

  static getInstance(): KappaLoopService {
    if (!KappaLoopService.instance) {
      KappaLoopService.instance = new KappaLoopService();
    }
    return KappaLoopService.instance;
  }

  // ============ 生命周期控制 ============

  /**
   * 启动卡帕斯循环
   */
  async start(): Promise<{ success: boolean; message: string }> {
    try {
      console.log('🚀 启动卡帕斯循环...');
      const result = await invoke<{
        success: boolean;
        message?: string;
        error?: string;
      }>('start_kappa_loop');
      
      if (result.success) {
        console.log('✅ 卡帕斯循环已启动');
        return { success: true, message: result.message || '已启动' };
      } else {
        console.error('❌ 启动失败:', result.error);
        return { success: false, message: result.error || '启动失败' };
      }
    } catch (error) {
      console.error('启动卡帕斯循环失败:', error);
      return { 
        success: false, 
        message: error instanceof Error ? error.message : String(error) 
      };
    }
  }

  /**
   * 停止卡帕斯循环
   */
  async stop(): Promise<{ success: boolean; message: string }> {
    try {
      console.log('🛑 停止卡帕斯循环...');
      const result = await invoke<{
        success: boolean;
        message?: string;
        error?: string;
      }>('stop_kappa_loop');
      
      if (result.success) {
        console.log('✅ 卡帕斯循环已停止');
        return { success: true, message: result.message || '已停止' };
      } else {
        return { success: false, message: result.error || '停止失败' };
      }
    } catch (error) {
      console.error('停止卡帕斯循环失败:', error);
      return { 
        success: false, 
        message: error instanceof Error ? error.message : String(error) 
      };
    }
  }

  /**
   * 暂停卡帕斯循环
   */
  async pause(): Promise<{ success: boolean; message: string }> {
    try {
      console.log('⏸️ 暂停卡帕斯循环...');
      const result = await invoke<{
        success: boolean;
        message?: string;
        error?: string;
      }>('pause_kappa_loop');
      
      if (result.success) {
        return { success: true, message: result.message || '已暂停' };
      } else {
        return { success: false, message: result.error || '暂停失败' };
      }
    } catch (error) {
      return { 
        success: false, 
        message: error instanceof Error ? error.message : String(error) 
      };
    }
  }

  /**
   * 恢复卡帕斯循环
   */
  async resume(): Promise<{ success: boolean; message: string }> {
    try {
      console.log('▶️ 恢复卡帕斯循环...');
      const result = await invoke<{
        success: boolean;
        message?: string;
        error?: string;
      }>('resume_kappa_loop');
      
      if (result.success) {
        return { success: true, message: result.message || '已恢复' };
      } else {
        return { success: false, message: result.error || '恢复失败' };
      }
    } catch (error) {
      return { 
        success: false, 
        message: error instanceof Error ? error.message : String(error) 
      };
    }
  }

  // ============ 状态查询 ============

  /**
   * 获取卡帕斯循环状态
   */
  async getState(): Promise<KappaLoopState | null> {
    try {
      const result = await invoke<{
        is_running: boolean;
        is_paused: boolean;
        current_iteration: number;
        total_cycles_completed: number;
        total_improvements: number;
        total_regressions: number;
        total_failures: number;
        health_score: number;
        circuit_breaker_open: boolean;
        last_error: string | null;
        experiments_log: KappaExperimentSummary[];
      }>('get_kappa_loop_state');
      
      return result;
    } catch (error) {
      console.error('获取卡帕斯循环状态失败:', error);
      return null;
    }
  }

  /**
   * 手动触发自修复
   */
  async triggerSelfRepair(): Promise<{
    success: boolean;
    improved?: boolean;
    error?: string;
  }> {
    try {
      console.log('🔧 手动触发自修复...');
      const result = await invoke<{
        success: boolean;
        improved?: boolean;
        error?: string;
      }>('trigger_kappa_self_repair');
      
      return result;
    } catch (error) {
      return { 
        success: false, 
        error: error instanceof Error ? error.message : String(error) 
      };
    }
  }

  /**
   * 更新配置
   */
  async updateConfig(config: Partial<KappaLoopConfig>): Promise<{
    success: boolean;
    message: string;
  }> {
    try {
      const result = await invoke<{
        success: boolean;
        message?: string;
      }>('update_kappa_loop_config', config);
      
      return result;
    } catch (error) {
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
  getStatusText(state: KappaLoopState): string {
    if (state.circuit_breaker_open) {
      return '熔断中';
    }
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
  getStatusColor(state: KappaLoopState): string {
    if (state.circuit_breaker_open) {
      return '#ff8800'; // 橙色 - 熔断
    }
    if (!state.is_running) {
      return '#ff4444'; // 红色 - 停止
    }
    if (state.is_paused) {
      return '#ffaa00'; // 黄色 - 暂停
    }
    if (state.health_score < 60) {
      return '#ff8800'; // 橙色 - 警告
    }
    return '#44ff44'; // 绿色 - 正常运行
  }

  /**
   * 获取健康评分颜色
   */
  getHealthColor(score: number): string {
    if (score >= 80) return '#44ff44'; // 绿色 - 优秀
    if (score >= 60) return '#ffaa00'; // 黄色 - 良好
    if (score >= 40) return '#ff8800'; // 橙色 - 警告
    return '#ff4444'; // 红色 - 危险
  }

  /**
   * 格式化统计信息
   */
  formatStats(state: KappaLoopState): string {
    return `改进: ${state.total_improvements} | 回退: ${state.total_regressions} | 失败: ${state.total_failures}`;
  }
}

export default KappaLoopService.getInstance();

// ============ 工具函数 ============

export function createKappaConfig(config?: Partial<KappaLoopConfig>): KappaLoopConfig {
  return { ...DEFAULT_CONFIG, ...config };
}

export function formatKappaState(state: KappaLoopState): string {
  const status = state.is_running ? '🔄 运行中' : '⏹️ 已停止';
  const health = `❤️ 健康: ${state.health_score.toFixed(1)}%`;
  const stats = `改进: ${state.total_improvements} | 回退: ${state.total_regressions}`;
  const cycles = `循环: ${state.total_cycles_completed}次`;
  
  return `${status} | ${health} | ${stats} | ${cycles}`;
}

export function getExperimentOutcomeLabel(outcome: string): { label: string; color: string } {
  switch (outcome) {
    case 'Improved':
      return { label: '改进', color: '#44ff44' };
    case 'Neutral':
      return { label: '中性', color: '#ffaa00' };
    case 'Regressed':
      return { label: '回退', color: '#ff4444' };
    case 'Failed':
      return { label: '失败', color: '#888888' };
    default:
      return { label: outcome, color: '#cccccc' };
  }
}