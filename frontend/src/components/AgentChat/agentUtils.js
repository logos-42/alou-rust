import agentAssetsService from '@/services/agentAssetsService'

export const fallbackAvatar = 'https://avatars.githubusercontent.com/u/16309930?v=4'

export const resolveAgentAvatar = (agent) => {
  if (!agent) return fallbackAvatar
  if (agent.avatar) return agent.avatar
  if (agent.avatar_url) return agent.avatar_url
  if (agent.avatarCid || agent.avatar_cid) {
    return agentAssetsService.resolveIpfsUri(agent.avatarCid || agent.avatar_cid)
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
  // 显示名称也优先使用 IPNS
  const displayName = cleanedAgent.display_name || cleanedAgent.name || nameFromIpns || nameFromDid || fallbackName
  const statusLabel = cleanedAgent.ipns
    ? 'IPNS 解析'
    : cleanedAgent.did
      ? 'DID 解析'
      : cleanedAgent.agent_type === 'claude_agent_sdk'
        ? '自定义智能体'
        : 'CID 解析'

  const avatar = resolveAgentAvatar(cleanedAgent)

  return {
    id,
    name: displayName,
    status: 'online',
    statusLabel,
    icon: '🛰️',
    avatar,
    color: 'linear-gradient(135deg,#6366f1,#8b5cf6)',
    updatedAt: Math.floor(Date.now() / 1000),
    meta: cleanedAgent,
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
    (selectedAgent.ipns && selectedAgent.ipns.replace(/^\/?ipns\//, '').slice(0, 42)) ||
    (selectedAgent.did && selectedAgent.did.split(':').filter(Boolean).slice(-1)[0]) ||
    selectedAgent.cid ||
    '解析智能体'

  const role = selectedAgent.did ? 'DID 智能体' : '去中心化智能体'

  return {
    name: displayName,
    role,
    avatar,
  }
}

