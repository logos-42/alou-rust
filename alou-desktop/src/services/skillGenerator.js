/**
 * SkillGenerator - AI 技能自动生成服务
 * 基于用户描述和智能体上下文自动生成技能定义
 */

class SkillGenerator {
  constructor() {
    this.apiEndpoint = '/api/agent/chat'; // 使用现有的聊天 API
    this.systemPrompt = `你是一个专业的技能定义专家。你的任务是根据用户的需求和描述，生成标准化的技能定义。

技能定义必须包含以下结构：
{
  "name": "技能名称（英文，下划线分隔）",
  "description": "技能描述（中文）",
  "version": "0.1.0",
  "category": "类别（automation/development/research/web3/other）",
  "actions": {
    "action_name": {
      "description": "操作描述",
      "parameters": {
        "type": "object",
        "properties": {
          "param_name": {
            "type": "string|number|boolean|array|object",
            "description": "参数描述"
          }
        },
        "required": ["必需参数列表"]
      }
    }
  }
}

请根据用户输入生成符合上述结构的技能定义，只返回 JSON 格式，不要包含其他文字。`;
  }

  /**
   * 生成技能定义
   * @param {string} description - 用户描述
   * @param {Object} agentInfo - 智能体信息
   * @param {Object} context - 上下文信息
   * @returns {Promise<Object>} 生成的技能定义
   */
  async generateSkill(description, agentInfo = {}, context = {}) {
    try {
      const prompt = this.buildPrompt(description, agentInfo, context);
      
      const response = await fetch(this.apiEndpoint, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          messages: [
            {
              role: 'system',
              content: this.systemPrompt
            },
            {
              role: 'user',
              content: prompt
            }
          ],
          temperature: 0.7,
          max_tokens: 2000
        })
      });

      if (!response.ok) {
        throw new Error(`API 请求失败: ${response.status}`);
      }

      const result = await response.json();
      const skillDefinition = this.parseSkillDefinition(result.choices?.[0]?.message?.content);
      
      return {
        success: true,
        skill: skillDefinition,
        rawResponse: result
      };
    } catch (error) {
      console.error('[SkillGenerator] 生成技能失败:', error);
      return {
        success: false,
        error: error.message,
        suggestion: '请检查网络连接或稍后重试'
      };
    }
  }

  /**
   * 构建提示词
   * @param {string} description - 用户描述
   * @param {Object} agentInfo - 智能体信息
   * @param {Object} context - 上下文信息
   * @returns {string} 完整提示词
   */
  buildPrompt(description, agentInfo, context) {
    let prompt = `请根据以下描述生成一个技能定义：\n\n用户描述：${description}\n\n`;
    
    if (agentInfo && Object.keys(agentInfo).length > 0) {
      prompt += `智能体信息：\n`;
      prompt += `- 名称：${agentInfo.name || '未命名'}\n`;
      prompt += `- 类型：${agentInfo.type || '通用'}\n`;
      prompt += `- 描述：${agentInfo.description || '无描述'}\n\n`;
    }
    
    if (context && Object.keys(context).length > 0) {
      prompt += `上下文信息：\n`;
      Object.entries(context).forEach(([key, value]) => {
        prompt += `- ${key}：${value}\n`;
      });
      prompt += '\n';
    }
    
    prompt += `请生成一个实用、完整的技能定义，确保：
1. 技能名称简洁明了，使用英文和下划线
2. 描述准确说明功能
3. 包含至少一个可执行的操作
4. 参数定义完整且合理
5. 选择合适的类别`;

    return prompt;
  }

  /**
   * 解析技能定义
   * @param {string} content - AI 返回的内容
   * @returns {Object} 解析后的技能定义
   */
  parseSkillDefinition(content) {
    try {
      // 尝试直接解析 JSON
      if (typeof content === 'string') {
        // 清理可能的 markdown 代码块标记
        const cleanContent = content.replace(/```json\s*|\s*```/g, '').trim();
        return JSON.parse(cleanContent);
      }
      return content;
    } catch (error) {
      console.error('[SkillGenerator] 解析技能定义失败:', error);
      
      // 尝试从文本中提取 JSON
      const jsonMatch = content.match(/\{[\s\S]*\}/);
      if (jsonMatch) {
        try {
          return JSON.parse(jsonMatch[0]);
        } catch (e) {
          console.error('[SkillGenerator] 提取 JSON 失败:', e);
        }
      }
      
      // 返回默认技能结构
      return this.getDefaultSkill();
    }
  }

  /**
   * 获取默认技能结构
   * @returns {Object} 默认技能定义
   */
  getDefaultSkill() {
    return {
      name: 'custom_skill',
      description: '自定义技能',
      version: '1.0.0',
      category: 'other',
      actions: {
        execute: {
          description: '执行自定义操作',
          parameters: {
            type: 'object',
            properties: {
              input: {
                type: 'string',
                description: '输入参数'
              }
            },
            required: ['input']
          }
        }
      }
    };
  }

  /**
   * 验证技能定义
   * @param {Object} skillDefinition - 技能定义
   * @returns {Object} 验证结果
   */
  validateSkillDefinition(skillDefinition) {
    const errors = [];
    const warnings = [];

    // 检查必需字段
    const requiredFields = ['name', 'description', 'version', 'category', 'actions'];
    requiredFields.forEach(field => {
      if (!skillDefinition[field]) {
        errors.push(`缺少必需字段: ${field}`);
      }
    });

    // 检查名称格式
    if (skillDefinition.name && !/^[a-z][a-z0-9_]*$/.test(skillDefinition.name)) {
      warnings.push('技能名称应使用小写字母、数字和下划线，并以字母开头');
    }

    // 检查版本格式
    if (skillDefinition.version && !/^\d+\.\d+\.\d+$/.test(skillDefinition.version)) {
      warnings.push('版本号应使用语义化版本格式 (如: 1.0.0)');
    }

    // 检查类别
    const validCategories = ['automation', 'development', 'research', 'web3', 'other'];
    if (skillDefinition.category && !validCategories.includes(skillDefinition.category)) {
      warnings.push(`类别应为以下之一: ${validCategories.join(', ')}`);
    }

    // 检查操作定义
    if (skillDefinition.actions) {
      Object.entries(skillDefinition.actions).forEach(([actionName, actionDef]) => {
        if (!actionDef.description) {
          warnings.push(`操作 ${actionName} 缺少描述`);
        }
        
        if (actionDef.parameters) {
          if (!actionDef.parameters.type || actionDef.parameters.type !== 'object') {
            errors.push(`操作 ${actionName} 的参数定义必须为 object 类型`);
          } else if (!actionDef.parameters.properties) {
            warnings.push(`操作 ${actionName} 的参数定义缺少 properties`);
          }
        }
      });
    }

    return {
      isValid: errors.length === 0,
      errors,
      warnings
    };
  }

  /**
   * 生成技能建议
   * @param {Object} agentInfo - 智能体信息
   * @param {Array} existingSkills - 现有技能列表
   * @returns {Promise<Array>} 技能建议列表
   */
  async generateSkillSuggestions(agentInfo, existingSkills = []) {
    try {
      const prompt = `基于以下智能体信息和现有技能，请生成 5 个技能改进建议：

智能体信息：
${JSON.stringify(agentInfo, null, 2)}

现有技能：
${existingSkills.map(skill => `- ${skill.name}: ${skill.description}`).join('\n')}

请为每个建议提供：
1. 技能名称（英文，下划线分隔）
2. 技能描述（中文）
3. 主要功能
4. 与现有技能的关系

请以 JSON 数组格式返回，格式如下：
[
  {
    "name": "skill_name",
    "description": "技能描述",
    "purpose": "主要功能",
    "relationship": "与现有技能的关系"
  }
]`;

      const response = await fetch(this.apiEndpoint, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          messages: [
            {
              role: 'system',
              content: '你是一个技能建议专家，请根据智能体信息提供实用的技能改进建议。'
            },
            {
              role: 'user',
              content: prompt
            }
          ],
          temperature: 0.8,
          max_tokens: 1500
        })
      });

      if (!response.ok) {
        throw new Error(`API 请求失败: ${response.status}`);
      }

      const result = await response.json();
      const content = result.choices?.[0]?.message?.content;
      
      try {
        const cleanContent = content.replace(/```json\s*|\s*```/g, '').trim();
        return JSON.parse(cleanContent);
      } catch (error) {
        console.error('[SkillGenerator] 解析建议失败:', error);
        return [];
      }
    } catch (error) {
      console.error('[SkillGenerator] 生成技能建议失败:', error);
      return [];
    }
  }

  /**
   * 从示例学习生成技能
   * @param {Array} examples - 示例列表
   * @param {string} targetDescription - 目标描述
   * @returns {Promise<Object>} 生成的技能定义
   */
  async learnFromExamples(examples, targetDescription) {
    try {
      const prompt = `基于以下示例，请生成一个新的技能定义来实现目标功能：

示例：
${examples.map((example, index) => 
  `示例${index + 1}：
  输入：${example.input}
  输出：${example.output}
  描述：${example.description}`
).join('\n\n')}

目标功能：${targetDescription}

请分析示例的模式，生成一个能够实现目标功能的完整技能定义。`;

      return await this.generateSkill(prompt);
    } catch (error) {
      console.error('[SkillGenerator] 从示例学习失败:', error);
      return {
        success: false,
        error: error.message
      };
    }
  }

  /**
   * 优化现有技能
   * @param {Object} currentSkill - 当前技能定义
   * @param {string} improvementGoal - 改进目标
   * @returns {Promise<Object>} 优化后的技能定义
   */
  async optimizeSkill(currentSkill, improvementGoal) {
    try {
      const prompt = `请优化以下技能定义以实现改进目标：

当前技能：
${JSON.stringify(currentSkill, null, 2)}

改进目标：${improvementGoal}

请保持技能的核心功能，但根据改进目标进行优化，确保：
1. 提高技能的实用性
2. 改进参数设计
3. 增强错误处理
4. 优化用户体验`;

      const response = await fetch(this.apiEndpoint, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          messages: [
            {
              role: 'system',
              content: '你是一个技能优化专家，请根据改进目标优化现有技能定义。'
            },
            {
              role: 'user',
              content: prompt
            }
          ],
          temperature: 0.6,
          max_tokens: 2000
        })
      });

      if (!response.ok) {
        throw new Error(`API 请求失败: ${response.status}`);
      }

      const result = await response.json();
      const optimizedSkill = this.parseSkillDefinition(result.choices?.[0]?.message?.content);
      
      return {
        success: true,
        skill: optimizedSkill,
        improvements: this.identifyImprovements(currentSkill, optimizedSkill)
      };
    } catch (error) {
      console.error('[SkillGenerator] 优化技能失败:', error);
      return {
        success: false,
        error: error.message
      };
    }
  }

  /**
   * 识别改进点
   * @param {Object} original - 原始技能
   * @param {Object} optimized - 优化后技能
   * @returns {Array} 改进点列表
   */
  identifyImprovements(original, optimized) {
    const improvements = [];
    
    // 比较字段变化
    Object.keys(optimized).forEach(key => {
      if (JSON.stringify(original[key]) !== JSON.stringify(optimized[key])) {
        improvements.push({
          field: key,
          original: original[key],
          optimized: optimized[key],
          type: 'modified'
        });
      }
    });
    
    return improvements;
  }
}

// 创建单例实例
const skillGenerator = new SkillGenerator();

export default skillGenerator;
