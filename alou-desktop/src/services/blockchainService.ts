// ============================================
// Blockchain Service - Query real blockchain data
// ============================================

interface NetworkInfo {
  chainId: string;
  name: string;
  symbol: string;
}

// 余额信息
export interface BalanceInfo {
  address: string;
  balance: string;
  symbol: string;
  chainId: string;
  network: string;
}

// 代币余额
export interface TokenBalance {
  address: string;
  tokenAddress: string;
  balance: string;
  symbol: string;
  decimals: number;
}

// 代理钱包
export interface AgentWallet {
  address: string;
  chain: string;
  balance: string;
  createdAt: string;
}

// API 基础 URL
const getApiBaseUrl = (): string => {
  const apiBaseUrl =
    import.meta.env.VITE_API_BASE_URL ||
    (import.meta.env.DEV ? '' : 'https://alou-edge.yuanjieliu65.workers.dev');
  return apiBaseUrl ? `${apiBaseUrl}/api` : '/api';
};

class BlockchainService {
  private ethereum: any | null = null;
  private apiBaseUrl: string;

  constructor() {
    this.ethereum = typeof window !== 'undefined' ? (window as any).ethereum : null;
    this.apiBaseUrl = getApiBaseUrl();
  }

  /**
   * Get real balance from blockchain
   */
  async getBalance(address: string, chainId?: string): Promise<BalanceInfo | null> {
    if (!this.ethereum) {
      return null;
    }

    try {
      const currentChainId = chainId || (await this.ethereum.request({ method: 'eth_chainId' }));

      const balance = await this.ethereum.request({
        method: 'eth_getBalance',
        params: [address, 'latest'],
      });

      // Convert from wei to ether
      const ethBalance = (parseInt(balance, 16) / 1e18).toFixed(6);

      const networkName = this.getNetworkName(currentChainId);
      const symbol = this.getNetworkSymbol(currentChainId);

      return {
        address,
        balance: ethBalance,
        symbol,
        chainId: currentChainId,
        network: networkName,
      };
    } catch (error) {
      console.error('Failed to get balance:', error);
      return null;
    }
  }

  /**
   * Get transaction history (simplified version)
   */
  async getTransactionHistory(): Promise<[]> {
    // In production, use Etherscan API or similar service
    // For now, return mock data
    return [];
  }

  async getTokenBalance(address: string, chain: string, tokenAddress: string): Promise<TokenBalance> {
    if (!address || !tokenAddress) {
      throw new Error('address 和 tokenAddress 不能为空');
    }

    const payload = {
      address,
      chain: chain || 'ethereum',
      token_address: tokenAddress,
    };

    const response = await fetch(`${this.apiBaseUrl}/blockchain/balance`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(payload),
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({}));
      throw new Error(error?.error || 'Failed to load token balance');
    }

    return response.json();
  }

  async listSupportedTokens(chain?: string): Promise<any[]> {
    const url = new URL(`${this.apiBaseUrl}/api/blockchain/tokens`);
    if (chain) {
      url.searchParams.set('chain', chain);
    }

    const response = await fetch(url.toString(), {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({}));
      throw new Error(error?.error || 'Failed to load supported tokens');
    }

    return response.json();
  }

  /**
   * Get network name from chain ID
   */
  getNetworkName(chainId: string): string {
    const networks: Record<string, string> = {
      '0x1': 'Ethereum Mainnet',
      '0xaa36a7': 'Ethereum Sepolia',
      '0x14a34': 'Base Sepolia',
      '0x13882': 'Polygon Amoy',
      '0x2105': 'Base Mainnet',
      '0x89': 'Polygon Mainnet',
    };
    return networks[chainId] || 'Unknown Network';
  }

  /**
   * Get network symbol from chain ID
   */
  getNetworkSymbol(chainId: string): string {
    const symbols: Record<string, string> = {
      '0x1': 'ETH',
      '0xaa36a7': 'ETH',
      '0x14a34': 'ETH',
      '0x13882': 'MATIC',
      '0x2105': 'ETH',
      '0x89': 'MATIC',
    };
    return symbols[chainId] || 'ETH';
  }

  /**
   * Query agent's wallet from backend
   */
  async getAgentWallet(sessionId: string, chain?: string): Promise<AgentWallet | null> {
    const baseUrl = getApiBaseUrl();

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
      });

      if (response.ok) {
        const data = await response.json();
        return data.wallet;
      }
      return null;
    } catch (error) {
      console.error('Failed to get agent wallet:', error);
      return null;
    }
  }

  /**
   * List all agent wallets
   */
  async listAgentWallets(sessionId: string): Promise<AgentWallet[]> {
    const baseUrl = getApiBaseUrl();

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
      });

      if (response.ok) {
        const data = await response.json();
        return data.wallets || [];
      }
      return [];
    } catch (error) {
      console.error('Failed to list agent wallets:', error);
      return [];
    }
  }
}

export const blockchainService = new BlockchainService();
export default blockchainService;
