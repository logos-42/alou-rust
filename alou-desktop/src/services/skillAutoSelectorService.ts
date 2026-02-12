/**
 * Skills 自动选择器服务
 * 
 * 提供AI根据消息内容自动选择工具的能力
 */

import { invoke } from '@tauri-apps/api/core';

/**
 * 工具匹配结果
 */
export interface ToolMatch {
  tool_id: string;
  name: string;
  confidence: number;
  reason: string;
}

/**
 * 分析结果
 */
export interface AnalyzeResult {
  success: boolean;
  tools: ToolMatch[];
  primary_intent: string | null;
  total_count: number;
  message: string;
}

/**
 * 配置参数
 */
export interface AutoSelectConfig {
  enabled: boolean;
  min_confidence: number;
  max_selections: number;
  allow_tool_chain: boolean;
}

/**
 * 工具信息
 */
export interface ToolInfo {
  id: string;
  description: string;
  keywords: string[];
  count: number;
}

/**
 * 执行计划
 */
export interface ExecutionPlan {
  success: boolean;
  primary_intent: string;
  confidence: number;
  plan: PlanStep[];
  total_tools: number;
  message: string;
}

/**
 * 计划步骤
 */
export interface PlanStep {
  step: number;
  tool: string;
  action: string;
  confidence: number;
  params: Record<string, unknown>;
}

class SkillAutoSelectorService {
  private static instance: SkillAutoSelectorService;

  private constructor() {}

  /**
   * 获取单例实例
   */
  static getInstance(): SkillAutoSelectorService {
    if (!SkillAutoSelectorService.instance) {
      SkillAutoSelectorService.instance = new SkillAutoSelectorService();
    }
    return SkillAutoSelectorService.instance;
  }

  /**
   * 分析消息并选择工具
   * 
   * @param message 用户消息
   * @param maxSelections 最大选择数量（可选，默认3）
   * @param minConfidence 最小置信度（可选，默认0.3）
   */
  async analyzeAndSelectTools(
    message: string,
    maxSelections?: number,
    minConfidence?: number
  ): Promise<AnalyzeResult> {
    try {
      const result = await invoke<AnalyzeResult>('analyze_and_select_tools', {
        message,
        maxSelections,
        minConfidence,
      });
      return result;
    } catch (error) {
      console.error('分析工具选择失败:', error);
      return {
        success: false,
        tools: [],
        primary_intent: null,
        total_count: 0,
        message: `分析失败: ${error}`,
      };
    }
  }

  /**
   * 生成执行计划
   * 
   * @param message 用户消息
   */
  async generateExecutionPlan(message: string): Promise<ExecutionPlan> {
    try {
      const result = await invoke<{
        success: boolean;
        primary_intent: string;
        confidence: number;
        plan: PlanStep[];
        total_tools: number;
        message: string;
      }>('generate_execution_plan', { message });
      
      return {
        success: result.success,
        primary_intent: result.primary_intent,
        confidence: result.confidence,
        plan: result.plan,
        total_tools: result.total_tools,
        message: result.message,
      };
    } catch (error) {
      console.error('生成执行计划失败:', error);
      return {
        success: false,
        primary_intent: '',
        confidence: 0,
        plan: [],
        total_tools: 0,
        message: `生成计划失败: ${error}`,
      };
    }
  }

  /**
   * 获取可用工具列表
   */
  async getAvailableTools(): Promise<ToolInfo[]> {
    try {
      const result = await invoke<{
        success: boolean;
        tools: ToolInfo[];
        total: number;
      }>('get_available_tools');
      
      if (result.success) {
        return result.tools;
      }
      return [];
    } catch (error) {
      console.error('获取工具列表失败:', error);
      return [];
    }
  }

  /**
   * 更新配置
   */
  async updateConfig(config: Partial<AutoSelectConfig>): Promise<boolean> {
    try {
      const result = await invoke<{
        success: boolean;
        config: AutoSelectConfig;
        message: string;
      }>('update_auto_select_config', config);
      
      return result.success;
    } catch (error) {
      console.error('更新配置失败:', error);
      return false;
    }
  }

  /**
   * 获取当前配置
   */
  async getConfig(): Promise<AutoSelectConfig | null> {
    try {
      const result = await invoke<{
        success: boolean;
        config: AutoSelectConfig;
      }>('get_auto_select_config');
      
      if (result.success) {
        return result.config;
      }
      return null;
    } catch (error) {
      console.error('获取配置失败:', error);
      return null;
    }
  }

  /**
   * 智能推荐工具
   * 
   * 根据消息自动分析并返回推荐的工具列表
   */
  async smartRecommend(message: string): Promise<{
    recommended: ToolMatch[];
    plan: ExecutionPlan;
  }> {
    const [analyzeResult, plan] = await Promise.all([
      this.analyzeAndSelectTools(message),
      this.generateExecutionPlan(message),
    ]);

    return {
      recommended: analyzeResult.tools,
      plan,
    };
  }
}

export default SkillAutoSelectorService.getInstance();
