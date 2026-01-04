/**
 * Skills Service - 技能管理服务
 * 管理所有可用的Skills，提供注册、发现和执行功能
 */
import { WorkflowSkill } from '../skills/WorkflowSkill.js';

export class SkillsService {
  constructor() {
    this.skills = new Map();
    this.skillRegistry = new Map();
    this.initializeSkills();
  }

  /**
   * 初始化内置技能
   */
  initializeSkills() {
    // 注册工作流管理技能
    this.registerSkill(new WorkflowSkill());
    
    console.log('[SkillsService] 技能初始化完成，已注册技能:', Array.from(this.skills.keys()));
  }

  /**
   * 注册技能
   * @param {Object} skill - 技能实例
   */
  registerSkill(skill) {
    if (!skill || !skill.name) {
      console.error('[SkillsService] 无效的技能:', skill);
      return false;
    }

    this.skills.set(skill.name, skill);
    this.skillRegistry.set(skill.name, {
      name: skill.name,
      description: skill.description,
      version: skill.version,
      category: skill.category,
      actions: skill.getDefinition().actions
    });

    console.log(`[SkillsService] 注册技能: ${skill.name} (${skill.description})`);
    return true;
  }

  /**
   * 获取所有已注册的技能
   */
  getAllSkills() {
    return Array.from(this.skillRegistry.values());
  }

  /**
   * 获取技能定义
   * @param {string} skillName - 技能名称
   */
  getSkillDefinition(skillName) {
    const skill = this.skills.get(skillName);
    if (!skill) {
      return null;
    }
    return skill.getDefinition();
  }

  /**
   * 执行技能
   * @param {string} skillName - 技能名称
   * @param {string} action - 操作名称
   * @param {Object} parameters - 操作参数
   */
  async executeSkill(skillName, action, parameters) {
    const skill = this.skills.get(skillName);
    if (!skill) {
      throw new Error(`未找到技能: ${skillName}`);
    }

    // 验证参数
    const validation = skill.validateParameters(action, parameters);
    if (!validation.valid) {
      throw new Error(`参数验证失败: ${validation.error}`);
    }

    try {
      console.log(`[SkillsService] 执行技能: ${skillName}.${action}`, parameters);
      
      const result = await skill.execute(action, parameters);
      
      console.log(`[SkillsService] 技能执行成功: ${skillName}.${action}`, result);
      return {
        success: true,
        skill: skillName,
        action,
        result,
        timestamp: new Date().toISOString()
      };
    } catch (error) {
      console.error(`[SkillsService] 技能执行失败: ${skillName}.${action}`, error);
      return {
        success: false,
        skill: skillName,
        action,
        error: error.message,
        timestamp: new Date().toISOString()
      };
    }
  }

  /**
   * 批量执行技能
   * @param {Array} skillCalls - 技能调用数组
   */
  async executeSkillsBatch(skillCalls) {
    const results = [];
    
    for (const skillCall of skillCalls) {
      const { skill_name, action, parameters } = skillCall;
      
      try {
        const result = await this.executeSkill(skill_name, action, parameters);
        results.push(result);
      } catch (error) {
        results.push({
          success: false,
          skill: skill_name,
          action,
          error: error.message,
          timestamp: new Date().toISOString()
        });
      }
    }
    
    return results;
  }

  /**
   * 发现可用技能
   * 可以扩展为从文件系统或网络发现技能
   */
  async discoverSkills() {
    // 这里可以添加从文件系统扫描技能的功能
    // 例如：扫描 .claude/skills/ 目录
    
    const discoveredSkills = [];
    
    // 示例：从本地目录发现技能
    // try {
    //   const skillFiles = await this.scanSkillDirectory();
    //   for (const file of skillFiles) {
    //     const skillModule = await import(file);
    //     const skill = new skillModule.default();
    //     this.registerSkill(skill);
    //     discoveredSkills.push(skill.name);
    //   }
    // } catch (error) {
    //   console.error('[SkillsService] 发现技能失败:', error);
    // }
    
    return {
      success: true,
      discovered: discoveredSkills,
      total: this.skills.size,
      skills: this.getAllSkills()
    };
  }

  /**
   * 创建技能调用示例
   */
  createSkillExamples() {
    const examples = [];
    
    // 工作流技能示例
    const workflowSkill = this.skills.get('workflow_manager');
    if (workflowSkill) {
      examples.push({
        skill: 'workflow_manager',
        action: 'create_workflow',
        description: '创建代码审查工作流',
        parameters: {
          name: '自动代码审查',
          description: '自动审查代码质量、安全性和性能',
          steps: [
            {
              id: 'analyze_code',
              name: '分析代码结构',
              tool: 'claude_analysis',
              args: { task: '分析代码结构和质量' }
            },
            {
              id: 'check_security',
              name: '安全检查',
              tool: 'claude_security',
              args: { task: '检查安全漏洞' },
              depends_on: ['analyze_code']
            }
          ]
        }
      });

      examples.push({
        skill: 'workflow_manager',
        action: 'execute_workflow',
        description: '执行工作流',
        parameters: {
          workflow_id: 'wf_123',
          api_key: 'your-claude-api-key'
        }
      });
    }
    
    return examples;
  }

  /**
   * 获取技能统计信息
   */
  getStats() {
    const skills = Array.from(this.skills.values());
    
    const categories = {};
    skills.forEach(skill => {
      const category = skill.category || 'uncategorized';
      categories[category] = (categories[category] || 0) + 1;
    });
    
    return {
      total: this.skills.size,
      categories,
      last_updated: new Date().toISOString()
    };
  }

  /**
   * 验证技能调用
   */
  validateSkillCall(skillCall) {
    const { skill_name, action, parameters } = skillCall;
    
    if (!skill_name || !action) {
      return { valid: false, error: '缺少必需字段: skill_name 或 action' };
    }
    
    const skill = this.skills.get(skill_name);
    if (!skill) {
      return { valid: false, error: `未找到技能: ${skill_name}` };
    }
    
    const definition = skill.getDefinition();
    if (!definition.actions[action]) {
      return { valid: false, error: `技能 ${skill_name} 不支持操作: ${action}` };
    }
    
    return { valid: true };
  }
}

// 创建单例实例
const skillsService = new SkillsService();

export default skillsService;
