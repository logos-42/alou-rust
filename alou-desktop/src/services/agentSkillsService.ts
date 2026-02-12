/**
 * Claude Agent Skills 服务
 * 
 * 与Claude官方Agent SDK Skills格式一致的技能系统
 * 支持 SKILL.md 格式、渐进式披露和自动执行
 */

import { invoke } from '@tauri-apps/api/core';

// ============ Skills 类型定义 ============

/**
 * Agent Skill 定义（标准 Claude SDK 格式）
 */
export interface AgentSkill {
  name: string;
  description: string;
  version?: string;
  license?: string;
  allowed_tools?: string[];
  metadata?: Record<string, unknown>;
  path: string;
  instructions?: string;
  loaded: boolean;
}

/**
 * Skill 元数据（发现阶段使用）
 */
export interface SkillMetadata {
  name: string;
  description: string;
  path: string;
}

/**
 * Skill 执行上下文
 */
export interface SkillExecutionContext {
  session_id: string;
  skill_name: string;
  execution_id: string;
  inputs: Record<string, unknown>;
  environment: Record<string, string>;
  working_dir: string;
  debug_mode: boolean;
}

/**
 * Skill 执行结果
 */
export interface SkillExecutionResult {
  success: boolean;
  output: Record<string, unknown>;
  execution_time_ms: number;
  error?: string;
  logs: string[];
}

// ============ Claude SDK Skills 服务 ============

class AgentSkillsService {
  private static instance: AgentSkillsService;

  private constructor() {}

  static getInstance(): AgentSkillsService {
    if (!AgentSkillsService.instance) {
      AgentSkillsService.instance = new AgentSkillsService();
    }
    return AgentSkillsService.instance;
  }

  // ============ Skill 发现 ============

  /**
   * 发现可用技能
   * Progressive Disclosure Level 1: 仅返回元数据
   */
  async discoverSkills(): Promise<SkillMetadata[]> {
    try {
      const result = await invoke<{
        success: boolean;
        skills: SkillMetadata[];
        message: string;
      }>('agent_skills_discover');

      if (result.success) {
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
   * Progressive Disclosure Level 2: 返回完整内容
   */
  async loadSkill(skillName: string): Promise<AgentSkill | null> {
    try {
      const result = await invoke<{
        success: boolean;
        skill: AgentSkill | null;
        message: string;
      }>('agent_skills_load', { skillName });

      if (result.success && result.skill) {
        return result.skill;
      }
      return null;
    } catch (error) {
      console.error('加载技能失败:', skillName, error);
      return null;
    }
  }

  // ============ Skill 执行 ============

  /**
   * 执行技能
   * 
   * @param skillName 技能名称
   * @param inputs 输入参数
   * @param debugMode 调试模式
   */
  async executeSkill(
    skillName: string,
    inputs: Record<string, unknown> = {},
    debugMode: boolean = false
  ): Promise<SkillExecutionResult> {
    try {
      const executionId = `exec_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
      
      const result = await invoke<SkillExecutionResult>('agent_skills_execute', {
        skillName,
        inputs,
        executionId,
        debugMode,
      });

      return result;
    } catch (error) {
      console.error('执行技能失败:', skillName, error);
      return {
        success: false,
        output: {},
        execution_time_ms: 0,
        error: String(error),
        logs: [],
      };
    }
  }

  // ============ Skill 管理 ============

  /**
   * 创建新技能
   */
  async createSkill(
    name: string,
    description: string,
    instructions: string,
    allowedTools: string[] = []
  ): Promise<boolean> {
    try {
      const result = await invoke<{
        success: boolean;
        skill: AgentSkill | null;
        message: string;
      }>('agent_skills_create', {
        name,
        description,
        instructions,
        allowedTools,
      });

      return result.success;
    } catch (error) {
      console.error('创建技能失败:', error);
      return false;
    }
  }

  /**
   * 删除技能
   */
  async deleteSkill(skillName: string): Promise<boolean> {
    try {
      const result = await invoke<{
        success: boolean;
        message: string;
      }>('agent_skills_delete', { skillName });

      return result.success;
    } catch (error) {
      console.error('删除技能失败:', skillName, error);
      return false;
    }
  }

  /**
   * 列出所有技能
   */
  async listSkills(): Promise<AgentSkill[]> {
    try {
      const result = await invoke<{
        success: boolean;
        skills: AgentSkill[];
        message: string;
      }>('agent_skills_list');

      if (result.success) {
        return result.skills;
      }
      return [];
    } catch (error) {
      console.error('列出技能失败:', error);
      return [];
    }
  }

  // ============ AI 集成 ============

  /**
   * AI自动选择并执行技能
   * 
   * 这是与 Skills 自动选择器 的集成点
   */
  async autoExecute(
    userIntent: string,
    preferredTools: string[] = []
  ): Promise<SkillExecutionResult> {
    // 1. 发现可用技能
    const availableSkills = await this.discoverSkills();
    
    if (availableSkills.length === 0) {
      return {
        success: false,
        output: {},
        execution_time_ms: 0,
        error: '没有可用的技能',
        logs: ['未发现任何技能，请先创建技能'],
      };
    }

    // 2. 根据用户意图选择最佳技能
    const selectedSkill = this.selectBestSkill(availableSkills, userIntent, preferredTools);

    if (!selectedSkill) {
      return {
        success: false,
        output: {},
        execution_time_ms: 0,
        error: '没有找到匹配的技能',
        logs: [`可用技能: ${availableSkills.map(s => s.name).join(', ')}`],
      };
    }

    // 3. 执行选中的技能
    console.log(`🎯 选择技能: ${selectedSkill.name}`);
    return await this.executeSkill(selectedSkill.name);
  }

  /**
   * 根据意图选择最佳技能
   */
  private selectBestSkill(
    skills: SkillMetadata[],
    intent: string,
    preferredTools: string[]
  ): SkillMetadata | null {
    const intentLower = intent.toLowerCase();
    const keywords = intentLower.split(/\s+/);

    // 评分函数
    const scoreSkill = (skill: SkillMetadata): number => {
      let score = 0;
      const descLower = skill.description.toLowerCase();

      // 1. 关键词匹配
      for (const keyword of keywords) {
        if (keyword.length > 2) {
          if (skill.name.toLowerCase().includes(keyword)) {
            score += 10;
          }
          if (descLower.includes(keyword)) {
            score += 5;
          }
        }
      }

      // 2. 首选工具优先
      if (preferredTools.length > 0) {
        // 尝试加载技能完整信息来检查 allowed_tools
        // 这里简化处理，假设技能名包含工具名
        for (const tool of preferredTools) {
          if (skill.name.toLowerCase().includes(tool.toLowerCase())) {
            score += 20;
          }
        }
      }

      return score;
    };

    // 找出最高分的技能
    let bestSkill: SkillMetadata | null = null;
    let maxScore = 0;

    for (const skill of skills) {
      const score = scoreSkill(skill);
      if (score > maxScore) {
        maxScore = score;
        bestSkill = skill;
      }
    }

    return maxScore > 0 ? bestSkill : (skills.length > 0 ? skills[0] : null);
  }

  // ============ 批量操作 ============

  /**
   * 执行多个技能（工具链）
   */
  async executeSkillChain(
    skills: Array<{
      name: string;
      inputs?: Record<string, unknown>;
    }>
  ): Promise<SkillExecutionResult[]> {
    const results: SkillExecutionResult[] = [];

    for (const skill of skills) {
      const result = await this.executeSkill(skill.name, skill.inputs);
      results.push(result);

      // 如果执行失败，停止链式执行
      if (!result.success && !result.output?.continue_on_error) {
        break;
      }
    }

    return results;
  }

  /**
   * 并行执行多个技能
   */
  async executeSkillsParallel(
    skills: Array<{
      name: string;
      inputs?: Record<string, unknown>;
    }>
  ): Promise<SkillExecutionResult[]> {
    const promises = skills.map(skill => 
      this.executeSkill(skill.name, skill.inputs)
    );

    return Promise.all(promises);
  }
}

export default AgentSkillsService.getInstance();

// ============ 示例 Skills ============

/**
 * 示例: README.md 技能
 * 路径: ~/.alou/skills/readme/SKILL.md
 */
export const README_SKILL = `
# name
readme

# description
读取并分析 README 文件，提供项目概述

# version
1.0.0

# license
MIT

# allowed_tools
- filesystem
- search

# parameters
{
  "type": "object",
  "properties": {
    "path": {
      "type": "string",
      "description": "README 文件路径，默认项目根目录"
    }
  },
  "required": ["path"]
}

# instructions
## 读取 README 文件

1. 首先读取指定路径的 README 文件
2. 解析文件内容，提取关键信息：
   - 项目名称和描述
   - 功能特性
   - 使用方法
   - 依赖要求
3. 生成简洁的摘要

## 输出格式

\`\`\`json
{
  "project_name": "项目名称",
  "description": "项目描述",
  "features": ["特性1", "特性2"],
  "usage": "使用方法",
  "requirements": ["要求1", "要求2"]
}
\`\`\`

## 注意事项

- 支持 Markdown 和纯文本格式
- 如果文件不存在，返回错误信息
- 如果文件过大，只读取前 100 行
`;

/**
 * 示例: 代码分析技能
 */
export const ANALYZE_CODE_SKILL = `
# name
analyze_code

# description
分析项目代码结构，生成分析报告

# version
1.0.0

# license
MIT

# allowed_tools
- filesystem
- search
- bash

# parameters
{
  "type": "object",
  "properties": {
    "directory": {
      "type": "string",
      "description": "要分析的代码目录"
    },
    "extensions": {
      "type": "array",
      "description": "要分析的文件扩展名"
    }
  },
  "required": ["directory"]
}

# instructions
## 分析代码结构

1. 扫描指定目录下的所有代码文件
2. 统计文件数量、行数
3. 识别主要模块和依赖关系
4. 生成分析报告

## 输出格式

\`\`\`json
{
  "total_files": 100,
  "total_lines": 10000,
  "by_extension": {
    ".ts": 50,
    ".tsx": 30,
    ".js": 20
  },
  "modules": ["module1", "module2"],
  "dependencies": ["dep1", "dep2"]
}
\`\`\`
`;
