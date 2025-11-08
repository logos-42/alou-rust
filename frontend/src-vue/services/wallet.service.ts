// ============================================
// Wallet Service - Handle wallet operations
// ============================================

export interface Network {
  chainId: string
  name: string
  type: string
  icon: string
  rpcUrl: string
  nativeCurrency?: {
    name: string
    symbol: string
    decimals: number
  }
}

export interface WalletInfo {
  address: string
  chainId: string
  walletType: string
  balance?: string
}

export interface WalletInstruction {
  type: 'wallet_operation' | 'query'
  method?: string
  params?: any
  fallback?: {
    method: string
    params: any
  }
}

class WalletService {
  private ethereum: any

  constructor() {
    this.ethereum = (window as any).ethereum
  }

  /**
   * Check if wallet is available
   */
  isWalletAvailable(): boolean {
    return typeof this.ethereum !== 'undefined'
  }

  /**
   * Get current wallet info
   */
  async getCurrentWalletInfo(): Promise<WalletInfo | null> {
    if (!this.isWalletAvailable()) {
      return null
    }

    try {
      const address = localStorage.getItem('wallet_address')
      const walletType = localStorage.getItem('wallet_type')
      const chainId = localStorage.getItem('wallet_chain_id')

      if (!address) {
        return null
      }

      return {
        address,
        chainId: chainId || '0x1',
        walletType: walletType || 'metamask'
      }
    } catch (error) {
      console.error('Failed to get wallet info:', error)
      return null
    }
  }

  /**
   * Get current network chain ID
   */
  async getCurrentChainId(): Promise<string> {
    if (!this.isWalletAvailable()) {
      throw new Error('Wallet not available')
    }

    try {
      const chainId = await this.ethereum.request({ method: 'eth_chainId' })
      localStorage.setItem('wallet_chain_id', chainId)
      return chainId
    } catch (error) {
      console.error('Failed to get chain ID:', error)
      throw error
    }
  }

  /**
   * Switch to a specific network
   */
  async switchNetwork(network: Network): Promise<boolean> {
    if (!this.isWalletAvailable()) {
      throw new Error('Wallet not available')
    }

    try {
      await this.ethereum.request({
        method: 'wallet_switchEthereumChain',
        params: [{ chainId: network.chainId }],
      })

      localStorage.setItem('wallet_chain_id', network.chainId)
      
      // Dispatch event for other components
      window.dispatchEvent(new CustomEvent('network-changed', {
        detail: { chainId: network.chainId, network }
      }))

      return true
    } catch (error: any) {
      // Network not added, try to add it
      if (error.code === 4902) {
        return await this.addNetwork(network)
      }
      console.error('Failed to switch network:', error)
      throw error
    }
  }

  /**
   * Add a new network to wallet
   */
  async addNetwork(network: Network): Promise<boolean> {
    if (!this.isWalletAvailable()) {
      throw new Error('Wallet not available')
    }

    try {
      await this.ethereum.request({
        method: 'wallet_addEthereumChain',
        params: [{
          chainId: network.chainId,
          chainName: network.name,
          rpcUrls: [network.rpcUrl],
          nativeCurrency: network.nativeCurrency || {
            name: 'Ether',
            symbol: 'ETH',
            decimals: 18
          }
        }],
      })

      localStorage.setItem('wallet_chain_id', network.chainId)
      
      window.dispatchEvent(new CustomEvent('network-changed', {
        detail: { chainId: network.chainId, network }
      }))

      return true
    } catch (error) {
      console.error('Failed to add network:', error)
      throw error
    }
  }

  /**
   * Get wallet balance
   */
  async getBalance(address?: string): Promise<string> {
    if (!this.isWalletAvailable()) {
      throw new Error('Wallet not available')
    }

    try {
      const targetAddress = address || localStorage.getItem('wallet_address')
      if (!targetAddress) {
        throw new Error('No wallet address available')
      }

      const balance = await this.ethereum.request({
        method: 'eth_getBalance',
        params: [targetAddress, 'latest']
      })

      // Convert from wei to ether
      const ethBalance = (parseInt(balance, 16) / 1e18).toFixed(6)
      return ethBalance
    } catch (error) {
      console.error('Failed to get balance:', error)
      throw error
    }
  }

  /**
   * Execute wallet instruction from agent
   */
  async executeInstruction(instruction: WalletInstruction): Promise<any> {
    if (!this.isWalletAvailable()) {
      throw new Error('Wallet not available')
    }

    try {
      if (instruction.type === 'wallet_operation') {
        if (instruction.method === 'wallet_switchEthereumChain') {
          const chainId = instruction.params?.chainId
          if (!chainId) {
            throw new Error('Missing chainId parameter')
          }

          try {
            await this.ethereum.request({
              method: instruction.method,
              params: [{ chainId }]
            })
            
            localStorage.setItem('wallet_chain_id', chainId)
            return { success: true, chainId }
          } catch (error: any) {
            // Try fallback if available
            if (error.code === 4902 && instruction.fallback) {
              await this.ethereum.request({
                method: instruction.fallback.method,
                params: [instruction.fallback.params]
              })
              
              localStorage.setItem('wallet_chain_id', chainId)
              return { success: true, chainId, addedNetwork: true }
            }
            throw error
          }
        }
      } else if (instruction.type === 'query') {
        if (instruction.method === 'eth_chainId') {
          return await this.getCurrentChainId()
        } else if (instruction.method === 'eth_getBalance') {
          const address = instruction.params?.[0]
          return await this.getBalance(address === 'current_wallet' ? undefined : address)
        }
      }

      throw new Error(`Unsupported instruction type: ${instruction.type}`)
    } catch (error) {
      console.error('Failed to execute instruction:', error)
      throw error
    }
  }

  /**
   * Listen to wallet events
   */
  onAccountsChanged(callback: (accounts: string[]) => void) {
    if (this.isWalletAvailable()) {
      this.ethereum.on('accountsChanged', callback)
    }
  }

  /**
   * Listen to network changes
   */
  onChainChanged(callback: (chainId: string) => void) {
    if (this.isWalletAvailable()) {
      this.ethereum.on('chainChanged', (chainId: string) => {
        localStorage.setItem('wallet_chain_id', chainId)
        callback(chainId)
      })
    }
  }

  /**
   * Remove event listeners
   */
  removeListener(event: string, callback: any) {
    if (this.isWalletAvailable()) {
      this.ethereum.removeListener(event, callback)
    }
  }
}

export const walletService = new WalletService()
