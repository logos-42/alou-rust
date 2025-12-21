/**
 * Agent Service - 与 Cloudflare Workers (alou-edge) 通信
 * 重构后：使用组合模式，将职责分离到不同的服务
 */
import apiClient from './api'
import agentResolverService from './agentResolverService'
import ipfsContentService from './ipfsContentService'
import { parseDidDocumentToAgent } from './didDocumentParser'

const DEFAULT_IPFS_API = import.meta.env.VITE_IPFS_API_URL || 'http://127.0.0.1:5001'
const DEFAULT_IPFS_GATEWAY = import.meta.env.VITE_IPFS_GATEWAY_URL || 'http://127.0.0.1:8080'

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
   */
  async sendMessage(sessionId, message, walletAddress) {
    const response = await apiClient.post('/agent/chat', {
      session_id: sessionId,
      message,
      wallet_address: walletAddress,
    })
    return response.data
  }

  /**
   * Resolve agent metadata via DIAP/IPFS
   * 解析后端返回的 did_document 来提取智能体元数据（名称、头像等）
   */
  async resolveAgent(target, sessionId) {
    const response = await apiClient.post('/agent/resolve', {
      target,
      session_id: sessionId,
    })
    
    const resolvedAgent = response.data
    
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
