/**
 * IPFS 内容服务 - 统一的 IPFS 内容获取和上传
 */

import { fetchJsonFromGateway } from './utils/gatewayUtils';

const DEFAULT_IPFS_API = import.meta.env.VITE_IPFS_API_URL || 'http://127.0.0.1:5001';

interface ContentResult<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  source: 'ipfs' | 'ipns' | 'cache';
}

interface UploadResult {
  success: boolean;
  cid?: string;
  error?: string;
  size?: number;
}

export class IpfsContentService {
  /**
   * 从 IPFS CID 获取内容
   * @param cid - IPFS CID
   * @returns 内容数据
   */
  async getContent<T = any>(cid: string): Promise<ContentResult<T>> {
    console.log(`[IpfsContentService] 从 IPFS 获取内容: ${cid}`);
    
    try {
      const data = await fetchJsonFromGateway<T>(`/ipfs/${cid}`);
      console.log(`[IpfsContentService] 获取内容成功: ${cid}`);
      return { success: true, data, source: 'ipfs' };
    } catch (error) {
      console.error(`[IpfsContentService] 获取内容失败: ${cid}`, error);
      return { 
        success: false, 
        error: (error as Error).message, 
        source: 'ipfs' 
      };
    }
  }
  
  /**
   * 从 IPNS 获取内容
   * @param ipnsName - IPNS 名称
   * @returns 内容数据
   */
  async getContentFromIpns<T = any>(ipnsName: string): Promise<ContentResult<T>> {
    console.log(`[IpfsContentService] 从 IPNS 获取内容: ${ipnsName}`);
    
    try {
      const data = await fetchJsonFromGateway<T>(`/ipns/${ipnsName}`);
      console.log(`[IpfsContentService] IPNS 获取内容成功: ${ipnsName}`);
      return { success: true, data, source: 'ipns' };
    } catch (error) {
      console.error(`[IpfsContentService] IPNS 获取内容失败: ${ipnsName}`, error);
      return { 
        success: false, 
        error: (error as Error).message, 
        source: 'ipns' 
      };
    }
  }

  /**
   * 解析 IPNS 到 CID
   * @param ipnsName - IPNS 名称
   * @returns CID 字符串
   */
  async resolveIpns(ipnsName: string): Promise<ContentResult<string>> {
    console.log(`[IpfsContentService] 解析 IPNS: ${ipnsName}`);
    
    try {
      // 通过 IPFS 网关解析 IPNS
      const response = await fetch(`${DEFAULT_IPFS_API}/api/v0/name/resolve/${ipnsName}`);
      
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const result = await response.json();
      const cid = result.Path?.replace('/ipfs/', '') || result.Cid;
      
      if (!cid) {
        throw new Error('无法从响应中提取 CID');
      }

      console.log(`[IpfsContentService] IPNS 解析成功: ${ipnsName} -> ${cid}`);
      return { success: true, data: cid, source: 'ipns' };
    } catch (error) {
      console.error(`[IpfsContentService] IPNS 解析失败: ${ipnsName}`, error);
      return { 
        success: false, 
        error: (error as Error).message, 
        source: 'ipns' 
      };
    }
  }

  /**
   * 上传内容到 IPFS
   * @param content - 要上传的内容
   * @param options - 上传选项
   * @returns 上传结果
   */
  async uploadContent(
    content: any, 
    options: {
      pin?: boolean;
      wrapWithDirectory?: boolean;
      timeout?: number;
    } = {}
  ): Promise<UploadResult> {
    console.log(`[IpfsContentService] 上传内容到 IPFS`);
    
    try {
      const formData = new FormData();
      formData.append('file', new Blob([JSON.stringify(content)], { type: 'application/json' }));

      const params = new URLSearchParams();
      if (options.pin) params.append('pin', 'true');
      if (options.wrapWithDirectory) params.append('wrap-with-directory', 'true');

      const response = await fetch(`${DEFAULT_IPFS_API}/api/v0/add?${params.toString()}`, {
        method: 'POST',
        body: formData,
        signal: options.timeout ? AbortSignal.timeout(options.timeout) : undefined,
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const result = await response.json();
      const cid = result.Hash || result.Cid;
      const size = result.Size || result.size;

      if (!cid) {
        throw new Error('无法从响应中提取 CID');
      }

      console.log(`[IpfsContentService] 上传成功: ${cid} (${size} bytes)`);
      return { success: true, cid, size };
    } catch (error) {
      console.error(`[IpfsContentService] 上传失败:`, error);
      return { 
        success: false, 
        error: (error as Error).message 
      };
    }
  }

  /**
   * Pin IPFS 内容
   * @param cid - 要 pin 的 CID
   * @returns Pin 结果
   */
  async pinContent(cid: string): Promise<ContentResult<void>> {
    console.log(`[IpfsContentService] Pin 内容: ${cid}`);
    
    try {
      const response = await fetch(`${DEFAULT_IPFS_API}/api/v0/pin/add?arg=${cid}`, {
        method: 'POST',
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const result = await response.json();
      
      if (result.Code !== 0) {
        throw new Error(result.Message || 'Pin 操作失败');
      }

      console.log(`[IpfsContentService] Pin 成功: ${cid}`);
      return { success: true, source: 'ipfs' };
    } catch (error) {
      console.error(`[IpfsContentService] Pin 失败: ${cid}`, error);
      return { 
        success: false, 
        error: (error as Error).message, 
        source: 'ipfs' 
      };
    }
  }

  /**
   * Unpin IPFS 内容
   * @param cid - 要 unpin 的 CID
   * @returns Unpin 结果
   */
  async unpinContent(cid: string): Promise<ContentResult<void>> {
    console.log(`[IpfsContentService] Unpin 内容: ${cid}`);
    
    try {
      const response = await fetch(`${DEFAULT_IPFS_API}/api/v0/pin/rm?arg=${cid}`, {
        method: 'POST',
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const result = await response.json();
      
      if (result.Code !== 0) {
        throw new Error(result.Message || 'Unpin 操作失败');
      }

      console.log(`[IpfsContentService] Unpin 成功: ${cid}`);
      return { success: true, source: 'ipfs' };
    } catch (error) {
      console.error(`[IpfsContentService] Unpin 失败: ${cid}`, error);
      return { 
        success: false, 
        error: (error as Error).message, 
        source: 'ipfs' 
      };
    }
  }

  /**
   * 检查内容是否被 Pin
   * @param cid - 要检查的 CID
   * @returns 检查结果
   */
  async isPinned(cid: string): Promise<ContentResult<boolean>> {
    console.log(`[IpfsContentService] 检查 Pin 状态: ${cid}`);
    
    try {
      const response = await fetch(`${DEFAULT_IPFS_API}/api/v0/pin/ls?arg=${cid}`);
      
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const result = await response.json();
      const isPinned = result.Keys && result.Keys[cid] !== undefined;

      console.log(`[IpfsContentService] Pin 状态检查: ${cid} -> ${isPinned}`);
      return { success: true, data: isPinned, source: 'ipfs' };
    } catch (error) {
      console.error(`[IpfsContentService] Pin 状态检查失败: ${cid}`, error);
      return { 
        success: false, 
        error: (error as Error).message, 
        source: 'ipfs' 
      };
    }
  }

  /**
   * 获取 IPFS 节点信息
   * @returns 节点信息
   */
  async getNodeInfo(): Promise<ContentResult<any>> {
    console.log(`[IpfsContentService] 获取节点信息`);
    
    try {
      const response = await fetch(`${DEFAULT_IPFS_API}/api/v0/id`);
      
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const data = await response.json();
      console.log(`[IpfsContentService] 节点信息获取成功`);
      return { success: true, data, source: 'ipfs' };
    } catch (error) {
      console.error(`[IpfsContentService] 获取节点信息失败:`, error);
      return { 
        success: false, 
        error: (error as Error).message, 
        source: 'ipfs' 
      };
    }
  }

  /**
   * 批量获取内容
   * @param cids - CID 列表
   * @param options - 获取选项
   * @returns 获取结果数组
   */
  async batchGetContent<T = any>(
    cids: string[], 
    options: {
      concurrency?: number;
      timeout?: number;
    } = {}
  ): Promise<ContentResult<T>[]> {
    const { concurrency = 5, timeout } = options;
    console.log(`[IpfsContentService] 批量获取内容: ${cids.length} 个 CID`);

    const results: ContentResult<T>[] = [];
    
    // 分批处理
    for (let i = 0; i < cids.length; i += concurrency) {
      const batch = cids.slice(i, i + concurrency);
      
      const batchPromises = batch.map(cid => 
        this.getContent<T>(cid)
          .catch(error => ({
            success: false,
            error: (error as Error).message,
            source: 'ipfs' as const,
          }))
      );

      const batchResults = await Promise.allSettled(batchPromises);
      
      batchResults.forEach((result, index) => {
        if (result.status === 'fulfilled') {
          results.push(result.value);
        } else {
          results.push({
            success: false,
            error: `获取 CID ${batch[index]} 失败: ${result.reason}`,
            source: 'ipfs',
          });
        }
      });
    }

    const successCount = results.filter(r => r.success).length;
    console.log(`[IpfsContentService] 批量获取完成: ${successCount}/${cids.length} 成功`);

    return results;
  }

  /**
   * 获取服务统计信息
   */
  getStats(): {
    defaultApi: string;
    supportedOperations: string[];
  } {
    return {
      defaultApi: DEFAULT_IPFS_API,
      supportedOperations: [
        'getContent',
        'getContentFromIpns',
        'resolveIpns',
        'uploadContent',
        'pinContent',
        'unpinContent',
        'isPinned',
        'getNodeInfo',
        'batchGetContent',
      ],
    };
  }
}

// 创建单例实例
const ipfsContentService = new IpfsContentService();

export default ipfsContentService;
export type { ContentResult, UploadResult };
