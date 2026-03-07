import { resolveAgentAvatar, type Agent } from './agentUtils';
import useAgentStore from '../../stores/agentStore';
import imageProxyService from '../../services/imageProxyService';
import agentAssetsService from '../../services/agentAssetsService';

/**
 * 头像管理模块
 * 集中处理头像的更新、同步和缓存
 */
class AvatarManager {
  public listeners = new Set<(agent: Agent) => void>();
  private avatarCache = new Map<string, string>();
  private readonly fallbackAvatar = 'https://avatars.githubusercontent.com/u/16309930?v=4';

  /**
   * 解析智能体头像
   * @param agent - 智能体对象
   * @returns 头像URL
   */
  async resolveAvatar(agent: Agent): Promise<string> {
    if (!agent) {
      return this.getFallbackAvatar();
    }

    // 检查缓存
    const cacheKey = this.getCacheKey(agent);
    if (this.avatarCache.has(cacheKey)) {
      return this.avatarCache.get(cacheKey)!;
    }

    try {
      // 使用现有的头像解析逻辑
      // 将 AgentMetadata 转换为 Agent 类型
      const avatarUrl = resolveAgentAvatar(agent as any);
      
      // 缓存结果
      this.avatarCache.set(cacheKey, avatarUrl);
      
      return avatarUrl;
    } catch (error) {
      console.error('[AvatarManager] 解析头像失败:', error);
      return this.getFallbackAvatar();
    }
  }

  /**
   * 获取缓存键
   * @param agent - 智能体对象
   * @returns 缓存键
   */
  private getCacheKey(agent: Agent): string {
    return `${agent.id || agent.did || 'unknown'}_${agent.avatar_cid || agent.avatar_url || 'default'}`;
  }

  /**
   * 获取默认头像
   * @returns 默认头像URL
   */
  getFallbackAvatar(): string {
    return this.fallbackAvatar;
  }

  /**
   * 更新智能体头像
   * @param agent - 智能体对象
   * @param newAvatarUrl - 新头像URL
   */
  async updateAvatar(agent: Agent, newAvatarUrl: string): Promise<void> {
    if (!agent || !newAvatarUrl) {
      return;
    }

    try {
      // 更新缓存
      const cacheKey = this.getCacheKey(agent);
      this.avatarCache.set(cacheKey, newAvatarUrl);

      // 更新 store 中的智能体信息
      const store = useAgentStore.getState();
      if (store.agents) {
        const agentId = agent.id || agent.sessionId;
        if (agentId) {
          store.updateAgent(agentId, {
            avatar_url: newAvatarUrl,
            avatar_cid: null,
          });
        }
      }

      // 通知监听器
      this.notifyListeners(agent);

      console.log('[AvatarManager] 头像更新成功:', agent.id);
    } catch (error) {
      console.error('[AvatarManager] 头像更新失败:', error);
    }
  }

  /**
   * 预加载头像
   * @param agents - 智能体列表
   */
  async preloadAvatars(agents: Agent[]): Promise<void> {
    if (!agents || agents.length === 0) {
      return;
    }

    const preloadPromises = agents.map(async (agent) => {
      try {
        await this.resolveAvatar(agent);
      } catch (error) {
        console.warn(`[AvatarManager] 预加载头像失败: ${agent.id}`, error);
      }
    });

    await Promise.allSettled(preloadPromises);
    console.log(`[AvatarManager] 预加载了 ${agents.length} 个头像`);
  }

  /**
   * 清除缓存
   * @param agent - 可选，指定智能体的缓存
   */
  clearCache(agent?: Agent): void {
    if (agent) {
      const cacheKey = this.getCacheKey(agent);
      this.avatarCache.delete(cacheKey);
    } else {
      // 清除所有缓存
      this.avatarCache.clear();
    }
  }

  /**
   * 添加头像变更监听器
   * @param listener - 监听器函数
   * @returns 清理函数
   */
  addListener(listener: (agent: Agent) => void): () => void {
    this.listeners.add(listener);
    
    // 返回清理函数
    return () => {
      this.listeners.delete(listener);
    };
  }

  /**
   * 移除头像变更监听器
   * @param listener - 监听器函数
   */
  removeListener(listener: (agent: Agent) => void): void {
    this.listeners.delete(listener);
  }

  /**
   * 通知所有监听器
   * @param agent - 更新头像的智能体
   */
  private notifyListeners(agent: Agent): void {
    this.listeners.forEach(listener => {
      try {
        listener(agent);
      } catch (error) {
        console.error('[AvatarManager] 监听器执行失败:', error);
      }
    });
  }

  /**
   * 获取缓存统计信息
   * @returns 缓存统计
   */
  getCacheStats(): {
    size: number;
    keys: string[];
  } {
    return {
      size: this.avatarCache.size,
      keys: Array.from(this.avatarCache.keys()),
    };
  }

  /**
   * 检查头像是否有效
   * @param avatarUrl - 头像URL
   * @returns 是否有效
   */
  async isValidAvatar(avatarUrl: string): Promise<boolean> {
    if (!avatarUrl) {
      return false;
    }

    try {
      // 使用图片代理服务检查
      const proxiedUrl = imageProxyService.getProxiedUrl(avatarUrl);
      const response = await fetch(proxiedUrl, { method: 'HEAD' });
      return response.ok;
    } catch (error) {
      console.warn('[AvatarManager] 头像验证失败:', avatarUrl, error);
      return false;
    }
  }

  /**
   * 批量验证头像
   * @param avatarUrls - 头像URL列表
   * @returns 验证结果
   */
  async batchValidateAvatars(avatarUrls: string[]): Promise<Record<string, boolean>> {
    const results: Record<string, boolean> = {};
    
    const validationPromises = avatarUrls.map(async (url) => {
      const isValid = await this.isValidAvatar(url);
      results[url] = isValid;
    });

    await Promise.allSettled(validationPromises);
    return results;
  }

  /**
   * 处理文件上传
   * @param file - 上传的文件
   * @returns 头像URL
   */
  async processFileUpload(file: File): Promise<string> {
    if (!file) {
      throw new Error('请选择头像文件')
    }

    // 验证文件类型
    if (!file.type.startsWith('image/')) {
      throw new Error('请选择图片文件')
    }

    // 验证文件大小 (最大 5MB)
    const maxSize = 5 * 1024 * 1024
    if (file.size > maxSize) {
      throw new Error('头像文件不能超过 5MB')
    }

    try {
      // 尝试上传到 IPFS
      const result = await agentAssetsService.uploadAvatar(file)
      if (result?.cid) {
        // IPFS 上传成功，使用 IPFS URL
        const avatarUrl = await agentAssetsService.getAvatar(result.cid)
        console.log('[AvatarManager] 头像上传到 IPFS 成功:', result.cid)
        return avatarUrl
      }
    } catch (error) {
      console.warn('[AvatarManager] IPFS 上传失败，将使用 base64:', error)
    }

    // IPFS 不可用，使用 base64 编码
    return new Promise((resolve, reject) => {
      const reader = new FileReader()
      reader.onload = () => {
        const base64 = reader.result as string
        console.log('[AvatarManager] 头像使用 base64 编码')
        resolve(base64)
      }
      reader.onerror = () => {
        reject(new Error('头像文件读取失败'))
      }
      reader.readAsDataURL(file)
    })
  }
}

// 创建单例实例
const avatarManager = new AvatarManager();

export default avatarManager;
export { AvatarManager };
