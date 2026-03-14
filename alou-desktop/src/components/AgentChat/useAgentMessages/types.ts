/**
 * useAgentMessages 模块类型定义
 * 
 * @module components/AgentChat/useAgentMessages/types
 */

/** Tauri 进度事件类型 */
export interface AgentProgressPayload {
  type: 'started' | 'thinking' | 'tool_calling' | 'tool_done' | 'tools_pending' | 'completed' | 'failed'
  task_id: string
  content?: string
  tool_name?: string
  success?: boolean
  preview?: string
  error?: string
  count?: number
  result?: string
}

/** 消息类型 */
export interface Message {
  id: string
  type: 'user' | 'assistant' | 'system' | 'error'
  content: string
  timestamp: number
  source?: string
  agentId?: string
  isLoading?: boolean
  metadata?: Record<string, unknown>
}

/** 群聊消息类型 */
export interface GroupChatMessage {
  groupId?: string
  fromName?: string
  content?: string
  text?: string
  timestamp?: number
  isMentioned?: boolean
  mentionedAgentIds?: string[]
}

/** AgentInfo 类型（兼容） */
export interface AgentInfo {
  id: string
  name: string
  display_name?: string
  avatar?: string
  roleDescription?: string
  maxTokens?: number
  [key: string]: unknown
}

/** useAgentMessages Hook 配置 */
export interface UseAgentMessagesConfig {
  sessionId: string
  setSessionId: (id: string) => void
  activeChain: string | null
  activeChannelId: string
  selectedAgent: AgentInfo | null
  createSession: () => Promise<void>
  setSessionReady: (ready: boolean) => void
  recordInteraction: (type: string, data: Record<string, unknown>) => void
  handleToolCalls: (toolCalls: unknown[], context: unknown) => Promise<void>
  conversationOverlayRef: unknown
  consoleDockRef: unknown
  contextEventsRef: unknown
  currentMode?: 'agent' | 'cluster'
  onRateLimitExceeded?: () => void
  onCreateAgent?: (agentInfo: AgentInfo) => Promise<boolean>
  onAutoCreateAgent?: (agentInfo: AgentInfo) => Promise<boolean>
}

/** 消息服务配置 */
export interface MessageServiceConfig {
  maxTokens?: number
  avgTokensPerMessage?: number
  responseBuffer?: number
}

/** IPFS 消息保存结果 */
export interface IpfsSaveResult {
  success: boolean
  cid?: string
  error?: string
}

/** 智能体执行结果 */
export interface AgentExecutionResult {
  success: boolean
  content?: string
  error?: string
  taskId?: string
}
