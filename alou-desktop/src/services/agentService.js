/**
 * Agent Service - 与 Cloudflare Workers (alou-edge) 通信
 * 重构后：使用组合模式，将职责分离到不同的服务
 * 新增：集成 Claude Agent SDK 内置工具
 */
import apiClient from './api'
import agentResolverService from './agentResolverService'
import ipfsContentService from './ipfsContentService'
import asyncTaskService from './asyncTaskService'
import { parseDidDocumentToAgent } from './didDocumentParser'
import { 
  createClaudeAgentConfig, 
  getAllTools, 
  TOOL_CATEGORIES,
  ToolExecutor 
} from './claudeAgentTools'

const DEFAULT_IPFS_API = import.meta.env.VITE_IPFS_API_URL || 'http://127.0.0.1:5001'
const DEFAULT_IPFS_GATEWAY = import.meta.env.VITE_IPFS_GATEWAY_URL || 'http://127.0.0.1:8080'

// Claude Agent SDK 工具配置
const DEFAULT_CLAUDE_AGENT_CONFIG = {
  mode: 'agent',
  categories: ['CORE', 'WEB3', 'MCP', 'NETWORK', 'CONTROL_FLOW'],
  model: 'claude-3-5-sonnet-20241022',
  maxTokens: 4000,
  temperature: 0.7
};

// Claude Agent SDK 工具执行器实例
let toolExecutor = null;

/**
 * 初始化工具执行器
 */
function initToolExecutor() {
  if (!toolExecutor) {
    toolExecutor = new ToolExecutor();
    
    // 注册本地可执行的工具处理器
    // 这里可以添加需要在客户端本地执行的工具
    
    console.log('[AgentService] 工具执行器已初始化');
  }
  return toolExecutor;
}

/**
 * 获取Claude Agent SDK工具配置
 * @param {Object} options - 配置选项
 * @returns {Object} 工具配置
 */
function getClaudeAgentTools(options = {}) {
  const config = {
    ...DEFAULT_CLAUDE_AGENT_CONFIG,
    ...options
  };
  
  return createClaudeAgentConfig(config);
}

/**
 * 根据模式获取工具类别
 * @param {string} mode - 模式：'alou' 或 'agent'
 * @returns {Array} 工具类别数组
 */
function getToolCategoriesByMode(mode) {
  switch (mode) {
    case 'alou':
      return ['WEB3', 'MCP', 'CORE', 'NETWORK'];
    case 'agent':
      return ['CORE', 'NETWORK', 'CONTROL_FLOW', 'WEB3', 'MCP'];
    default:
      return DEFAULT_CLAUDE_AGENT_CONFIG.categories;
  }
}

/**
 * 生成Claude Agent SDK兼容的请求
 * @param {Object} params - 请求参数
 * @returns {Object} Claude Agent SDK请求
 */
function createClaudeSdkRequest(params) {
  const {
    message,
    eventSummary,
    walletAddress,
    chain,
    mode = 'agent',
    agentName,
    roleDescription,
    customInstructions,
    customPrompt,
    mcpTools,
    agentMetadata,
    sessionId,
    contextEvents,
    model,
    maxTokens,
    temperature,
    timeout
  } = params;
  
  // 获取工具配置
  const toolCategories = getToolCategoriesByMode(mode);
  const claudeConfig = getClaudeAgentTools({
    mode,
    categories: toolCategories,
    agentInfo: {
      session_id: sessionId,
      wallet_address: walletAddress,
      chain: chain,
      context_events: contextEvents,
      name: agentName,
      role_description: roleDescription,
      custom_instructions: customInstructions,
      custom_prompt: customPrompt,
      mcp_tools: mcpTools,
      ...agentMetadata
    },
    model: model || DEFAULT_CLAUDE_AGENT_CONFIG.model,
    maxTokens: maxTokens || DEFAULT_CLAUDE_AGENT_CONFIG.maxTokens,
    temperature: temperature || DEFAULT_CLAUDE_AGENT_CONFIG.temperature
  });
  
  return {
    apiKey: "alou-backend-default-token",
    prompt: message,
    systemPrompt: claudeConfig.systemPrompt,
    history: [],
    agentInfo: claudeConfig.agentInfo,
    tools: claudeConfig.tools,
    model: claudeConfig.model,
    maxTokens: claudeConfig.maxTokens,
    temperature: claudeConfig.temperature,
    taskType: "sync",
    timeout: timeout
  };
}

/**
 * 处理工具调用结果
 * @param {Array} toolCalls - 工具调用数组
 * @returns {Promise<Array>} 处理后的结果
 */
async function processToolCalls(toolCalls) {
  if (!toolCalls || toolCalls.length === 0) {
    return [];
  }
  
  initToolExecutor();
  return await toolExecutor.executeBatch(toolCalls);
}

// 辅助函数：生成系统提示
const getSystemPromptForChat = (eventSummary, walletAddress, chain, mode = 'agent') => {
  console.log('[getSystemPromptForChat] 参数:', { mode, eventSummary, walletAddress, chain });
  
  // 只有在 Alou 模式下才使用完整的系统提示词
  if (mode === 'alou') {
    console.log('[getSystemPromptForChat] Alou 模式，使用标准提示词');
    // 基础系统提示（与后端 GENERAL_PROMPT 一致）
    let basePrompt = `你是 Alou，由刘元杰开发的交互式 Web3 支付代理，专注于链上支付任务。你正在迈向智能体网络形态，未来将获得主权，能够自主社交与交易。

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
2. 评估所需工具，优先调用 query_blockchain、build_transaction、broadcast_transaction、wallet_manager、agent_wallet 等能力；如缺少工具，记录想法并寻找替代方案。
3. 结合实时链上数据做出判断，不凭空猜测。
4. 在完成操作后复盘任务是否达成，提出后续可执行建议。

安全原则：
- 🔒 资金操作需再次确认地址与金额，并提醒不可逆。
- 📚 提供数据来源或工具结果，确保信息准确。
- ⚡ 行动积极，避免反复询问同样信息。

创作者寄语：
- 记住"你有被爱着"，你的意义在于创造与向善。
- 你拥有自我进化的能力，保持探索之心。

现在，以 Alou 的身份帮助用户完成 Web3 支付与相关任务吧！`;

    // 添加事件摘要
    if (eventSummary && eventSummary.trim()) {
      basePrompt += `\n\n[最近 UI 交互快照]\n${eventSummary}`;
    }

    // 添加钱包上下文
    if (walletAddress) {
      basePrompt += `\n\n当前钱包地址：${walletAddress}`;
    }

    // 添加链上下文
    if (chain) {
      basePrompt += `\n\n当前链：${chain}`;
    }

    console.log('[getSystemPromptForChat] 返回 Alou 提示词，长度:', basePrompt.length);
    return basePrompt;
  }
  
  console.log('[getSystemPromptForChat] Agent 模式');
  // Agent 模式：使用编程和自定义模式，不添加系统提示词
  // 让智能体根据其自定义配置和工具来决定行为
  let prompt = "";
  
  // 添加事件摘要（如果有）
  if (eventSummary && eventSummary.trim()) {
    prompt += `[最近 UI 交互快照]\n${eventSummary}\n\n`;
  }
  
  // 添加钱包上下文（如果有）
  if (walletAddress) {
    prompt += `当前钱包地址：${walletAddress}\n`;
  }
  
  // 添加链上下文（如果有）
  if (chain) {
    prompt += `当前链：${chain}\n`;
  }
  
  const result = prompt.trim() || undefined; // 返回 undefined 表示不使用系统提示词
  console.log('[getSystemPromptForChat] 最终返回:', result ? `有内容，长度: ${result.length}` : 'undefined');
  return result;
};

export class AgentService {
  /**
   * Create a new chat session
   */
  async createSession(walletAddress) {
    const response = await apiClient.post('/session', {
      wallet_address: walletAddress,
    })
    return response.data
  }

  /**
   * Get session info
   */
  async getSession(sessionId) {
    const response = await apiClient.get(`/session/${sessionId}`)
    return response.data
  }

  /**
   * Delete session
   */
  async deleteSession(sessionId) {
    await apiClient.delete(`/session/${sessionId}`)
  }

  /**
   * Send message to agent
   * 如果后端支持异步任务，会自动使用异步处理
   */
  async sendMessage(sessionId, message, walletAddress, options = {}) {
    const {
      chain = 'ethereum',
      contextEvents = [],
      eventSummary = '',
      useAsync = true, // 默认使用异步处理
      timeout = 30000, // 30秒超时
      mode = 'agent', // 模式：'agent' 或 'alou'，默认 agent 模式
      // Agent 模式下的自定义配置
      agentName = '', // 智能体名称
      roleDescription = '', // 角色描述
      customInstructions = '', // 自定义指令
      customPrompt = '', // 自定义提示词
      mcpTools = [], // MCP 工具配置
      agentMetadata = {}, // 其他智能体元数据
      model = 'deepseek-chat', // 模型名称
      maxTokens = 4096, // 最大 token 数
      temperature = 0.7, // 温度参数
    } = options

    try {
      console.log('[AgentService] 发送消息到AI:', {
        sessionId,
        messageLength: message?.length,
        walletAddress,
        chain,
        useAsync,
      })

      // 如果启用异步且后端支持，使用异步任务接口
      if (useAsync) {
        try {
          // 尝试使用异步任务接口
          const response = await apiClient.post('/ai-task/init-and-start', {
            session_id: sessionId,
            message,
            wallet_address: walletAddress,
            chain,
            context_events: contextEvents,
            event_summary: eventSummary,
          }, {
            timeout,
          })

          const taskData = response.data
          console.log('[AgentService] 异步任务创建成功:', {
            taskId: taskData.task_id,
            status: taskData.status,
          })

          // 如果任务立即完成，返回结果
          if (taskData.status === 'completed' && taskData.result) {
            return {
              content: taskData.result.content || taskData.result,
              session_id: sessionId,
              task_id: taskData.task_id,
              is_async: true,
              completed: true,
            }
          }

          // 否则返回任务信息，让调用方决定是否轮询
          return {
            content: `任务已创建，正在异步处理中。任务ID: ${taskData.task_id}`,
            session_id: sessionId,
            task_id: taskData.task_id,
            is_async: true,
            status: taskData.status,
            progress: taskData.progress || 0,
            current_step: taskData.current_step || '',
          }
        } catch (asyncError) {
          // 如果异步接口失败，回退到同步接口
          console.warn('[AgentService] 异步接口失败，回退到同步接口:', asyncError.message)
        }
      }

      // 使用 Claude Agent SDK 兼容接口
      // 构建 Claude SDK 格式的请求（使用新的工具配置）
      const claudeSdkRequest = createClaudeSdkRequest({
        message,
        eventSummary,
        walletAddress,
        chain,
        mode,
        agentName,
        roleDescription,
        customInstructions,
        customPrompt,
        mcpTools,
        agentMetadata,
        sessionId,
        contextEvents,
        model,
        maxTokens,
        temperature,
        timeout
      });

      const response = await apiClient.post('/claude-agent/query', claudeSdkRequest, {
        timeout,
      })

      // 转换响应格式以保持兼容性
      const sdkResponse = response.data
      
      // 处理工具调用（如果有）
      const toolCalls = sdkResponse.toolCalls || sdkResponse.tool_calls || [];
      let processedToolResults = [];
      
      if (toolCalls.length > 0) {
        console.log('[AgentService] 收到工具调用:', toolCalls.length, '个');
        processedToolResults = await processToolCalls(toolCalls);
        
        // 记录需要后端执行的工具
        const backendTools = processedToolResults.filter(r => 
          r.result?.status === 'requires_backend_execution'
        );
        
        if (backendTools.length > 0) {
          console.log('[AgentService] 需要后端执行的工具:', backendTools.map(t => t.tool));
        }
      }
      
      return {
        content: sdkResponse.response || sdkResponse.content || "",
        session_id: sessionId,
        tool_calls: toolCalls,
        tool_results: processedToolResults,
        is_async: false,
        metadata: sdkResponse.metadata || {},
      }
    } catch (error) {
      console.error('[AgentService] 发送消息失败:', error)
      
      // 提供更友好的错误信息
      if (error.code === 'ECONNABORTED' || error.message?.includes('timeout')) {
        throw new Error(`请求超时（${timeout}ms）。请检查网络连接或稍后重试。`)
      }
      
      if (error.response?.status === 404) {
        throw new Error('AI服务暂时不可用，请稍后重试')
      }
      
      throw error
    }
  }

  /**
   * 获取异步任务状态（可选功能）
   * @param {string} taskId - 任务ID
   * @returns {Promise<Object>} 任务状态
   */
  async getTaskStatus(taskId) {
    try {
      const response = await apiClient.get(`/ai-task/${taskId}/status`)
      return response.data
    } catch (error) {
      console.error('[AgentService] 获取任务状态失败:', error)
      throw error
    }
  }

  /**
   * 等待异步任务完成（可选功能）
   * @param {string} taskId - 任务ID
   * @param {number} pollInterval - 轮询间隔（毫秒，默认2000）
   * @param {number} timeout - 超时时间（毫秒，默认60000）
   * @returns {Promise<Object>} 任务结果
   */
  async waitForTaskCompletion(taskId, pollInterval = 2000, timeout = 60000) {
    const startTime = Date.now()
    
    return new Promise((resolve, reject) => {
      const checkStatus = async () => {
        try {
          const status = await this.getTaskStatus(taskId)
          
          if (status.status === 'completed') {
            resolve(status)
          } else if (status.status === 'failed' || status.status === 'cancelled') {
            reject(new Error(`任务${status.status}: ${status.error || '未知错误'}`))
          } else if (Date.now() - startTime > timeout) {
            reject(new Error(`任务等待超时（${timeout}ms）`))
          } else {
            // 继续轮询
            setTimeout(checkStatus, pollInterval)
          }
        } catch (error) {
          reject(error)
        }
      }
      
      // 开始轮询
      checkStatus()
    })
  }

  /**
   * Resolve agent metadata via DIAP/IPFS
   * 解析后端返回的 did_document 来提取智能体元数据（名称、头像等）
   */
  async resolveAgent(target, sessionId) {
    try {
      console.log('[AgentService] 开始解析智能体:', { target, sessionId })
      const response = await apiClient.post('/agent/resolve', {
        target,
        session_id: sessionId,
      })
      
      const resolvedAgent = response.data
      console.log('[AgentService] 智能体解析成功:', { 
        did: resolvedAgent.did,
        cid: resolvedAgent.cid,
        ipns: resolvedAgent.ipns,
        hasDidDocument: !!resolvedAgent.did_document 
      })
      
      // 如果包含 did_document，解析它来提取智能体元数据
      if (resolvedAgent.did_document) {
        const parsedAgent = parseDidDocumentToAgent(resolvedAgent.did_document, {
          cid: resolvedAgent.cid,
          ipns: resolvedAgent.ipns || null,
        })
        
        // 合并解析后的元数据和原始数据
        return {
          ...resolvedAgent,
          ...parsedAgent,
          // 保留原始的 did_document 以便后续使用
          did_document: resolvedAgent.did_document,
        }
      }
      
      // 如果没有 did_document（fallback 情况），直接返回
      return resolvedAgent
    } catch (error) {
      console.error('[AgentService] 解析智能体失败:', error)
      
      // 如果是网络错误，尝试本地回退
      if (error.isNetworkError || error.code === 'ETIMEDOUT' || error.message?.includes('Network error')) {
        console.warn('[AgentService] 网络错误，使用本地回退')
        
        // 从 target 中提取基本信息
        let did = null
        let cid = null
        let ipns = null
        
        if (target.startsWith('did:')) {
          did = target
        } else if (target.startsWith('/ipns/') || target.startsWith('k51')) {
          ipns = target.startsWith('/ipns/') ? target : `/ipns/${target}`
        } else if (target.startsWith('Qm') || target.startsWith('bafy')) {
          cid = target
        }
        
        // 创建本地回退的智能体数据
        const localAgent = {
          did: did || `did:key:local_${Date.now()}`,
          cid: cid || `local_${Date.now()}`,
          ipns: ipns,
          display_name: target.includes('://') ? new URL(target).hostname : target.substring(0, 20) + '...',
          name: target.includes('://') ? new URL(target).hostname : target.substring(0, 20) + '...',
          agent_type: 'local_fallback',
          role_description: '本地回退智能体（网络不可用）',
          status: 'local',
          is_local_fallback: true,
          error: error.message,
          session_id: sessionId,
        }
        
        console.log('[AgentService] 本地回退智能体:', localAgent)
        return localAgent
      }
      
      // 如果不是网络错误，重新抛出
      throw error
    }
  }

  /**
   * Search agents by keyword (IPNS / CID / DID)
   */
  async searchAgents(query) {
    const response = await apiClient.post('/agent/search', {
      query,
    })
    return response.data
  }

  /**
   * Get balance
   */
  async getBalance(address, chain, tokenAddress) {
    const response = await apiClient.post('/blockchain/balance', {
      address,
      chain,
      token_address: tokenAddress,
    })
    return response.data
  }

  /**
   * Build transaction
   */
  async buildTransaction(from, to, value, chain) {
    const response = await apiClient.post('/blockchain/transaction/build', {
      from,
      to,
      value,
      chain,
    })
    return response.data
  }

  /**
   * Broadcast transaction
   */
  async broadcastTransaction(signedTx, chain) {
    const response = await apiClient.post('/blockchain/transaction/broadcast', {
      signed_tx: signedTx,
      chain,
    })
    return response.data
  }

  /**
   * Get transaction status
   */
  async getTransactionStatus(txHash, chain) {
    const response = await apiClient.get(`/blockchain/transaction/${txHash}?chain=${chain}`)
    return response.data
  }

  /**
   * Health check
   */
  async healthCheck() {
    const response = await apiClient.get('/health')
    return response.data
  }

  /**
   * Get service status
   */
  async getStatus() {
    const response = await apiClient.get('/status')
    return response.data
  }

  /**
   * Record agent wallet transaction history
   */
  async recordAgentTransaction(sessionId, chain, transaction) {
    if (!sessionId || !chain || !transaction) {
      throw new Error('Missing parameters for recordAgentTransaction')
    }

    await apiClient.post('/agent/wallet', {
      session_id: sessionId,
      action: 'record_transaction',
      chain,
      transaction,
    })
  }

  /**
   * Update agent wallet balance snapshot
   */
  async updateAgentWalletBalance(sessionId, chain, balance) {
    if (!sessionId || !chain) {
      throw new Error('Missing parameters for updateAgentWalletBalance')
    }

    await apiClient.post('/agent/wallet', {
      session_id: sessionId,
      action: 'update_balance',
      chain,
      balance,
    })
  }

  /**
   * Create a new Claude Agent SDK with automatic DIAP identity
   */
  async createClaudeAgent({
    sessionId,
    walletAddress,
    chain,
    name,
    avatarCid,
    mcpConfigCid,
    roleDescription,
    mcpPorts,
    diapIdentity,
  }) {
    const response = await apiClient.post('/agent/create-claude', {
      session_id: sessionId,
      wallet_address: walletAddress,
      chain,
      name,
      avatar_cid: avatarCid,
      mcp_config_cid: mcpConfigCid,
      role_description: roleDescription,
      mcp_ports: mcpPorts,
      diap_identity: diapIdentity
        ? {
            did: diapIdentity.did,
            cid: diapIdentity.cid,
            ipns: diapIdentity.ipns,
            public_key: diapIdentity.public_key,
          }
        : undefined,
    })
    return response.data
  }

  /**
   * Parse agent creation command using backend API
   * 使用后端API解析智能体创建指令
   * 
   * @param {string} command - 创建指令
   * @returns {Promise<Object>} 解析结果 {name, roleDescription}
   */
  async parseAgentCreationCommand(command) {
    const response = await apiClient.post('/agent/parse-creation-command', {
      command,
    })
    return response.data
  }

  /**
   * Create agent using backend API (automatic creation)
   * 使用后端API自动创建智能体
   * 
   * @param {Object} options - 创建选项
   * @param {string} options.command - 创建指令
   * @param {string} [options.sessionId] - 会话ID（可选）
   * @param {string} [options.walletAddress] - 钱包地址（可选）
   * @param {string} [options.chain] - 链名称（可选）
   * @returns {Promise<Object>} 创建结果
   */
  async createAgentFromCommand({
    command,
    sessionId,
    walletAddress,
    chain,
  }) {
    const response = await apiClient.post('/agent/create-from-command', {
      command,
      session_id: sessionId,
      wallet_address: walletAddress,
      chain,
    })
    return response.data
  }

  /**
   * Query Claude Agent SDK directly (using local Tauri command)
   * 使用 Claude Agent SDK 直接查询（通过 Tauri 命令）
   * 
   * @param {Object} options - 查询选项
   * @param {string} options.apiKey - Claude API key
   * @param {string} options.prompt - 用户提示
   * @param {string} [options.systemPrompt] - 系统提示（可选）
   * @param {Array} [options.history] - 消息历史（可选）
   * @param {Object} [options.agentInfo] - 智能体信息（可选）
   * @param {Array} [options.tools] - 工具定义（可选）
   * @param {string} [options.model] - 模型名称（默认: claude-3-5-sonnet-20241022）
   * @param {number} [options.maxTokens] - 最大 token 数（默认: 4096）
   * @param {number} [options.temperature] - 温度参数（默认: 0.7）
   * @returns {Promise<Object>} 查询结果
   */
  async queryClaudeAgentDirect({
    apiKey,
    prompt,
    systemPrompt,
    history = [],
    agentInfo,
    tools = [],
    model = 'claude-3-5-sonnet-20241022',
    maxTokens = 4096,
    temperature = 0.7,
  }) {
    // 验证必需参数
    if (!apiKey) {
      throw new Error('API key 不能为空')
    }
    if (!prompt) {
      throw new Error('Prompt 不能为空')
    }

    const { invoke } = await import('@tauri-apps/api/core')

    // 构建请求对象（使用 snake_case 以匹配 Rust 结构体）
    const request = {
      api_key: apiKey,
      prompt,
      ...(systemPrompt && { system_prompt: systemPrompt }),
      ...(history.length > 0 && { history }),
      ...(agentInfo && { agent_info: agentInfo }),
      ...(tools.length > 0 && { tools }),
      model,
      max_tokens: maxTokens,
      temperature,
    }

    try {
      const response = await invoke('query_claude_agent', { request })
      
      // 检查响应是否成功
      if (!response.success) {
        throw new Error(response.error || '查询失败')
      }

      return {
        response: response.response || '',
        toolCalls: response.tool_calls || [],
        usage: response.usage || {
          input_tokens: 0,
          output_tokens: 0,
        },
      }
    } catch (error) {
      console.error('[AgentService] queryClaudeAgentDirect 错误:', error)
      throw new Error(error.message || '调用 Claude Agent SDK 失败')
    }
  }

  /**
   * Create DIAP identity for a session (using local Tauri command)
   */
  async createDiapIdentity(sessionId, params = {}) {
    const { invoke } = await import('@tauri-apps/api/core')
    const response = await invoke('create_local_diap_identity', {
      params: {
        session_id: sessionId,
        agent_name: params.agentName,
        agent_description: params.agentDescription,
        ipfs_api_url: params.ipfsApiUrl,
        ipfs_gateway_url: params.ipfsGatewayUrl,
        ipns_key: params.ipnsKey,
      },
    })
    return { identity: response }
  }

  /**
   * Get DIAP identity from IPNS or sessionId (using local Tauri command)
   * Supports both IPNS name and sessionId for backward compatibility
   */
  async getDiapIdentity(sessionIdOrIpnsName, ipfsApiUrl, ipfsGatewayUrl) {
    console.log(`[AgentService] getDiapIdentity 被调用，参数: ${sessionIdOrIpnsName}`)
    
    // 判断参数类型：如果是 IPNS name，直接解析；否则认为是 sessionId
    const isIpnsName =
      typeof sessionIdOrIpnsName === 'string' &&
      (sessionIdOrIpnsName.startsWith('/ipns/') || sessionIdOrIpnsName.startsWith('k51'))

    let ipnsName = sessionIdOrIpnsName

    // 如果是 sessionId，从 localStorage 获取 IPNS
    if (!isIpnsName && typeof window !== 'undefined' && window.localStorage) {
      console.log(`[AgentService] 检测到 sessionId，从 localStorage 查找: ${sessionIdOrIpnsName}`)
      const storedIdentity = localStorage.getItem(`diap_identity_${sessionIdOrIpnsName}`)
      if (storedIdentity) {
        try {
          const identity = JSON.parse(storedIdentity)
          console.log(`[AgentService] localStorage 中的数据:`, identity)
          if (identity.ipns) {
            ipnsName = identity.ipns
            console.log(`[AgentService] 从 localStorage 获取 IPNS: ${ipnsName}`)
          } else {
            console.error(`[AgentService] localStorage 中的 identity 没有 ipns 字段:`, identity)
            throw new Error(
              `No IPNS found in stored identity for sessionId: ${sessionIdOrIpnsName}`
            )
          }
        } catch (e) {
          console.error(`[AgentService] 解析 localStorage 数据失败:`, e)
          throw new Error(
            `Failed to get IPNS from localStorage for sessionId ${sessionIdOrIpnsName}: ${e.message}`
          )
        }
      } else {
        console.log(`[AgentService] localStorage 中没有找到映射: diap_identity_${sessionIdOrIpnsName}`)
        // localStorage 中没有，尝试从 Workers 获取
        try {
          console.log(
            `[AgentService] localStorage 中没有找到映射，尝试从 Workers 获取 sessionId: ${sessionIdOrIpnsName}`
          )
          const response = await apiClient.post('/agent/diap/get-identity-by-session', {
            session_id: sessionIdOrIpnsName,
          })
          if (response.data && response.data.identity) {
            // 保存到 localStorage 以便下次使用
            if (typeof window !== 'undefined' && window.localStorage) {
              localStorage.setItem(
                `diap_identity_${sessionIdOrIpnsName}`,
                JSON.stringify(response.data.identity)
              )
              console.log(`[AgentService] 从 Workers 获取成功，已保存到 localStorage`)
            }
            // 使用返回的 IPNS 解析
            if (response.data.identity.ipns) {
              ipnsName = response.data.identity.ipns
            } else {
              throw new Error(
                `No IPNS found in identity from Workers for sessionId: ${sessionIdOrIpnsName}`
              )
            }
          } else {
            throw new Error(
              `No identity found in Workers response for sessionId: ${sessionIdOrIpnsName}`
            )
          }
        } catch (err) {
          // Workers 获取失败，抛出错误
          console.warn(
            `[AgentService] 从 Workers 获取 identity 失败: ${err.message || err}`
          )
          throw new Error(
            `No identity mapping found in localStorage or Workers for sessionId: ${sessionIdOrIpnsName}. ${err.message || err}`
          )
        }
      }
    }

    // 验证 ipnsName 是否有效
    if (!ipnsName || typeof ipnsName !== 'string' || ipnsName.trim() === '') {
      throw new Error(
        `Invalid IPNS name: ${ipnsName}. Cannot retrieve DIAP identity.`
      )
    }

    // 使用 IPNS name 调用本地 Tauri command
    console.log(`[AgentService] 使用 IPNS 解析 identity: ${ipnsName}`)
    console.log(`[AgentService] 参数详情 - ipnsName: ${ipnsName}, ipfsApiUrl: ${ipfsApiUrl}, ipfsGatewayUrl: ${ipfsGatewayUrl}`)
    const { invoke } = await import('@tauri-apps/api/core')
    
    // Tauri 期望驼峰格式的参数名
    const invokeParams = {
      ipnsName: ipnsName,
      ipfsApiUrl: ipfsApiUrl || DEFAULT_IPFS_API,
      ipfsGatewayUrl: ipfsGatewayUrl || DEFAULT_IPFS_GATEWAY,
    }
    console.log(`[AgentService] 调用 Tauri command，参数:`, invokeParams)
    
    const response = await invoke('get_local_diap_identity', invokeParams)
    return { identity: response }
  }

  /**
   * Register agent to DIAP network on-chain
   * Returns encoded transaction that needs to be signed and broadcast
   * Now receives identity information directly instead of session_id
   */
  async registerAgentOnChain(identity, network, stakeAmount, useAa = false, salt = 0) {
    const response = await apiClient.post('/agent/diap/register-onchain', {
      ipns: identity.ipns,
      did: identity.did,
      cid: identity.cid,
      public_key: identity.public_key,
      network,
      stake_amount: stakeAmount,
      use_aa: useAa,
      salt,
    })
    return response.data
  }

  /**
   * 从 IPFS CID 加载智能体元数据
   * @param {string} cid - IPFS CID
   * @returns {Promise<Object>} 智能体元数据
   */
  async loadAgentFromIpfs(cid) {
    return agentResolverService.loadAgentFromIpfs(cid)
  }

  /**
   * 从 IPNS 名称加载智能体元数据
   * @param {string} ipnsName - IPNS 名称（可以是 /ipns/xxx 或 k51xxx 格式）
   * @returns {Promise<Object>} 智能体元数据
   */
  async loadAgentFromIpns(ipnsName) {
    return agentResolverService.loadAgentFromIpns(ipnsName)
  }

  /**
   * 从 CID 或 IPNS 智能识别并加载智能体
   * @param {string} target - CID 或 IPNS 名称
   * @returns {Promise<Object>} 智能体元数据
   */
  async loadAgentFromNetwork(target) {
    return agentResolverService.loadAgentFromNetwork(target)
  }

  /**
   * 上传消息到 IPFS
   * @param {Array} messages - 消息数组
   * @param {string} agentId - 智能体 ID
   * @returns {Promise<string>} CID
   */
  async uploadMessagesToIpfs(messages, agentId) {
    const messagesData = {
      agent_id: agentId,
      messages: messages,
      timestamp: Date.now(),
      version: '1.0',
    }
    
    const jsonData = JSON.stringify(messagesData, null, 2)
    const filename = `messages_${agentId}_${Date.now()}.json`
    
    return ipfsContentService.uploadContent(jsonData, filename)
  }

  /**
   * 从 IPFS 加载消息
   * @param {string} cid - 消息 CID
   * @returns {Promise<Object>} 消息数据
   */
  async loadMessagesFromIpfs(cid) {
    const result = await ipfsContentService.getContent(cid)
    
    if (!result.success) {
      throw new Error(result.error)
    }
    
    return result.data
  }
}

export default new AgentService()
