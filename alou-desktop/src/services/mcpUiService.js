import apiClient from './api'

export const MCP_UI_TARGETS = {
  agentProfile: 'agent_profile',
  channelDetail: 'channel_detail',
  channelCreate: 'channel_create',
  conversationDetail: 'conversation_detail',
  channelList: 'channel_list',
  walletOverview: 'wallet_overview',
  transactionDetail: 'transaction_detail',
}

const normalizeResourcePayload = (payload) => {
  if (!payload) {
    return { resource: null, metadata: null, raw: null }
  }

  let resource = null
  if (payload.resource) {
    resource = payload.resource
  } else if (Array.isArray(payload.resources) && payload.resources.length > 0) {
    resource = payload.resources[0]
  } else {
    resource = payload
  }

  const metadata = (resource && (resource._meta || resource.metadata)) || payload.metadata || null

  return {
    resource,
    metadata,
    raw: payload,
  }
}

export const requestMcpUiResource = async (target, params = {}) => {
  try {
    const response = await apiClient.post('/mcp/ui-resource', {
      target,
      params,
    })

    return normalizeResourcePayload(response.data)
  } catch (error) {
    // 静默处理MCP UI资源加载错误，避免阻塞应用其他功能
    console.warn(`[MCP UI] 资源加载失败 (${target}):`, error.message)
    
    // 返回空的资源对象，避免上层代码崩溃
    return {
      resource: null,
      metadata: null,
      raw: null
    }
  }
}
