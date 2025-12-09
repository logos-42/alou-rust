/**
 * Agent Service - 与 Cloudflare Workers (alou-edge) 通信
 */
import apiClient from './api'

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
    
    // 使用默认值如果参数未提供（与 diapService.js 保持一致）
    const DEFAULT_IPFS_API = import.meta.env.VITE_IPFS_API_URL || 'http://127.0.0.1:5001'
    const DEFAULT_IPFS_GATEWAY = import.meta.env.VITE_IPFS_GATEWAY_URL || 'http://127.0.0.1:8080'
    
    // Tauri 期望驼峰格式的参数名（根据错误信息）
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
      
      // 尝试通过 Tauri 命令解析 IPNS
      const { invoke } = await import('@tauri-apps/api/core')
      
      const identity = await invoke('get_local_diap_identity', {
        ipnsName: normalizedName,
        ipfsApiUrl: DEFAULT_IPFS_API,
        ipfsGatewayUrl: DEFAULT_IPFS_GATEWAY,
      })
      
      console.log(`[AgentService] IPNS 解析结果:`, identity)
      
      // 使用解析得到的 CID 获取完整文档
      const loadResult = await this.loadAgentFromIpfs(identity.cid)
      
      if (loadResult.success) {
        // 合并 IPNS 信息
        loadResult.agent.ipns = normalizedName
        loadResult.agent.did = identity.did
        loadResult.source = 'ipns'
      }
      
      return loadResult
    } catch (error) {
      console.error(`[AgentService] 从 IPNS 加载智能体失败:`, error)
      
      // 降级：尝试直接通过 Gateway 访问
      try {
        console.log(`[AgentService] 尝试通过 Gateway 降级访问 IPNS...`)
        const normalizedName = ipnsName.replace(/^\/ipns\//, '')
        const gatewayUrl = `${DEFAULT_IPFS_GATEWAY}/ipns/${normalizedName}`
        
        const response = await fetch(gatewayUrl, {
          method: 'GET',
          headers: { 'Accept': 'application/json' },
        })
        
        if (!response.ok) {
          throw new Error(`Gateway 返回错误: ${response.status}`)
        }
        
        const didDocument = await response.json()
        const agentMetadata = this._parseDidDocumentToAgent(didDocument, { 
          ipns: ipnsName.startsWith('/ipns/') ? ipnsName : `/ipns/${ipnsName}`
        })
        
        return {
          success: true,
          agent: agentMetadata,
          didDocument,
          source: 'ipns-gateway',
        }
      } catch (fallbackError) {
        console.error(`[AgentService] Gateway 降级也失败:`, fallbackError)
        return {
          success: false,
          error: error.message || '加载失败',
          source: 'ipns',
        }
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
    console.log('[AgentService] 解析 DID 文档:', JSON.stringify(didDocument, null, 2))
    
    // 从 DID 文档中提取智能体信息
    const did = didDocument.id || didDocument.did || null
    
    // 从 service 数组中提取智能体配置
    const services = didDocument.service || []
    console.log('[AgentService] 找到 services:', services.length, services)
    
    // 尝试多种方式查找 AgentEndpoint 服务
    let agentService = services.find(s => 
      s.type === 'AgentEndpoint' || s.type === 'agent' || s.id?.includes('#agent')
    )
    
    // 如果没有找到，尝试查找第一个包含 serviceEndpoint 的服务
    if (!agentService) {
      agentService = services.find(s => s.serviceEndpoint)
    }
    
    // 如果还是没有找到，使用第一个服务
    if (!agentService && services.length > 0) {
      agentService = services[0]
    }
    
    console.log('[AgentService] 找到 agentService:', agentService)
    
    // 提取 serviceEndpoint，支持多种结构
    let serviceEndpoint = {}
    if (agentService) {
      // 如果 serviceEndpoint 是对象
      if (typeof agentService.serviceEndpoint === 'object' && agentService.serviceEndpoint !== null) {
        serviceEndpoint = agentService.serviceEndpoint
      }
      // 如果整个 service 对象就是配置
      else if (agentService.name || agentService.avatar_cid) {
        serviceEndpoint = agentService
      }
    }
    
    console.log('[AgentService] 提取的 serviceEndpoint:', serviceEndpoint)
    
    // 提取 metadata，支持多种位置
    const metadata = didDocument['alou:metadata'] || 
                     didDocument.metadata || 
                     didDocument['@context']?.metadata ||
                     {}
    
    console.log('[AgentService] 提取的 metadata:', metadata)
    
    // 提取 PubSub 主题
    const pubsubTopics = agentService?.pubsubTopics || 
                        agentService?.pubsub_topics ||
                        serviceEndpoint.pubsub_topics ||
                        []
    
    // 提取加密的 PeerID
    const encryptedPeerIdService = services.find(s => 
      s.type === 'EncryptedPeerID' || 
      s.type === 'encryptedPeerId' ||
      s.id?.includes('#encryptedPeerId')
    )
    
    // 提取名字，支持多种字段名和位置
    const name = serviceEndpoint.name || 
                 serviceEndpoint.display_name ||
                 metadata.agent_name ||
                 metadata.name ||
                 didDocument.name ||
                 '未命名智能体'
    
    // 提取头像，支持多种字段名
    const avatar_cid = serviceEndpoint.avatar_cid || 
                      serviceEndpoint.avatarCid ||
                      metadata.avatar_cid ||
                      metadata.avatarCid ||
                      null
    
    const avatar_url = serviceEndpoint.avatar_url ||
                      serviceEndpoint.avatarUrl ||
                      metadata.avatar_url ||
                      metadata.avatarUrl ||
                      null
    
    console.log('[AgentService] 解析结果 - name:', name, 'avatar_cid:', avatar_cid, 'avatar_url:', avatar_url)
    
    return {
      id: additionalInfo.cid || additionalInfo.ipns || did || `agent_${Date.now()}`,
      did,
      cid: additionalInfo.cid || null,
      ipns: additionalInfo.ipns || null,
      name,
      display_name: name,
      role_description: serviceEndpoint.description || 
                       serviceEndpoint.role_description ||
                       metadata.agent_description ||
                       metadata.description ||
                       metadata.role_description ||
                       '',
      avatar_cid,
      avatar_url,
      mcp_config_cid: serviceEndpoint.mcp_config_cid || 
                      serviceEndpoint.mcpConfigCid ||
                      metadata.mcp_config_cid ||
                      null,
      mcp_ports: serviceEndpoint.mcp_ports || 
                serviceEndpoint.mcpPorts ||
                metadata.mcp_ports ||
                [],
      agent_type: serviceEndpoint.agent_type || 
                 serviceEndpoint.agentType ||
                 metadata.agent_type ||
                 'claude_agent_sdk',
      customPrompt: serviceEndpoint.custom_prompt || 
                   serviceEndpoint.customPrompt ||
                   metadata.custom_prompt ||
                   null,
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
      // 检查是否在桌面环境（Tauri）
      const isDesktop = typeof window !== 'undefined' && window.__TAURI__ !== undefined
      
      if (isDesktop) {
        // 桌面版：使用 Tauri 命令（避免 CORS 问题）
        const { invoke } = await import('@tauri-apps/api/core')
        
        // 将 JSON 数据转换为 base64（与 agentAssetsService 保持一致）
        const base64Data = btoa(unescape(encodeURIComponent(jsonData)))
        
        const result = await invoke('ipfs_add_base64', {
          dataBase64: base64Data,
          fileName: `messages_${agentId}_${Date.now()}.json`,
          ipfsApiUrl: DEFAULT_IPFS_API,
        })
        
        console.log('[AgentService] 通过 Tauri 上传消息到 IPFS 成功:', result.cid)
        return result.cid
      } else {
        // 网页版：使用 IPFS HTTP API（需要配置 CORS）
        const response = await fetch(`${DEFAULT_IPFS_API}/api/v0/add`, {
          method: 'POST',
          body: new Blob([jsonData], { type: 'application/json' }),
        })
        
        if (!response.ok) {
          throw new Error(`IPFS 上传失败: ${response.status}`)
        }
        
        const result = await response.json()
        return result.Hash
      }
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
