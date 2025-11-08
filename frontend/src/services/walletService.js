// ============================================
// Wallet Service - Handle wallet operations
// ============================================

class WalletService {
  constructor() {
    this.ethereum = typeof window !== 'undefined' ? window.ethereum : null
  }

  getProvider() {
    if (typeof window !== 'undefined' && window.ethereum) {
      this.ethereum = window.ethereum
    }
    return this.ethereum
  }

  /**
   * Check if wallet is available
   */
  isWalletAvailable() {
    return Boolean(this.getProvider())
  }

  /**
   * Get current wallet info
   */
  async getCurrentWalletInfo() {
    if (!this.isWalletAvailable()) {
      return null
    }

    try {
      const address = typeof window !== 'undefined' ? localStorage.getItem('wallet_address') : null
      const walletType = typeof window !== 'undefined' ? localStorage.getItem('wallet_type') : null
      const chainId = typeof window !== 'undefined' ? localStorage.getItem('wallet_chain_id') : null

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
  async getCurrentChainId() {
    if (!this.isWalletAvailable()) {
      throw new Error('Wallet not available')
    }

    try {
      const provider = this.getProvider()
      const chainId = await provider.request({ method: 'eth_chainId' })
      if (typeof window !== 'undefined') {
        localStorage.setItem('wallet_chain_id', chainId)
      }
      return chainId
    } catch (error) {
      console.error('Failed to get chain ID:', error)
      throw error
    }
  }

  /**
   * Switch to a specific network
   */
  async switchNetwork(network) {
    if (!this.isWalletAvailable()) {
      throw new Error('Wallet not available')
    }

    try {
      const provider = this.getProvider()

      await provider.request({
        method: 'wallet_switchEthereumChain',
        params: [{ chainId: network.chainId }],
      })

      if (typeof window !== 'undefined') {
        localStorage.setItem('wallet_chain_id', network.chainId)
      }
      
      // Dispatch event for other components
      if (typeof window !== 'undefined') {
        window.dispatchEvent(new CustomEvent('network-changed', {
          detail: { chainId: network.chainId, network }
        }))
      }

      return true
    } catch (error) {
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
  async addNetwork(network) {
    if (!this.isWalletAvailable()) {
      throw new Error('Wallet not available')
    }

    try {
      const provider = this.getProvider()

      await provider.request({
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

      if (typeof window !== 'undefined') {
        localStorage.setItem('wallet_chain_id', network.chainId)
      }
      
      if (typeof window !== 'undefined') {
        window.dispatchEvent(new CustomEvent('network-changed', {
          detail: { chainId: network.chainId, network }
        }))
      }

      return true
    } catch (error) {
      console.error('Failed to add network:', error)
      throw error
    }
  }

  /**
   * Get wallet balance
   */
  async getBalance(address) {
    if (!this.isWalletAvailable()) {
      throw new Error('Wallet not available')
    }

    try {
      const targetAddress = address || (typeof window !== 'undefined' ? localStorage.getItem('wallet_address') : null)
      if (!targetAddress) {
        throw new Error('No wallet address available')
      }

      const provider = this.getProvider()
      const balance = await provider.request({
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
  async executeInstruction(instruction) {
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
            const provider = this.getProvider()
            await provider.request({
              method: instruction.method,
              params: [{ chainId }]
            })
            
            if (typeof window !== 'undefined') {
              localStorage.setItem('wallet_chain_id', chainId)
            }
            return { success: true, chainId }
          } catch (error) {
            // Try fallback if available
            if (error.code === 4902 && instruction.fallback) {
              const provider = this.getProvider()
              await provider.request({
                method: instruction.fallback.method,
                params: [instruction.fallback.params]
              })
              
              if (typeof window !== 'undefined') {
                localStorage.setItem('wallet_chain_id', chainId)
              }
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
  onAccountsChanged(callback) {
    if (this.isWalletAvailable()) {
      const provider = this.getProvider()
      provider.on('accountsChanged', callback)
    }
  }

  /**
   * Listen to network changes
   */
  onChainChanged(callback) {
    if (this.isWalletAvailable()) {
      const provider = this.getProvider()
      provider.on('chainChanged', (chainId) => {
        if (typeof window !== 'undefined') {
          localStorage.setItem('wallet_chain_id', chainId)
        }
        callback(chainId)
      })
    }
  }

  /**
   * Remove event listeners
   */
  removeListener(event, callback) {
    if (this.isWalletAvailable()) {
      const provider = this.getProvider()
      provider.removeListener(event, callback)
    }
  }
}

export const walletService = new WalletService()
