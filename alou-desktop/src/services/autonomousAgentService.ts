/**
 * Alou 自主智能体服务
 * 
 * 整合Skills自动选择、工具执行、Claude官方Skills格式支持
 */

import { invoke } from '@tauri-apps/api/core';
import skillAutoSelectorService from './skillAutoSelectorService';
import agentSkillsService, { AgentSkill, SkillExecutionResult } from './agentSkillsService';

// ============ 类型定义 ============

// 工具匹配结果
export interface ToolMatch {
  tool_id: string;
  name: string;
  confidence: number;
  reason: string;
}

// 分析结果
export interface AnalyzeResult {
  success: boolean;
  tools: ToolMatch[];
  primary_intent: string | null;
  total_count: number;
  message: string;
}

// 执行计划步骤
export interface PlanStep {
  step: number;
  tool: string;
  action: string;
  confidence: number;
  params: Record<string, unknown>;
}

// 执行计划
export interface ExecutionPlan {
  success: boolean;
  primary_intent: string;
  confidence: number;
  plan: PlanStep[];
  total_tools: number;
  message: string;
}

// 执行结果
export interface ExecutionResult {
  success: boolean;
  tool_id: string;
  result: Record<string, unknown>;
  error: string | null;
  execution_time_ms: number;
}

// Claude Skills 执行结果
export interface ClaudeSkillResult {
  success: boolean;
  skill_name: string;
  output: Record<string, unknown>;
  execution_time_ms: number;
  error?: string;
  logs: string[];
}

// 执行配置
export interface ExecutorConfig {
  enabled: boolean;
  auto_execute: boolean;
  max_concurrent: number;
  timeout_seconds: number;
  retry_on_failure: boolean;
  max_retries: number;
}

// 响应配置
export interface RespondConfig {
  enabled: boolean;
  respond_to_mentions: boolean;
  respond_to_keywords: boolean;
  respond_to_all: boolean;
  keywords: string[];
  ignore_users: string[];
  cooldown_seconds: number;
}

// 群聊消息
export interface ChatMessage {
  id: string;
  topic: string;
  content: string;
  sender: string;
  timestamp: number;
  message_type: string;
}

// ============ Claude Skills 自主智能体服务 ============

class AutonomousAgentService {
  private static instance: AutonomousAgentService;

  private constructor() {}

  static getInstance(): AutonomousAgentService {
    if (!AutonomousAgentService.instance) {
      AutonomousAgentService.instance = new AutonomousAgentService();
    }
    return AutonomousAgentService.instance;
  }

  // ============ 需求分析 ============

  /**
   * 分析用户需求，选择合适的工具
   */
  async analyzeRequirement(message: string): Promise<AnalyzeResult> {
    return skillAutoSelectorService.analyzeAndSelectTools(message);
  }

  /**
   * 生成执行计划
   */
  async generatePlan(message: string): Promise<ExecutionPlan> {
    return skillAutoSelectorService.smartRecommend(message).then(r => r.plan);
  }

  /**
   * 完整分析并生成计划
   */
  async analyzeAndPlan(message: string): Promise<{
    analysis: AnalyzeResult;
    plan: ExecutionPlan;
  }> {
    const [analysis, plan] = await Promise.all([
      this.analyzeRequirement(message),
      this.generatePlan(message),
    ]);

    return { analysis, plan };
  }

  // ============ Claude Skills 执行 ============

  /**
   * AI自动选择并执行Claude Skills
   * 
   * 这是核心功能：AI分析需求 → 选择最佳Skill → 执行Skill
   */
  async executeClaudeSkill(
    userIntent: string,
    preferredSkills: string[] = []
  ): Promise<ClaudeSkillResult> {
    console.log('🧠 开始Claude Skills执行流程...');
    console.log('📝 用户需求:', userIntent);

    // 1. 发现可用Skills
    console.log('📚 发现可用Skills...');
    const skills = await agentSkillsService.discoverSkills();
    
    if (skills.length === 0) {
      console.log('⚠️ 没有可用的Skills');
      return {
        success: false,
        skill_name: '',
        output: {},
        execution_time_ms: 0,
        error: '没有可用的Skills，请先创建Skills',
        logs: ['发现Skills: 0个'],
      };
    }
    
    console.log(`✅ 发现 ${skills.length} 个Skills`);

    // 2. 根据用户意图选择最佳Skill
    console.log('🎯 选择最佳Skill...');
    const selectedSkill = await this.selectBestSkill(skills, userIntent, preferredSkills);
    
    if (!selectedSkill) {
      console.log('⚠️ 没有找到匹配的Skill');
      return {
        success: false,
        skill_name: '',
        output: {},
        execution_time_ms: 0,
        error: '没有找到匹配的Skill',
        logs: [`可用Skills: ${skills.map(s => s.name).join(', ')}`],
      };
    }
    
    console.log(`✅ 选择Skill: ${selectedSkill.name}`);

    // 3. 加载Skill完整内容
    console.log('📖 加载Skill内容...');
    const skillDetail = await agentSkillsService.loadSkill(selectedSkill.name);
    
    if (!skillDetail) {
      return {
        success: false,
        skill_name: selectedSkill.name,
        output: {},
        execution_time_ms: 0,
        error: '无法加载Skill内容',
        logs: [`选择Skill: ${selectedSkill.name}`, '加载失败'],
      };
    }

    // 4. 执行Skill
    console.log('⚙️ 执行Skill...');
    const startTime = Date.now();
    const result = await agentSkillsService.executeSkill(selectedSkill.name);
    const executionTime = Date.now() - startTime;

    console.log(`⏱️ 执行完成 (${executionTime}ms)`);

    return {
      success: result.success,
      skill_name: selectedSkill.name,
      output: result.output,
      execution_time_ms: executionTime,
      error: result.error || undefined,
      logs: result.logs,
    };
  }

  /**
   * 根据意图选择最佳Skill
   */
  private async selectBestSkill(
    skills: Array<{ name: string; description: string; path: string }>,
    intent: string,
    preferredSkills: string[]
  ): Promise<{ name: string; description: string; path: string } | null> {
    const intentLower = intent.toLowerCase();
    const keywords = intentLower.split(/\s+/).filter(k => k.length > 2);

    // 评分函数
    const scoreSkill = (skill: { name: string; description: string }): number => {
      let score = 0;
      const nameLower = skill.name.toLowerCase();
      const descLower = skill.description.toLowerCase();

      // 1. 首选Skills优先
      if (preferredSkills.includes(skill.name)) {
        score += 50;
      }

      // 2. 关键词匹配
      for (const keyword of keywords) {
        if (nameLower.includes(keyword)) {
          score += 10;
        }
        if (descLower.includes(keyword)) {
          score += 5;
        }
      }

      return score;
    };

    // 找出最高分的Skill
    let bestSkill = skills[0];
    let maxScore = 0;

    for (const skill of skills) {
      const score = scoreSkill(skill);
      if (score > maxScore) {
        maxScore = score;
        bestSkill = skill;
      }
    }

    return maxScore > 0 ? bestSkill : null;
  }

  /**
   * 执行Skill链（多个Skills顺序执行）
   */
  async executeSkillChain(
    skills: Array<{
      name: string;
      inputs?: Record<string, unknown>;
    }>
  ): Promise<ClaudeSkillResult[]> {
    console.log(`🔗 执行Skill链 (${skills.length} 个Skills)`);
    
    const results: ClaudeSkillResult[] = [];

    for (const skill of skills) {
      console.log(`⚙️ 执行Skill: ${skill.name}`);
      
      const startTime = Date.now();
      const result = await agentSkillsService.executeSkill(skill.name, skill.inputs);
      const executionTime = Date.now() - startTime;

      results.push({
        success: result.success,
        skill_name: skill.name,
        output: result.output,
        execution_time_ms: executionTime,
        error: result.error || undefined,
        logs: result.logs,
      });

      // 如果执行失败，停止链式执行
      if (!result.success) {
        console.log(`❌ Skill执行失败: ${skill.name}`);
        break;
      }
    }

    return results;
  }

  // ============ 工具执行 ============

  /**
   * 执行计划
   */
  async executePlan(plan: ExecutionPlan, autoConfirm: boolean = false): Promise<ExecutionResult[]> {
    try {
      const planJson = {
        success: plan.success,
        primary_intent: plan.primary_intent,
        confidence: plan.confidence,
        plan: plan.plan.map(step => ({
          step: step.step,
          tool: step.tool,
          action: step.action,
          confidence: step.confidence,
          params: step.params,
        })),
        total_tools: plan.total_tools,
        message: plan.message,
      };

      const result = await invoke<{
        success: boolean;
        total_steps: number;
        executed: number;
        results: ExecutionResult[];
        message: string;
      }>('execute_plan', { plan: planJson, autoConfirm });

      return result.results;
    } catch (error) {
      console.error('执行计划失败:', error);
      return [];
    }
  }

  /**
   * 执行单个工具
   */
  async executeTool(toolId: string, params: Record<string, unknown>): Promise<ExecutionResult | null> {
    try {
      const result = await invoke<ExecutionResult>('execute_tool', {
        toolId,
        params,
      });
      return result;
    } catch (error) {
      console.error('执行工具失败:', toolId, error);
      return null;
    }
  }

  /**
   * 获取执行历史
   */
  async getExecutionHistory(): Promise<ExecutionResult[]> {
    try {
      return await invoke<ExecutionResult[]>('get_execution_history');
    } catch (error) {
      console.error('获取执行历史失败:', error);
      return [];
    }
  }

  /**
   * 清除执行历史
   */
  async clearExecutionHistory(): Promise<boolean> {
    try {
      return await invoke<boolean>('clear_execution_history');
    } catch (error) {
      console.error('清除执行历史失败:', error);
      return false;
    }
  }

  // ============ 配置管理 ============

  /**
   * 更新执行器配置
   */
  async updateExecutorConfig(config: Partial<ExecutorConfig>): Promise<boolean> {
    try {
      const result = await invoke<{
        success: boolean;
        config: ExecutorConfig;
        message: string;
      }>('update_executor_config', config);
      return result.success;
    } catch (error) {
      console.error('更新执行器配置失败:', error);
      return false;
    }
  }

  /**
   * 获取执行器配置
   */
  async getExecutorConfig(): Promise<ExecutorConfig | null> {
    try {
      const result = await invoke<{
        success: boolean;
        config: ExecutorConfig;
      }>('get_executor_config');
      return result.success ? result.config : null;
    } catch (error) {
      console.error('获取执行器配置失败:', error);
      return null;
    }
  }

  /**
   * 更新响应配置
   */
  async updateRespondConfig(config: Partial<RespondConfig>): Promise<boolean> {
    try {
      const result = await invoke<{
        success: boolean;
        message: string;
      }>('update_respond_config', config);
      return result.success;
    } catch (error) {
      console.error('更新响应配置失败:', error);
      return false;
    }
  }

  /**
   * 获取响应配置
   */
  async getRespondConfig(): Promise<RespondConfig | null> {
    try {
      const result = await invoke<{
        success: boolean;
        config: RespondConfig;
      }>('get_respond_config');
      return result.success ? result.config : null;
    } catch (error) {
      console.error('获取响应配置失败:', error);
      return null;
    }
  }

  // ============ 群聊自动响应 ============

  /**
   * 处理群聊消息
   */
  async handleChatMessage(message: ChatMessage): Promise<{
    responded: boolean;
    response: string | null;
  }> {
    try {
      const result = await invoke<{
        responded: boolean;
        response: string | null;
      }>('handle_chat_message', {
        id: message.id,
        topic: message.topic,
        content: message.content,
        sender: message.sender,
        timestamp: message.timestamp,
        messageType: message.message_type,
      });
      return result;
    } catch (error) {
      console.error('处理群聊消息失败:', error);
      return { responded: false, response: null };
    }
  }

  // ============ Claude Skills 完整工作流 ============

  /**
   * 端到端Claude Skills工作流
   * 
   * 这是与Claude官方SDK一致的核心功能：
   * 1. 用户输入需求
   * 2. AI分析意图
   * 3. 选择最佳Skill
   * 4. 执行Skill
   * 5. 返回结果
   */
  async runClaudeSkillsWorkflow(
    userInput: string,
    preferredSkills: string[] = []
  ): Promise<{
    success: boolean;
    skill_name: string;
    output: Record<string, unknown>;
    execution_time_ms: number;
    message: string;
  }> {
    console.log('\n' + '='.repeat(50));
    console.log('🚀 Claude Skills 工作流开始');
    console.log('='.repeat(50));
    console.log('📝 用户输入:', userInput);
    console.log('='.repeat(50));

    // 执行Claude Skill
    const result = await this.executeClaudeSkill(userInput, preferredSkills);

    console.log('='.repeat(50));
    console.log('📊 工作流结果');
    console.log('='.repeat(50));
    console.log(`✅ 成功: ${result.success}`);
    console.log(`🎯 Skill: ${result.skill_name}`);
    console.log(`⏱️ 时间: ${result.execution_time_ms}ms`);
    if (result.error) {
      console.log(`❌ 错误: ${result.error}`);
    }
    console.log('='.repeat(50));

    return {
      success: result.success,
      skill_name: result.skill_name,
      output: result.output,
      execution_time_ms: result.execution_time_ms,
      message: result.success 
        ? `Claude Skill "${result.skill_name}" 执行成功`
        : `执行失败: ${result.error}`,
    };
  }

  /**
   * 混合工作流：同时使用Skills和工具
   */
  async runHybridWorkflow(
    userInput: string
  ): Promise<{
    success: boolean;
    claude_skill_result: ClaudeSkillResult | null;
    tool_results: ExecutionResult[];
    message: string;
  }> {
    console.log('\n🔀 混合工作流开始');
    console.log('📝 用户输入:', userInput);

    // 1. 分析用户需求
    const analysis = await this.analyzeRequirement(userInput);
    console.log('🧠 分析结果:', analysis.primary_intent);
    console.log('🛠️ 推荐工具:', analysis.tools.map(t => t.tool_id).join(', '));

    // 2. 如果有匹配的Skill，先执行Skill
    let skillResult: ClaudeSkillResult | null = null;
    if (analysis.primary_intent) {
      skillResult = await this.executeClaudeSkill(analysis.primary_intent);
    }

    // 3. 如果没有Skill或Skill执行完成，执行工具
    const toolResults = await this.executePlan({
      success: analysis.success,
      primary_intent: analysis.primary_intent || '',
      confidence: analysis.tools[0]?.confidence || 0,
      plan: analysis.tools.map((tool, idx) => ({
        step: idx + 1,
        tool: tool.tool_id,
        action: tool.reason,
        confidence: tool.confidence,
        params: {},
      })),
      total_tools: analysis.tools.length,
      message: analysis.message,
    });

    // 4. 生成总结
    const skillSuccess = skillResult?.success || false;
    const toolSuccess = toolResults.length > 0 && toolResults.every(r => r.success);

    return {
      success: skillSuccess || toolSuccess,
      claude_skill_result: skillResult,
      tool_results: toolResults,
      message: `工作流完成: Skill ${skillSuccess ? '✅' : '❌'}, 工具 ${toolSuccess ? '✅' : '⚠️'}`,
    };
  }
}

export default AutonomousAgentService.getInstance();
