import { getToolCategoriesByMode as getToolCategoriesFromAgentTools, getToolsByCategories as getToolsByCategoriesFromAgentTools } from './utils/agentTools'

// AgentInfo 类型定义
interface AgentInfo {
  id?: string;
  name?: string;
  display_name?: string;
  did?: string;
  cid?: string;
  ipns?: string;
}

// ToolCategory 类型定义
interface ToolCategory {
  name: string;
  description?: string;
  tools?: unknown[];
}

export const fallbackAvatar = 'https://avatars.githubusercontent.com/u/16309930?v=4'

// IPFS 网关列表 - 按优先级排序
const IPFS_GATEWAYS = [
  'https://gateway.ipfs.io/ipfs',
  'https://ipfs.io/ipfs',
  'https://cloudflare-ipfs.com/ipfs',
  'https://dweb.link/ipfs',
]

// 本地 IPFS 网关（优先使用）
const getLocalGateway = (): string => {
  const localGateway = import.meta.env.VITE_IPFS_GATEWAY_URL || 'http://127.0.0.1:8080'
  return `${localGateway}/ipfs`
}

/**
 * 构建 IPFS URL
 * @param cid - IPFS CID
 * @param preferLocal - 是否优先使用本地网关
 * @returns 完整的 IPFS URL
 */
export const buildIpfsUrl = (cid: string, preferLocal: boolean = true): string => {
  if (!cid) return fallbackAvatar
  
  // 如果已经是完整 URL
  if (cid.startsWith('http')) return cid
  
  // 如果是 data URL
  if (cid.startsWith('data:')) return cid
  
  // 确保 CID 格式正确
  if (!cid.startsWith('Qm') && !cid.startsWith('bafy') && !cid.startsWith('bafk')) {
    console.warn('[buildIpfsUrl] 未知的 CID 格式:', cid)
    return fallbackAvatar
  }
  
  // 优先使用本地网关（如果在桌面环境）
  if (preferLocal && typeof window !== 'undefined' && (window as any).__TAURI__) {
    return `${getLocalGateway()}/${cid}`
  }
  
  // 使用公共网关
  return `${IPFS_GATEWAYS[0]}/${cid}`
}

/**
 * 获取备用 IPFS URL 列表（用于重试）
 * @param cid - IPFS CID
 * @returns URL 列表
 */
export const getIpfsFallbackUrls = (cid: string): string[] => {
  if (!cid || cid.startsWith('http') || cid.startsWith('data:')) return []
  
  const urls = []
  
  // 本地网关
  urls.push(`${getLocalGateway()}/${cid}`)
  
  // 公共网关
  for (const gateway of IPFS_GATEWAYS) {
    urls.push(`${gateway}/${cid}`)
  }
  
  return urls
}

// 智能体类型
export interface Agent {
  id?: string;
  name?: string;
  display_name?: string;
  avatar?: string;
  avatar_url?: string | null;
  avatar_cid?: string | null;
  avatarCid?: string;
  ipns?: string | null;
  did?: string | null;
  cid?: string | null;
  diapIdentity?: {
    avatar_cid?: string;
    did?: string;
    ipns?: string;
    cid?: string;
  } | null;
  serviceEndpoint?: {
    avatar_cid?: string;
  };
  meta?: Agent;
  did_document?: {
    service?: Array<{
      serviceEndpoint?: {
        avatar_cid?: string;
      };
    }>;
  };
  status?: string;
  agent_type?: string;
  mode?: string;
  role_description?: string;
  sessionId?: string | undefined;
}

// 频道类型
export interface Channel {
  id: string;
  name: string;
  status: string;
  statusLabel: string;
  icon: string;
  avatar: string;
  color: string;
  updatedAt: number;
  meta: Agent;
}

// 扩展Agent类型以包含hasLogged属性
declare global {
  interface Function {
    hasLogged?: boolean;
  }
}

/**
 * 解析智能体头像 URL
 * 支持多种来源：直接 URL、IPFS CID、嵌套对象中的头像
 */
export const resolveAgentAvatar = (agent: Agent | null | undefined): string => {
  if (!agent) return fallbackAvatar

  // 只在启动时输出一次调试日志
  if (!resolveAgentAvatar.hasLogged) {
    console.warn('[resolveAgentAvatar] 开始解析头像:', {
      id: agent.id,
      name: agent.name || agent.display_name,
      hasAvatar: !!agent.avatar,
      hasAvatarUrl: !!agent.avatar_url,
      hasAvatarCid: !!(agent.avatarCid || agent.avatar_cid),
      avatarUrl: agent.avatar_url?.substring(0, 100),
      avatarCid: agent.avatar_cid || agent.avatarCid
    })
    resolveAgentAvatar.hasLogged = true
  }
  
  // 1. 优先使用 avatar 字段（包含base64数据）
  if (agent.avatar) {
    // 如果是 data URL（base64），直接返回
    if (agent.avatar.startsWith('data:')) {
      return agent.avatar
    }
    // 如果是 http URL，直接返回
    if (agent.avatar.startsWith('http')) {
      return agent.avatar
    }
  }
  
  // 2. 使用 avatar_url 字段（支持 http URL 和 base64 data URL）
  if (agent.avatar_url && (agent.avatar_url.startsWith('http') || agent.avatar_url.startsWith('data:'))) {
    return agent.avatar_url
  }
  
  // 3. IPFS CID（支持多种格式）- 使用新的构建函数
  const avatarCid = agent.avatarCid || agent.avatar_cid
  if (avatarCid) {
    const url = buildIpfsUrl(avatarCid)
    if (url !== fallbackAvatar) return url
  }
  
  // 4. 从当前智能体的 diapIdentity 中获取（避免跨智能体获取）
  if (agent.diapIdentity?.avatar_cid && 
      (agent.diapIdentity.did === agent.did || 
       agent.diapIdentity.ipns === agent.ipns || 
       agent.diapIdentity.cid === agent.cid)) {
    const url = buildIpfsUrl(agent.diapIdentity.avatar_cid)
    if (url !== fallbackAvatar) return url
  }
  
  // 5. 从 serviceEndpoint 中获取（当前智能体的服务端点）
  if (agent.serviceEndpoint?.avatar_cid) {
    const url = buildIpfsUrl(agent.serviceEndpoint.avatar_cid)
    if (url !== fallbackAvatar) return url
  }
  
  // 6. 从 meta 对象中获取（频道数据结构）
  if (agent.meta) {
    const metaAvatar = resolveAgentAvatar(agent.meta)
    // 只有当 meta 头像不属于其他智能体时才使用
    if (metaAvatar !== fallbackAvatar && agent.meta.id === agent.id) {
      return metaAvatar
    }
  }
  
  // 7. 从 did_document 的 service 中提取（当前智能体的 DID 文档）
  if (agent.did_document?.service) {
    const services = agent.did_document.service
    for (const svc of services) {
      const endpoint = svc.serviceEndpoint
      if (endpoint?.avatar_cid) {
        const url = buildIpfsUrl(endpoint.avatar_cid)
        if (url !== fallbackAvatar) return url
      }
    }
  }
  
  return fallbackAvatar
}

export const buildChannelFromAgent = (agent: Agent | null): Channel | null => {
  if (!agent) {
    console.error('[buildChannelFromAgent] agent 为 null！')
    return null
  }

  // 只在启动时输出一次调试日志
  if (!buildChannelFromAgent.hasLogged) {
    console.warn('[buildChannelFromAgent] 开始构建频道:', {
      id: agent.id,
      name: agent.name || agent.display_name,
      hasAvatar: !!agent.avatar,
      hasAvatarUrl: !!agent.avatar_url,
      hasAvatarCid: !!(agent.avatar_cid || agent.avatarCid),
      hasMeta: !!agent.meta,
      keys: Object.keys(agent)
    })
    buildChannelFromAgent.hasLogged = true
  }
  
  // 过滤掉 mock IPNS 值
  const mockIpns = 'k51qzi5uqu5dihfll965owckn1s0zsrip0twrzaa4939vs6e0mccc33namyv0s'
  const ipnsValue = agent.ipns
  const isMockIpns = ipnsValue && (
    ipnsValue.includes(mockIpns) ||
    ipnsValue === mockIpns ||
    ipnsValue === `/ipns/${mockIpns}`
  )
  
  // 创建清理后的 agent 对象
  const cleanedAgent = { ...agent }
  if (isMockIpns) {
    delete cleanedAgent.ipns
  }
  
  // IPNS 应该优先于 DID 作为标识符
  const id = cleanedAgent.ipns || cleanedAgent.did || cleanedAgent.cid || `agent_${Date.now()}_${Math.random().toString(36).slice(2, 9)}`
  const nameFromIpns = cleanedAgent.ipns ? cleanedAgent.ipns.replace(/^\/?ipns\//, '') : null
  const nameFromDid = cleanedAgent.did ? cleanedAgent.did.split(':').filter(Boolean).slice(-1)[0] : null
  const fallbackName = cleanedAgent.cid || id
  
  const providedName = cleanedAgent.display_name || cleanedAgent.name
  const isUnnamed = providedName === '未命名智能体' || providedName === 'Unnamed Agent' || !providedName
  
  let displayName: string | null = null
  if (!isUnnamed) {
    displayName = providedName || null
  } else {
    if (cleanedAgent.did) {
      const didParts = cleanedAgent.did.split(':')
      if (didParts.length > 2) {
        const lastPart = didParts[didParts.length - 1]
        displayName = lastPart.length > 16 ? lastPart.substring(0, 8) : lastPart
      }
    }
    if (!displayName) {
      displayName = nameFromIpns || nameFromDid || null
    }
  }
  if (!displayName) {
    displayName = fallbackName
  }
  
  if (!id) {
    console.error('[buildChannelFromAgent] 生成的 ID 为空！', {
      ipns: cleanedAgent.ipns,
      did: cleanedAgent.did,
      cid: cleanedAgent.cid,
    })
  }
  
  const isCreating = cleanedAgent.status === 'creating' || 
    (cleanedAgent.cid && cleanedAgent.cid.startsWith('temp_')) ||
    (id && id.startsWith('temp_'))
  
  const statusLabel = isCreating
    ? '创建中...'
    : cleanedAgent.ipns
      ? 'IPNS 解析'
      : cleanedAgent.did
        ? 'DID 解析'
        : cleanedAgent.agent_type === 'claude_agent_sdk'
          ? '自定义智能体'
          : 'CID 解析'

  const avatar = resolveAgentAvatar(cleanedAgent)

  // 确保 meta 对象有 mode 字段（默认为 'agent'）
  const metaWithMode: Agent = {
    ...cleanedAgent,
    mode: cleanedAgent.mode || 'agent',
  }

  return {
    id,
    name: displayName,
    status: isCreating ? 'creating' : 'online',
    statusLabel,
    icon: '🛰️',
    avatar,
    color: 'linear-gradient(135deg,#6366f1,#8b5cf6)',
    updatedAt: Math.floor(Date.now() / 1000),
    meta: metaWithMode,
  }
}

export const extractAgentTarget = (agent: Agent | null): string | null => {
  if (!agent) return null
  if (agent.ipns) return agent.ipns
  if (agent.did) return agent.did
  if (agent.cid) return agent.cid
  if (agent.meta) {
    return agent.meta.ipns || agent.meta.did || agent.meta.cid || null
  }
  return null
}

export const extractErrorMessage = (error: { response?: { data?: { error?: string } }; message?: string } | null): string => {
  if (!error) return '未知错误'
  if (error.response?.data?.error) {
    return error.response.data.error
  }
  if (typeof error.message === 'string' && error.message.trim().length > 0) {
    return error.message
  }
  return '请求失败'
}

// 智能体档案类型
interface AgentProfile {
  name: string;
  role: string;
  avatar: string;
}

export const computeAgentProfile = (selectedAgent: Agent | null): AgentProfile => {
  const avatar = resolveAgentAvatar(selectedAgent)
  if (!selectedAgent) {
    return {
      name: 'alou',
      role: 'Web3 Multi-Agent Coordinator',
      avatar,
    }
  }

  const displayName =
    selectedAgent.display_name ||
    selectedAgent.name ||
    (selectedAgent.ipns && selectedAgent.ipns.replace(/^\/?ipns\//, '').slice(0, 42)) ||
    (selectedAgent.did && selectedAgent.did.split(':').filter(Boolean).slice(-1)[0]) ||
    selectedAgent.cid ||
    '智能体'

  const role = selectedAgent.did ? 'DID 智能体' : '去中心化智能体'

  return {
    name: displayName,
    role,
    avatar,
  }
}

/**
 * 根据模式获取工具类别
 * @param mode - 当前模式: 'agent' 或 'alou'
 * @param agentInfo - 智能体信息
 * @returns 工具类别数组
 */
export function getToolCategoriesByMode(mode: string, agentInfo: AgentInfo): string[] {
  return getToolCategoriesFromAgentTools(mode, agentInfo)
}

/**
 * 根据类别获取工具列表
 * @param categories - 工具类别数组
 * @returns 工具列表
 */
export function getToolsByCategories(categories: string[]): ToolCategory[] {
  return getToolsByCategoriesFromAgentTools(categories)
}

export default {
  fallbackAvatar,
  resolveAgentAvatar,
  buildChannelFromAgent,
  extractAgentTarget,
  extractErrorMessage,
  computeAgentProfile,
  getToolCategoriesByMode,
  getToolsByCategories,
}
