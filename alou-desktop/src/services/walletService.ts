// ============================================
// Wallet Service - Handle wallet operations
// ============================================

import type { BlockchainNetworkInfo } from '@/shared/types';

// 钱包指令类型
interface WalletInstruction {
  type: 'wallet_operation' | 'query';
  method: string;
  params?: Record<string, any> | any[];
  fallback?: {
    method: string;
    params: Record<string, any>;
  };
  keys?: string[];
}

// 交易参数类型
interface TransactionParams {
  [key: string]: string | undefined;
  from?: string;
  to?: string;
  value?: string;
  data?: string;
  gas?: string;
  gasPrice?: string;
}

// 钱包信息类型
export interface CurrentWalletInfo {
  address: string;
  chainId: string;
  walletType: string;
}

class WalletService {
  private ethereum: any | null = null;
  private desktopWalletService: any = null;

  // 延迟加载 desktopWalletService 以避免循环依赖
  async getDesktopWalletService(): Promise<any> {
    if (!this.desktopWalletService && typeof window !== 'undefined') {
      try {
        // 动态导入以避免循环依赖
        const { desktopWalletService } = await import('@/services/desktopWalletService');
        this.desktopWalletService = desktopWalletService;
      } catch (error) {
        // 如果导入失败，返回 null
        console.warn('Failed to load desktopWalletService:', error);
      }
    }
    return this.desktopWalletService;
  }

  isDesktop(): boolean {
    if (typeof window === 'undefined') return false;
    return window.__TAURI__ !== undefined;
  }

  async getProvider(): Promise<any> {
    try {
      // 如果是桌面环境且没有浏览器钱包，尝试使用桌面钱包服务
      if (this.isDesktop() && typeof window !== 'undefined' && !window.ethereum) {
        try {
          const desktopService = await this.getDesktopWalletService();
          if (desktopService && desktopService.isConnected && desktopService.isConnected()) {
            // 返回桌面钱包的 provider（如果已连接）
            const provider = await desktopService.getProvider();
            if (provider) {
              return provider;
            }
          }
        } catch (error) {
          console.warn('Failed to get desktop wallet provider:', error);
          // 继续尝试浏览器钱包
        }
      }

      if (typeof window !== 'undefined' && window.ethereum) {
        this.ethereum = window.ethereum;
        return this.ethereum;
      }
    } catch (error) {
      console.error('Error in getProvider:', error);
    }

    return null;
  }

  /**
   * Request wallet accounts, optionally forcing the provider to show the
   * account selection dialog (MetaMask, etc.).
   */
  async requestAccounts({ forceSelect = false }: { forceSelect?: boolean } = {}): Promise<string[]> {
    if (!this.isWalletAvailable()) {
      throw new Error('Wallet not available');
    }

    const provider = await this.getProvider();
    let accounts: string[] = [];

    if (forceSelect) {
      try {
        await provider.request({
          method: 'wallet_requestPermissions',
          params: [{ eth_accounts: {} }],
        });
        accounts = await provider.request({ method: 'eth_accounts' });
      } catch (error: any) {
        if (error?.code === 4001) {
          // User rejected the permission request – rethrow so caller can handle
          throw error;
        }

        if (error?.code !== -32601) {
          console.warn(
            'wallet_requestPermissions failed, falling back to eth_requestAccounts',
            error,
          );
        }
        // If method not supported or other non-blocking error, fall back below
      }
    }

    if (!accounts || accounts.length === 0) {
      accounts = await provider.request({ method: 'eth_requestAccounts' });
    }

    if (!accounts || accounts.length === 0) {
      throw new Error('No wallet accounts available');
    }

    return accounts;
  }

  /**
   * Get currently authorized accounts without prompting the user.
   */
  async getAccounts(): Promise<string[]> {
    if (!this.isWalletAvailable()) {
      return [];
    }

    try {
      const provider = await this.getProvider();
      const accounts = await provider.request({ method: 'eth_accounts' });
      return Array.isArray(accounts) ? accounts : [];
    } catch (error) {
      console.warn('Failed to get wallet accounts silently:', error);
      return [];
    }
  }

  /**
   * Check if wallet is available
   */
  isWalletAvailable(): boolean {
    // 检查浏览器钱包
    if (typeof window !== 'undefined' && window.ethereum) {
      return true;
    }

    // 桌面环境：如果有保存的钱包地址，返回 true
    // 注意：这里不检查桌面钱包是否已连接，因为这会触发异步操作
    // 实际连接状态会在 getCurrentWalletInfo 中检查
    if (this.isDesktop()) {
      const savedAddress = typeof window !== 'undefined' ? localStorage.getItem('wallet_address') : null;
      return savedAddress !== null;
    }

    return false;
  }

  /**
   * Get current wallet info
   */
  async getCurrentWalletInfo(): Promise<CurrentWalletInfo | null> {
    try {
      const address = typeof window !== 'undefined' ? localStorage.getItem('wallet_address') : null;
      const walletType = typeof window !== 'undefined' ? localStorage.getItem('wallet_type') : null;
      const chainId = typeof window !== 'undefined' ? localStorage.getItem('wallet_chain_id') : null;

      if (!address) {
        return null;
      }

      // 如果是桌面环境且钱包类型是桌面钱包，尝试从桌面钱包服务获取最新信息
      if (this.isDesktop() && (walletType === 'walletconnect' || walletType === 'local')) {
        try {
          const desktopService = await this.getDesktopWalletService();
          if (desktopService && desktopService.isConnected()) {
            try {
              const accounts = await desktopService.getAccounts();
              const currentChainId = await desktopService.getCurrentChainId();
              if (accounts.length > 0) {
                return {
                  address: accounts[0],
                  chainId: currentChainId || chainId || '0x1',
                  walletType: walletType || 'walletconnect',
                };
              }
            } catch (error) {
              console.warn('Failed to get wallet info from desktop service:', error);
              // 继续使用 localStorage 中的信息
            }
          }
        } catch (error) {
          console.warn('Failed to load desktop wallet service:', error);
          // 继续使用 localStorage 中的信息
        }
      }

      return {
        address,
        chainId: chainId || '0x1',
        walletType: walletType || 'metamask',
      };
    } catch (error) {
      console.error('Failed to get wallet info:', error);
      return null;
    }
  }

  /**
   * Get current network chain ID
   */
  async getCurrentChainId(): Promise<string> {
    try {
      // 如果是桌面环境且使用桌面钱包，使用桌面钱包服务
      if (this.isDesktop()) {
        try {
          const desktopService = await this.getDesktopWalletService();
          const walletType = typeof window !== 'undefined' ? localStorage.getItem('wallet_type') : null;
          if (desktopService && (walletType === 'walletconnect' || walletType === 'local')) {
            try {
              return await desktopService.getCurrentChainId();
            } catch (error) {
              console.warn('Failed to get chain ID from desktop service:', error);
              // 继续尝试使用浏览器钱包或 localStorage
            }
          }
        } catch (error) {
          console.warn('Failed to load desktop wallet service:', error);
          // 继续尝试使用浏览器钱包或 localStorage
        }
      }

      // 使用浏览器钱包
      if (!this.isWalletAvailable()) {
        // 如果钱包不可用，尝试从 localStorage 读取
        const savedChainId = typeof window !== 'undefined' ? localStorage.getItem('wallet_chain_id') : null;
        if (savedChainId) {
          return savedChainId;
        }
        throw new Error('Wallet not available');
      }

      const provider = await this.getProvider();
      if (!provider) {
        const savedChainId = typeof window !== 'undefined' ? localStorage.getItem('wallet_chain_id') : null;
        if (savedChainId) {
          return savedChainId;
        }
        throw new Error('No wallet provider available');
      }

      // 处理不同的 provider 类型
      if (provider.request) {
        const chainId = await provider.request({ method: 'eth_chainId' });
        if (typeof window !== 'undefined') {
          localStorage.setItem('wallet_chain_id', chainId);
        }
        return chainId;
      }

      // 处理 ethers Provider
      if (provider.getNetwork) {
        const network = await provider.getNetwork();
        const chainId = `0x${network.chainId.toString(16)}`;
        if (typeof window !== 'undefined') {
          localStorage.setItem('wallet_chain_id', chainId);
        }
        return chainId;
      }

      // 回退到 localStorage
      const savedChainId = typeof window !== 'undefined' ? localStorage.getItem('wallet_chain_id') : null;
      if (savedChainId) {
        return savedChainId;
      }

      throw new Error('Unsupported wallet provider');
    } catch (error) {
      console.error('Failed to get chain ID:', error);
      // 最后尝试从 localStorage 读取
      const savedChainId = typeof window !== 'undefined' ? localStorage.getItem('wallet_chain_id') : null;
      if (savedChainId) {
        return savedChainId;
      }
      throw error;
    }
  }

  /**
   * Switch to a specific network
   */
  async switchNetwork(network: BlockchainNetworkInfo): Promise<boolean> {
    if (!this.isWalletAvailable()) {
      throw new Error('Wallet not available');
    }

    try {
      const provider = await this.getProvider();

      await provider.request({
        method: 'wallet_switchEthereumChain',
        params: [{ chainId: `0x${network.chainId.toString(16)}` }],
      });

      if (typeof window !== 'undefined') {
        localStorage.setItem('wallet_chain_id', `0x${network.chainId.toString(16)}`);
      }

      // Dispatch event for other components
      if (typeof window !== 'undefined') {
        window.dispatchEvent(
          new CustomEvent('network-changed', {
            detail: { chainId: network.chainId, network },
          }),
        );
      }

      return true;
    } catch (error: any) {
      // Network not added, try to add it
      if (error.code === 4902) {
        return await this.addNetwork(network);
      }
      console.error('Failed to switch network:', error);
      throw error;
    }
  }

  /**
   * Add a new network to wallet
   */
  async addNetwork(network: BlockchainNetworkInfo): Promise<boolean> {
    if (!this.isWalletAvailable()) {
      throw new Error('Wallet not available');
    }

    try {
      const provider = await this.getProvider();

      await provider.request({
        method: 'wallet_addEthereumChain',
        params: [
          {
            chainId: `0x${network.chainId.toString(16)}`,
            chainName: network.name,
            rpcUrls: [network.rpcUrl],
            nativeCurrency: network.nativeCurrency || {
              name: 'Ether',
              symbol: 'ETH',
              decimals: 18,
            },
          },
        ],
      });

      if (typeof window !== 'undefined') {
        localStorage.setItem('wallet_chain_id', String(network.chainId));
      }

      if (typeof window !== 'undefined') {
        window.dispatchEvent(
          new CustomEvent('network-changed', {
            detail: { chainId: network.chainId, network },
          }),
        );
      }

      return true;
    } catch (error) {
      console.error('Failed to add network:', error);
      throw error;
    }
  }

  /**
   * Get wallet balance
   */
  async getBalance(address?: string): Promise<string> {
    try {
      const targetAddress =
        address || (typeof window !== 'undefined' ? localStorage.getItem('wallet_address') : null);
      if (!targetAddress) {
        throw new Error('No wallet address available');
      }

      // 如果是桌面环境且使用桌面钱包，使用桌面钱包服务
      if (this.isDesktop() && typeof window !== 'undefined') {
        try {
          const desktopService = await this.getDesktopWalletService();
          const walletType = localStorage.getItem('wallet_type');
          if (desktopService && (walletType === 'walletconnect' || walletType === 'local')) {
            try {
              if (desktopService.isConnected && desktopService.isConnected()) {
                const balance = await desktopService.getBalance(targetAddress);
                if (balance !== undefined && balance !== null) {
                  return balance;
                }
              }
            } catch (error) {
              console.warn('Failed to get balance from desktop service:', error);
              // 继续尝试使用浏览器钱包或返回默认值
            }
          }
        } catch (error) {
          console.warn('Failed to load desktop wallet service:', error);
          // 继续尝试使用浏览器钱包
        }
      }

      // 使用浏览器钱包
      if (!this.isWalletAvailable()) {
        // 如果没有钱包，返回 0 而不是抛出错误
        return '0';
      }

      try {
        const provider = await this.getProvider();
        if (!provider) {
          // 如果没有 provider，返回 0
          return '0';
        }

        // 处理 ethers.js provider
        if (provider.request && typeof provider.request === 'function') {
          const balance = await provider.request({
            method: 'eth_getBalance',
            params: [targetAddress, 'latest'],
          });
          // Convert from wei to ether
          const ethBalance = (parseInt(balance as string, 16) / 1e18).toFixed(6);
          return ethBalance;
        }

        // 处理 ethers Provider
        if (provider.getBalance && typeof provider.getBalance === 'function') {
          const { ethers } = await import('ethers');
          const balance = await provider.getBalance(targetAddress);
          return ethers.formatEther(balance);
        }

        // 如果都不支持，返回默认值
        return '0';
      } catch (error) {
        console.warn('Failed to get balance from provider:', error);
        // 返回默认值而不是抛出错误
        return '0';
      }
    } catch (error) {
      console.error('Failed to get balance:', error);
      // 返回默认值而不是抛出错误，避免组件崩溃
      return '0';
    }
  }

  /**
   * Execute wallet instruction from agent
   */
  async executeInstruction(instruction: WalletInstruction): Promise<any> {
    if (!this.isWalletAvailable()) {
      throw new Error('Wallet not available');
    }

    try {
      if (instruction.type === 'wallet_operation') {
        if (instruction.method === 'eth_sendTransaction') {
          const txParams = Array.isArray(instruction.params)
            ? instruction.params[0]
            : instruction.params;
          if (!txParams) {
            throw new Error('Missing transaction params');
          }
          return await this.sendTransaction(txParams);
        }

        if (instruction.method === 'wallet_switchEthereumChain') {
          const params = instruction.params as Record<string, any>;
          const chainId = params?.chainId;
          if (!chainId) {
            throw new Error('Missing chainId parameter');
          }

          try {
            const provider = await this.getProvider();
            await provider.request({
              method: instruction.method,
              params: [{ chainId }],
            });

            if (typeof window !== 'undefined') {
              localStorage.setItem('wallet_chain_id', chainId);
            }
            return { success: true, chainId };
          } catch (error: any) {
            // Try fallback if available
            if (error.code === 4902 && instruction.fallback) {
              const provider = await this.getProvider();
              await provider.request({
                method: instruction.fallback.method,
                params: [instruction.fallback.params],
              });

              if (typeof window !== 'undefined') {
                localStorage.setItem('wallet_chain_id', chainId);
              }
              return { success: true, chainId, addedNetwork: true };
            }
            throw error;
          }
        }
      } else if (instruction.type === 'query') {
        if (instruction.method === 'eth_chainId') {
          return await this.getCurrentChainId();
        }

        if (instruction.method === 'eth_getBalance') {
          const addressParam = Array.isArray(instruction.params)
            ? instruction.params[0]
            : (instruction.params as Record<string, any>)?.address;
          const targetAddress =
            addressParam === 'current_wallet' || !addressParam ? undefined : addressParam;
          return await this.getBalance(targetAddress);
        }

        if (Array.isArray(instruction.keys) && typeof window !== 'undefined') {
          const result: Record<string, any> = {};
          instruction.keys.forEach((key: string) => {
            try {
              result[key] = window.localStorage?.getItem?.(key) ?? null;
            } catch (error) {
              console.warn('Failed to read wallet info key from localStorage:', key, error);
              result[key] = null;
            }
          });
          return result;
        }
      }

      throw new Error(`Unsupported instruction type: ${instruction.type}`);
    } catch (error) {
      console.error('Failed to execute instruction:', error);
      throw error;
    }
  }

  /**
   * Send transaction with current wallet (eth_sendTransaction)
   */
  async sendTransaction(rawParams: TransactionParams): Promise<string> {
    if (!this.isWalletAvailable()) {
      throw new Error('Wallet not available');
    }

    const provider = await this.getProvider();
    const params = { ...(rawParams || {}) };

    if (!params.from) {
      const address = typeof window !== 'undefined' ? localStorage.getItem('wallet_address') : null;
      if (!address) {
        throw new Error('Missing sender address. 请先连接钱包');
      }
      params.from = address;
    }

    // Remove undefined / null values to avoid RPC errors
    Object.keys(params).forEach((key: string) => {
      if (params[key] === undefined || params[key] === null || params[key] === '') {
        delete params[key];
      }
    });

    const txHash = await provider.request({
      method: 'eth_sendTransaction',
      params: [params],
    });

    if (typeof window !== 'undefined') {
      window.dispatchEvent(
        new CustomEvent('wallet-transaction-sent', {
          detail: { txHash, params },
        }),
      );
    }

    return txHash;
  }

  /**
   * Listen to wallet events
   */
  async onAccountsChanged(callback: (accounts: string[]) => void): Promise<void> {
    if (this.isWalletAvailable()) {
      const provider = await this.getProvider();
      if (provider && typeof provider.on === 'function') {
        provider.on('accountsChanged', callback);
      }
    }
  }

  /**
   * Listen to network changes
   */
  async onChainChanged(callback: (chainId: string) => void): Promise<void> {
    if (this.isWalletAvailable()) {
      const provider = await this.getProvider();
      if (provider && typeof provider.on === 'function') {
        provider.on('chainChanged', (chainId: string) => {
          if (typeof window !== 'undefined') {
            localStorage.setItem('wallet_chain_id', chainId);
          }
          callback(chainId);
        });
      }
    }
  }

  /**
   * Remove event listeners
   */
  async removeListener(event: string, callback: (...args: any[]) => void): Promise<void> {
    if (this.isWalletAvailable()) {
      const provider = await this.getProvider();
      if (provider && typeof provider.removeListener === 'function') {
        provider.removeListener(event, callback);
      }
    }
  }
}

export const walletService = new WalletService();
export default walletService;
