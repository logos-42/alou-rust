/**
 * TypeScript Agent - 类型定义
 */

// 自定义智能体信息（与 Rust 代码兼容）
export interface CustomAgentInfo {
  name: string;
  role_description: string;
  custom_instructions?: string;
  did?: string;
  ipns?: string;
  cid?: string;
  avatar_cid?: string;
  mcp_config_cid?: string;
  pubsub_topics?: string[];
}

// 聊天消息（与 Rust Message 结构兼容）
export interface ChatMessage {
  role: 'user' | 'assistant' | 'tool';
  content: string;
  timestamp?: number;
  tool_calls?: ToolCall[];
}

// 工具调用（与 Rust ToolCallInfo 结构兼容）
export interface ToolCall {
  id: string;
  name: string;
  input: Record<string, unknown>;
  output?: string;
}

// Session 结构（与 Rust Session 结构兼容）
export interface Session {
  session_id: string;
  wallet_address?: string;
  chain?: string;
  messages: Message[];
  recent_events?: any[];
  agent_metadata?: any;
  diap_identity?: any;
  created_at: number;
  updated_at: number;
}

// Message 结构（与 Rust Message 结构兼容）
export interface Message {
  role: string;
  content: string;
  timestamp: number;
  tool_call_id?: string | null;
}

