/**
 * Agent Service - 与 Cloudflare Workers (alou-edge) 通信
 */
import apiClient from './api'

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
   */
  async resolveAgent(target, sessionId) {
    const response = await apiClient.post('/agent/resolve', {
      target,
      session_id: sessionId,
    })
    return response.data
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
          const response = await apiClient.post('/api/agent/diap/get-identity-by-session', {
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
}

export default new AgentService()
