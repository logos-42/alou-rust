// ============================================
// Desktop Wallet Service - 前端版钱包服务
// 支持 WalletConnect、本地钱包文件和消息签名验证
// 适配版本：去掉Tauri依赖，使Tauri功能可选
// ============================================

import { EthereumProvider } from '@walletconnect/ethereum-provider'
import { ethers } from 'ethers'

class DesktopWalletService {
  constructor() {
    this.walletConnectProvider = null
    this.localWallet = null
    this.walletType = null // 'walletconnect', 'local', 'browser'
    this.provider = null
  }

  /**
   * 检测是否在桌面环境
   */
  isDesktop() {
    if (typeof window === 'undefined') {
      return false
    }

    // 前端版不需要Tauri环境，但保留检测以便兼容
    const hasTauriApi = typeof window.__TAURI__ !== 'undefined'
    const hasTauriIPC = typeof window.__TAURI_IPC__ !== 'undefined'
    const hasTauriEnv = typeof import.meta !== 'undefined' && Boolean(import.meta.env?.TAURI_PLATFORM)
    const userAgent = typeof navigator !== 'undefined' ? navigator.userAgent || '' : ''
    const uaMatchesTauri = /tauri/i.test(userAgent)

    return hasTauriApi || hasTauriIPC || hasTauriEnv || uaMatchesTauri
  }

  /**
   * 获取当前可用的钱包提供者
   */
  async getProvider() {
    // 优先使用已连接的钱包
    if (this.provider) {
      return this.provider
    }

    // 尝试浏览器钱包（如果可用）
    if (typeof window !== 'undefined' && window.ethereum) {
      return new ethers.BrowserProvider(window.ethereum)
    }

    return null
  }

  /**
   * 初始化 WalletConnect（返回Provider但不连接）
   */
  async initWalletConnect() {
    // 如果已经初始化，直接返回
    if (this.walletConnectProvider) {
      return this.walletConnectProvider
    }

    // 如果正在初始化，等待完成
    if (this._initWalletConnectPromise) {
      return this._initWalletConnectPromise
    }

    // 开始初始化
    this._initWalletConnectPromise = (async () => {
      try {
        const projectId = import.meta.env.VITE_WALLETCONNECT_PROJECT_ID || 'YOUR_WALLETCONNECT_PROJECT_ID'

        // 验证 projectId 是否有效
        if (!projectId || projectId === 'YOUR_WALLETCONNECT_PROJECT_ID') {
          const error = new Error(
            'WalletConnect Project ID 未配置。请在 .env 文件中设置 VITE_WALLETCONNECT_PROJECT_ID，或访问 https://cloud.walletconnect.com 获取 Project ID'
          )
          error.code = 'WALLETCONNECT_PROJECT_ID_MISSING'
          throw error
        }

        // WalletConnect 配置选项
        const providerOptions = {
          projectId,
          chains: [1], // Ethereum Mainnet
          optionalChains: [5, 137, 80001], // Goerli, Polygon, Mumbai
          showQrModal: false, // 我们使用自定义的QR码显示
          metadata: {
            name: 'Alou',
            description: 'Alou - Web3 AI Agent',
            url: window.location.origin, // 使用当前页面 URL 而不是硬编码
            icons: ['https://alou.app/icon.png'],
          },
        }

        console.log('[DesktopWalletService] Initializing WalletConnect with options:', {
          projectId: projectId.substring(0, 10) + '...',
          chains: providerOptions.chains,
          url: providerOptions.metadata.url,
        })

        this.walletConnectProvider = await EthereumProvider.init(providerOptions)

        // 监听连接错误
        this.walletConnectProvider.on?.('disconnect', (error) => {
          console.error('[DesktopWalletService] WalletConnect disconnected:', error)
        })

        // 监听会话过期
        this.walletConnectProvider.on?.('session_delete', (event) => {
          console.log('[DesktopWalletService] WalletConnect session deleted:', event)
        })

        return this.walletConnectProvider
      } catch (error) {
        console.error('WalletConnect initialization error:', error)
        // 清除初始化 Promise，允许重试
        this._initWalletConnectPromise = null
        throw error
      } finally {
        // 初始化完成后清除 Promise
        this._initWalletConnectPromise = null
      }
    })()

    return this._initWalletConnectPromise
  }

  /**
   * 连接 WalletConnect
   */
  async connectWalletConnect() {
    try {
      if (!this.walletConnectProvider) {
        await this.initWalletConnect()
      }

      // 连接
      await this.walletConnectProvider.enable()

      // 创建 ethers provider
      this.provider = new ethers.BrowserProvider(this.walletConnectProvider)
      this.walletType = 'walletconnect'

      // 获取账户
      const signer = await this.provider.getSigner()
      const address = await signer.getAddress()

      // 监听断开连接
      this.walletConnectProvider.on('disconnect', () => {
        console.log('[DesktopWalletService] WalletConnect disconnected')
        this.disconnect()
      })

      // 监听 session 变化
      this.walletConnectProvider.on?.('session_update', (event) => {
        console.log('[DesktopWalletService] Session updated:', event)
      })

      return {
        address,
        provider: this.provider,
      }
    } catch (error) {
      console.error('WalletConnect connection error:', error)
      throw error
    }
  }

  /**
   * 获取 WalletConnect QR URI（用于显示二维码）
   */
  async getWalletConnectQrUri() {
    if (!this.walletConnectProvider) {
      await this.initWalletConnect()
    }

    // 如果已经有session，返回null（已连接）
    if (this.walletConnectProvider.session) {
      console.log('[DesktopWalletService] Session already exists, skipping QR generation')
      return null
    }

    // 如果URI已经存在，直接返回
    if (this.walletConnectProvider.uri) {
      console.log('[DesktopWalletService] URI already exists, returning existing URI')
      return this.walletConnectProvider.uri
    }

    return new Promise((resolve, reject) => {
      let resolved = false
      
      // 监听 URI 事件
      const uriHandler = (uri) => {
        if (resolved) return
        resolved = true
        console.log('[DesktopWalletService] display_uri event received:', uri ? 'URI received' : 'null')
        this.walletConnectProvider.off('display_uri', uriHandler)
        if (uri) {
          resolve(uri)
        } else {
          reject(new Error('URI is empty'))
        }
      }

      this.walletConnectProvider.on('display_uri', uriHandler)

      // 触发enable以生成URI
      console.log('[DesktopWalletService] Calling provider.enable() to generate QR URI...')
      this.walletConnectProvider.enable().then(() => {
        // enable() 成功后，URI 应该通过 display_uri 事件传递
        // 但如果 URI 已经存在，直接返回
        if (!resolved && this.walletConnectProvider.uri) {
          resolved = true
          console.log('[DesktopWalletService] URI available after enable()')
          this.walletConnectProvider.off('display_uri', uriHandler)
          resolve(this.walletConnectProvider.uri)
        }
      }).catch((err) => {
        if (!resolved) {
          resolved = true
          console.error('[DesktopWalletService] enable() failed:', err)
          this.walletConnectProvider.off('display_uri', uriHandler)
          reject(err)
        }
      })

      // 超时处理（增加到60秒，因为网络请求可能需要时间）
      setTimeout(() => {
        if (!resolved) {
          resolved = true
          console.error('[DesktopWalletService] QR code generation timeout after 60s')
          this.walletConnectProvider.off('display_uri', uriHandler)
          reject(new Error('QR code generation timeout (60s)'))
        }
      }, 60000)
    })
  }

  /**
   * 创建或导入本地钱包
   */
  async connectLocalWallet(privateKeyOrMnemonic, isMnemonic = false) {
    try {
      let wallet

      if (isMnemonic) {
        // 从助记词创建钱包
        wallet = ethers.Wallet.fromPhrase(privateKeyOrMnemonic)
      } else {
        // 从私钥创建钱包
        wallet = new ethers.Wallet(privateKeyOrMnemonic)
      }

      // 使用 JSON-RPC Provider（需要配置RPC节点）
      const rpcUrl = this.getDefaultRpcUrl()
      const provider = new ethers.JsonRpcProvider(rpcUrl)
      wallet = wallet.connect(provider)

      this.localWallet = wallet
      this.provider = provider
      this.walletType = 'local'

      return {
        address: wallet.address,
        provider: this.provider,
        wallet: wallet,
      }
    } catch (error) {
      console.error('Local wallet connection error:', error)
      throw new Error('无效的私钥或助记词')
    }
  }

  /**
   * 创建新钱包
   */
  async createNewWallet() {
    const wallet = ethers.Wallet.createRandom()
    const rpcUrl = this.getDefaultRpcUrl()
    const provider = new ethers.JsonRpcProvider(rpcUrl)
    const connectedWallet = wallet.connect(provider)

    this.localWallet = connectedWallet
    this.provider = provider
    this.walletType = 'local'

    return {
      address: wallet.address,
      privateKey: wallet.privateKey,
      mnemonic: wallet.mnemonic.phrase,
      provider: this.provider,
      wallet: connectedWallet,
    }
  }

  /**
   * 消息签名（用于验证钱包所有权）
   */
  async signMessage(message) {
    if (!this.provider) {
      throw new Error('未连接钱包')
    }

    try {
      let signature

      if (this.walletType === 'local' && this.localWallet) {
        // 本地钱包直接签名
        signature = await this.localWallet.signMessage(message)
      } else {
        // WalletConnect 或浏览器钱包
        const signer = await this.provider.getSigner()
        signature = await signer.signMessage(message)
      }

      return signature
    } catch (error) {
      console.error('Sign message error:', error)
      throw error
    }
  }

  /**
   * 验证消息签名（优先使用ethers.js，备用后端）
   */
  async verifySignature(address, message, signature) {
    try {
      // 优先使用 ethers.js 验证签名（更准确，支持keccak256）
      const recoveredAddress = ethers.verifyMessage(message, signature)
      
      // 地址比较（不区分大小写）
      return recoveredAddress.toLowerCase() === address.toLowerCase()
    } catch (error) {
      console.warn('Ethers verification failed:', error)
      
      // 前端版不使用Tauri后端验证，直接返回false
      return false
    }
  }

  /**
   * 连接钱包并验证（完整流程）
   */
  async connectAndVerify(walletType, options = {}) {
    let address, provider

    // 1. 连接钱包
    if (walletType === 'walletconnect') {
      const result = await this.connectWalletConnect()
      address = result.address
      provider = result.provider
    } else if (walletType === 'local') {
      const { privateKeyOrMnemonic, isMnemonic } = options
      if (!privateKeyOrMnemonic) {
        throw new Error('需要提供私钥或助记词')
      }
      const result = await this.connectLocalWallet(privateKeyOrMnemonic, isMnemonic)
      address = result.address
      provider = result.provider
    } else {
      throw new Error('不支持的钱包类型')
    }

    // 2. 生成验证消息
    const verificationMessage = this.generateVerificationMessage(address)

    // 3. 签名消息
    const signature = await this.signMessage(verificationMessage)

    // 4. 验证签名
    const isValid = await this.verifySignature(address, verificationMessage, signature)

    if (!isValid) {
      throw new Error('签名验证失败')
    }

    // 5. 保存连接信息
    await this.saveWalletConnection(address, walletType)

    return {
      address,
      provider,
      walletType,
    }
  }

  /**
   * 生成验证消息
   */
  generateVerificationMessage(address) {
    const timestamp = Date.now()
    return `Alou 钱包验证\n\n地址: ${address}\n时间戳: ${timestamp}\n\n请确认这是您的钱包地址。`
  }

  /**
   * 获取账户列表
   */
  async getAccounts() {
    if (!this.provider) {
      console.log('[DesktopWalletService] getAccounts: No provider available')
      // 如果 WalletConnect provider 存在但 ethers provider 不存在，尝试创建
      if (this.walletConnectProvider && this.walletType === 'walletconnect') {
        try {
          console.log('[DesktopWalletService] Creating provider from WalletConnect provider')
          const { BrowserProvider } = await import('ethers')
          this.provider = new BrowserProvider(this.walletConnectProvider)
          console.log('[DesktopWalletService] Provider created successfully')
        } catch (createError) {
          console.error('[DesktopWalletService] Failed to create provider:', createError)
          return []
        }
      } else {
        return []
      }
    }

    try {
      if (this.walletType === 'local' && this.localWallet) {
        return [this.localWallet.address]
      }

      // 对于 WalletConnect，确保 session 存在
      if (this.walletType === 'walletconnect' && this.walletConnectProvider) {
        const hasSession = this.walletConnectProvider.session || 
                          this.walletConnectProvider.client?.session
        if (!hasSession) {
          console.log('[DesktopWalletService] WalletConnect session not established yet')
          return []
        }
      }

      const signer = await this.provider.getSigner()
      const address = await signer.getAddress()
      console.log('[DesktopWalletService] getAccounts: Successfully got address:', address)
      return [address]
    } catch (error) {
      console.error('[DesktopWalletService] Get accounts error:', error)
      return []
    }
  }

  /**
   * 获取当前链ID
   */
  async getCurrentChainId() {
    if (!this.provider) {
      throw new Error('未连接钱包')
    }

    try {
      const network = await this.provider.getNetwork()
      return `0x${network.chainId.toString(16)}`
    } catch (error) {
      console.error('Get chain ID error:', error)
      throw error
    }
  }

  /**
   * 获取余额
   */
  async getBalance(address) {
    if (!this.provider) {
      throw new Error('未连接钱包')
    }

    try {
      const balance = await this.provider.getBalance(address)
      return ethers.formatEther(balance)
    } catch (error) {
      console.error('Get balance error:', error)
      throw error
    }
  }

  /**
   * 发送交易
   */
  async sendTransaction(txParams) {
    if (!this.provider) {
      throw new Error('未连接钱包')
    }

    try {
      let signer

      if (this.walletType === 'local' && this.localWallet) {
        signer = this.localWallet
      } else {
        signer = await this.provider.getSigner()
      }

      const tx = await signer.sendTransaction(txParams)
      return tx.hash
    } catch (error) {
      console.error('Send transaction error:', error)
      throw error
    }
  }

  /**
   * 断开连接
   */
  async disconnect() {
    if (this.walletConnectProvider) {
      await this.walletConnectProvider.disconnect()
      this.walletConnectProvider = null
    }

    this.localWallet = null
    this.provider = null
    this.walletType = null

    // 清除本地存储
    if (typeof window !== 'undefined') {
      localStorage.removeItem('wallet_address')
      localStorage.removeItem('wallet_type')
      localStorage.removeItem('wallet_chain_id')
    }
  }

  /**
   * 保存钱包连接信息
   */
  async saveWalletConnection(address, walletType) {
    if (typeof window !== 'undefined') {
      localStorage.setItem('wallet_address', address)
      localStorage.setItem('wallet_type', walletType)
      
      const chainId = await this.getCurrentChainId()
      localStorage.setItem('wallet_chain_id', chainId)
    }
  }

  /**
   * 获取默认 RPC URL
   */
  getDefaultRpcUrl() {
    // 可以根据环境变量或配置来设置
    return import.meta.env.VITE_ETH_RPC_URL || 'https://eth.llamarpc.com'
  }

  /**
   * 检查是否已连接
   */
  isConnected() {
    return this.provider !== null
  }
}

export const desktopWalletService = new DesktopWalletService()

