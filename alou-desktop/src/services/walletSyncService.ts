// ============================================
// Wallet Sync Service - 浏览器和桌面应用之间的钱包信息同步
// ============================================

interface WalletSyncParams {
  address: string;
  chainId?: string;
  walletType?: string;
  token?: string;
}

interface SyncResult {
  success: boolean;
  message?: string;
  method?: string;
}

/**
 * 检测是否在桌面环境
 */
function isDesktop(): boolean {
  if (typeof window === 'undefined') return false;
  return typeof window.__TAURI__ !== 'undefined' ||
         typeof window.__TAURI_IPC__ !== 'undefined' ||
         (typeof import.meta !== 'undefined' && Boolean(import.meta.env?.TAURI_PLATFORM));
}

/**
 * 同步钱包信息到桌面应用
 * 在浏览器登录成功后调用此函数
 */
export async function syncWalletToDesktop(params: WalletSyncParams): Promise<SyncResult> {
  try {
    // 如果在桌面环境中，不需要同步（已经在桌面应用中了）
    if (isDesktop()) {
      console.log('[WalletSync] Already in desktop environment, no sync needed');
      return { success: true, message: 'Already in desktop' };
    }

    const { address, chainId, walletType, token } = params;

    // 方案1：尝试使用深度链接（如果桌面应用注册了协议）
    try {
      const deepLink = `alou://wallet-sync?address=${encodeURIComponent(address)}&chainId=${encodeURIComponent(chainId || '0x1')}&walletType=${encodeURIComponent(walletType || 'metamask')}&token=${encodeURIComponent(token || '')}`;
      
      console.log('[WalletSync] Trying deep link:', deepLink);
      
      // 创建一个隐藏的iframe来触发深度链接
      const iframe = document.createElement('iframe');
      iframe.style.display = 'none';
      iframe.src = deepLink;
      document.body.appendChild(iframe);
      
      // 等待一段时间后移除iframe
      setTimeout(() => {
        if (iframe.parentNode) {
          iframe.parentNode.removeChild(iframe);
        }
      }, 1000);
      
      return { success: true, message: 'Deep link triggered', method: 'deep-link' };
    } catch (error) {
      console.warn('[WalletSync] Deep link failed:', error);
    }

    // 方案2：使用localStorage作为桥梁（桌面应用定期检查）
    try {
      const syncData = {
        address,
        chainId: chainId || '0x1',
        walletType: walletType || 'metamask',
        token: token || '',
        timestamp: Date.now(),
      };
      
      localStorage.setItem('alou_wallet_sync', JSON.stringify(syncData));
      console.log('[WalletSync] Data stored in localStorage');
      
      return { success: true, message: 'Data stored in localStorage', method: 'localStorage' };
    } catch (error) {
      console.warn('[WalletSync] localStorage failed:', error);
    }

    // 方案3：使用BroadcastChannel（如果支持）
    try {
      if (typeof BroadcastChannel !== 'undefined') {
        const channel = new BroadcastChannel('alou-wallet-sync');
        channel.postMessage({
          type: 'wallet-sync',
          data: {
            address,
            chainId: chainId || '0x1',
            walletType: walletType || 'metamask',
            token: token || '',
          },
        });
        
        setTimeout(() => {
          channel.close();
        }, 1000);
        
        return { success: true, message: 'BroadcastChannel message sent', method: 'broadcast-channel' };
      }
    } catch (error) {
      console.warn('[WalletSync] BroadcastChannel failed:', error);
    }

    // 所有方案都失败
    return { success: false, message: 'All sync methods failed' };
  } catch (error) {
    console.error('[WalletSync] Sync failed:', error);
    return { success: false, message: (error as Error).message };
  }
}

/**
 * 从localStorage获取同步的钱包信息
 * 桌面应用调用此函数来获取浏览器同步的数据
 */
export function getSyncedWalletFromStorage(): WalletSyncParams | null {
  try {
    const syncData = localStorage.getItem('alou_wallet_sync');
    if (!syncData) return null;

    const parsed = JSON.parse(syncData);
    
    // 检查数据是否过期（5分钟）
    const now = Date.now();
    if (now - parsed.timestamp > 5 * 60 * 1000) {
      localStorage.removeItem('alou_wallet_sync');
      return null;
    }

    return {
      address: parsed.address,
      chainId: parsed.chainId,
      walletType: parsed.walletType,
      token: parsed.token,
    };
  } catch (error) {
    console.error('[WalletSync] Failed to get synced wallet:', error);
    return null;
  }
}

/**
 * 清除同步的钱包信息
 */
export function clearSyncedWallet(): void {
  try {
    localStorage.removeItem('alou_wallet_sync');
    console.log('[WalletSync] Synced wallet data cleared');
  } catch (error) {
    console.error('[WalletSync] Failed to clear synced wallet:', error);
  }
}

/**
 * 监听钱包同步事件
 * 桌面应用可以调用此函数来监听浏览器的同步事件
 */
export function listenToWalletSync(callback: (data: WalletSyncParams) => void): () => void {
  let channel: BroadcastChannel | null = null;

  try {
    if (typeof BroadcastChannel !== 'undefined') {
      channel = new BroadcastChannel('alou-wallet-sync');
      
      channel.onmessage = (event) => {
        if (event.data.type === 'wallet-sync') {
          callback(event.data.data);
        }
      };
    }
  } catch (error) {
    console.warn('[WalletSync] BroadcastChannel not supported:', error);
  }

  // 返回清理函数
  return () => {
    if (channel) {
      channel.close();
    }
  };
}

/**
 * 定期检查localStorage中的同步数据
 * 桌面应用可以定期调用此函数
 */
export function pollWalletSync(callback: (data: WalletSyncParams | null) => void, intervalMs: number = 1000): () => void {
  let lastTimestamp = 0;
  
  const poll = () => {
    const data = getSyncedWalletFromStorage();
    
    if (data && data !== null) {
      const syncData = localStorage.getItem('alou_wallet_sync');
      if (syncData) {
        const parsed = JSON.parse(syncData);
        if (parsed.timestamp > lastTimestamp) {
          lastTimestamp = parsed.timestamp;
          callback(data);
        }
      }
    }
  };

  const intervalId = setInterval(poll, intervalMs);
  
  // 立即执行一次
  poll();

  // 返回清理函数
  return () => {
    clearInterval(intervalId);
  };
}

/**
 * 检查是否支持钱包同步
 */
export function isWalletSyncSupported(): {
  deepLink: boolean;
  localStorage: boolean;
  broadcastChannel: boolean;
  any: boolean;
} {
  return {
    deepLink: true, // 深度链接总是支持的
    localStorage: typeof Storage !== 'undefined',
    broadcastChannel: typeof BroadcastChannel !== 'undefined',
    any: typeof Storage !== 'undefined' || typeof BroadcastChannel !== 'undefined',
  };
}

/**
 * 获取最佳同步方法
 */
export function getBestSyncMethod(): 'deep-link' | 'localStorage' | 'broadcast-channel' | 'none' {
  const supported = isWalletSyncSupported();
  
  if (supported.broadcastChannel) {
    return 'broadcast-channel';
  }
  
  if (supported.localStorage) {
    return 'localStorage';
  }
  
  if (supported.deepLink) {
    return 'deep-link';
  }
  
  return 'none';
}

/**
 * 钱包同步管理器类
 */
export class WalletSyncManager {
  private cleanupFunctions: (() => void)[] = [];
  private isListening = false;

  /**
   * 开始监听钱包同步
   */
  startListening(callback: (data: WalletSyncParams | null) => void): void {
    if (this.isListening) {
      console.warn('[WalletSync] Already listening');
      return;
    }

    this.isListening = true;

    // 监听BroadcastChannel
    const cleanupChannel = listenToWalletSync((data) => callback(data));
    this.cleanupFunctions.push(cleanupChannel);

    // 定期检查localStorage
    const cleanupPoll = pollWalletSync(callback, 2000);
    this.cleanupFunctions.push(cleanupPoll);

    console.log('[WalletSync] Started listening for wallet sync');
  }

  /**
   * 停止监听钱包同步
   */
  stopListening(): void {
    if (!this.isListening) {
      return;
    }

    this.cleanupFunctions.forEach(cleanup => {
      try {
        cleanup();
      } catch (error) {
        console.error('[WalletSync] Error during cleanup:', error);
      }
    });

    this.cleanupFunctions = [];
    this.isListening = false;

    console.log('[WalletSync] Stopped listening for wallet sync');
  }

  /**
   * 手动同步钱包到桌面
   */
  async syncToDesktop(params: WalletSyncParams): Promise<SyncResult> {
    return syncWalletToDesktop(params);
  }

  /**
   * 获取当前同步状态
   */
  getStatus(): {
    isListening: boolean;
    supported: ReturnType<typeof isWalletSyncSupported>;
    bestMethod: ReturnType<typeof getBestSyncMethod>;
    hasSyncedData: boolean;
  } {
    return {
      isListening: this.isListening,
      supported: isWalletSyncSupported(),
      bestMethod: getBestSyncMethod(),
      hasSyncedData: getSyncedWalletFromStorage() !== null,
    };
  }

  /**
   * 销毁管理器
   */
  destroy(): void {
    this.stopListening();
    clearSyncedWallet();
  }
}

// 创建默认实例
export const walletSyncManager = new WalletSyncManager();

export default walletSyncManager;
export type { WalletSyncParams, SyncResult };
