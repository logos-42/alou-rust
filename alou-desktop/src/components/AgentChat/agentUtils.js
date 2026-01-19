import agentAssetsService from '@/services/agentAssetsService'
import { SDK_TOOL_CATEGORIES, SDK_AGENT_TOOLS } from '@/services/sdkAgentTools'
import { getToolCategoriesByMode as getToolCategoriesFromAgentTools, getToolsByCategories as getToolsByCategoriesFromAgentTools } from './utils/agentTools'

export const fallbackAvatar = 'https://avatars.githubusercontent.com/u/16309930?v=4'

/**
 * 解析智能体头像 URL
 * 支持多种来源：直接 URL、IPFS CID、嵌套对象中的头像
 */
export const resolveAgentAvatar = (agent) => {
  if (!agent) return fallbackAvatar
  
  // 1. 直接 URL（已经是完整的 http/https URL）或 data URL
  if (agent.avatar) {
    if (agent.avatar.startsWith('http') || agent.avatar.startsWith('data:')) {
      return agent.avatar
    }
  }
  if (agent.avatar_url && agent.avatar_url.startsWith('http')) return agent.avatar_url
  
  // 2. IPFS CID（支持多种格式）- 只从当前智能体获取
  const avatarCid = agent.avatarCid || agent.avatar_cid
  if (avatarCid) {
    // 如果已经是完整 URL
    if (avatarCid.startsWith('http')) return avatarCid
    // 如果是 IPFS CID 格式
    if (avatarCid.startsWith('Qm') || avatarCid.startsWith('bafy') || avatarCid.startsWith('bafk')) {
      return agentAssetsService.resolveIpfsUri(avatarCid)
    }
  }
  
  // 3. 从当前智能体的 diapIdentity 中获取（避免跨智能体获取）
  // 只有当 diapIdentity 属于当前智能体时才使用
  if (agent.diapIdentity?.avatar_cid && 
      (agent.diapIdentity.did === agent.did || 
       agent.diapIdentity.ipns === agent.ipns || 
       agent.diapIdentity.cid === agent.cid)) {
    const cid = agent.diapIdentity.avatar_cid
    if (cid.startsWith('http')) return cid
    return agentAssetsService.resolveIpfsUri(cid)
  }
  
  // 4. 从 serviceEndpoint 中获取（当前智能体的服务端点）
  if (agent.serviceEndpoint?.avatar_cid) {
    const cid = agent.serviceEndpoint.avatar_cid
    if (cid.startsWith('http')) return cid
    return agentAssetsService.resolveIpfsUri(cid)
  }
  
  // 5. 从 meta 对象中获取（频道数据结构）
  if (agent.meta) {
    const metaAvatar = resolveAgentAvatar(agent.meta)
    // 只有当 meta 头像不属于其他智能体时才使用
    if (metaAvatar !== fallbackAvatar && agent.meta.id === agent.id) {
      return metaAvatar
    }
  }
  
  // 6. 从 did_document 的 service 中提取（当前智能体的 DID 文档）
  if (agent.did_document?.service) {
    const services = agent.did_document.service
    for (const svc of services) {
      const endpoint = svc.serviceEndpoint
      if (endpoint?.avatar_cid) {
        const cid = endpoint.avatar_cid
        if (cid.startsWith('http')) return cid
        return agentAssetsService.resolveIpfsUri(cid)
      }
    }
  }
  
  return fallbackAvatar
}

export const buildChannelFromAgent = (agent) => {
  if (!agent) {
    return null
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
  const id = cleanedAgent.ipns || cleanedAgent.did || cleanedAgent.cid || `agent_${Date.now()}`
  const nameFromIpns = cleanedAgent.ipns ? cleanedAgent.ipns.replace(/^\/?ipns\//, '') : null
  const nameFromDid = cleanedAgent.did ? cleanedAgent.did.split(':').filter(Boolean).slice(-1)[0] : null
  const fallbackName = cleanedAgent.cid || id
  // 显示名称也优先使用 IPNS，但如果所有名称都为空，使用 DID 的最后部分作为 fallback
  // 如果名称是 "未命名智能体"，也尝试从 DID 提取名称
  const providedName = cleanedAgent.display_name || cleanedAgent.name
  const isUnnamed = providedName === '未命名智能体' || providedName === 'Unnamed Agent' || !providedName
  
  let displayName = null
  if (!isUnnamed) {
    displayName = providedName
  } else {
    // 如果名称为空或是默认值，尝试从 DID 提取可读名称
    if (cleanedAgent.did) {
      const didParts = cleanedAgent.did.split(':')
      if (didParts.length > 2) {
        const lastPart = didParts[didParts.length - 1]
        // 如果 DID 最后部分是哈希值，只显示前8个字符
        displayName = lastPart.length > 16 ? lastPart.substring(0, 8) : lastPart
      }
    }
    // 如果 DID 提取失败，尝试使用 IPNS 或 CID
    if (!displayName) {
      displayName = nameFromIpns || nameFromDid
    }
  }
  // 最后的 fallback
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
  
  // 判断是否是临时频道（正在创建中）
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
  const metaWithMode = {
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

export const extractAgentTarget = (agent) => {
  if (!agent) return null
  if (agent.ipns) return agent.ipns
  if (agent.did) return agent.did
  if (agent.cid) return agent.cid
  if (agent.meta) {
    return agent.meta.ipns || agent.meta.did || agent.meta.cid || null
  }
  return null
}

export const extractErrorMessage = (error) => {
  if (!error) return '未知错误'
  if (error.response?.data?.error) {
    return error.response.data.error
  }
  if (typeof error.message === 'string' && error.message.trim().length > 0) {
    return error.message
  }
  return '请求失败'
}

export const computeAgentProfile = (selectedAgent) => {
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
 * @param {string} mode - 当前模式: 'agent' 或 'alou'
 * @param {Object} agentInfo - 智能体信息
 * @returns {Array<string>} 工具类别数组
 */
export function getToolCategoriesByMode(mode, agentInfo) {
  // 使用 agentTools.js 中的函数获取工具类别
  return getToolCategoriesFromAgentTools(mode, agentInfo)
}

/**
 * 根据类别获取工具列表
 * @param {Array<string>} categories - 工具类别数组
 * @returns {Array<Object>} 工具列表
 */
export function getToolsByCategories(categories) {
  // 使用 agentTools.js 中的函数获取工具
  return getToolsByCategoriesFromAgentTools(categories)
}

