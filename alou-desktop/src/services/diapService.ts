/**
 * DIAP 服务 - 去中心化身份和代理协议服务
 */
import { invoke } from '@tauri-apps/api/core';

const DEFAULT_IPFS_API = import.meta.env.VITE_IPFS_API_URL || 'http://127.0.0.1:5001';
const DEFAULT_IPFS_GATEWAY =
  import.meta.env.VITE_IPFS_GATEWAY_URL || 'http://127.0.0.1:8080';

export interface CreateIdentityParams {
  name?: string;
  description?: string;
  sessionId?: string;
  avatarCid?: string;
  mcpConfigCid?: string;
  customPrompt?: string;
  ipfsApiUrl?: string;
  ipfsGatewayUrl?: string;
}

export interface DIAPIdentity {
  did: string;
  ipns: string;
  cid: string;
  publicKey: string;
  avatar_cid?: string;
}

export interface DIAPOperationResult {
  success: boolean;
  data?: DIAPIdentity;
  error?: string;
}

class DiapService {
  /**
   * 带重试的 IPFS 操作包装器
   */
  async withRetry<T>(
    operation: () => Promise<T>,
    operationName: string,
    maxRetries = 3,
    delayMs = 1500
  ): Promise<T> {
    let lastError: Error | undefined;
    for (let i = 0; i < maxRetries; i++) {
      try {
        return await operation();
      } catch (error) {
        lastError = error instanceof Error ? error : new Error(String(error));
        const errorMsg = lastError.message || lastError.toString() || '';

        // 如果是 502 错误，等待后重试
        if (errorMsg.includes('502') || errorMsg.includes('Bad Gateway')) {
          if (i < maxRetries - 1) {
            console.log(
              `[DiapService] ${operationName} 遇到 502 错误，等待 ${delayMs}ms 后重试 (${i + 1}/${maxRetries})`
            );
            await new Promise((resolve) => setTimeout(resolve, delayMs));
            continue;
          }
        }

        // 其他错误或重试次数用完，直接抛出
        throw error;
      }
    }
    throw lastError;
  }

  async createLocalIdentity(params: CreateIdentityParams = {}): Promise<DIAPIdentity> {
    const {
      name,
      description,
      sessionId,
      avatarCid,
      mcpConfigCid,
      customPrompt,
      ipfsApiUrl,
      ipfsGatewayUrl,
    } = params;

    return this.withRetry(
      async () => {
        const response = await invoke<DIAPIdentity>('create_local_diap_identity', {
          params: {
            agent_name: name,
            agent_description: description,
            session_id: sessionId,
            avatar_cid: avatarCid,
            mcp_config_cid: mcpConfigCid,
            custom_prompt: customPrompt,
            ipfs_api_url: ipfsApiUrl || DEFAULT_IPFS_API,
            ipfs_gateway_url: ipfsGatewayUrl || DEFAULT_IPFS_GATEWAY,
          },
        });
        return response;
      },
      '创建 DIAP Identity',
      3,
      1500
    );
  }

  async getLocalIdentity(
    ipnsName: string,
    ipfsApiUrl?: string,
    ipfsGatewayUrl?: string
  ): Promise<DIAPIdentity> {
    return this.withRetry(
      async () => {
        const response = await invoke<DIAPIdentity>('get_local_diap_identity', {
          ipns_name: ipnsName,
          ipfs_api_url: ipfsApiUrl || DEFAULT_IPFS_API,
          ipfs_gateway_url: ipfsGatewayUrl || DEFAULT_IPFS_GATEWAY,
        });
        return response;
      },
      '获取 DIAP Identity',
      3,
      1500
    );
  }

  async updateLocalIdentity(
    ipnsKey: string,
    cid: string,
    ipfsApiUrl?: string,
    ipfsGatewayUrl?: string
  ): Promise<DIAPIdentity> {
    return this.withRetry(
      async () => {
        const response = await invoke<DIAPIdentity>('update_local_diap_identity', {
          ipns_key: ipnsKey,
          cid: cid,
          ipfs_api_url: ipfsApiUrl || DEFAULT_IPFS_API,
          ipfs_gateway_url: ipfsGatewayUrl || DEFAULT_IPFS_GATEWAY,
        });
        return response;
      },
      '更新 DIAP Identity',
      3,
      1500
    );
  }
}

export default new DiapService();
