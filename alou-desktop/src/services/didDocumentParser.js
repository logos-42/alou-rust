/**
 * DID 文档解析器
 */

/**
 * 解析 DID 文档，提取智能体元数据
 * @param {Object} didDocument - DID 文档
 * @param {Object} additionalInfo - 附加信息 { cid, ipns }
 * @returns {Object} 智能体元数据
 */
export function parseDidDocumentToAgent(didDocument, additionalInfo = {}) {
  console.log('[DidDocumentParser] 解析 DID 文档:', JSON.stringify(didDocument, null, 2))
  
  // 从 DID 文档中提取智能体信息
  const did = didDocument.id || didDocument.did || null
  
  // 从 service 数组中提取智能体配置
  const services = didDocument.service || []
  console.log('[DidDocumentParser] 找到 services:', services.length, services)
  
  // 如果提供了 IPNS，验证 DID 文档中的 IPNS 是否匹配
  if (additionalInfo.ipns) {
    validateIpnsMatch(didDocument, additionalInfo.ipns)
  }
  
  // 尝试多种方式查找 AgentEndpoint 服务
  let agentService = services.find(s => 
    s.type === 'AgentEndpoint' || s.type === 'agent' || s.id?.includes('#agent')
  )
  
  // 如果没有找到，尝试查找第一个包含 serviceEndpoint 的服务
  if (!agentService) {
    agentService = services.find(s => s.serviceEndpoint)
  }
  
  // 如果还是没有找到，使用第一个服务
  if (!agentService && services.length > 0) {
    agentService = services[0]
  }
  
  console.log('[DidDocumentParser] 找到 agentService:', agentService)
  
  // 提取 serviceEndpoint，支持多种结构
  let serviceEndpoint = {}
  if (agentService) {
    // 如果 serviceEndpoint 是对象
    if (typeof agentService.serviceEndpoint === 'object' && agentService.serviceEndpoint !== null) {
      serviceEndpoint = agentService.serviceEndpoint
    }
    // 如果整个 service 对象就是配置
    else if (agentService.name || agentService.avatar_cid) {
      serviceEndpoint = agentService
    }
  }
  
  console.log('[DidDocumentParser] 提取的 serviceEndpoint:', serviceEndpoint)
  
  // 提取 metadata，支持多种位置
  const metadata = didDocument['alou:metadata'] || 
                   didDocument.metadata || 
                   didDocument['@context']?.metadata ||
                   {}
  
  console.log('[DidDocumentParser] 提取的 metadata:', metadata)
  
  // 提取 PubSub 主题
  const pubsubTopics = agentService?.pubsubTopics || 
                      agentService?.pubsub_topics ||
                      serviceEndpoint.pubsub_topics ||
                      []
  
  // 提取加密的 PeerID
  const encryptedPeerIdService = services.find(s => 
    s.type === 'EncryptedPeerID' || 
    s.type === 'encryptedPeerId' ||
    s.id?.includes('#encryptedPeerId')
  )
  
  // 提取名字，支持多种字段名和位置
  const name = serviceEndpoint.name || 
               serviceEndpoint.display_name ||
               metadata.agent_name ||
               metadata.name ||
               didDocument.name ||
               '未命名智能体'
  
  // 提取头像，支持多种字段名
  const avatar_cid = serviceEndpoint.avatar_cid || 
                    serviceEndpoint.avatarCid ||
                    metadata.avatar_cid ||
                    metadata.avatarCid ||
                    null
  
  const avatar_url = serviceEndpoint.avatar_url ||
                    serviceEndpoint.avatarUrl ||
                    metadata.avatar_url ||
                    metadata.avatarUrl ||
                    null
  
  console.log('[DidDocumentParser] 解析结果 - name:', name, 'avatar_cid:', avatar_cid, 'avatar_url:', avatar_url)
  
  return {
    id: additionalInfo.cid || additionalInfo.ipns || did || `agent_${Date.now()}`,
    did,
    cid: additionalInfo.cid || null,
    ipns: additionalInfo.ipns || null,
    name,
    display_name: name,
    role_description: serviceEndpoint.description || 
                     serviceEndpoint.role_description ||
                     metadata.agent_description ||
                     metadata.description ||
                     metadata.role_description ||
                     '',
    avatar_cid,
    avatar_url,
    mcp_config_cid: serviceEndpoint.mcp_config_cid || 
                   serviceEndpoint.mcpConfigCid ||
                   metadata.mcp_config_cid ||
                   null,
    mcp_ports: serviceEndpoint.mcp_ports || 
              serviceEndpoint.mcpPorts ||
              metadata.mcp_ports ||
              [],
    agent_type: serviceEndpoint.agent_type || 
               serviceEndpoint.agentType ||
               metadata.agent_type ||
               'ai_agent_sdk',
    customPrompt: serviceEndpoint.custom_prompt || 
                 serviceEndpoint.customPrompt ||
                 metadata.custom_prompt ||
                 null,
    pubsub_topics: pubsubTopics,
    encrypted_peer_id: encryptedPeerIdService?.serviceEndpoint || null,
    created_at: didDocument.created ? new Date(didDocument.created).getTime() : Date.now(),
    updated_at: Date.now(),
    imported_from_network: true,
    diapIdentity: {
      did,
      cid: additionalInfo.cid,
      ipns: additionalInfo.ipns,
      public_key: didDocument.verificationMethod?.[0]?.publicKeyMultibase || null,
    },
  }
}

/**
 * 验证 DID 文档中的 IPNS 是否匹配请求的 IPNS
 * @param {Object} didDocument - DID 文档
 * @param {string} requestedIpns - 请求的 IPNS
 */
export function validateIpnsMatch(didDocument, requestedIpns) {
  const services = didDocument.service || []
  const requestedIpnsKey = requestedIpns.replace(/^\/ipns\//, '')
  
  const hasMatchingIpns = services.some(svc => {
    const endpoint = svc.serviceEndpoint
    if (typeof endpoint === 'string') {
      return endpoint.includes(requestedIpnsKey) || endpoint.includes(requestedIpns)
    }
    return false
  })
  
  if (!hasMatchingIpns && services.length > 0) {
    console.warn(`[DidDocumentParser] ⚠️ DID 文档中的 IPNS 与请求的 IPNS 不匹配！`)
    console.warn(`[DidDocumentParser] 请求的 IPNS: ${requestedIpns}`)
    console.warn(`[DidDocumentParser] DID 文档 services:`, services.map(s => ({ type: s.type, endpoint: s.serviceEndpoint })))
  }
}

