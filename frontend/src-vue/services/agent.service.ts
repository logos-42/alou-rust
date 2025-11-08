/**
 * Agent Service - 与 Cloudflare Workers (alou-edge) 通信
 */
import apiClient from './api'

interface ChatMessage {
  role: string
  content: string
}

interface ChatResponse {
  content: string
  session_id: string
  tool_calls?: Array<{
    id: string
    name: string
    result: any
  }>
}

interface SessionResponse {
  session_id: string
}

interface BalanceResponse {
  address: string
  chain: string
  balance: string
}

interface TransactionData {
  from: string
  to: string
  value: string
  data?: string
  gas?: string
  gas_price?: string
  nonce?: string
}

interface BroadcastResponse {
  tx_hash: string
  chain: string
}

interface TransactionReceipt {
  tx_hash: string
  status: string
  block_number?: string
}

export class AgentService {
  private baseUrl: string

  constructor() {
    this.baseUrl = import.meta.env.VITE_AGENT_API_URL || 'https://api.alou.onl'
  }

  /**
   * Create a new chat session
   */
  async createSession(walletAddress?: string): Promise<SessionResponse> {
    const response = await apiClient.post(`${this.baseUrl}/api/session`, {
      wallet_address: walletAddress
    })
    return response.data
  }

  /**
   * Get session info
   */
  async getSession(sessionId: string): Promise<any> {
    const response = await apiClient.get(`${this.baseUrl}/api/session/${sessionId}`)
    return response.data
  }

  /**
   * Delete session
   */
  async deleteSession(sessionId: string): Promise<void> {
    await apiClient.delete(`${this.baseUrl}/api/session/${sessionId}`)
  }

  /**
   * Send message to agent
   */
  async sendMessage(
    sessionId: string,
    message: string,
    walletAddress?: string
  ): Promise<ChatResponse> {
    const response = await apiClient.post(`${this.baseUrl}/api/agent/chat`, {
      session_id: sessionId,
      message,
      wallet_address: walletAddress
    })
    return response.data
  }

  /**
   * Get balance
   */
  async getBalance(
    address: string,
    chain: string,
    tokenAddress?: string
  ): Promise<BalanceResponse> {
    const response = await apiClient.post(`${this.baseUrl}/api/blockchain/balance`, {
      address,
      chain,
      token_address: tokenAddress
    })
    return response.data
  }

  /**
   * Build transaction
   */
  async buildTransaction(
    from: string,
    to: string,
    value: number,
    chain: string
  ): Promise<TransactionData> {
    const response = await apiClient.post(`${this.baseUrl}/api/blockchain/transaction/build`, {
      from,
      to,
      value,
      chain
    })
    return response.data
  }

  /**
   * Broadcast transaction
   */
  async broadcastTransaction(signedTx: string, chain: string): Promise<BroadcastResponse> {
    const response = await apiClient.post(`${this.baseUrl}/api/blockchain/transaction/broadcast`, {
      signed_tx: signedTx,
      chain
    })
    return response.data
  }

  /**
   * Get transaction status
   */
  async getTransactionStatus(txHash: string, chain: string): Promise<TransactionReceipt> {
    const response = await apiClient.get(
      `${this.baseUrl}/api/blockchain/transaction/${txHash}?chain=${chain}`
    )
    return response.data
  }

  /**
   * Health check
   */
  async healthCheck(): Promise<{ status: string; timestamp: string }> {
    const response = await apiClient.get(`${this.baseUrl}/api/health`)
    return response.data
  }

  /**
   * Get service status
   */
  async getStatus(): Promise<any> {
    const response = await apiClient.get(`${this.baseUrl}/api/status`)
    return response.data
  }
}

export default new AgentService()
