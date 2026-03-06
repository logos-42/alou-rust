/**
 * 群聊@智能体消息解析器
 * 解析消息中的@提及并匹配到智能体
 */

export interface Agent {
  id: string
  did?: string
  name?: string
  display_name?: string
  agent_name?: string
}

export interface ParseResult {
  mentionedAgents: Agent[]
  rawMentions: string[]
  cleanContent: string
  hasMention: boolean
}

/**
 * 解析消息中的@提及
 * @param message 原始消息内容
 * @param agents 群聊中的智能体列表
 * @returns 解析结果
 */
export function parseMentions(message: string, agents: Agent[]): ParseResult {
  if (!message || !agents || agents.length === 0) {
    return {
      mentionedAgents: [],
      rawMentions: [],
      cleanContent: message || '',
      hasMention: false
    }
  }

  // 提取所有@提及 (支持 @名称 或 @ID 格式)
  const mentionRegex = /@([a-zA-Z0-9_\-\u4e00-\u9fa5]+)/g
  const rawMentions: string[] = []
  let match: RegExpExecArray | null

  while ((match = mentionRegex.exec(message)) !== null) {
    rawMentions.push(match[1])
  }

  // 构建智能体名称到智能体的映射
  const agentMap = new Map<string, Agent>()
  
  for (const agent of agents) {
    // 使用多种名称进行匹配
    const names = [
      agent.name?.toLowerCase(),
      agent.display_name?.toLowerCase(),
      agent.agent_name?.toLowerCase(),
      agent.id?.toLowerCase(),
      agent.did?.toLowerCase()
    ].filter(Boolean)

    for (const name of names) {
      if (name && !agentMap.has(name)) {
        agentMap.set(name, agent)
      }
    }
  }

  // 匹配被@的智能体
  const mentionedAgents: Agent[] = []
  const matchedNames = new Set<string>()

  for (const mention of rawMentions) {
    const lowerMention = mention.toLowerCase()
    
    // 精确匹配
    if (agentMap.has(lowerMention) && !matchedNames.has(lowerMention)) {
      const agent = agentMap.get(lowerMention)!
      if (!mentionedAgents.some(a => a.id === agent.id)) {
        mentionedAgents.push(agent)
        matchedNames.add(lowerMention)
      }
    } else {
      // 部分匹配 (例如 @claude 可以匹配 @claude-code)
      for (const [name, agent] of agentMap) {
        if (name.startsWith(lowerMention) && !matchedNames.has(name)) {
          if (!mentionedAgents.some(a => a.id === agent.id)) {
            mentionedAgents.push(agent)
            matchedNames.add(name)
          }
          break
        }
      }
    }
  }

  // 清理消息内容，移除@提及部分
  const cleanContent = message.replace(mentionRegex, '').trim()

  return {
    mentionedAgents,
    rawMentions,
    cleanContent,
    hasMention: rawMentions.length > 0
  }
}

/**
 * 检查智能体是否被@提及
 * @param agentId 智能体ID
 * @param message 消息内容
 * @param agents 智能体列表
 * @returns 是否被提及
 */
export function isAgentMentioned(
  agentId: string,
  message: string,
  agents: Agent[]
): boolean {
  const { mentionedAgents } = parseMentions(message, agents)
  return mentionedAgents.some(agent => agent.id === agentId)
}

/**
 * 格式化@提及列表为可读字符串
 * @param agents 被@的智能体列表
 * @returns 格式化的字符串
 */
export function formatMentions(agents: Agent[]): string {
  if (!agents || agents.length === 0) return ''
  
  return agents
    .map(agent => `@${agent.name || agent.display_name || agent.id}`)
    .join(', ')
}

/**
 * 获取智能体名称列表（用于UI自动补全）
 * @param agents 智能体列表
 * @returns 名称列表
 */
export function getAgentNameList(agents: Agent[]): string[] {
  if (!agents || agents.length === 0) return []

  const names = new Set<string>()
  
  for (const agent of agents) {
    if (agent.name) names.add(agent.name)
    if (agent.display_name) names.add(agent.display_name)
    if (agent.agent_name) names.add(agent.agent_name)
    if (agent.id) names.add(agent.id)
  }

  return Array.from(names)
}

export default {
  parseMentions,
  isAgentMentioned,
  formatMentions,
  getAgentNameList
}
