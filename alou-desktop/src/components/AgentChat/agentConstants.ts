// Constants used in AgentChat component

export const MOCK_IPNS = 'k51qzi5uqu5dihfll965owckn1s0zsrip0twrzaa4939vs6e0mccc33namyv0s';

export const CONNECTION_STATUS_LABELS: Record<string, string> = {
  connected: '已连接',
  connecting: '连接中',
  error: '服务异常',
  disconnected: '未连接',
};

// Agent status types
export type AgentStatus = 'connected' | 'connecting' | 'error' | 'disconnected';

// Message role types
export type MessageRole = 'user' | 'assistant' | 'system' | 'tool';

// Chat mode types
export type ChatMode = 'agent' | 'alou' | 'group';

// Connection status types
export type ConnectionStatus = 'connected' | 'connecting' | 'error' | 'disconnected';

// Default constants
export const DEFAULT_AGENT_CONFIG = {
  name: 'Alou Assistant',
  description: 'AI智能助手',
  avatar: 'https://avatars.githubusercontent.com/u/16309930?v=4',
  model: 'gpt-4',
  temperature: 0.7,
  maxTokens: 2000,
} as const;

// API endpoints
export const API_ENDPOINTS = {
  CHAT: '/api/agent/chat',
  STREAM: '/api/agent/stream',
  WALLET: '/api/agent/wallet',
  STATUS: '/api/agent/status',
} as const;

// Error messages
export const ERROR_MESSAGES = {
  NETWORK_ERROR: '网络连接失败',
  TIMEOUT_ERROR: '请求超时',
  AUTH_ERROR: '认证失败',
  INVALID_RESPONSE: '响应格式错误',
  AGENT_NOT_FOUND: '智能体未找到',
  WALLET_NOT_CONNECTED: '钱包未连接',
} as const;

// Success messages
export const SUCCESS_MESSAGES = {
  AGENT_CREATED: '智能体创建成功',
  WALLET_CONNECTED: '钱包连接成功',
  MESSAGE_SENT: '消息发送成功',
  FILE_UPLOADED: '文件上传成功',
} as const;
