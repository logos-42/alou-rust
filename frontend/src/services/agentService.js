/**
 * Agent Service - 与 Cloudflare Workers (alou-edge) 通信
 */
import apiClient from './api'

const DEFAULT_IPFS_API = import.meta.env.VITE_IPFS_API_URL || 'http://127.0.0.1:5001'
const DEFAULT_IPFS_GATEWAY = import.meta.env.VITE_IPFS_GATEWAY_URL || 'http://127.0.0.1:8080'

export class AgentService {
  constructor() {
    const defaultBase =
      import.meta.env.VITE_API_BASE_URL ||
      (import.meta.env.DEV ? 'http://127.0.0.1:8787' : 'https://alou-edge.yuanjieliu65.workers.dev')
    this.baseUrl = import.meta.env.VITE_AGENT_API_URL || defaultBase
  }

  /**
   * Create a new chat session
   */
  async createSession(walletAddress) {
    const response = await apiClient.post(`${this.baseUrl}/api/session`, {
      wallet_address: walletAddress,
    })
    return response.data
  }

  /**
   * Get session info
   */
  async getSession(sessionId) {
    const response = await apiClient.get(`${this.baseUrl}/api/session/${sessionId}`)
    return response.data
  }

  /**
   * Delete session
   */
  async deleteSession(sessionId) {
    await apiClient.delete(`${this.baseUrl}/api/session/${sessionId}`)
  }

  /**
   * Send message to agent
   */
  async sendMessage(sessionId, message, walletAddress) {
    const response = await apiClient.post(`${this.baseUrl}/api/agent/chat`, {
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
    const response = await apiClient.post(`${this.baseUrl}/api/agent/resolve`, {
      target,
      session_id: sessionId,
    })
    
    const resolvedAgent = response.data
    
    // 如果包含 did_document，解析它来提取智能体元数据
    if (resolvedAgent.did_document) {
      const parsedAgent = this._parseDidDocumentToAgent(resolvedAgent.did_document, {
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
    const response = await apiClient.post(`${this.baseUrl}/api/agent/search`, {
      query,
    })
    return response.data
  }

  /**
   * Get balance
   */
  async getBalance(address, chain, tokenAddress) {
    const response = await apiClient.post(`${this.baseUrl}/api/blockchain/balance`, {
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
    const response = await apiClient.post(`${this.baseUrl}/api/blockchain/transaction/build`, {
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
    const response = await apiClient.post(`${this.baseUrl}/api/blockchain/transaction/broadcast`, {
      signed_tx: signedTx,
      chain,
    })
    return response.data
  }

  /**
   * Get transaction status
   */
  async getTransactionStatus(txHash, chain) {
    const response = await apiClient.get(
      `${this.baseUrl}/api/blockchain/transaction/${txHash}?chain=${chain}`,
    )
    return response.data
  }

  /**
   * Health check
   */
  async healthCheck() {
    const response = await apiClient.get(`${this.baseUrl}/api/health`)
    return response.data
  }

  /**
   * Get service status
   */
  async getStatus() {
    const response = await apiClient.get(`${this.baseUrl}/api/status`)
    return response.data
  }

  /**
   * Record agent wallet transaction history
   */
  async recordAgentTransaction(sessionId, chain, transaction) {
    if (!sessionId || !chain || !transaction) {
      throw new Error('Missing parameters for recordAgentTransaction')
    }

    await apiClient.post(`${this.baseUrl}/api/agent/wallet`, {
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

    await apiClient.post(`${this.baseUrl}/api/agent/wallet`, {
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
    const response = await apiClient.post(`${this.baseUrl}/api/agent/create-claude`, {
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
   * Create DIAP identity for a session
   */
  async createDiapIdentity(sessionId) {
    const response = await apiClient.post(`${this.baseUrl}/api/agent/diap/create-identity`, {
      session_id: sessionId,
    })
    return response.data
  }

  /**
   * Get DIAP identity for a session
   */
  async getDiapIdentity(sessionId) {
    const response = await apiClient.post(`${this.baseUrl}/api/agent/diap/get-identity`, {
      session_id: sessionId,
    })
    return response.data
  }

  /**
   * Register agent to DIAP network on-chain
   * Returns encoded transaction that needs to be signed and broadcast
   */
  async registerAgentOnChain(sessionId, network, stakeAmount, useAa = false, salt = 0) {
    const response = await apiClient.post(`${this.baseUrl}/api/agent/diap/register-onchain`, {
      session_id: sessionId,
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
    console.log(`[AgentService] 从 IPFS 加载智能体: ${cid}`)
    
    try {
      // 通过 IPFS Gateway 获取 DID 文档
      const gatewayUrl = `${DEFAULT_IPFS_GATEWAY}/ipfs/${cid}`
      const response = await fetch(gatewayUrl, {
        method: 'GET',
        headers: { 'Accept': 'application/json' },
      })
      
      if (!response.ok) {
        throw new Error(`IPFS Gateway 返回错误: ${response.status} ${response.statusText}`)
      }
      
      const didDocument = await response.json()
      console.log(`[AgentService] 获取到 DID 文档:`, didDocument)
      
      // 解析 DID 文档，提取智能体元数据
      const agentMetadata = this._parseDidDocumentToAgent(didDocument, { cid })
      
      return {
        success: true,
        agent: agentMetadata,
        didDocument,
        source: 'ipfs',
      }
    } catch (error) {
      console.error(`[AgentService] 从 IPFS 加载智能体失败:`, error)
      return {
        success: false,
        error: error.message || '加载失败',
        source: 'ipfs',
      }
    }
  }

  /**
   * 从 IPNS 名称加载智能体元数据
   * @param {string} ipnsName - IPNS 名称（可以是 /ipns/xxx 或 k51xxx 格式）
   * @returns {Promise<Object>} 智能体元数据
   */
  async loadAgentFromIpns(ipnsName) {
    console.log(`[AgentService] 从 IPNS 加载智能体: ${ipnsName}`)
    
    try {
      // 规范化 IPNS 名称
      const normalizedName = ipnsName.startsWith('/ipns/') 
        ? ipnsName 
        : `/ipns/${ipnsName}`
      
      // 前端版：直接通过 Gateway 访问 IPNS
      const normalizedNameForGateway = ipnsName.replace(/^\/ipns\//, '')
      const gatewayUrl = `${DEFAULT_IPFS_GATEWAY}/ipns/${normalizedNameForGateway}`
      
      const response = await fetch(gatewayUrl, {
        method: 'GET',
        headers: { 'Accept': 'application/json' },
      })
      
      if (!response.ok) {
        throw new Error(`Gateway 返回错误: ${response.status}`)
      }
      
      const didDocument = await response.json()
      const agentMetadata = this._parseDidDocumentToAgent(didDocument, { 
        ipns: normalizedName
      })
      
      return {
        success: true,
        agent: agentMetadata,
        didDocument,
        source: 'ipns-gateway',
      }
    } catch (error) {
      console.error(`[AgentService] 从 IPNS 加载智能体失败:`, error)
      return {
        success: false,
        error: error.message || '加载失败',
        source: 'ipns',
      }
    }
  }

  /**
   * 从 CID 或 IPNS 智能识别并加载智能体
   * @param {string} target - CID 或 IPNS 名称
   * @returns {Promise<Object>} 智能体元数据
   */
  async loadAgentFromNetwork(target) {
    if (!target || typeof target !== 'string') {
      return { success: false, error: '无效的目标标识' }
    }
    
    const trimmed = target.trim()
    
    // 判断是 IPNS 还是 CID
    const isIpns = trimmed.startsWith('/ipns/') || 
                   trimmed.startsWith('k51') || 
                   trimmed.startsWith('k2')
    
    const isCid = trimmed.startsWith('Qm') || 
                  trimmed.startsWith('bafy') || 
                  trimmed.startsWith('bafk')
    
    if (isIpns) {
      return this.loadAgentFromIpns(trimmed)
    } else if (isCid) {
      return this.loadAgentFromIpfs(trimmed)
    } else {
      // 尝试作为 IPNS 解析
      console.log(`[AgentService] 未知格式，尝试作为 IPNS 解析: ${trimmed}`)
      return this.loadAgentFromIpns(trimmed)
    }
  }

  /**
   * 解析 DID 文档，提取智能体元数据
   * @private
   */
  _parseDidDocumentToAgent(didDocument, additionalInfo = {}) {
    // 从 DID 文档中提取智能体信息
    const did = didDocument.id || didDocument.did || null
    
    // 从 service 数组中提取智能体配置
    const services = didDocument.service || []
    const agentService = services.find(s => 
      s.type === 'AgentEndpoint' || s.id?.includes('#agent')
    )
    
    const serviceEndpoint = agentService?.serviceEndpoint || {}
    
    // 提取 metadata
    const metadata = didDocument['alou:metadata'] || {}
    
    // 提取 PubSub 主题
    const pubsubTopics = agentService?.pubsubTopics || []
    
    // 提取加密的 PeerID
    const encryptedPeerIdService = services.find(s => 
      s.type === 'EncryptedPeerID' || s.id?.includes('#encryptedPeerId')
    )
    
    return {
      id: additionalInfo.cid || additionalInfo.ipns || did || `agent_${Date.now()}`,
      did,
      cid: additionalInfo.cid || null,
      ipns: additionalInfo.ipns || null,
      name: serviceEndpoint.name || metadata.agent_name || '未命名智能体',
      display_name: serviceEndpoint.name || metadata.agent_name || '未命名智能体',
      role_description: serviceEndpoint.description || metadata.agent_description || '',
      avatar_cid: serviceEndpoint.avatar_cid || null,
      avatar_url: serviceEndpoint.avatar_url || null,
      mcp_config_cid: serviceEndpoint.mcp_config_cid || null,
      mcp_ports: serviceEndpoint.mcp_ports || [],
      agent_type: serviceEndpoint.agent_type || 'claude_agent_sdk',
      customPrompt: serviceEndpoint.custom_prompt || null,
      pubsub_topics: pubsubTopics,
      encrypted_peer_id: encryptedPeerIdService?.serviceEndpoint || null,
      created_at: didDocument.created ? new Date(didDocument.created).getTime() : Date.now(),
      updated_at: Date.now(),
      imported_from_network: true,
      diapIdentity: {
        did,
        cid: additionalInfo.cid,
        ipns: additionalInfo.ipns,
        public_key: didDocument.verificationMethod?.[0]?.publicKeyMultibase || null,
      },
    }
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
    
    try {
      // 使用 IPFS HTTP API 上传
      const formData = new FormData()
      const blob = new Blob([jsonData], { type: 'application/json' })
      formData.append('file', blob, 'messages.json')
      
      const response = await fetch(`${DEFAULT_IPFS_API}/api/v0/add`, {
        method: 'POST',
        body: formData,
      })
      
      if (!response.ok) {
        throw new Error(`IPFS 上传失败: ${response.status}`)
      }
      
      const result = await response.json()
      return result.Hash
    } catch (error) {
      console.error('[AgentService] 上传消息到 IPFS 失败:', error)
      throw error
    }
  }

  /**
   * 从 IPFS 加载消息
   * @param {string} cid - 消息 CID
   * @returns {Promise<Object>} 消息数据
   */
  async loadMessagesFromIpfs(cid) {
    try {
      const gatewayUrl = `${DEFAULT_IPFS_GATEWAY}/ipfs/${cid}`
      const response = await fetch(gatewayUrl, {
        method: 'GET',
        headers: { 'Accept': 'application/json' },
      })
      
      if (!response.ok) {
        throw new Error(`IPFS Gateway 返回错误: ${response.status}`)
      }
      
      const data = await response.json()
      return data
    } catch (error) {
      console.error('[AgentService] 从 IPFS 加载消息失败:', error)
      throw error
    }
  }
}

export default new AgentService()
