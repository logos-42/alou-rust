/**
 * Alou Edge TypeScript - 类型定义
 */

// 环境变量类型
export interface Env {
  // Cloudflare bindings
  DB: D1Database;
  SESSIONS: KVNamespace;
  CACHE: KVNamespace;
  
  // API Keys
  ANTHROPIC_API_KEY: string;
  
  // 环境配置
  ENVIRONMENT: string;
}

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

// 会话数据
export interface SessionData {
  id: string;
  wallet_address?: string;
  chain?: string;
  created_at: number;
  updated_at: number;
  agent_info?: CustomAgentInfo;
  messages: ChatMessage[];
}

// 聊天消息
export interface ChatMessage {
  role: 'user' | 'assistant';
  content: string;
  timestamp: number;
  tool_calls?: ToolCall[];
}

// 工具调用
export interface ToolCall {
  id: string;
  name: string;
  input: Record<string, unknown>;
  output?: string;
}

// API 请求/响应类型
export interface ChatRequest {
  session_id: string;
  message: string;
  wallet_address?: string;
  chain?: string;
}

export interface ChatResponse {
  success: boolean;
  message?: string;
  response?: string;
  error?: string;
  tool_calls?: ToolCall[];
}

export interface CreateSessionRequest {
  wallet_address?: string;
}

export interface CreateSessionResponse {
  success: boolean;
  session_id: string;
  created_at: number;
}

export interface CreateAgentRequest {
  session_id: string;
  name: string;
  role_description?: string;
  avatar_cid?: string;
  mcp_config_cid?: string;
  diap_identity?: {
    did?: string;
    cid?: string;
    ipns?: string;
    public_key?: string;
  };
}

export interface ResolveAgentRequest {
  target: string;
  session_id?: string;
}

// DIAP 身份信息
export interface DiapIdentity {
  did: string;
  cid: string;
  ipns: string;
  public_key: string;
  gateway_url?: string;
  pubsub_topics?: string[];
  encrypted_node_id?: {
    ciphertext: string;
    nonce: string;
    signature: string;
    method: string;
  };
}

