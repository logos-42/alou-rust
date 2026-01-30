/**
 * DID 文档解析器
 */

// 本地类型定义
interface AgentInfo {
  id: string;
  name: string;
  description?: string;
  mode?: string;
  did?: string;
  ipns?: string;
  capabilities?: string[];
  publicKey?: string;
  avatar_url?: string;
  [key: string]: unknown;
}

interface DIDDocument {
  '@context'?: string | string[];
  id: string;
  name?: string;
  description?: string;
  created?: string;
  updated?: string;
  verificationMethod?: Array<{
    id: string;
    type: string;
    controller: string;
    publicKeyPem?: string;
    publicKeyBase58?: string;
  }>;
  service?: Array<{
    id: string;
    type: string;
    serviceEndpoint: string | Record<string, any>;
  }>;
}

interface AdditionalInfo {
  cid?: string;
  ipns?: string;
  [key: string]: unknown;
}

interface ServiceEndpoint {
  id: string;
  type: string;
  serviceEndpoint: string | Record<string, unknown>;
}

/**
 * 解析 DID 文档，提取智能体元数据
 * @param didDocument - DID 文档
 * @param additionalInfo - 附加信息 { cid, ipns }
 * @returns 智能体元数据
 */
export function parseDidDocumentToAgent(
  didDocument: DIDDocument, 
  additionalInfo: AdditionalInfo = {}
): AgentInfo {
  console.log('[DidDocumentParser] 解析 DID 文档:', JSON.stringify(didDocument, null, 2));
  
  // 从 DID 文档中提取智能体信息
  const did = didDocument.id || undefined;
  
  // 从 service 数组中提取智能体配置
  const services = didDocument.service || [];
  console.log('[DidDocumentParser] 找到 services:', services.length, services);
  
  // 如果提供了 IPNS，验证 DID 文档中的 IPNS 是否匹配
  if (additionalInfo.ipns) {
    validateIpnsMatch(didDocument, additionalInfo.ipns);
  }
  
  // 尝试多种方式查找 AgentEndpoint 服务
  let agentService = services.find((s: ServiceEndpoint) => 
    s.type === 'AgentEndpoint' || s.type === 'agent' || s.id?.includes('#agent')
  );
  
  // 如果没找到，尝试查找其他可能的服务类型
  if (!agentService) {
    agentService = services.find((s: ServiceEndpoint) => 
      s.type === 'DecentralizedWebNode' || 
      s.type === 'DIDCommMessaging' ||
      s.serviceEndpoint
    );
  }

  const agentInfo: Partial<AgentInfo> = {
    did,
    ipns: additionalInfo.ipns,
    cid: additionalInfo.cid,
  };

  // 从服务中提取信息
  if (agentService) {
    console.log('[DidDocumentParser] 使用 agentService:', agentService);
    
    const endpoint = agentService.serviceEndpoint;
    
    if (typeof endpoint === 'string') {
      // 如果是字符串，尝试解析为 URL 或直接使用
      try {
        const url = new URL(endpoint);
        agentInfo.serviceEndpoint = {
          url: endpoint,
          type: agentService.type,
        };
        
        // 从 URL 中提取信息
        if (url.hostname) {
          agentInfo.name = url.hostname;
        }
        if (url.pathname) {
          agentInfo.description = url.pathname;
        }
      } catch {
        // 如果不是有效 URL，直接作为描述
        agentInfo.description = endpoint;
      }
    } else if (typeof endpoint === 'object' && endpoint !== null) {
      // 如果是对象，直接使用
      agentInfo.serviceEndpoint = endpoint;
      
      // 从对象中提取常见字段
      if (endpoint.name) {
        agentInfo.name = endpoint.name;
      }
      if (endpoint.description) {
        agentInfo.description = endpoint.description;
      }
      if (endpoint.avatar_cid || endpoint.avatarCid) {
        agentInfo.avatar_cid = endpoint.avatar_cid || endpoint.avatarCid;
        agentInfo.avatar_url = `https://ipfs.io/ipfs/${agentInfo.avatar_cid}`;
      }
      if (endpoint.role_description) {
        agentInfo.role_description = endpoint.role_description;
      }
      if (endpoint.mode) {
        agentInfo.mode = endpoint.mode as 'agent' | 'alou';
      }
    }
  }

  // 从 DID 文档的其他部分提取信息
  if (!agentInfo.name && didDocument.name) {
    agentInfo.name = didDocument.name;
  }
  
  if (!agentInfo.description && didDocument.description) {
    agentInfo.description = didDocument.description;
  }

  // 从 verificationMethod 中提取公钥信息
  if (didDocument.verificationMethod && didDocument.verificationMethod.length > 0) {
    const firstMethod = didDocument.verificationMethod[0];
    if (firstMethod.publicKeyBase58) {
      agentInfo.publicKey = firstMethod.publicKeyBase58;
    }
  }

  // 设置默认值
  agentInfo.name = agentInfo.name || 'Unknown Agent';
  agentInfo.description = agentInfo.description || 'No description available';
  agentInfo.mode = agentInfo.mode || 'agent';
  agentInfo.avatar_url = agentInfo.avatar_url || 'https://avatars.githubusercontent.com/u/16309930?v=4';

  console.log('[DidDocumentParser] 解析结果:', agentInfo);
  
  return agentInfo as AgentInfo;
}

/**
 * 验证 IPNS 匹配
 * @param didDocument - DID 文档
 * @param ipns - IPNS 名称
 */
function validateIpnsMatch(didDocument: DIDDocument, ipns: string): void {
  // 检查 DID 文档中是否包含相同的 IPNS
  const services = didDocument.service || [];
  const ipnsService = services.find((s: ServiceEndpoint) => 
    s.serviceEndpoint && 
    typeof s.serviceEndpoint === 'string' && 
    s.serviceEndpoint.includes(ipns)
  );

  if (ipnsService) {
    console.log('[DidDocumentParser] IPNS 验证成功:', ipns);
  } else {
    console.warn('[DidDocumentParser] IPNS 验证失败，未找到匹配的服务:', ipns);
  }
}

/**
 * 从 DID 文档中提取服务端点
 * @param didDocument - DID 文档
 * @param serviceType - 服务类型
 * @returns 服务端点
 */
export function extractServiceEndpoint(
  didDocument: DIDDocument, 
  serviceType: string
): ServiceEndpoint | null {
  if (!didDocument.service || !Array.isArray(didDocument.service)) {
    return null;
  }

  return didDocument.service.find((s: ServiceEndpoint) => s.type === serviceType) || null;
}

/**
 * 获取 DID 文档中的所有服务类型
 * @param didDocument - DID 文档
 * @returns 服务类型列表
 */
export function getServiceTypes(didDocument: DIDDocument): string[] {
  if (!didDocument.service || !Array.isArray(didDocument.service)) {
    return [];
  }

  return didDocument.service.map((s: ServiceEndpoint) => s.type).filter(Boolean);
}

/**
 * 检查 DID 文档是否有效
 * @param didDocument - DID 文档
 * @returns 验证结果
 */
export function validateDidDocument(didDocument: any): {
  isValid: boolean;
  errors: string[];
  warnings: string[];
} {
  const errors: string[] = [];
  const warnings: string[] = [];

  // 检查必需字段
  if (!didDocument.id && !didDocument.did) {
    errors.push('DID 文档必须包含 id 或 did 字段');
  }

  if (!didDocument.service || !Array.isArray(didDocument.service)) {
    warnings.push('DID 文档没有 service 数组或格式不正确');
  }

  // 检查服务格式
  if (didDocument.service && Array.isArray(didDocument.service)) {
    didDocument.service.forEach((service: any, index: number) => {
      if (!service.id) {
        warnings.push(`服务 ${index} 缺少 id 字段`);
      }
      if (!service.type) {
        warnings.push(`服务 ${index} 缺少 type 字段`);
      }
      if (!service.serviceEndpoint) {
        warnings.push(`服务 ${index} 缺少 serviceEndpoint 字段`);
      }
    });
  }

  // 检查验证方法
  if (!didDocument.verificationMethod || !Array.isArray(didDocument.verificationMethod)) {
    warnings.push('DID 文档没有 verificationMethod 数组');
  }

  return {
    isValid: errors.length === 0,
    errors,
    warnings,
  };
}

/**
 * 标准化 DID 文档格式
 * @param didDocument - 原始 DID 文档
 * @returns 标准化的 DID 文档
 */
export function normalizeDidDocument(didDocument: any): DIDDocument {
  const normalized: DIDDocument = {
    id: didDocument.id || didDocument.did,
    service: [],
    verificationMethod: [],
  };

  // 标准化服务
  if (didDocument.service && Array.isArray(didDocument.service)) {
    normalized.service = didDocument.service.map((service: any) => ({
      id: service.id || `service-${Date.now()}`,
      type: service.type || 'Unknown',
      serviceEndpoint: service.serviceEndpoint || '',
    }));
  }

  // 标准化验证方法
  if (didDocument.verificationMethod && Array.isArray(didDocument.verificationMethod)) {
    normalized.verificationMethod = didDocument.verificationMethod.map((method: any) => ({
      id: method.id || `key-${Date.now()}`,
      type: method.type || 'Ed25519VerificationKey2018',
      controller: method.controller || normalized.id,
      publicKeyBase58: method.publicKeyBase58 || '',
    }));
  }

  // 复制其他字段
  if (didDocument.name) normalized.name = didDocument.name;
  if (didDocument.description) normalized.description = didDocument.description;
  if (didDocument.created) normalized.created = didDocument.created;
  if (didDocument.updated) normalized.updated = didDocument.updated;

  return normalized;
}

/**
 * 从 DID 文档中提取头像信息
 * @param didDocument - DID 文档
 * @returns 头像 URL 或 null
 */
export function extractAvatarFromDidDocument(didDocument: DIDDocument): string | null {
  if (!didDocument.service) return null;

  // 查找包含头像信息的服务
  const avatarService = didDocument.service.find((s: ServiceEndpoint) => {
    const endpoint = s.serviceEndpoint;
    if (typeof endpoint === 'object' && endpoint !== null) {
      return endpoint.avatar_cid || endpoint.avatarCid || endpoint.avatar_url || endpoint.avatar;
    }
    return false;
  });

  if (avatarService) {
    const endpoint = avatarService.serviceEndpoint as Record<string, any>;
    
    if (endpoint.avatar_url) {
      return endpoint.avatar_url;
    }
    
    if (endpoint.avatar_cid || endpoint.avatarCid) {
      const cid = endpoint.avatar_cid || endpoint.avatarCid;
      return `https://ipfs.io/ipfs/${cid}`;
    }
    
    if (endpoint.avatar) {
      return endpoint.avatar;
    }
  }

  return null;
}
