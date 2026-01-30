/**
 * Agent 解析器服务 - 统一的 Agent 解析逻辑
 */
import { cleanTarget, normalizeIpns, cleanIpnsKey } from './utils/ipnsUtils';
import { tryWithFallback } from './utils/fallbackStrategy';
import { parseDidDocumentToAgent } from './didDocumentParser';
import ipfsContentService from './ipfsContentService';
import apiClient from './api';

// 智能体元数据
export interface AgentMetadata {
  id: string;
  name?: string;
  description?: string;
  display_name?: string;
  did?: string;
  ipns?: string;
  cid?: string;
  role_description?: string;
  [key: string]: any;
}

// 加载结果
export interface LoadAgentResult {
  success: boolean;
  agent?: AgentMetadata;
  didDocument?: any;
  source?: string;
  error?: string;
}

// IPNS 解析身份
export interface IpnsIdentity {
  cid: string;
  did: string;
}

const DEFAULT_IPFS_API = (import.meta as any).env?.VITE_IPFS_API_URL || 'http://127.0.0.1:5001';
const DEFAULT_IPFS_GATEWAY = (import.meta as any).env?.VITE_IPFS_GATEWAY_URL || 'http://127.0.0.1:8080';

export class AgentResolverService {
  /**
   * 从网络加载智能体（自动识别 IPNS 或 CID）
   * @param target - IPNS 或 CID
   * @returns 智能体元数据
   */
  async loadAgentFromNetwork(target: string): Promise<LoadAgentResult> {
    if (!target || typeof target !== 'string') {
      return { success: false, error: '无效的目标标识' };
    }

    const cleaned = cleanTarget(target);

    if (cleaned.type === 'ipns') {
      return this.loadAgentFromIpns(cleaned.value);
    } else if (cleaned.type === 'cid') {
      return this.loadAgentFromIpfs(cleaned.value);
    } else {
      // 未知格式，尝试作为 IPNS 解析
      console.log(`[AgentResolverService] 未知格式，尝试作为 IPNS 解析: ${cleaned.value}`);
      return this.loadAgentFromIpns(cleaned.value);
    }
  }

  /**
   * 从 IPFS CID 加载智能体
   * @param cid - IPFS CID
   * @returns 智能体元数据
   */
  async loadAgentFromIpfs(cid: string): Promise<LoadAgentResult> {
    console.log(`[AgentResolverService] 从 IPFS 加载智能体: ${cid}`);

    const result = await ipfsContentService.getContent(cid);

    if (!result.success) {
      return result as LoadAgentResult;
    }

    const didDocument = result.data;
    const agentMetadata = parseDidDocumentToAgent(didDocument, { cid });

    return {
      success: true,
      agent: agentMetadata,
      didDocument,
      source: result.source,
    };
  }

  /**
   * 从 IPNS 名称加载智能体
   * @param ipnsName - IPNS 名称
   * @returns 智能体元数据
   */
  async loadAgentFromIpns(ipnsName: string): Promise<LoadAgentResult> {
    console.log(`[AgentResolverService] 从 IPNS 加载智能体: ${ipnsName}`);

    // 规范化 IPNS 名称
    const normalizedName = normalizeIpns(ipnsName) || ipnsName;

    // 定义解析策略
    const strategies: Array<() => Promise<LoadAgentResult>> = [
      // 策略 1: 通过 Tauri 解析 IPNS
      async () => {
        const identity = await this._resolveViaTauri(normalizedName);
        const loadResult = await this.loadAgentFromIpfs(identity.cid);

        if (loadResult.success && loadResult.agent) {
          loadResult.agent.ipns = normalizedName;
          loadResult.agent.did = identity.did;
          loadResult.source = 'ipns-tauri';
          return loadResult;
        }

        // Tauri 解析成功但加载 DID 文档失败，尝试 Workers API
        throw new Error('Tauri resolved but DID document load failed');
      },

      // 策略 2: 通过 Workers API 解析 IPNS
      async () => {
        const identity = await this._resolveViaWorkersApi(ipnsName);
        const loadResult = await this.loadAgentFromIpfs(identity.cid);

        if (loadResult.success && loadResult.agent) {
          loadResult.agent.ipns = normalizedName;
          loadResult.agent.did = identity.did;
          loadResult.source = 'ipns-workers-api';
          return loadResult;
        }

        throw new Error('Workers API resolved but DID document load failed');
      },

      // 策略 3: 通过 Gateway 直接访问 IPNS
      async () => {
        const result = await ipfsContentService.getContentFromIpns(ipnsName);

        if (!result.success) {
          throw new Error(result.error);
        }

        const didDocument = result.data;
        const agentMetadata = parseDidDocumentToAgent(didDocument, { ipns: normalizedName });

        return {
          success: true,
          agent: agentMetadata,
          didDocument,
          source: 'ipns-gateway',
        };
      },
    ];

    try {
      const { result } = await tryWithFallback(strategies, {
        onError: (index: number, error: Error) => {
          console.warn(`[AgentResolverService] 策略 ${index} 失败:`, error.message || error);
        },
      });

      return result;
    } catch (error: any) {
      console.error(`[AgentResolverService] 所有解析策略都失败:`, error);
      return {
        success: false,
        error: error.message || '加载失败',
        source: 'ipns',
      };
    }
  }

  /**
   * 通过 Tauri 解析 IPNS
   * @private
   */
  private async _resolveViaTauri(ipnsName: string): Promise<IpnsIdentity> {
    const { invoke } = await import('@tauri-apps/api/core');

    const identity = await invoke('get_local_diap_identity', {
      ipnsName,
      ipfsApiUrl: DEFAULT_IPFS_API,
      ipfsGatewayUrl: DEFAULT_IPFS_GATEWAY,
    }) as IpnsIdentity;

    console.log(`[AgentResolverService] Tauri 解析 IPNS 成功:`, identity);
    return identity;
  }

  /**
   * 通过 Workers API 解析 IPNS
   * @private
   */
  private async _resolveViaWorkersApi(ipnsName: string): Promise<IpnsIdentity> {
    // 清理 IPNS key
    const ipnsKey = cleanIpnsKey(ipnsName);
    if (!ipnsKey) {
      throw new Error(`Invalid IPNS name: ${ipnsName}`);
    }

    const response = await apiClient.post('/agent/diap/get-identity', {
      ipns_name: ipnsKey,
    });

    if (!response.data || !response.data.identity) {
      throw new Error('Workers API 返回的数据格式不正确');
    }

    console.log(`[AgentResolverService] Workers API 解析 IPNS 成功:`, response.data.identity);
    return response.data.identity;
  }
}

export default new AgentResolverService();
