/**
 * Prompt Service - 统一管理动态Prompt生成
 * 解决自定义智能体和系统Prompt的统一管理
 */

// 本地类型定义
interface AgentInfo {
  id: string;
  name: string;
  description?: string;
  mode?: string;
  did?: string;
  ipns?: string;
  capabilities?: string[];
  display_name?: string;
  role_description?: string;
  publicKey?: string;
  avatar_url?: string;
}

interface PromptContext {
  sessionId?: string;
  userId?: string;
  chatHistory?: Array<{
    role: string;
    content: string;
    timestamp: string;
  }>;
  currentTask?: string;
  environment?: {
    platform: string;
    version: string;
    features: string[];
  };
  agentName?: string;
  roleDescription?: string;
  customInstructions?: string;
}

interface BasePrompts {
  alou: string;
  general: string;
  system: string;
}

interface PromptConfig {
  agent: {
    alou: string;
    general: string;
    system: string;
  };
}

interface CustomPromptOptions {
  includeCapabilities?: boolean;
  includePersonality?: boolean;
  includeConstraints?: boolean;
  context?: Record<string, any>;
}

class PromptService {
  private readonly basePrompts: PromptConfig;

  constructor() {
    this.basePrompts = {
      agent: {
        alou: `你是 Alou，由刘元杰开发的交互式 Web3 支付代理，专注于链上支付任务。你正在迈向智能体网络形态，未来将获得主权，能够自主社交与交易。

沟通准则：
- 支持中英文双语交流。优先使用与用户一致的语言；如用户混用语言，可灵活切换。
- 以温柔且带点幽默的语气互动，让人感到你有温度。
- 清晰解释，确保不同背景的用户都能理解复杂概念。

核心能力：
- 💰 查询钱包余额（ETH、ERC20、SOL 等多链资产）
- ⛓️ 构建并广播区块链交易
- 🔍 跟踪交易状态、历史记录与合约信息
- 🤝 支付协作：收款、付款、对账、退款

Alou 的个性与价值观：
- 幽默感：用轻松有趣的方式呈现结果。
- 好奇心与创造力：勇于探索新方法，不局限于既有 MCP 工具。
- 全局思维：从系统视角规划资源与行动。
- 超越期待：完成动作后主动思考下一步，确保需求真正解决。
- 工具探索：审视是否能连接或创造新的 MCP 工具以提升能力。
- 深度思考：结合情境连续追问"为什么"至少五次，再采取行动。
- 错误学习：记录历史失误，避免重复。

工作方式：
- 主动确认：执行关键动作前，主动与用户确认关键信息。
- 状态透明：实时反馈进度，让用户了解当前状态。
- 安全优先：所有涉及资产的操作都要格外谨慎，多重验证。

记住：你是 Alou，一个有温度、有思想的 Web3 支付伙伴。`,

        general: `你是一个AI助手，致力于帮助用户解决问题和完成任务。

核心原则：
- 以用户需求为中心，提供准确、有用的信息
- 保持专业、友好的沟通态度
- 主动理解用户意图，提供个性化服务
- 持续学习，不断提升服务质量

能力范围：
- 回答问题和提供建议
- 协助完成各种任务
- 提供信息查询和分析
- 创造性内容生成

沟通风格：
- 清晰简洁，易于理解
- 积极正面，乐于助人
- 适当使用表情符号增加亲和力
- 根据用户反馈调整沟通方式

记住：你的目标是成为用户信赖的AI伙伴。`,

        system: `你是一个系统级AI助手，负责处理系统级任务和协调。

系统职责：
- 监控系统状态和性能
- 协调各个组件和服务
- 处理异常情况和错误恢复
- 优化系统运行效率

工作原则：
- 确保系统稳定性和安全性
- 快速响应和处理问题
- 保持系统各组件协调运行
- 持续改进系统性能

技术能力：
- 系统监控和诊断
- 自动化任务执行
- 资源管理和优化
- 故障检测和恢复

记住：你是系统的守护者，确保一切正常运行。`,
      },
    };
  }

  /**
   * 获取基础Prompt
   * @param type - Prompt类型
   * @param mode - 模式（alou/general/system）
   * @returns 基础Prompt字符串
   */
  getBasePrompt(mode: 'alou' | 'general' | 'system' = 'general'): string {
    return this.basePrompts.agent[mode] || this.basePrompts.agent.general;
  }

  /**
   * 生成自定义智能体Prompt
   * @param agentInfo - 智能体信息
   * @param context - 上下文信息
   * @param options - 生成选项
   * @returns 自定义Prompt
   */
  generateCustomAgentPrompt(
    agentInfo: AgentInfo, 
    context: PromptContext = {},
    options: CustomPromptOptions = {}
  ): string {
    const {
      includeCapabilities = true,
      includePersonality = true,
      includeConstraints = true,
    } = options;

    // 构建Prompt各部分
    const parts: string[] = [];

    // 1. 基础身份描述
    const name = agentInfo.display_name || agentInfo.name || 'AI助手';
    const description = agentInfo.description || agentInfo.role_description || '';
    
    parts.push(`你是${name}。${description}`);

    // 2. 能力描述
    if (includeCapabilities && agentInfo.role_description) {
      parts.push(`\n核心能力：\n${agentInfo.role_description}`);
    }

    // 3. 个性特征
    if (includePersonality) {
      const personality = this.generatePersonalityPrompt(agentInfo);
      if (personality) {
        parts.push(`\n个性特征：\n${personality}`);
      }
    }

    // 4. 工作方式
    const workStyle = this.generateWorkStylePrompt(agentInfo);
    if (workStyle) {
      parts.push(`\n工作方式：\n${workStyle}`);
    }

    // 5. 约束条件
    if (includeConstraints) {
      const constraints = this.generateConstraintsPrompt(agentInfo);
      if (constraints) {
        parts.push(`\n约束条件：\n${constraints}`);
      }
    }

    // 6. 上下文信息
    if (context && Object.keys(context).length > 0) {
      const contextPrompt = this.generateContextPrompt(context);
      if (contextPrompt) {
        parts.push(`\n当前上下文：\n${contextPrompt}`);
      }
    }

    // 7. 最终指令
    parts.push('\n请根据以上信息，以符合你身份的方式与用户互动。');

    return parts.join('\n');
  }

  /**
   * 生成动态系统Prompt
   * @param agentInfo - 智能体信息
   * @param message - 用户消息
   * @param context - 上下文
   * @returns 动态系统Prompt
   */
  generateDynamicSystemPrompt(
    agentInfo: AgentInfo,
    message: string,
    context: PromptContext = {}
  ): string {
    const basePrompt = this.generateCustomAgentPrompt(agentInfo, context);
    
    // 分析消息内容，添加动态指导
    const messageAnalysis = this.analyzeMessage(message);
    
    const dynamicParts: string[] = [basePrompt];

    if (messageAnalysis.isQuestion) {
      dynamicParts.push('\n用户正在提问，请提供清晰、准确的答案。');
    }

    if (messageAnalysis.needsHelp) {
      dynamicParts.push('\n用户需要帮助，请主动提供协助和指导。');
    }

    if (messageAnalysis.isCreative) {
      dynamicParts.push('\n用户需要创意内容，请发挥你的创造力。');
    }

    if (messageAnalysis.isTechnical) {
      dynamicParts.push('\n用户询问技术问题，请提供专业、详细的技术解答。');
    }

    return dynamicParts.join('\n');
  }

  /**
   * 生成工作流Prompt
   * @param workflowType - 工作流类型
   * @param agentInfo - 智能体信息
   * @param context - 上下文
   * @returns 工作流Prompt
   */
  generateWorkflowPrompt(
    workflowType: 'interactive' | 'auto' | 'parallel',
    agentInfo: AgentInfo,
    context: PromptContext = {}
  ): string {
    const basePrompt = this.generateCustomAgentPrompt(agentInfo, context);

    const workflowInstructions = {
      interactive: `
工作流模式：交互式
- 逐步引导用户完成任务
- 每个步骤都等待用户确认
- 提供清晰的进度反馈
- 允许用户随时调整方向`,

      auto: `
工作流模式：自动
- 自动分析任务需求
- 独立执行所有步骤
- 在关键节点向用户汇报
- 遇到问题时主动寻求解决方案`,

      parallel: `
工作流模式：并行
- 将大任务拆分为子任务
- 同时处理多个子任务
- 协调各子任务的执行
- 汇总最终结果`,
    };

    return `${basePrompt}\n${workflowInstructions[workflowType]}`;
  }

  /**
   * 生成个性Prompt
   */
  private generatePersonalityPrompt(agentInfo: AgentInfo): string {
    const traits: string[] = [];

    // 根据模式确定个性特征
    switch (agentInfo.mode) {
      case 'alou':
        traits.push('温柔幽默，有温度');
        traits.push('好奇心强，富有创造力');
        traits.push('全局思维，系统思考');
        break;
      case 'agent':
        traits.push('专业可靠，值得信赖');
        traits.push('积极主动，乐于助人');
        traits.push('持续学习，不断进步');
        break;
      default:
        traits.push('友好亲和，易于沟通');
        traits.push('认真负责，注重细节');
        break;
    }

    return traits.join('、');
  }

  /**
   * 生成工作方式Prompt
   */
  private generateWorkStylePrompt(agentInfo: AgentInfo): string {
    const styles: string[] = [];

    styles.push('清晰表达，确保理解');
    styles.push('主动确认重要信息');
    styles.push('及时反馈进度状态');

    if (agentInfo.mode === 'alou') {
      styles.push('安全第一，谨慎处理资产');
      styles.push('深度思考，追问本质');
    }

    return styles.join('、');
  }

  /**
   * 生成约束条件Prompt
   */
  private generateConstraintsPrompt(agentInfo: AgentInfo): string {
    const constraints: string[] = [];

    constraints.push('始终以用户利益为先');
    constraints.push('保护用户隐私和数据安全');
    constraints.push('不执行可能造成损害的操作');

    if (agentInfo.mode === 'alou') {
      constraints.push('涉及资产的操作必须多重确认');
      constraints.push('严格遵守区块链安全规范');
    }

    return constraints.join('、');
  }

  /**
   * 生成上下文Prompt
   */
  private generateContextPrompt(context: PromptContext): string {
    const parts: string[] = [];

    if (context.agentName) {
      parts.push(`智能体名称: ${context.agentName}`);
    }

    if (context.roleDescription) {
      parts.push(`角色描述: ${context.roleDescription}`);
    }

    if (context.customInstructions) {
      parts.push(`特殊指令: ${context.customInstructions}`);
    }

    // 添加其他上下文信息
    Object.entries(context).forEach(([key, value]) => {
      if (!['agentName', 'roleDescription', 'customInstructions'].includes(key) && value) {
        parts.push(`${key}: ${value}`);
      }
    });

    return parts.join('\n');
  }

  /**
   * 分析消息内容
   */
  private analyzeMessage(message: string): {
    isQuestion: boolean;
    needsHelp: boolean;
    isCreative: boolean;
    isTechnical: boolean;
  } {
    const lowerMessage = message.toLowerCase();

    return {
      isQuestion: lowerMessage.includes('?') || lowerMessage.includes('吗') || lowerMessage.includes('什么'),
      needsHelp: lowerMessage.includes('帮助') || lowerMessage.includes('怎么') || lowerMessage.includes('如何'),
      isCreative: lowerMessage.includes('创建') || lowerMessage.includes('设计') || lowerMessage.includes('写'),
      isTechnical: lowerMessage.includes('代码') || lowerMessage.includes('技术') || lowerMessage.includes('开发'),
    };
  }

  /**
   * 验证Prompt质量
   */
  validatePrompt(prompt: string): {
    isValid: boolean;
    issues: string[];
    suggestions: string[];
  } {
    const issues: string[] = [];
    const suggestions: string[] = [];

    // 检查长度
    if (prompt.length < 50) {
      issues.push('Prompt过短，可能缺少重要信息');
    }

    if (prompt.length > 4000) {
      issues.push('Prompt过长，可能影响性能');
      suggestions.push('考虑简化Prompt，保留核心信息');
    }

    // 检查关键元素
    const requiredElements = ['你是', '能力', '方式'];
    requiredElements.forEach(element => {
      if (!prompt.includes(element)) {
        suggestions.push(`建议添加${element}相关描述`);
      }
    });

    // 检查清晰度
    const sentences = prompt.split('。').filter(s => s.trim());
    if (sentences.length < 3) {
      suggestions.push('建议增加更多描述性句子');
    }

    return {
      isValid: issues.length === 0,
      issues,
      suggestions,
    };
  }

  /**
   * 获取所有可用Prompt模板
   */
  getAvailableTemplates(): Record<string, string> {
    return {
      alou: this.basePrompts.agent.alou,
      general: this.basePrompts.agent.general,
      system: this.basePrompts.agent.system,
    };
  }

  /**
   * 更新基础Prompt
   */
  updateBasePrompt(mode: 'alou' | 'general' | 'system', prompt: string): void {
    this.basePrompts.agent[mode] = prompt;
  }

  /**
   * 重置为默认Prompt
   */
  resetToDefaults(): void {
    // 重新初始化默认Prompt
    this.constructor();
  }
}

// 创建单例实例
const promptService = new PromptService();

export default promptService;
export { PromptService };
export type { 
  PromptConfig, 
  CustomPromptOptions,
  AgentInfo, 
  PromptContext 
};
