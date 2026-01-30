/**
 * IPFS Gateway 工具函数
 */

const DEFAULT_IPFS_GATEWAY = import.meta.env.VITE_IPFS_GATEWAY_URL || 'http://127.0.0.1:8080';

/**
 * 公共 IPFS Gateway 列表
 */
export const PUBLIC_GATEWAYS = [
  'https://ipfs.io',
  'https://gateway.ipfs.io',
  'https://dweb.link',
] as const;

interface FetchOptions {
  headers?: Record<string, string>;
  timeout?: number;
  retries?: number;
}

interface GatewayResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  gateway?: string;
}

/**
 * 从本地 Gateway 获取内容
 * @param path - IPFS 路径 (/ipfs/... 或 /ipns/...)
 * @param options - 选项
 * @returns Response
 */
export async function fetchFromLocalGateway(
  path: string, 
  options: FetchOptions = {}
): Promise<Response> {
  const gatewayUrl = `${DEFAULT_IPFS_GATEWAY}${path}`;
  return fetch(gatewayUrl, {
    method: 'GET',
    headers: { 'Accept': 'application/json', ...options.headers },
    signal: options.timeout ? AbortSignal.timeout(options.timeout) : undefined,
    ...options,
  });
}

/**
 * 从公共 Gateway 获取内容
 * @param path - IPFS 路径
 * @param options - 选项
 * @returns Response
 */
export async function fetchFromPublicGateway(
  path: string, 
  options: FetchOptions = {}
): Promise<Response> {
  const gatewayUrl = `${PUBLIC_GATEWAYS[0]}${path}`;
  return fetch(gatewayUrl, {
    method: 'GET',
    headers: { 'Accept': 'application/json', ...options.headers },
    signal: options.timeout ? AbortSignal.timeout(options.timeout) : undefined,
    ...options,
  });
}

/**
 * 从 Gateway 获取 JSON 数据
 * @param path - IPFS 路径
 * @param options - 选项
 * @returns JSON 数据
 */
export async function fetchJsonFromGateway<T = any>(
  path: string, 
  options: FetchOptions = {}
): Promise<T> {
  const response = await fetchFromLocalGateway(path, options);
  
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
  }

  return response.json();
}

/**
 * 使用回退策略获取内容
 * @param path - IPFS 路径
 * @param options - 选项
 * @returns 内容数据
 */
export async function fetchWithFallback<T = any>(
  path: string, 
  options: FetchOptions = {}
): Promise<GatewayResponse<T>> {
  const errors: string[] = [];
  
  // 尝试本地 Gateway
  try {
    const data = await fetchJsonFromGateway<T>(path, options);
    return {
      success: true,
      data,
      gateway: DEFAULT_IPFS_GATEWAY,
    };
  } catch (error) {
    errors.push(`Local Gateway: ${(error as Error).message}`);
  }

  // 尝试公共 Gateway
  for (const gateway of PUBLIC_GATEWAYS) {
    try {
      const response = await fetch(`${gateway}${path}`, {
        method: 'GET',
        headers: { 'Accept': 'application/json', ...options.headers },
        signal: options.timeout ? AbortSignal.timeout(options.timeout) : undefined,
      });

      if (response.ok) {
        const data = await response.json();
        return {
          success: true,
          data,
          gateway,
        };
      }
    } catch (error) {
      errors.push(`${gateway}: ${(error as Error).message}`);
    }
  }

  return {
    success: false,
    error: `所有 Gateway 都无法访问: ${errors.join('; ')}`,
  };
}

/**
 * 验证 Gateway 可用性
 * @param gatewayUrl - Gateway URL
 * @returns 是否可用
 */
export async function testGatewayAvailability(gatewayUrl: string): Promise<boolean> {
  try {
    const response = await fetch(`${gatewayUrl}/ipfs/QmYwAPJzv5CZsnA625s3Xf2nemtYgP57d3uE3SyNvfbzmj`, {
      method: 'HEAD',
      signal: AbortSignal.timeout(5000),
    });
    return response.ok;
  } catch {
    return false;
  }
}

/**
 * 获取可用的 Gateway 列表
 * @returns 可用的 Gateway 列表
 */
export async function getAvailableGateways(): Promise<string[]> {
  const gateways = [DEFAULT_IPFS_GATEWAY, ...PUBLIC_GATEWAYS];
  const available: string[] = [];

  const testPromises = gateways.map(async (gateway) => {
    const isAvailable = await testGatewayAvailability(gateway);
    return { gateway, isAvailable };
  });

  const results = await Promise.allSettled(testPromises);
  
  results.forEach((result) => {
    if (result.status === 'fulfilled' && result.value.isAvailable) {
      available.push(result.value.gateway);
    }
  });

  return available;
}

/**
 * 构建 IPFS URL
 * @param cid - IPFS CID
 * @param gateway - Gateway URL（可选）
 * @returns 完整的 IPFS URL
 */
export function buildIpfsUrl(cid: string, gateway?: string): string {
  const baseUrl = gateway || DEFAULT_IPFS_GATEWAY;
  return `${baseUrl}/ipfs/${cid}`;
}

/**
 * 构建 IPNS URL
 * @param ipns - IPNS 名称
 * @param gateway - Gateway URL（可选）
 * @returns 完整的 IPNS URL
 */
export function buildIpnsUrl(ipns: string, gateway?: string): string {
  const baseUrl = gateway || DEFAULT_IPFS_GATEWAY;
  return `${baseUrl}/ipns/${ipns}`;
}

/**
 * 解析 IPFS URL
 * @param url - IPFS URL
 * @returns 解析结果
 */
export function parseIpfsUrl(url: string): {
  type: 'ipfs' | 'ipns' | 'invalid';
  gateway?: string;
  identifier?: string;
} {
  try {
    const urlObj = new URL(url);
    
    if (urlObj.pathname.startsWith('/ipfs/')) {
      return {
        type: 'ipfs',
        gateway: `${urlObj.protocol}//${urlObj.host}`,
        identifier: urlObj.pathname.replace('/ipfs/', ''),
      };
    }
    
    if (urlObj.pathname.startsWith('/ipns/')) {
      return {
        type: 'ipns',
        gateway: `${urlObj.protocol}//${urlObj.host}`,
        identifier: urlObj.pathname.replace('/ipns/', ''),
      };
    }
    
    return { type: 'invalid' };
  } catch {
    return { type: 'invalid' };
  }
}

/**
 * 验证 CID 格式
 * @param cid - CID 字符串
 * @returns 是否为有效的 CID
 */
export function isValidCid(cid: string): boolean {
  if (!cid || typeof cid !== 'string') {
    return false;
  }

  // 基本的 CID 格式检查
  const cidPatterns = [
    /^Qm[1-9A-HJ-NP-Za-km-z]{44,}$/, // CIDv0
    /^bafy[1-9A-HJ-NP-Za-km-z]{55,}$/, // CIDv1 (default base32)
    /^bafk[1-9A-HJ-NP-Za-km-z]{55,}$/, // CIDv1 (base32)
    /^z[1-9A-HJ-NP-Za-km-z]{48,}$/, // CIDv1 (base58)
  ];

  return cidPatterns.some(pattern => pattern.test(cid));
}

/**
 * 验证 IPNS 格式
 * @param ipns - IPNS 字符串
 * @returns 是否为有效的 IPNS
 */
export function isValidIpns(ipns: string): boolean {
  if (!ipns || typeof ipns !== 'string') {
    return false;
  }

  // IPNS 名称通常以 k51qzi5uqu5 开头
  const ipnsPattern = /^k51qzi5uqu5[1-9A-HJ-NP-Za-km-z]{59,}$/;
  return ipnsPattern.test(ipns) || ipns.startsWith('/ipns/');
}

/**
 * 标准化 IPFS 路径
 * @param path - 原始路径
 * @returns 标准化的路径
 */
export function normalizeIpfsPath(path: string): string {
  if (!path) return '';

  // 移除开头的斜杠
  let normalized = path.startsWith('/') ? path.slice(1) : path;
  
  // 确保以 ipfs/ 或 ipns/ 开头
  if (!normalized.startsWith('ipfs/') && !normalized.startsWith('ipns/')) {
    // 如果是 CID，添加 ipfs/ 前缀
    if (isValidCid(normalized)) {
      normalized = `ipfs/${normalized}`;
    } else if (isValidIpns(normalized)) {
      normalized = `ipns/${normalized}`;
    }
  }

  return normalized;
}

/**
 * 获取 Gateway 工具统计信息
 */
export function getGatewayUtilsStats(): {
  defaultGateway: string;
  publicGateways: readonly string[];
  supportedOperations: string[];
} {
  return {
    defaultGateway: DEFAULT_IPFS_GATEWAY,
    publicGateways: PUBLIC_GATEWAYS,
    supportedOperations: [
      'fetchFromLocalGateway',
      'fetchFromPublicGateway',
      'fetchJsonFromGateway',
      'fetchWithFallback',
      'testGatewayAvailability',
      'getAvailableGateways',
      'buildIpfsUrl',
      'buildIpnsUrl',
      'parseIpfsUrl',
      'isValidCid',
      'isValidIpns',
      'normalizeIpfsPath',
    ],
  };
}
