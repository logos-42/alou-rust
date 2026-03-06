/**
 * @mention 解析工具
 * 用于解析群聊消息中的 @智能体 提令
 */

export interface MentionTarget {
  /** 智能体ID */
  agentId: string
  /** 智能体名称 */
  agentName: string
  /** @提令的起始位置 */
  startIndex: number
  /** @提令的结束位置 */
  endIndex: number
  /** 提令文本（如 @agent1） */
  mentionText: string
}

export interface ParsedMessage {
  /** 原始消息内容 */
  rawContent: string
  /** 解析后的纯文本（去除 @ 提令标记） */
  cleanContent: string
  /** 检测到的 @ 提令列表 */
  mentions: MentionTarget[]
  /** 是否包含提令 */
  hasMentions: boolean
  /** 是否为仅针对特定智能体的消息（私密提令） */
  isPrivateMention: boolean
}

/**
 * 解析消息中的 @ 提令
 * 支持格式: @agentName, @agentId, @agentName@domain
 * 
 * @param message - 原始消息内容
 * @param availableAgents - 可用的智能体列表
 * @returns 解析结果
 */
export function parseMentions(
  message: string,
  availableAgents: Array<{ id: string; name: string; agent_id?: string; did?: string }> = []
): ParsedMessage {
  const mentions: MentionTarget[] = []
  
  // 检测 @ 提令的正则表达式
  // 匹配 @名称 或 @ID 格式
  const mentionRegex = /@(\S+)/g
  
  let match
  while ((match = mentionRegex.exec(message)) !== null) {
    const mentionText = match[0] // 如 @agent1
    const mentionedName = match[1] // 如 agent1
    const startIndex = match.index
    const endIndex = startIndex + mentionText.length
    
    // 尝试匹配智能体
    let matchedAgent = null
    
    // 首先尝试精确匹配名称（忽略大小写）
    matchedAgent = availableAgents.find(
      agent => agent.name?.toLowerCase() === mentionedName.toLowerCase()
    )
    
    // 然后尝试匹配 ID
    if (!matchedAgent) {
      matchedAgent = availableAgents.find(
        agent => 
          agent.id?.toLowerCase() === mentionedName.toLowerCase() ||
          agent.agent_id?.toLowerCase() === mentionedName.toLowerCase() ||
          agent.did?.toLowerCase() === mentionedName.toLowerCase()
      )
    }
    
    // 部分匹配（以输入开头）
    if (!matchedAgent) {
      matchedAgent = availableAgents.find(
        agent => 
          agent.name?.toLowerCase().startsWith(mentionedName.toLowerCase()) ||
          agent.id?.toLowerCase().startsWith(mentionedName.toLowerCase())
      )
    }
    
    if (matchedAgent) {
      mentions.push({
        agentId: matchedAgent.id || matchedAgent.agent_id || matchedAgent.did || '',
        agentName: matchedAgent.name || '',
        startIndex,
        endIndex,
        mentionText
      })
    }
  }
  
  // 生成纯文本内容（保留提令标记但移除位置信息，用于显示）
  const cleanContent = message
  
  // 判断是否为私密提令（只有一个提令且在消息开头）
  const isPrivateMention = 
    mentions.length === 1 && 
    mentions[0].startIndex === 0
  
  return {
    rawContent: message,
    cleanContent,
    mentions,
    hasMentions: mentions.length > 0,
    isPrivateMention
  }
}

/**
 * 提取消息中的所有提令名称（用于自动补全）
 * 
 * @param message - 消息内容
 * @returns 提令名称列表
 */
export function extractMentionNames(message: string): string[] {
  const names: string[] = []
  const mentionRegex = /@(\S+)/g
  let match
  
  while ((match = mentionRegex.exec(message)) !== null) {
    names.push(match[1])
  }
  
  return names
}

/**
 * 获取光标位置前的提令名称（用于自动补全）
 * 
 * @param message - 消息内容
 * @param cursorPosition - 光标位置
 * @returns 当前正在输入的提令名称（不包含 @）
 */
export function getPartialMentionAtCursor(
  message: string, 
  cursorPosition: number
): string | null {
  // 找到光标前最近的 @ 符号
  const textBeforeCursor = message.substring(0, cursorPosition)
  const lastAtIndex = textBeforeCursor.lastIndexOf('@')
  
  if (lastAtIndex === -1) {
    return null
  }
  
  // 检查 @ 后面是否有空格或其他分隔符
  const textAfterAt = textBeforeCursor.substring(lastAtIndex + 1)
  
  // 如果 @ 后面有空格，说明不是当前正在输入的提令
  if (textAfterAt.includes(' ')) {
    return null
  }
  
  // 返回 @ 后面的文本（不包含 @）
  return textAfterAt
}

/**
 * 检查消息是否提令了特定的智能体
 * 
 * @param message - 消息内容
 * @param agentId - 智能体ID
 * @returns 是否提令了该智能体
 */
export function isMentioned(
  message: string, 
  agentId: string,
  availableAgents: Array<{ id: string; name: string; agent_id?: string; did?: string }> = []
): boolean {
  const parsed = parseMentions(message, availableAgents)
  return parsed.mentions.some(mention => 
    mention.agentId.toLowerCase() === agentId.toLowerCase()
  )
}

/**
 * 格式化消息用于显示（高亮提令）
 * 
 * @param message - 原始消息
 * @returns HTML 格式的消息（用于渲染）
 */
export function formatMessageWithMentions(message: string): string {
  // 将 @提令 转换为带高亮的 HTML
  return message.replace(
    /@(\S+)/g, 
    '<span class="mention">@$1</span>'
  )
}

/**
 * 生成智能体选择列表
 * 
 * @param partialName - 用户输入的部分名称
 * @param availableAgents - 可用的智能体列表
 * @param maxResults - 最大返回数量
 * @returns 匹配的智能体列表
 */
export function getMentionSuggestions(
  partialName: string,
  availableAgents: Array<{ id: string; name: string; agent_id?: string; did?: string }> = [],
  maxResults: number = 5
): Array<{ id: string; name: string; avatar?: string }> {
  if (!partialName) {
    // 如果没有输入，返回所有智能体
    return availableAgents.slice(0, maxResults).map(agent => ({
      id: agent.id || agent.agent_id || agent.did || '',
      name: agent.name || ''
    }))
  }
  
  const lowerName = partialName.toLowerCase()
  
  // 优先匹配名称开头
  const matched = availableAgents
    .filter(agent => {
      const name = agent.name?.toLowerCase() || ''
      const id = agent.id?.toLowerCase() || ''
      return name.startsWith(lowerName) || id.startsWith(lowerName)
    })
    .slice(0, maxResults)
    .map(agent => ({
      id: agent.id || agent.agent_id || agent.did || '',
      name: agent.name || ''
    }))
  
  return matched
}
