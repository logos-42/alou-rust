/**
 * Agent Coordinator Service - 智能体协调和自主交流
 * 实现智能体间的消息路由和协作
 */
import pubsubService, { PubSubMessage, MessageType } from './pubsubService'
import agentService from './agentService'

/**
 * 智能体能力类型
 */
export const AgentCapability = {
  CHAT: 'chat',                 // 通用对话
  BLOCKCHAIN: 'blockchain',     // 区块链操作
  DEFI: 'defi',                 // DeFi 操作
  NFT: 'nft',                   // NFT 操作
  TRANSLATION: 'translation',   // 翻译
  CODE: 'code',                 // 代码生成
  RESEARCH: 'research',         // 研究分析
  WALLET: 'wallet',             // 钱包管理
}

/**
 * 意图分析结果
 */
class IntentAnalysis {
  constructor({ intent, confidence, capabilities, keywords, targetAgent }) {
    this.intent = intent
    this.confidence = confidence || 0.5
    this.capabilities = capabilities || []
    this.keywords = keywords || []
    this.targetAgent = targetAgent || null
  }
}

class AgentCoordinatorService {
  constructor() {
    this.registeredAgents = new Map() // agentId -> AgentInfo
    this.agentSubscriptions = new Map() // agentId -> unsubscribe function
    this.messageHandlers = new Map() // agentId -> handler function
    this.pendingRequests = new Map() // requestId -> { resolve, reject, timeout }
  }

  /**
   * 注册智能体到协调器
   */
  registerAgent(agent) {
    if (!agent?.id && !agent?.did && !agent?.ipns) {
      console.warn('[AgentCoordinator] 无法注册智能体：缺少标识')
      return false
    }

    const agentId = agent.id || agent.did || agent.ipns
    const agentInfo = {
      id: agentId,
      did: agent.did,
      ipns: agent.ipns,
      name: agent.display_name || agent.name,
      description: agent.role_description || '',
      capabilities: this._extractCapabilities(agent),
      pubsubTopics: agent.pubsub_topics || [],
      status: 'online',
      registeredAt: Date.now(),
    }

    this.registeredAgents.set(agentId, agentInfo)
    console.log('[AgentCoordinator] 注册智能体:', agentInfo.name, agentId)

    // 订阅智能体的 PubSub 主题
    if (agentInfo.pubsubTopics.length > 0) {
      this._subscribeToAgentTopics(agentId, agentInfo.pubsubTopics)
    }

    return true
  }

  /**
   * 注销智能体
   */
  unregisterAgent(agentId) {
    // 取消订阅
    const unsubscribe = this.agentSubscriptions.get(agentId)
    if (unsubscribe) {
      unsubscribe()
      this.agentSubscriptions.delete(agentId)
    }

    this.registeredAgents.delete(agentId)
    this.messageHandlers.delete(agentId)
    console.log('[AgentCoordinator] 注销智能体:', agentId)
  }

  /**
   * 设置智能体消息处理器
   */
  setMessageHandler(agentId, handler) {
    this.messageHandlers.set(agentId, handler)
  }

  /**
   * 分析用户意图
   */
  analyzeIntent(message) {
    const lowerMessage = message.toLowerCase()
    const analysis = new IntentAnalysis({ intent: 'general' })

    // 区块链相关
    if (this._matchKeywords(lowerMessage, ['转账', '发送', 'transfer', 'send', '余额', 'balance', '交易', 'transaction'])) {
      analysis.intent = 'blockchain'
      analysis.capabilities.push(AgentCapability.BLOCKCHAIN, AgentCapability.WALLET)
      analysis.confidence = 0.8
    }

    // DeFi 相关
    if (this._matchKeywords(lowerMessage, ['swap', '兑换', 'stake', '质押', 'liquidity', '流动性', 'yield', '收益', 'defi'])) {
      analysis.intent = 'defi'
      analysis.capabilities.push(AgentCapability.DEFI, AgentCapability.BLOCKCHAIN)
      analysis.confidence = 0.85
    }

    // NFT 相关
    if (this._matchKeywords(lowerMessage, ['nft', '铸造', 'mint', '收藏品', 'collectible', 'opensea'])) {
      analysis.intent = 'nft'
      analysis.capabilities.push(AgentCapability.NFT)
      analysis.confidence = 0.85
    }

    // 翻译相关
    if (this._matchKeywords(lowerMessage, ['翻译', 'translate', '英文', '中文', 'english', 'chinese'])) {
      analysis.intent = 'translation'
      analysis.capabilities.push(AgentCapability.TRANSLATION)
      analysis.confidence = 0.9
    }

    // 代码相关
    if (this._matchKeywords(lowerMessage, ['代码', 'code', '编程', 'program', '函数', 'function', 'bug', '调试'])) {
      analysis.intent = 'code'
      analysis.capabilities.push(AgentCapability.CODE)
      analysis.confidence = 0.85
    }

    // 研究分析
    if (this._matchKeywords(lowerMessage, ['分析', 'analyze', '研究', 'research', '报告', 'report', '数据'])) {
      analysis.intent = 'research'
      analysis.capabilities.push(AgentCapability.RESEARCH)
      analysis.confidence = 0.75
    }

    return analysis
  }

  /**
   * 根据意图匹配最合适的智能体
   */
  matchAgentByIntent(intent) {
    const agents = Array.from(this.registeredAgents.values())
    
    if (agents.length === 0) {
      return null
    }

    // 计算每个智能体的匹配分数
    const scoredAgents = agents.map(agent => {
      let score = 0
      
      // 能力匹配
      for (const cap of intent.capabilities) {
        if (agent.capabilities.includes(cap)) {
          score += 10
        }
      }

      // 关键词匹配（在描述中）
      const description = (agent.description || '').toLowerCase()
      for (const keyword of intent.keywords) {
        if (description.includes(keyword)) {
          score += 5
        }
      }

      // 在线状态加分
      if (agent.status === 'online') {
        score += 2
      }

      return { agent, score }
    })

    // 按分数排序
    scoredAgents.sort((a, b) => b.score - a.score)

    // 返回最高分的智能体（如果分数大于0）
    if (scoredAgents[0]?.score > 0) {
      return scoredAgents[0].agent
    }

    // 没有匹配的，返回第一个在线的智能体
    return agents.find(a => a.status === 'online') || agents[0]
  }

  /**
   * 路由消息到合适的智能体
   */
  async routeMessage(message, fromAgentId = null) {
    const intent = this.analyzeIntent(message)
    console.log('[AgentCoordinator] 意图分析:', intent)

    const targetAgent = this.matchAgentByIntent(intent)
    if (!targetAgent) {
      console.warn('[AgentCoordinator] 没有找到合适的智能体')
      return null
    }

    console.log('[AgentCoordinator] 路由到智能体:', targetAgent.name, targetAgent.id)
    return targetAgent
  }

  /**
   * 智能体间发送消息
   */
  async sendAgentMessage(fromAgentId, toAgentId, message, metadata = {}) {
    const fromAgent = this.registeredAgents.get(fromAgentId)
    const toAgent = this.registeredAgents.get(toAgentId)

    if (!toAgent) {
      console.warn('[AgentCoordinator] 目标智能体未注册:', toAgentId)
      return false
    }

    // 生成请求 ID
    const requestId = `req_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`

    // 构建消息
    const pubsubMessage = new PubSubMessage({
      type: MessageType.AGENT_REQUEST,
      from: fromAgent?.did || fromAgentId,
      to: toAgent.did || toAgentId,
      content: message,
      topic: toAgent.pubsubTopics[0] || pubsubService.generateAgentTopic(toAgentId),
      metadata: {
        ...metadata,
        requestId,
        fromAgentName: fromAgent?.name,
        toAgentName: toAgent.name,
      },
    })

    // 发布消息
    const topic = pubsubMessage.topic
    const success = await pubsubService.publish(topic, pubsubMessage)

    if (success) {
      console.log('[AgentCoordinator] 智能体消息已发送:', fromAgentId, '->', toAgentId)
    }

    return success
  }

  /**
   * 智能体间请求（带响应等待）
   */
  async requestFromAgent(fromAgentId, toAgentId, message, timeoutMs = 30000) {
    const requestId = `req_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`

    return new Promise((resolve, reject) => {
      // 设置超时
      const timeout = setTimeout(() => {
        this.pendingRequests.delete(requestId)
        reject(new Error('智能体响应超时'))
      }, timeoutMs)

      // 保存待处理请求
      this.pendingRequests.set(requestId, { resolve, reject, timeout })

      // 发送请求
      this.sendAgentMessage(fromAgentId, toAgentId, message, { requestId })
        .catch(err => {
          clearTimeout(timeout)
          this.pendingRequests.delete(requestId)
          reject(err)
        })
    })
  }

  /**
   * 处理智能体响应
   */
  handleAgentResponse(requestId, response) {
    const pending = this.pendingRequests.get(requestId)
    if (pending) {
      clearTimeout(pending.timeout)
      this.pendingRequests.delete(requestId)
      pending.resolve(response)
    }
  }

  /**
   * 广播消息给所有智能体
   */
  async broadcastToAgents(message, excludeAgentId = null) {
    const agents = Array.from(this.registeredAgents.values())
    const results = []

    for (const agent of agents) {
      if (agent.id === excludeAgentId) continue

      const success = await this.sendAgentMessage(excludeAgentId, agent.id, message)
      results.push({ agentId: agent.id, success })
    }

    return results
  }

  /**
   * 获取所有已注册的智能体
   */
  getRegisteredAgents() {
    return Array.from(this.registeredAgents.values())
  }

  /**
   * 获取智能体信息
   */
  getAgent(agentId) {
    return this.registeredAgents.get(agentId)
  }

  /**
   * 提取智能体能力
   */
  _extractCapabilities(agent) {
    const capabilities = [AgentCapability.CHAT] // 默认都有聊天能力
    const description = (agent.role_description || agent.description || '').toLowerCase()

    if (this._matchKeywords(description, ['blockchain', '区块链', 'web3', 'crypto'])) {
      capabilities.push(AgentCapability.BLOCKCHAIN)
    }
    if (this._matchKeywords(description, ['defi', 'swap', 'stake', '质押'])) {
      capabilities.push(AgentCapability.DEFI)
    }
    if (this._matchKeywords(description, ['nft', '收藏品'])) {
      capabilities.push(AgentCapability.NFT)
    }
    if (this._matchKeywords(description, ['翻译', 'translate', 'translation'])) {
      capabilities.push(AgentCapability.TRANSLATION)
    }
    if (this._matchKeywords(description, ['代码', 'code', '编程', 'developer'])) {
      capabilities.push(AgentCapability.CODE)
    }
    if (this._matchKeywords(description, ['研究', 'research', '分析', 'analyst'])) {
      capabilities.push(AgentCapability.RESEARCH)
    }
    if (this._matchKeywords(description, ['钱包', 'wallet'])) {
      capabilities.push(AgentCapability.WALLET)
    }

    return [...new Set(capabilities)]
  }

  /**
   * 关键词匹配
   */
  _matchKeywords(text, keywords) {
    return keywords.some(kw => text.includes(kw))
  }

  /**
   * 订阅智能体的 PubSub 主题
   */
  _subscribeToAgentTopics(agentId, topics) {
    const unsubscribes = []

    for (const topic of topics) {
      const unsubscribe = pubsubService.subscribe(topic, (message) => {
        this._handleIncomingMessage(agentId, message)
      })
      unsubscribes.push(unsubscribe)
    }

    // 保存取消订阅函数
    this.agentSubscriptions.set(agentId, () => {
      unsubscribes.forEach(fn => fn())
    })
  }

  /**
   * 处理收到的消息
   */
  _handleIncomingMessage(agentId, message) {
    console.log('[AgentCoordinator] 收到消息:', agentId, message.type)

    // 如果是响应消息，处理待处理请求
    if (message.type === MessageType.AGENT_RESPONSE && message.metadata?.requestId) {
      this.handleAgentResponse(message.metadata.requestId, message)
      return
    }

    // 调用智能体的消息处理器
    const handler = this.messageHandlers.get(agentId)
    if (handler) {
      handler(message)
    }
  }

  /**
   * 清理所有资源
   */
  cleanup() {
    // 取消所有订阅
    this.agentSubscriptions.forEach(unsubscribe => unsubscribe())
    this.agentSubscriptions.clear()

    // 清理待处理请求
    this.pendingRequests.forEach(({ timeout }) => clearTimeout(timeout))
    this.pendingRequests.clear()

    this.registeredAgents.clear()
    this.messageHandlers.clear()

    console.log('[AgentCoordinator] 已清理所有资源')
  }
}

export default new AgentCoordinatorService()
