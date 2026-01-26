/**
 * Agent Service - 与 Cloudflare Workers (alou-edge) 通信
 * 重构后：使用组合模式，将职责分离到不同的服务
 */
import apiClient from './api'
import agentResolverService from './agentResolverService'
import ipfsContentService from './ipfsContentService'
import asyncTaskService from './asyncTaskService'
import promptService from './promptService'
import { parseDidDocumentToAgent } from './didDocumentParser'

const DEFAULT_IPFS_API = import.meta.env.VITE_IPFS_API_URL || 'http://127.0.0.1:5001'
const DEFAULT_IPFS_GATEWAY = import.meta.env.VITE_IPFS_GATEWAY_URL || 'http://127.0.0.1:8080'

// 辅助函数：生成系统提示（使用统一Prompt服务）
const getSystemPromptForChat = (agentInfo, context = {}) => {
  console.log('[getSystemPromptForChat] 使用统一Prompt服务:', { agentInfo, context });
  
  try {
    // 使用PromptService生成动态Prompt
    const prompt = promptService.generateCustomAgentPrompt(agentInfo, context);
    
    console.log('[getSystemPromptForChat] 生成Prompt成功，长度:', prompt.length);
    return prompt;
  } catch (error) {
    console.error('[getSystemPromptForChat] 生成Prompt失败:', error);
    
    // 降级处理：使用基础Prompt
    const mode = agentInfo?.mode || 'agent';
    const basePrompt = promptService.getBasePrompt('agent');
    const fallbackPrompt = mode === 'alou' ? basePrompt.alou : basePrompt.general;
    
    console.log('[getSystemPromptForChat] 使用降级Prompt，长度:', fallbackPrompt.length);
    return fallbackPrompt;
  }
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

      // 获取系统提示词
      const systemPrompt = getSystemPromptForChat(
        eventSummary,
        walletAddress,
        chain,
        mode
      )

      // 构建请求
      const requestBody = {
        message,
        session_id: sessionId,
        system_prompt: systemPrompt,
        wallet_address: walletAddress,
        chain,
        mode,
        agent_name: agentName,
        role_description: roleDescription,
        custom_instructions: customInstructions,
        custom_prompt: customPrompt,
        mcp_tools: mcpTools,
        agent_metadata: agentMetadata,
        context_events: contextEvents,
        model,
        max_tokens: maxTokens,
        temperature,
      }

      const response = await apiClient.post('/agent/chat', requestBody, {
        timeout,
      })

      // 转换响应格式以保持兼容性
      const data = response.data

      return {
        content: data.response || data.content || "",
        session_id: sessionId,
        tool_calls: data.tool_calls || [],
        tool_results: [],
        is_async: false,
        metadata: data.metadata || {},
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
   * Create agent using backend API
   * 使用后端API创建智能体
   * 
   * @param {Object} options - 创建选项
   * @param {string} options.sessionId - 会话ID
   * @param {string} [options.walletAddress] - 钱包地址（可选）
   * @param {string} [options.chain] - 链名称（可选）
   * @param {string} [options.name] - 智能体名称（可选）
   * @param {string} [options.roleDescription] - 角色描述（可选）
   * @param {string} [options.avatarCid] - 头像CID（可选）
   * @param {string} [options.mcpConfigCid] - MCP配置CID（可选）
   * @param {Array} [options.mcpPorts] - MCP端口配置（可选）
   * @param {Object} [options.diapIdentity] - DIAP身份信息（可选）
   * @returns {Promise<Object>} 创建结果
   */
  async createAgent({
    sessionId,
    walletAddress,
    chain,
    name,
    roleDescription,
    avatarCid,
    mcpConfigCid,
    mcpPorts,
    diapIdentity,
  }) {
    console.log('[AgentService] 开始创建智能体:', {
      sessionId,
      walletAddress,
      chain,
      name,
      roleDescription,
      avatarCid,
      mcpConfigCid,
      mcpPorts,
      diapIdentity,
    })

    try {
      const response = await apiClient.post('/agent/create_agent', {
        // session_id: sessionId, // 让后端自动创建session
        wallet_address: walletAddress,
        chain: chain,
        name: name,
        role_description: roleDescription,
        avatar_cid: avatarCid,
        mcp_config_cid: mcpConfigCid,
        mcp_ports: mcpPorts,
        diap_identity: diapIdentity,
      })

      console.log('[AgentService] 智能体创建成功:', response.data)
      
      // 调试：检查后端返回的avatar相关字段
      if (response.data) {
        console.log('[AgentService] 后端返回的avatar数据:', {
          avatar_cid: response.data.avatar_cid,
          avatar_url: response.data.avatar_url,
          avatar: response.data.avatar,
          hasAvatarCid: !!response.data.avatar_cid,
          hasAvatarUrl: !!response.data.avatar_url,
          hasAvatar: !!response.data.avatar
        })
      }
      
      return response.data
    } catch (error) {
      console.error('[AgentService] 智能体创建失败:', error)
      throw new Error(`创建智能体失败: ${error.message}`)
    }
  }

  /**
   * Create DIAP identity for a session (统一入口，避免重复)
   * 使用新的DiapIntegrationService统一处理
   */
  async createDiapIdentity(sessionId, params = {}) {
    console.log('[AgentService] 使用统一的DIAP身份创建服务:', { sessionId, params })
    
    try {
      // 导入并使用DiapIntegrationService
      const { default: diapIntegrationService } = await import('./diapIntegrationService')
      
      const result = await diapIntegrationService.createDiapIdentity(sessionId, params)
      
      console.log('[AgentService] DIAP身份创建完成')
      return result
      
    } catch (error) {
      console.error('[AgentService] DIAP身份创建失败:', error)
      throw new Error(`DIAP身份创建失败: ${error.message}`)
    }
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