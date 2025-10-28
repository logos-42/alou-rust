// ============================================
// Blockchain Service - Query real blockchain data
// ============================================

export interface BalanceInfo {
  address: string
  balance: string
  symbol: string
  chainId: string
  network: string
}

export interface Transaction {
  hash: string
  from: string
  to: string
  value: string
  timestamp: number
  status: 'confirmed' | 'pending' | 'failed'
  type: 'send' | 'receive' | 'contract'
  token: string
}

class BlockchainService {
  private ethereum: any

  constructor() {
    this.ethereum = (window as any).ethereum
  }

  /**
   * Get real balance from blockchain
   */
  async getBalance(address: string, chainId?: string): Promise<BalanceInfo | null> {
    if (!this.ethereum) {
      return null
    }

    try {
      const currentChainId = chainId || await this.ethereum.request({ method: 'eth_chainId' })
      
      const balance = await this.ethereum.request({
        method: 'eth_getBalance',
        params: [address, 'latest']
      })

      // Convert from wei to ether
      const ethBalance = (parseInt(balance, 16) / 1e18).toFixed(6)
      
      const networkName = this.getNetworkName(currentChainId)
      const symbol = this.getNetworkSymbol(currentChainId)

      return {
        address,
        balance: ethBalance,
        symbol,
        chainId: currentChainId,
        network: networkName
      }
    } catch (error) {
      console.error('Failed to get balance:', error)
      return null
    }
  }

  /**
   * Get transaction history (simplified version)
   */
  async getTransactionHistory(address: string): Promise<Transaction[]> {
    // In production, use Etherscan API or similar service
    // For now, return mock data
    return []
  }

  /**
   * Get network name from chain ID
   */
  private getNetworkName(chainId: string): string {
    const networks: Record<string, string> = {
      '0x1': 'Ethereum Mainnet',
      '0xaa36a7': 'Ethereum Sepolia',
      '0x14a34': 'Base Sepolia',
      '0x13882': 'Polygon Amoy',
      '0x2105': 'Base Mainnet',
      '0x89': 'Polygon Mainnet'
    }
    return networks[chainId] || 'Unknown Network'
  }

  /**
   * Get network symbol from chain ID
   */
  private getNetworkSymbol(chainId: string): string {
    const symbols: Record<string, string> = {
      '0x1': 'ETH',
      '0xaa36a7': 'ETH',
      '0x14a34': 'ETH',
      '0x13882': 'MATIC',
      '0x2105': 'ETH',
      '0x89': 'MATIC'
    }
    return symbols[chainId] || 'ETH'
  }

  /**
   * Query agent's wallet from backend
   */
  async getAgentWallet(sessionId: string, chain: string): Promise<any> {
    const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 
      (import.meta.env.DEV ? 'http://localhost:8787' : 'https://alou-edge.yuanjieliu65.workers.dev')

    try {
      const response = await fetch(`${API_BASE_URL}/api/agent/wallet`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          session_id: sessionId,
          action: 'get_wallet',
          chain
        })
      })

      if (response.ok) {
        const data = await response.json()
        return data.wallet
      }
      return null
    } catch (error) {
      console.error('Failed to get agent wallet:', error)
      return null
    }
  }

  /**
   * List all agent wallets
   */
  async listAgentWallets(sessionId: string): Promise<any[]> {
    const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 
      (import.meta.env.DEV ? 'http://localhost:8787' : 'https://alou-edge.yuanjieliu65.workers.dev')

    try {
      const response = await fetch(`${API_BASE_URL}/api/agent/wallet`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          session_id: sessionId,
          action: 'list_wallets'
        })
      })

      if (response.ok) {
        const data = await response.json()
        return data.wallets || []
      }
      return []
    } catch (error) {
      console.error('Failed to list agent wallets:', error)
      return []
    }
  }
}

export const blockchainService = new BlockchainService()
