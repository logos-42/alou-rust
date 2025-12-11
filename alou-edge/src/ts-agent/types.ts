/**
 * TypeScript Agent - 类型定义
 */

// 自定义智能体信息
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

// 聊天消息
export interface ChatMessage {
  role: 'user' | 'assistant';
  content: string;
  timestamp?: number;
  tool_calls?: ToolCall[];
}

// 工具调用
export interface ToolCall {
  id: string;
  name: string;
  input: Record<string, unknown>;
  output?: string;
}

