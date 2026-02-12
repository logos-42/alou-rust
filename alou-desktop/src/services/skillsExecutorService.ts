/**
 * Alou Skills 执行服务
 * 
 * 连接到自主循环，实现真正的 Skills 执行
 */

import { invoke } from '@tauri-apps/api/core';

// ============ 类型定义 ============

/**
 * 技能元数据
 */
export interface SkillMetadata {
  name: string;
  description: string;
  path: string;
}

/**
 * 技能完整信息
 */
export interface SkillDetail {
  name: string;
  description: string;
  instructions: string;
  allowed_tools: string[];
  version: string;
  license: string;
}

/**
 * 技能执行请求
 */
export interface SkillExecuteRequest {
  skill_name: string;
  inputs: Record<string, unknown>;
  session_id?: string;
  debug_mode?: boolean;
}

/**
 * 技能执行结果
 */
export interface SkillExecuteResult {
  success: boolean;
  output: Record<string, unknown>;
  execution_time_ms: number;
  error?: string;
  logs: string[];
}

/**
 * 任务匹配结果
 */
export interface TaskMatchResult {
  task: string;
  matched_skill: string;
  confidence: number;
  reasons: string[];
}

/**
 * 执行上下文
 */
export interface ExecutionContext {
  session_id: string;
  task: string;
  skill: string;
  inputs: Record<string, unknown>;
  start_time: number;
  logs: string[];
}

// ============ Skills 执行服务 ============

class SkillsExecutorService {
  private static instance: SkillsExecutorService;

  private constructor() {}

  static getInstance(): SkillsExecutorService {
    if (!SkillsExecutorService.instance) {
      SkillsExecutorService.instance = new SkillsExecutorService();
    }
    return SkillsExecutorService.instance;
  }

  // ============ 技能发现 ============

  /**
   * 发现可用技能
   */
  async discoverSkills(): Promise<SkillMetadata[]> {
    try {
      const result = await invoke<{
        success: boolean;
        skills: SkillMetadata[];
        message: string;
      }>('agent_skills', {
        skill_name: null,
      });

      if (result.success && result.skills) {
        return result.skills;
      }
      return [];
    } catch (error) {
      console.error('发现技能失败:', error);
      return [];
    }
  }

  /**
   * 获取技能详情
   */
  async getSkillDetail(skillName: string): Promise<SkillDetail | null> {
    try {
      const result = await invoke<{
        success: boolean;
        skill: SkillDetail;
        message: string;
      }>('agent_skills', {
        skill_name: skillName,
      });

      if (result.success && result.skill) {
        return result.skill;
      }
      return null;
    } catch (error) {
      console.error('获取技能详情失败:', error);
      return null;
    }
  }

  // ============ 任务匹配 ============

  /**
   * 分析任务，选择最佳技能
   */
  async matchTaskToSkill(task: string): Promise<TaskMatchResult> {
    const skills = await this.discoverSkills();
    
    if (skills.length === 0) {
      return {
        task,
        matched_skill: 'unknown',
        confidence: 0,
        reasons: ['没有发现可用技能'],
      };
    }

    // 简单的关键词匹配
    const taskLower = task.toLowerCase();
    const taskWords = taskLower.split(/\s+/).filter(w => w.length > 2);

    const scores = [];

    for (const skill of skills) {
      let score = 0;
      const reasons = [];
      const skillLower = skill.description.toLowerCase();

      // 关键词匹配
      for (const word of taskWords) {
        if (skillLower.includes(word) || skill.name.toLowerCase().includes(word)) {
          score += 10;
          reasons.push(`包含关键词"${word}"`);
        }
      }

      // 任务类型匹配
      const patterns = [
        { keywords: ['email', '邮件', '检查邮件'], skill: 'EmailChecker' },
        { keywords: ['workflow', '工作流', '流程', '任务'], skill: 'WorkflowSkill' },
        { keywords: ['search', '搜索', '查找', '资讯'], skill: 'WebSearch' },
        { keywords: ['code', '代码', '分析', '结构'], skill: 'CodeAnalyzer' },
        { keywords: ['git', '提交', '分支', 'commit'], skill: 'GitHelper' },
      ];

      for (const pattern of patterns) {
        if (pattern.keywords.some(kw => taskLower.includes(kw))) {
          if (skill.name.includes(pattern.skill)) {
            score += 30;
            reasons.push(`任务类型匹配`);
          }
        }
      }

      scores.push({
        skill: skill.name,
        score,
        reasons,
      });
    }

    // 排序
    scores.sort((a, b) => b.score - a.score);

    const best = scores[0];

    // 归一化置信度
    const confidence = Math.min(best.score / 50 * 100, 100);

    return {
      task,
      matched_skill: best.skill,
      confidence,
      reasons: best.reasons.length > 0 ? best.reasons : ['无明显匹配'],
    };
  }

  // ============ 技能执行 ============

  /**
   * 执行技能
   */
  async executeSkill(
    skillName: string,
    inputs: Record<string, unknown> = {}
  ): Promise<SkillExecuteResult> {
    const startTime = Date.now();
    const logs: string[] = [];

    logs.push(`[${new Date().toISOString()}] 开始执行技能: ${skillName}`);
    logs.push(`[输入] ${JSON.stringify(inputs)}`);

    try {
      // 调用后端执行
      const result = await invoke<{
        success: boolean;
        output: Record<string, unknown>;
        error?: string;
        logs: string[];
      }>('agent_skills', {
        skill_name: skillName,
        inputs,
        session_id: `autonomous_${Date.now()}`,
        debug_mode: false,
      });

      logs.push(`[${new Date().toISOString()}] 执行完成`);

      const executionTime = Date.now() - startTime;

      if (result.success) {
        logs.push(`[成功] 输出: ${JSON.stringify(result.output)}`);

        return {
          success: true,
          output: result.output || {},
          execution_time_ms: executionTime,
          logs,
        };
      } else {
        const errorMsg = result.error || '未知错误';
        logs.push(`[失败] ${errorMsg}`);

        return {
          success: false,
          output: {},
          execution_time_ms: executionTime,
          error: errorMsg,
          logs,
        };
      }
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      logs.push(`[异常] ${errorMsg}`);

      return {
        success: false,
        output: {},
        execution_time_ms: Date.now() - startTime,
        error: errorMsg,
        logs,
      };
    }
  }

  /**
   * 自主任务执行（发现→选择→执行）
   */
  async autonomousExecute(task: string): Promise<{
    matched_skill: string;
    confidence: number;
    result: SkillExecuteResult;
  }> {
    console.log(`\n🎯 自主执行任务: ${task}`);

    // Step 1: 匹配技能
    const match = await this.matchTaskToSkill(task);
    console.log(`  📝 匹配: ${match.matched_skill} (${match.confidence.toFixed(0)}%)`);

    // Step 2: 执行技能
    const result = await this.executeSkill(match.matched_skill, {
      task,
      timestamp: Date.now(),
    });

    // Step 3: 返回结果
    return {
      matched_skill: match.matched_skill,
      confidence: match.confidence,
      result,
    };
  }

  // ============ 学习功能 ============

  /**
   * 记录执行历史
   */
  async recordExecution(
    task: string,
    skill: string,
    result: SkillExecuteResult
  ): Promise<void> {
    const execution = {
      task,
      skill,
      success: result.success,
      execution_time_ms: result.execution_time_ms,
      timestamp: Date.now(),
    };

    // 保存到本地存储
    const history = await this.getExecutionHistory();
    history.push(execution);

    // 只保留最近 100 条
    if (history.length > 100) {
      history.shift();
    }

    localStorage.setItem('alou_execution_history', JSON.stringify(history));
  }

  /**
   * 获取执行历史
   */
  async getExecutionHistory(): Promise<any[]> {
    try {
      const data = localStorage.getItem('alou_execution_history');
      return data ? JSON.parse(data) : [];
    } catch {
      return [];
    }
  }

  /**
   * 计算技能统计
   */
  async getSkillStats(): Promise<Record<string, {
    count: number;
    success: number;
    avgTime: number;
  }>> {
    const history = await this.getExecutionHistory();
    const stats: Record<string, any> = {};

    for (const entry of history) {
      if (!stats[entry.skill]) {
        stats[entry.skill] = { count: 0, success: 0, totalTime: 0 };
      }

      stats[entry.skill].count++;
      if (entry.success) {
        stats[entry.skill].success++;
      }
      stats[entry.skill].totalTime += entry.execution_time_ms || 0;
    }

    // 计算平均值
    for (const skill of Object.keys(stats)) {
      const s = stats[skill];
      s.avgTime = s.count > 0 ? Math.round(s.totalTime / s.count) : 0;
    }

    return stats;
  }
}

export default SkillsExecutorService.getInstance();
