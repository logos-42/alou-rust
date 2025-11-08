import apiClient from './api'

export const MCP_UI_TARGETS = {
  agentProfile: 'agent_profile',
  channelDetail: 'channel_detail',
  channelCreate: 'channel_create',
  conversationDetail: 'conversation_detail',
  walletOverview: 'wallet_overview',
  transactionDetail: 'transaction_detail',
}

const normalizeResourcePayload = (payload) => {
  if (!payload) {
    return null
  }

  if (payload.resource) {
    return payload.resource
  }

  if (Array.isArray(payload.resources) && payload.resources.length > 0) {
    return payload.resources[0]
  }

  return payload
}

export const requestMcpUiResource = async (target, params = {}) => {
  const response = await apiClient.post('/mcp/ui-resource', {
    target,
    params,
  })

  return normalizeResourcePayload(response.data)
}


