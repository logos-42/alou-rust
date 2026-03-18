/**
 * Agent Runtime Service - 前端桥接服务
 * 
 * 连接前端与后端 Agent Runtime
 * 设计原则：
 * - 后端 = 真正的大脑（认知循环、工具执行）
 * - 前端 = 纯 UI 展示（消息显示、用户输入）
 */

import { invoke } from '@tauri-apps/api/core'
import { listen, UnlistenFn } from '@tauri-apps/api/event'

/// Agent 信息
export interface AgentInfo {
  id: string
  name: string
  display_name: string
  avatar_url?: string
  capabilities: string[]
  groups: string[]
  script_path: string
  enabled: boolean
}

/// Agent 配置
export interface AgentConfig {
  mention_only: boolean
  auto_reply: boolean
  reply_delay_ms: number
  max_context_messages: number
  custom_prompt?: string
}

/// 运行时状态
export interface RuntimeStatus {
  initialized: boolean
  active_agents: number
  message_bus_subscribers: number
}

/// 群聊消息
export interface GroupChatMessage {
  id: string
  group_id: string
  sender_id: string
  sender_name: string
  content: string
  timestamp: number
}

// 事件监听器管理
let unlistenBackendMessage: UnlistenFn | null = null
const messageHandlers: Set<(message: GroupChatMessage) => void> = new Set()

/**
 * 初始化 Agent Runtime
 */
export async function initAgentRuntime(): Promise<boolean> {
  return await invoke<boolean>('init_agent_runtime')
}

/**
 * 注册后端 Agent
 */
export async function registerBackendAgent(
  agent: AgentInfo,
  config: AgentConfig
): Promise<string> {
  return await invoke<string>('register_backend_agent', {
    agent,
    config,
  })
}

/**
 * Agent 加入群聊
 */
export async function agentJoinGroupChat(
  agent_id: string,
  group_id: string
): Promise<void> {
  return await invoke<void>('agent_join_group_chat', {
    agentId: agent_id,
    groupId: group_id,
  })
}

/**
 * 发送用户消息到群聊
 */
export async function sendUserMessage(
  group_id: string,
  sender_id: string,
  sender_name: string,
  content: string
): Promise<void> {
  return await invoke<void>('send_user_message', {
    groupId: group_id,
    senderId: sender_id,
    senderName: sender_name,
    content,
  })
}

/**
 * 获取活跃的 Agents
 */
export async function getActiveAgents(): Promise<string[]> {
  return await invoke<string[]>('get_active_agents')
}

/**
 * 停止 Agent
 */
export async function stopAgent(agent_id: string): Promise<void> {
  return await invoke<void>('stop_agent', {
    agentId: agent_id,
  })
}

/**
 * 获取群聊历史消息
 */
export async function getGroupHistory(
  group_id: string,
  limit: number = 100
): Promise<Array<{
  id: string
  group_id: string
  sender_id: string
  content: string
  timestamp: number
}>> {
  return await invoke('get_group_history', {
    groupId: group_id,
    limit,
  })
}

/**
 * 提交任务到队列
 */
export async function submitTask(
  agent_id: string,
  action: 'send_message' | 'check_activity',
  payload: Record<string, unknown>
): Promise<string> {
  return await invoke<string>('submit_task', {
    agentId: agent_id,
    action,
    payload,
  })
}

/**
 * 列出可用工具
 */
export async function listTools(): Promise<Array<{
  name: string
  description: string
}>> {
  return await invoke('list_tools')
}

/**
 * 执行工具
 */
export async function executeTool(
  tool_name: string,
  args: Record<string, unknown>
): Promise<unknown> {
  return await invoke('execute_tool', {
    toolName: tool_name,
    args,
  })
}

/**
 * 获取运行时状态
 */
export async function getRuntimeStatus(): Promise<RuntimeStatus> {
  return await invoke<RuntimeStatus>('get_runtime_status')
}

/**
 * 检查运行时是否已初始化
 */
export async function ensureRuntimeInitialized(): Promise<boolean> {
  try {
    const status = await getRuntimeStatus()
    if (!status.initialized) {
      await initAgentRuntime()
    }
    return true
  } catch (error) {
    console.error('[AgentRuntime] 初始化失败:', error)
    return false
  }
}

/**
 * 监听后端 Agent 消息（纯 UI 层）
 */
export function onBackendAgentMessage(
  handler: (message: GroupChatMessage) => void
): UnlistenFn {
  messageHandlers.add(handler)
  
  // 如果还没开始监听，启动监听
  if (!unlistenBackendMessage) {
    startBackendMessageListening()
  }
  
  // 返回取消监听函数
  return () => {
    messageHandlers.delete(handler)
    if (messageHandlers.size === 0 && unlistenBackendMessage) {
      unlistenBackendMessage()
      unlistenBackendMessage = null
    }
  }
}

/**
 * 启动后端消息监听
 */
async function startBackendMessageListening() {
  try {
    unlistenBackendMessage = await listen('backend-agent-message', (event) => {
      const message = event.payload as GroupChatMessage
      
      // 通知所有 UI 处理器
      messageHandlers.forEach(handler => {
        try {
          handler(message)
        } catch (error) {
          console.error('[AgentRuntime] 消息处理错误:', error)
        }
      })
    })
    
    console.log('[AgentRuntime] 后端消息监听已启动')
  } catch (error) {
    console.error('[AgentRuntime] 启动消息监听失败:', error)
  }
}

/**
 * 停止监听
 */
export function stopBackendMessageListening() {
  if (unlistenBackendMessage) {
    unlistenBackendMessage()
    unlistenBackendMessage = null
  }
  messageHandlers.clear()
}
