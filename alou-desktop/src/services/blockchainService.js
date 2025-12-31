// ============================================
// Blockchain Service - Query real blockchain data
// ============================================

const API_BASE_URL =
  import.meta.env.VITE_API_BASE_URL ||
  (import.meta.env.DEV ? '' : 'https://alou-edge.yuanjieliu65.workers.dev')

class BlockchainService {
  constructor() {
    this.ethereum = typeof window !== 'undefined' ? window.ethereum : null
    this.apiBaseUrl = API_BASE_URL ? `${API_BASE_URL}/api` : '/api'
  }

  /**
   * Get real balance from blockchain
   */
  async getBalance(address, chainId) {
    if (!this.ethereum) {
      return null
    }

    try {
      const currentChainId = chainId || (await this.ethereum.request({ method: 'eth_chainId' }))

      const balance = await this.ethereum.request({
        method: 'eth_getBalance',
        params: [address, 'latest'],
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
        network: networkName,
      }
    } catch (error) {
      console.error('Failed to get balance:', error)
      return null
    }
  }

  /**
   * Get transaction history (simplified version)
   */
  async getTransactionHistory() {
    // In production, use Etherscan API or similar service
    // For now, return mock data
    return []
  }

  async getTokenBalance(address, chain, tokenAddress) {
    if (!address || !tokenAddress) {
      throw new Error('address 和 tokenAddress 不能为空')
    }

    const payload = {
      address,
      chain: chain || 'ethereum',
      token_address: tokenAddress,
    }

    const response = await fetch(`${this.apiBaseUrl}/blockchain/balance`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(payload),
    })

    if (!response.ok) {
      const error = await response.json().catch(() => ({}))
      throw new Error(error?.error || 'Failed to load token balance')
    }

    return response.json()
  }

  async listSupportedTokens(chain) {
    const url = new URL(`${this.apiBaseUrl}/api/blockchain/tokens`)
    if (chain) {
      url.searchParams.set('chain', chain)
    }

    const response = await fetch(url.toString(), {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
    })

    if (!response.ok) {
      const error = await response.json().catch(() => ({}))
      throw new Error(error?.error || 'Failed to load supported tokens')
    }

    return response.json()
  }

  /**
   * Get network name from chain ID
   */
  getNetworkName(chainId) {
    const networks = {
      '0x1': 'Ethereum Mainnet',
      '0xaa36a7': 'Ethereum Sepolia',
      '0x14a34': 'Base Sepolia',
      '0x13882': 'Polygon Amoy',
      '0x2105': 'Base Mainnet',
      '0x89': 'Polygon Mainnet',
    }
    return networks[chainId] || 'Unknown Network'
  }

  /**
   * Get network symbol from chain ID
   */
  getNetworkSymbol(chainId) {
    const symbols = {
      '0x1': 'ETH',
      '0xaa36a7': 'ETH',
      '0x14a34': 'ETH',
      '0x13882': 'MATIC',
      '0x2105': 'ETH',
      '0x89': 'MATIC',
    }
    return symbols[chainId] || 'ETH'
  }

  /**
   * Query agent's wallet from backend
   */
  async getAgentWallet(sessionId, chain) {
    const API_BASE_URL =
      import.meta.env.VITE_API_BASE_URL ||
      (import.meta.env.DEV ? '' : 'https://alou-edge.yuanjieliu65.workers.dev')
    const baseUrl = API_BASE_URL ? `${API_BASE_URL}/api` : '/api'

    try {
      const response = await fetch(`${baseUrl}/agent/wallet`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          session_id: sessionId,
          action: 'get_wallet',
          chain,
        }),
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
  async listAgentWallets(sessionId) {
    const API_BASE_URL =
      import.meta.env.VITE_API_BASE_URL ||
      (import.meta.env.DEV ? '' : 'https://alou-edge.yuanjieliu65.workers.dev')
    const baseUrl = API_BASE_URL ? `${API_BASE_URL}/api` : '/api'

    try {
      const response = await fetch(`${baseUrl}/agent/wallet`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          session_id: sessionId,
          action: 'list_wallets',
        }),
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
