/**
 * 图片代理服务
 * 在Tauri环境中将外部图片URL转换为data URL
 */

interface CacheEntry {
  dataUrl: string;
  timestamp: number;
}

class ImageProxyService {
  private cache = new Map<string, CacheEntry>();
  private loadingPromises = new Map<string, Promise<string | null>>();
  private readonly CACHE_DURATION = 24 * 60 * 60 * 1000; // 24小时
  private readonly MAX_CACHE_SIZE = 100; // 最大缓存数量

  /**
   * 获取图片的data URL
   * @param imageUrl - 图片URL
   * @returns data URL
   */
  async getImageDataUrl(imageUrl: string): Promise<string | null> {
    if (!imageUrl) {
      return null;
    }

    // 如果已经是data URL，直接返回
    if (imageUrl.startsWith('data:')) {
      return imageUrl;
    }

    // 检查缓存
    const cached = this.getCachedEntry(imageUrl);
    if (cached) {
      return cached.dataUrl;
    }

    // 检查是否正在加载
    const existingPromise = this.loadingPromises.get(imageUrl);
    if (existingPromise) {
      return existingPromise;
    }

    // 开始加载
    const loadingPromise = this.loadImageDataUrl(imageUrl);
    this.loadingPromises.set(imageUrl, loadingPromise);

    try {
      const dataUrl = await loadingPromise;
      return dataUrl;
    } finally {
      // 清除加载中的Promise
      this.loadingPromises.delete(imageUrl);
    }
  }

  /**
   * 获取代理后的URL（用于img标签的src属性）
   * @param imageUrl - 原始图片URL
   * @returns 代理URL或data URL
   */
  getProxiedUrl(imageUrl: string): string {
    if (!imageUrl) {
      return '';
    }

    // 如果已经是data URL，直接返回
    if (imageUrl.startsWith('data:')) {
      return imageUrl;
    }

    // 在Tauri环境中，使用代理URL
    if (this.isTauriEnvironment()) {
      return `tauri://image-proxy?url=${encodeURIComponent(imageUrl)}`;
    }

    // 在浏览器环境中，直接返回原URL
    return imageUrl;
  }

  /**
   * 预加载图片
   * @param imageUrls - 图片URL数组
   * @returns Promise数组
   */
  async preloadImages(imageUrls: string[]): Promise<void> {
    if (!imageUrls || imageUrls.length === 0) {
      return;
    }

    const preloadPromises = imageUrls
      .filter(url => url && !url.startsWith('data:'))
      .map(url => this.getImageDataUrl(url).catch(error => {
        console.warn('[ImageProxyService] 预加载图片失败:', url, error);
        return null;
      }));

    await Promise.allSettled(preloadPromises);
    console.log(`[ImageProxyService] 预加载了 ${preloadPromises.length} 张图片`);
  }

  /**
   * 清除缓存
   * @param imageUrl - 可选，指定要清除的图片URL
   */
  clearCache(imageUrl?: string): void {
    if (imageUrl) {
      this.cache.delete(imageUrl);
      this.loadingPromises.delete(imageUrl);
    } else {
      // 清除所有缓存
      this.cache.clear();
      this.loadingPromises.clear();
    }
  }

  /**
   * 清除过期缓存
   */
  clearExpiredCache(): void {
    const now = Date.now();
    const expiredKeys: string[] = [];

    for (const [url, entry] of this.cache.entries()) {
      if (now - entry.timestamp > this.CACHE_DURATION) {
        expiredKeys.push(url);
      }
    }

    expiredKeys.forEach(key => {
      this.cache.delete(key);
    });

    if (expiredKeys.length > 0) {
      console.log(`[ImageProxyService] 清除了 ${expiredKeys.length} 个过期缓存`);
    }
  }

  /**
   * 获取缓存统计信息
   */
  getCacheStats(): {
    size: number;
    loadingCount: number;
    memoryUsage: string;
  } {
    const loadingCount = this.loadingPromises.size;
    let totalSize = 0;

    for (const entry of this.cache.values()) {
      // 估算data URL的大小（字符数）
      totalSize += entry.dataUrl.length;
    }

    return {
      size: this.cache.size,
      loadingCount,
      memoryUsage: `${(totalSize / 1024 / 1024).toFixed(2)} MB`,
    };
  }

  /**
   * 从缓存中获取条目
   */
  private getCachedEntry(imageUrl: string): CacheEntry | null {
    const entry = this.cache.get(imageUrl);
    if (!entry) {
      return null;
    }

    // 检查是否过期
    if (Date.now() - entry.timestamp > this.CACHE_DURATION) {
      this.cache.delete(imageUrl);
      return null;
    }

    return entry;
  }

  /**
   * 加载图片数据URL
   */
  private async loadImageDataUrl(imageUrl: string): Promise<string | null> {
    try {
      let dataUrl: string;

      if (this.isTauriEnvironment()) {
        // 在Tauri环境中使用Tauri API
        dataUrl = await this.loadImageViaTauri(imageUrl);
      } else {
        // 在浏览器环境中使用fetch
        dataUrl = await this.loadImageViaFetch(imageUrl);
      }

      // 缓存结果
      this.cacheEntry(imageUrl, dataUrl);
      
      return dataUrl;
    } catch (error) {
      console.error('[ImageProxyService] 加载图片失败:', imageUrl, error);
      return null;
    }
  }

  /**
   * 通过Tauri API加载图片
   */
  private async loadImageViaTauri(imageUrl: string): Promise<string> {
    // 暂时直接使用 fetch，避免 Tauri 依赖问题
    console.warn('[ImageProxyService] 暂时使用 fetch 方式加载图片');
    return this.loadImageViaFetch(imageUrl);
  }

  /**
   * 通过fetch加载图片
   */
  private async loadImageViaFetch(imageUrl: string): Promise<string> {
    const response = await fetch(imageUrl, {
      mode: 'cors',
      credentials: 'omit',
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    const blob = await response.blob();
    return this.blobToDataUrl(blob);
  }

  /**
   * 将Blob转换为data URL
   */
  private blobToDataUrl(blob: Blob): Promise<string> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      
      reader.onload = () => {
        const dataUrl = reader.result as string;
        resolve(dataUrl);
      };
      
      reader.onerror = () => {
        reject(new Error('Failed to convert blob to data URL'));
      };
      
      reader.readAsDataURL(blob);
    });
  }

  /**
   * 缓存条目
   */
  private cacheEntry(imageUrl: string, dataUrl: string): void {
    // 检查缓存大小限制
    if (this.cache.size >= this.MAX_CACHE_SIZE) {
      // 删除最旧的条目
      let oldestKey = '';
      let oldestTime = Date.now();

      for (const [key, entry] of this.cache.entries()) {
        if (entry.timestamp < oldestTime) {
          oldestTime = entry.timestamp;
          oldestKey = key;
        }
      }

      if (oldestKey) {
        this.cache.delete(oldestKey);
      }
    }

    // 添加新条目
    this.cache.set(imageUrl, {
      dataUrl,
      timestamp: Date.now(),
    });
  }

  /**
   * 检查是否在Tauri环境中
   */
  private isTauriEnvironment(): boolean {
    return typeof window !== 'undefined' && 
           (window as any).__TAURI__ !== undefined;
  }

  /**
   * 验证图片URL
   */
  isValidImageUrl(url: string): boolean {
    if (!url || typeof url !== 'string') {
      return false;
    }

    // 如果已经是data URL，检查格式
    if (url.startsWith('data:')) {
      return url.startsWith('data:image/');
    }

    try {
      const urlObj = new URL(url);
      const validProtocols = ['http:', 'https:', 'ipfs:', 'ipns:'];
      return validProtocols.includes(urlObj.protocol);
    } catch {
      return false;
    }
  }

  /**
   * 批量验证图片URL
   */
  batchValidateUrls(urls: string[]): Record<string, boolean> {
    const results: Record<string, boolean> = {};
    
    urls.forEach(url => {
      results[url] = this.isValidImageUrl(url);
    });

    return results;
  }

  /**
   * 销毁服务
   */
  destroy(): void {
    this.cache.clear();
    this.loadingPromises.clear();
  }
}

// 创建单例实例
const imageProxyService = new ImageProxyService();

export default imageProxyService;
export { ImageProxyService };
export type { CacheEntry };
