import apiClient from './api'

/**
 * MCP UI 目标类型
 */
export enum MCP_UI_TARGETS {
  agentProfile = 'agent_profile',
  channelDetail = 'channel_detail',
  channelCreate = 'channel_create',
  conversationDetail = 'conversation_detail',
  channelList = 'channel_list',
  walletOverview = 'wallet_overview',
  transactionDetail = 'transaction_detail',
}

/**
 * MCP UI 资源负载
 */
export interface McpResourcePayload {
  resource?: any
  resources?: any[]
  metadata?: any
  _meta?: any
  [key: string]: any
}

/**
 * 规范化后的 MCP 资源
 */
export interface NormalizedMcpResource {
  resource: any
  metadata: any
  raw: McpResourcePayload | null
}

/**
 * 请求 MCP UI 资源的参数
 */
export interface RequestMcpUiResourceParams {
  [key: string]: any
}

const normalizeResourcePayload = (payload: McpResourcePayload | null): NormalizedMcpResource => {
  if (!payload) {
    return { resource: null, metadata: null, raw: null }
  }

  let resource: any = null
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

export const requestMcpUiResource = async (
  target: string,
  params: RequestMcpUiResourceParams = {}
): Promise<NormalizedMcpResource> => {
  try {
    const response = await apiClient.post<McpResourcePayload>('/mcp/ui-resource', {
      target,
      params,
    })

    return normalizeResourcePayload(response.data)
  } catch (error) {
    // 静默处理MCP UI资源加载错误，避免阻塞应用其他功能
    console.warn(`[MCP UI] 资源加载失败 (${target}):`, (error as Error).message)
    
    // 返回空的资源对象，避免上层代码崩溃
    return {
      resource: null,
      metadata: null,
      raw: null
    }
  }
}

// 默认导出
const mcpUiService = {
  MCP_UI_TARGETS,
  requestMcpUiResource,
}

export default mcpUiService
