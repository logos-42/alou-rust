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
   * Get DIAP identity from IPNS (using local Tauri command)
   */
  async getDiapIdentity(ipnsName, ipfsApiUrl, ipfsGatewayUrl) {
    const { invoke } = await import('@tauri-apps/api/core')
    const response = await invoke('get_local_diap_identity', {
      ipns_name: ipnsName,
      ipfs_api_url: ipfsApiUrl,
      ipfs_gateway_url: ipfsGatewayUrl,
    })
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
