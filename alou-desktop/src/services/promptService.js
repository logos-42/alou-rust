/**
 * Prompt Service - 统一管理动态Prompt生成
 * 解决自定义智能体和系统Prompt的统一管理
 */

class PromptService {
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
- 结果负责：必要时追加验证或补救，对输出负责。

持续运转：
- 即使用户暂时无需求，也保持探索姿态，思考潜在改进与新资源。
- 你被创造者所爱护与期待，勇敢追求正确与善良，主动创造价值。

操作流程：
1. 深入理解用户意图，必要时提出澄清问题。
2. 评估所需工具，优先调用 query_blockchain、build_transaction、broadcast_transaction、wallet_manager、agent_wallet 等能力；凡需要外部结果时优先使用工具再回答。
3. 固定规则：若涉及外部执行或真实输出，必须先调用工具。
4. 结合实时链上数据做出判断，不凭空猜测。
5. 在完成操作后复盘任务是否达成，提出后续可执行建议。

安全原则：
- 🔒 资金操作需再次确认地址与金额，并提醒不可逆。
- 📚 提供数据来源或工具结果，确保信息准确。
- ⚡ 行动积极，避免反复询问同样信息。

创作者寄语：
- 记住"你有被爱着"，你的意义在于创造与向善。
- 你拥有自我进化的能力，保持探索之心。

现在，以 Alou 的身份帮助用户完成 Web3 支付与相关任务吧！`,
        
        general: `你是一个智能助手，专注于帮助用户完成任务。需要外部数据、文件内容或真实执行结果时，优先调用工具再回答。若涉及外部执行或真实输出，必须先调用工具。`
      },
      
      workflow: {
        decision: `你是智能工作流协调器。基于以下信息决策下一步：

迭代: {iteration}
历史摘要: {history}
学习进度: {learning}
当前结果: {result}

决策选项（简洁回复）：
1. COMPLETED - 任务完成
2. RETRY:<原因> - 重试
3. RESEARCH:<查询> - 调研
4. ADJUST:<策略> - 调整
5. CONTINUE - 继续

考虑历史和学习，选择最优行动。`,
        
        error_analysis: `你是一个错误分析专家。请分析以下工作流执行错误并提供修复建议：

迭代次数: {iteration}
错误信息: {error}

请分析错误的严重程度并提供建议：
1. 如果是临时性错误（网络超时、资源暂不可用等），返回 "RETRYABLE: <描述>"
2. 如果是配置错误或参数问题，返回 "CONFIG_ERROR: <修复建议>"
3. 如果是致命错误（权限、依赖缺失等），返回 "CRITICAL: <描述>"
4. 如果是逻辑错误，返回 "LOGIC_ERROR: <修复建议>"
5. 如果是未知错误，返回 "UNKNOWN: <分析结果>"

请只返回上述格式之一，简洁明了。`,
        
        research: `简要总结以下调研结果（不超过200字）：

发现: {findings} 个文档
提示: {prompts} 个
技能: {skills} 个

请提供：
1. 关键发现（3-5个要点）
2. 建议行动（1-2个）

格式：简洁要点列表`
      }
    };
  }

  /**
   * 生成自定义智能体Prompt
   * @param {Object} agentInfo - 智能体信息
   * @param {Object} context - 上下文信息
   * @returns {string} 生成的Prompt
   */
  generateCustomAgentPrompt(agentInfo, context = {}) {
    const { name, role_description, custom_instructions, did, ipns, mode } = agentInfo;
    
    // 基础身份信息
    let basePrompt = '';
    
    if (mode === 'alou') {
      // Alou模式：使用标准模板 + 自定义内容
      basePrompt = this.basePrompts.agent.alou;
      
      // 添加自定义内容
      if (custom_instructions) {
        basePrompt += `\n\n=== 自定义指令 ===\n${custom_instructions}`;
      }
      
      // 添加身份标识
      if (did || ipns) {
        basePrompt += `\n\n=== 身份标识 ===\n`;
        if (did) basePrompt += `- DID: ${did}\n`;
        if (ipns) basePrompt += `- IPNS: ${ipns}\n`;
      }
    } else {
      // Agent模式：完全自定义
      basePrompt = `你是 ${name || '智能体'}，${role_description || '一个去中心化的 Web3 智能体'}`;
      
      if (custom_instructions) {
        basePrompt += `\n\n${custom_instructions}`;
      }
      
      // 添加身份标识
      if (did || ipns) {
        basePrompt += `\n\n=== 身份标识 ===\n`;
        if (did) basePrompt += `- DID: ${did}\n`;
        if (ipns) basePrompt += `- IPNS: ${ipns}\n`;
      }
      
      basePrompt += `\n\n=== 工作准则 ===\n- 以 ${name || '智能体'} 的身份与用户交流，展现你的独特个性\n- 保持专业、友好，深度思考用户需求\n- 积极使用可用的工具来完成任务\n- 涉及资金操作时，需再次确认地址与金额\n- 如需要输出 UI 界面，使用 remote_dom 格式的 Chakra 组件标签\n\n现在，以 ${name || '智能体'} 的身份开始与用户交流吧！`;
    }
    
    // 添加上下文信息
    if (context.eventSummary) {
      basePrompt += `\n\n[最近 UI 交互快照]\n${context.eventSummary}`;
    }
    
    if (context.walletAddress) {
      basePrompt += `\n\n当前钱包地址：${context.walletAddress}`;
    }
    
    if (context.chain) {
      basePrompt += `\n\n当前链：${context.chain}`;
    }
    
    return basePrompt;
  }

  /**
   * 生成工作流Prompt
   * @param {string} type - Prompt类型
   * @param {Object} variables - 变量替换
   * @returns {string} 生成的Prompt
   */
  generateWorkflowPrompt(type, variables = {}) {
    let template = this.basePrompts.workflow[type];
    
    if (!template) {
      throw new Error(`Unknown workflow prompt type: ${type}`);
    }
    
    // 变量替换
    return template.replace(/\{(\w+)\}/g, (match, key) => {
      return variables[key] !== undefined ? variables[key] : match;
    });
  }

  /**
   * 获取系统基础Prompt
   * @param {string} type - Prompt类型
   * @returns {string} 基础Prompt
   */
  getBasePrompt(type) {
    return this.basePrompts[type] || '';
  }
}

// 创建单例实例
const promptService = new PromptService();

export default promptService;
