/**
 * Agent Service - 与 Cloudflare Workers (alou-edge) 通信
 */
import apiClient from './api'

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
   */
  async resolveAgent(target, sessionId) {
    const response = await apiClient.post(`${this.baseUrl}/api/agent/resolve`, {
      target,
      session_id: sessionId,
    })
    return response.data
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
  async createClaudeAgent(sessionId, walletAddress, chain, name) {
    const response = await apiClient.post(`${this.baseUrl}/api/agent/create-claude`, {
      session_id: sessionId,
      wallet_address: walletAddress,
      chain,
      name,
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
}

export default new AgentService()
