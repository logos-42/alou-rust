// ============================================
// Desktop Wallet Service - 桌面版钱包服务
// 支持 WalletConnect、本地钱包文件和消息签名验证
// ============================================

import { EthereumProvider } from '@walletconnect/ethereum-provider';
import { ethers } from 'ethers';
import { invoke } from '@tauri-apps/api/core';

interface WalletInfo {
  address: string;
  chainId: string;
  walletType: 'walletconnect' | 'local' | 'browser';
  name?: string;
  icon?: string;
}

interface LocalWalletData {
  address: string;
  privateKey: string;
  mnemonic?: string;
  name?: string;
  chainId?: string;
  createdAt: string;
  updatedAt: string;
}

interface WalletConnectSession {
  topic: string;
  symKey: string;
  peer: {
    metadata: {
      name: string;
      description: string;
      url: string;
      icons: string[];
    };
  };
}

interface SignMessageResult {
  success: boolean;
  signature?: string;
  error?: string;
}

interface TransactionResult {
  success: boolean;
  hash?: string;
  error?: string;
}

class DesktopWalletService {
  private walletConnectProvider: InstanceType<typeof EthereumProvider> | null = null;
  private localWallet: ethers.Wallet | null = null;
  private walletType: 'walletconnect' | 'local' | 'browser' | null = null;
  private provider: ethers.BrowserProvider | null = null;

  constructor() {
    this.walletConnectProvider = null;
    this.localWallet = null;
    this.walletType = null;
    this.provider = null;
  }

  /**
   * 检测是否在桌面环境
   */
  isDesktop(): boolean {
    if (typeof window === 'undefined') {
      return false;
    }

    const hasTauriApi = typeof window.__TAURI__ !== 'undefined';
    const hasTauriIPC = typeof window.__TAURI_IPC__ !== 'undefined';
    const hasTauriEnv = typeof import.meta !== 'undefined' && Boolean(import.meta.env?.TAURI_PLATFORM);
    const userAgent = typeof navigator !== 'undefined' ? navigator.userAgent || '' : '';
    const uaMatchesTauri = /tauri/i.test(userAgent);

    return hasTauriApi || hasTauriIPC || hasTauriEnv || uaMatchesTauri;
  }

  /**
   * 初始化钱包服务
   */
  async initialize(): Promise<{ success: boolean; error?: string }> {
    try {
      console.warn('[DesktopWalletService] 初始化钱包服务');

      // 检查是否在桌面环境
      if (!this.isDesktop()) {
        console.warn('[DesktopWalletService] 不在桌面环境，跳过初始化');
        return { success: true };
      }

      // 尝试加载本地钱包
      await this.loadLocalWallet();

      console.warn('[DesktopWalletService] 钱包服务初始化成功');
      return { success: true };
    } catch (error) {
      console.error('[DesktopWalletService] 初始化失败:', error);
      return {
        success: false,
        error: (error as Error).message,
      };
    }
  }

  /**
   * 连接 WalletConnect
   */
  async connectWalletConnect(
    projectId: string,
    metadata: {
      name: string;
      description: string;
      url: string;
      icons: string[];
    }
  ): Promise<{ success: boolean; uri?: string; error?: string }> {
    try {
      console.warn('[DesktopWalletService] 连接 WalletConnect');

      // 创建 WalletConnect 提供者（ethers v6）
      this.walletConnectProvider = await EthereumProvider.init({
        projectId,
        metadata,
        showQrModal: true,
        chains: [1], // Ethereum Mainnet chainId
      });

      // 启动连接
      await this.walletConnectProvider.enable();

      // 设置提供商
      this.provider = new ethers.BrowserProvider(this.walletConnectProvider as any);
      this.walletType = 'walletconnect';

      console.warn('[DesktopWalletService] WalletConnect 连接成功');
      return { success: true };
    } catch (error) {
      console.error('[DesktopWalletService] WalletConnect 连接失败:', error);
      return {
        success: false,
        error: (error as Error).message,
      };
    }
  }

  /**
   * 创建本地钱包
   */
  async createLocalWallet(options: {
    mnemonic?: string;
    password?: string;
    name?: string;
  } = {}): Promise<{ success: boolean; address?: string; mnemonic?: string; error?: string }> {
    try {
      console.warn('[DesktopWalletService] 创建本地钱包');

      let wallet: ethers.HDNodeWallet;

      if (options.mnemonic) {
        // 使用助记词创建钱包
        wallet = ethers.HDNodeWallet.fromPhrase(options.mnemonic);
      } else {
        // 生成随机钱包
        wallet = ethers.HDNodeWallet.createRandom();
      }

      const address = await wallet.getAddress();
      const mnemonic = wallet.mnemonic?.phrase;

      // 保存钱包数据
      const walletData: LocalWalletData = {
        address,
        privateKey: wallet.privateKey,
        mnemonic,
        name: options.name || `本地钱包 ${address.slice(0, 6)}...${address.slice(-4)}`,
        chainId: '0x1', // Ethereum Mainnet
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
      };

      await this.saveLocalWallet(walletData);

      this.localWallet = wallet as unknown as ethers.Wallet;
      // 对于本地钱包，我们创建一个自定义的provider
      this.provider = this.createLocalProvider(wallet as unknown as ethers.Wallet);
      this.walletType = 'local';

      console.warn('[DesktopWalletService] 本地钱包创建成功:', address);
      return {
        success: true,
        address,
        mnemonic,
      };
    } catch (error) {
      console.error('[DesktopWalletService] 创建本地钱包失败:', error);
      return {
        success: false,
        error: (error as Error).message,
      };
    }
  }

  /**
   * 导入本地钱包
   */
  async importLocalWallet(privateKey: string, options: {
    name?: string;
    password?: string;
  } = {}): Promise<{ success: boolean; address?: string; error?: string }> {
    try {
      console.warn('[DesktopWalletService] 导入本地钱包');

      // ethers v6: 使用 Wallet 而不是 HDNodeWallet
      const wallet = new ethers.Wallet(privateKey);
      const address = await wallet.getAddress();

      const walletData: LocalWalletData = {
        address,
        privateKey,
        name: options.name || `导入钱包 ${address.slice(0, 6)}...${address.slice(-4)}`,
        chainId: '0x1',
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
      };

      await this.saveLocalWallet(walletData);

      this.localWallet = wallet;
      this.provider = this.createLocalProvider(wallet);
      this.walletType = 'local';

      console.warn('[DesktopWalletService] 本地钱包导入成功:', address);
      return {
        success: true,
        address,
      };
    } catch (error) {
      console.error('[DesktopWalletService] 导入本地钱包失败:', error);
      return {
        success: false,
        error: (error as Error).message,
      };
    }
  }

  /**
   * Agent 创建钱包（使用 Web3 工具）
   */
  async agentCreateWallet(options: {
    name?: string;
    chain?: string;
  } = {}): Promise<{
    success: boolean;
    address?: string;
    mnemonic?: string;
    privateKey?: string;
    error?: string;
  }> {
    try {
      console.warn('[DesktopWalletService] Agent 创建钱包');

      // 使用 ethers 生成随机助记词和钱包
      const mnemonic = ethers.Mnemonic.entropyToPhrase(ethers.randomBytes(16));
      const wallet = ethers.HDNodeWallet.fromPhrase(mnemonic);
      const address = await wallet.getAddress();
      const privateKey = wallet.privateKey;

      // 保存钱包数据
      const walletData: LocalWalletData = {
        address,
        privateKey,
        mnemonic,
        name: options.name || `Agent 钱包 ${address.slice(0, 6)}...${address.slice(-4)}`,
        chainId: options.chain || '0x1',
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
      };

      await this.saveLocalWallet(walletData);

      this.localWallet = wallet as unknown as ethers.Wallet;
      this.provider = this.createLocalProvider(this.localWallet);
      this.walletType = 'local';

      console.warn('[DesktopWalletService] Agent 钱包创建成功:', address);
      return {
        success: true,
        address,
        mnemonic,
        privateKey,
      };
    } catch (error) {
      console.error('[DesktopWalletService] Agent 创建钱包失败:', error);
      return {
        success: false,
        error: (error as Error).message,
      };
    }
  }

  /**
   * 获取当前钱包信息
   */
  async getWalletInfo(): Promise<WalletInfo | null> {
    try {
      if (!this.provider) {
        return null;
      }

      // ethers v6: getSigner() 返回 Promise，需要 await
      const signer = await this.provider.getSigner();
      const address = await signer.getAddress();
      const network = await this.provider.getNetwork();

      let name = '未知钱包';
      let icon = '';

      if (this.walletType === 'walletconnect' && this.walletConnectProvider) {
        const session = (this.walletConnectProvider as any).session;
        name = session?.peer?.metadata?.name || 'WalletConnect';
        icon = session?.peer?.metadata?.icons?.[0] || '';
      } else if (this.walletType === 'local') {
        name = '本地钱包';
        icon = '🔑';
      }

      return {
        address,
        // ethers v6: chainId 是 bigint，需要转换为 string
        chainId: `0x${network.chainId.toString(16)}`,
        walletType: this.walletType!,
        name,
        icon,
      };
    } catch (error) {
      console.error('[DesktopWalletService] 获取钱包信息失败:', error);
      return null;
    }
  }

  /**
   * 签名消息
   */
  async signMessage(message: string): Promise<SignMessageResult> {
    try {
      if (!this.provider) {
        throw new Error('钱包未连接');
      }

      // ethers v6: getSigner() 返回 Promise
      const signer = await this.provider.getSigner();
      const signature = await signer.signMessage(message);

      console.warn('[DesktopWalletService] 消息签名成功');
      return {
        success: true,
        signature,
      };
    } catch (error) {
      console.error('[DesktopWalletService] 消息签名失败:', error);
      return {
        success: false,
        error: (error as Error).message,
      };
    }
  }

  /**
   * 签名交易
   */
  async signTransaction(transaction: ethers.TransactionRequest): Promise<TransactionResult> {
    try {
      if (!this.provider) {
        throw new Error('钱包未连接');
      }

      // ethers v6: getSigner() 返回 Promise
      const signer = await this.provider.getSigner();
      const signedTx = await signer.signTransaction(transaction);

      console.warn('[DesktopWalletService] 交易签名成功');
      // 在ethers v6中，signedTx是签名的十六进制字符串
      // 要获取哈希，我们需要解析这个交易
      const tx = ethers.Transaction.from(signedTx);
      return {
        success: true,
        hash: tx.hash || undefined,
      };
    } catch (error) {
      console.error('[DesktopWalletService] 交易签名失败:', error);
      return {
        success: false,
        error: (error as Error).message,
      };
    }
  }

  /**
   * 发送交易
   */
  async sendTransaction(transaction: ethers.TransactionRequest): Promise<TransactionResult> {
    try {
      if (!this.provider) {
        throw new Error('钱包未连接');
      }

      // ethers v6: getSigner() 返回 Promise
      const signer = await this.provider.getSigner();
      const tx = await signer.sendTransaction(transaction);

      console.warn('[DesktopWalletService] 交易发送成功:', tx.hash);
      return {
        success: true,
        hash: tx.hash,
      };
    } catch (error) {
      console.error('[DesktopWalletService] 交易发送失败:', error);
      return {
        success: false,
        error: (error as Error).message,
      };
    }
  }

  /**
   * 获取余额
   */
  async getBalance(address?: string): Promise<{ success: boolean; balance?: string; error?: string }> {
    try {
      if (!this.provider) {
        throw new Error('钱包未连接');
      }

      const signer = await this.provider.getSigner();
      const targetAddress = address || await signer.getAddress();
      const balance = await this.provider.getBalance(targetAddress);

      const etherBalance = ethers.formatEther(balance);

      console.warn('[DesktopWalletService] 余额查询成功:', etherBalance);
      return {
        success: true,
        balance: etherBalance,
      };
    } catch (error) {
      console.error('[DesktopWalletService] 余额查询失败:', error);
      return {
        success: false,
        error: (error as Error).message,
      };
    }
  }

  /**
   * 获取代币余额
   */
  async getTokenBalance(
    tokenAddress: string, 
    address?: string
  ): Promise<{ success: boolean; balance?: string; error?: string }> {
    try {
      if (!this.provider) {
        throw new Error('钱包未连接');
      }

      const signer = await this.provider.getSigner();
      const targetAddress = address || await signer.getAddress();
      
      // ERC20 ABI (只包含余额查询方法)
      const erc20Abi = [
        'function balanceOf(address) view returns (uint256)',
        'function decimals() view returns (uint8)',
      ];

      const contract = new ethers.Contract(tokenAddress, erc20Abi, this.provider);
      const balance = await contract.balanceOf(targetAddress);
      const decimals = await contract.decimals();

      const formattedBalance = ethers.formatUnits(balance, decimals);

      console.warn('[DesktopWalletService] 代币余额查询成功:', formattedBalance);
      return {
        success: true,
        balance: formattedBalance,
      };
    } catch (error) {
      console.error('[DesktopWalletService] 代币余额查询失败:', error);
      return {
        success: false,
        error: (error as Error).message,
      };
    }
  }

  /**
   * 切换网络
   */
  async switchNetwork(chainId: string): Promise<{ success: boolean; error?: string }> {
    try {
      if (!this.provider) {
        throw new Error('钱包未连接');
      }

      await this.provider.send('wallet_switchEthereumChain', [{ chainId }]);
      
      console.warn('[DesktopWalletService] 网络切换成功:', chainId);
      return { success: true };
    } catch (error) {
      console.error('[DesktopWalletService] 网络切换失败:', error);
      return {
        success: false,
        error: (error as Error).message,
      };
    }
  }

  /**
   * 添加网络
   */
  async addNetwork(networkParams: {
    chainId: string;
    chainName: string;
    rpcUrls: string[];
    nativeCurrency: {
      name: string;
      symbol: string;
      decimals: number;
    };
    blockExplorerUrls?: string[];
  }): Promise<{ success: boolean; error?: string }> {
    try {
      if (!this.provider) {
        throw new Error('钱包未连接');
      }

      await this.provider.send('wallet_addEthereumChain', [networkParams]);
      
      console.warn('[DesktopWalletService] 网络添加成功:', networkParams.chainName);
      return { success: true };
    } catch (error) {
      console.error('[DesktopWalletService] 网络添加失败:', error);
      return {
        success: false,
        error: (error as Error).message,
      };
    }
  }

  /**
   * 断开钱包连接
   */
  async disconnect(): Promise<{ success: boolean; error?: string }> {
    try {
      console.warn('[DesktopWalletService] 断开钱包连接');

      if (this.walletConnectProvider) {
        await this.walletConnectProvider.disconnect();
        this.walletConnectProvider = null;
      }

      this.localWallet = null;
      this.provider = null;
      this.walletType = null;

      console.warn('[DesktopWalletService] 钱包连接已断开');
      return { success: true };
    } catch (error) {
      console.error('[DesktopWalletService] 断开钱包连接失败:', error);
      return {
        success: false,
        error: (error as Error).message,
      };
    }
  }

  /**
   * 保存本地钱包数据
   */
  private async saveLocalWallet(walletData: LocalWalletData): Promise<void> {
    try {
      if (this.isDesktop()) {
        // 在桌面环境中使用 Tauri API
        await invoke('save_local_wallet', { walletData });
      } else {
        // 在浏览器环境中使用 localStorage
        localStorage.setItem('alou_local_wallet', JSON.stringify(walletData));
      }
    } catch (error) {
      console.error('[DesktopWalletService] 保存本地钱包失败:', error);
      throw error;
    }
  }

  /**
   * 加载本地钱包数据
   */
  private async loadLocalWallet(): Promise<void> {
    try {
      let walletData: LocalWalletData | null = null;

      if (this.isDesktop()) {
        // 在桌面环境中使用 Tauri API
        const result = await invoke('load_local_wallet');
        // 安全类型转换，检查是否具有必要的属性
        if (result && typeof result === 'object' && 'address' in result && 'privateKey' in result) {
          walletData = result as LocalWalletData;
        }
      } else {
        // 在浏览器环境中使用 localStorage
        const stored = localStorage.getItem('alou_local_wallet');
        if (stored) {
          walletData = JSON.parse(stored);
        }
      }

      if (walletData && walletData.privateKey) {
        this.localWallet = new ethers.Wallet(walletData.privateKey);
        // ethers v6: 创建本地钱包的provider
        this.provider = this.createLocalProvider(this.localWallet);
        this.walletType = 'local';

        console.warn('[DesktopWalletService] 本地钱包加载成功:', walletData.address);
      }
    } catch (error) {
      console.error('[DesktopWalletService] 加载本地钱包失败:', error);
    }
  }

  /**
   * 删除本地钱包
   */
  async deleteLocalWallet(): Promise<{ success: boolean; error?: string }> {
    try {
      console.warn('[DesktopWalletService] 删除本地钱包');

      if (this.isDesktop()) {
        await invoke('delete_local_wallet');
      } else {
        localStorage.removeItem('alou_local_wallet');
      }

      this.localWallet = null;
      this.provider = null;
      this.walletType = null;

      console.warn('[DesktopWalletService] 本地钱包已删除');
      return { success: true };
    } catch (error) {
      console.error('[DesktopWalletService] 删除本地钱包失败:', error);
      return {
        success: false,
        error: (error as Error).message,
      };
    }
  }

  /**
   * 获取钱包状态
   */
  getStatus(): {
    isConnected: boolean;
    walletType: 'walletconnect' | 'local' | 'browser' | null;
    isDesktop: boolean;
    hasLocalWallet: boolean;
  } {
    return {
      isConnected: !!this.provider,
      walletType: this.walletType,
      isDesktop: this.isDesktop(),
      hasLocalWallet: !!this.localWallet,
    };
  }

  /**
   * 验证消息签名
   */
  async verifySignature(message: string, signature: string, address: string): Promise<boolean> {
    try {
      const recoveredAddress = ethers.verifyMessage(message, signature);
      return recoveredAddress.toLowerCase() === address.toLowerCase();
    } catch (error) {
      console.error('[DesktopWalletService] 签名验证失败:', error);
      return false;
    }
  }

  /**
   * 获取支持的链列表
   */
  getSupportedChains(): Array<{
    chainId: string;
    name: string;
    rpcUrls: string[];
    nativeCurrency: {
      name: string;
      symbol: string;
      decimals: number;
    };
  }> {
    return [
      {
        chainId: '0x1',
        name: 'Ethereum Mainnet',
        rpcUrls: ['https://mainnet.infura.io/v3/'],
        nativeCurrency: {
          name: 'Ether',
          symbol: 'ETH',
          decimals: 18,
        },
      },
      {
        chainId: '0x3',
        name: 'Ropsten Testnet',
        rpcUrls: ['https://ropsten.infura.io/v3/'],
        nativeCurrency: {
          name: 'Ether',
          symbol: 'ETH',
          decimals: 18,
        },
      },
      {
        chainId: '0x89',
        name: 'Polygon Mainnet',
        rpcUrls: ['https://polygon-rpc.com/'],
        nativeCurrency: {
          name: 'MATIC',
          symbol: 'MATIC',
          decimals: 18,
        },
      },
      {
        chainId: '0x13882',
        name: 'Polygon Mumbai',
        rpcUrls: ['https://rpc-mumbai.maticvigil.com'],
        nativeCurrency: {
          name: 'MATIC',
          symbol: 'MATIC',
          decimals: 18,
        },
      },
      {
        chainId: '0x2105',
        name: 'Base Mainnet',
        rpcUrls: ['https://mainnet.base.org'],
        nativeCurrency: {
          name: 'Ether',
          symbol: 'ETH',
          decimals: 18,
        },
      },
      {
        chainId: '0x14a34',
        name: 'Base Sepolia Testnet',
        rpcUrls: ['https://sepolia.base.org'],
        nativeCurrency: {
          name: 'Ether',
          symbol: 'ETH',
          decimals: 18,
        },
      },
    ];
  }

  /**
   * 为本地钱包创建自定义的 EIP-1193 provider
   */
  private createLocalProvider(wallet: ethers.Wallet): ethers.BrowserProvider {
    // 创建一个自定义的 EIP-1193 provider 包装器
    const customProvider = {
      request: async (request: { method: string; params?: any[] }) => {
        switch (request.method) {
          case 'eth_accounts':
            return [await wallet.getAddress()];
          case 'eth_chainId':
            return '0x1'; // Ethereum Mainnet
          case 'personal_sign':
          case 'eth_sign':
          case 'eth_signTypedData':
          case 'eth_signTypedData_v4':
            if (request.params && request.params.length > 0) {
              const [message, address] = request.params;
              // 验证地址匹配
              const walletAddress = await wallet.getAddress();
              if (address.toLowerCase() !== walletAddress.toLowerCase()) {
                throw new Error('Address does not match wallet');
              }
              // 签名消息
              return await wallet.signMessage(message);
            }
            throw new Error('Missing parameters for signing');
          case 'eth_sendTransaction':
            if (request.params && request.params.length > 0) {
              const [tx] = request.params;
              return await wallet.sendTransaction(tx);
            }
            throw new Error('Missing transaction parameters');
          default:
            throw new Error(`Method ${request.method} not supported by local wallet`);
        }
      },
    };

    return new ethers.BrowserProvider(customProvider);
  }
}

// 创建单例实例
const desktopWalletService = new DesktopWalletService();

export default desktopWalletService;
export { DesktopWalletService, desktopWalletService };
export type {
  WalletInfo,
  LocalWalletData,
  WalletConnectSession,
  SignMessageResult,
  TransactionResult,
};
