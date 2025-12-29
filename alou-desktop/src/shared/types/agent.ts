/**
 * 代理相关类型定义
 * 从原有src/types/agent.ts迁移而来
 */

export interface AgentProfile {
  name: string;
  role: string;
  avatar: string;
}

export interface Channel {
  id: string;
  name: string;
  status: 'online' | 'offline' | 'busy';
  statusLabel: string;
  icon: string;
  color: string;
  updatedAt: number;
}

export interface TransactionItem {
  id: string;
  direction: 'in' | 'out';
  amount: string;
  token: string;
  counterparty: string;
  status: 'pending' | 'confirmed' | 'failed';
  statusLabel: string;
  timestamp: number;
}

export interface WalletSnapshot {
  address: string;
  chainId: string;
  balance: string;
  balanceFiat: string;
  networkLabel: string;
}

export interface Message {
  id: string;
  type: 'user' | 'assistant';
  content: string;
  timestamp: number;
  source?: string;
}

export interface InteractionLog {
  id: string;
  action: string;
  label: string;
  timestamp: number;
  detail?: Record<string, unknown>;
}

export interface ContextEvent {
  action: string;
  detail?: Record<string, unknown>;
  timestamp: number;
}

// 新增类型：代理配置
export interface AgentConfig {
  id?: string;
  name: string;
  description?: string;
  model: string;
  temperature: number;
  maxTokens: number;
  systemPrompt?: string;
  avatar?: string;
  createdAt?: Date;
  updatedAt?: Date;
}

// 新增类型：代理状态
export type AgentStatus = 'idle' | 'active' | 'error' | 'offline' | 'initializing';

// 新增类型：代理会话
export interface AgentSession {
  id: string;
  agentId: string;
  userId: string;
  messages: Message[];
  createdAt: Date;
  updatedAt: Date;
  status: 'active' | 'closed' | 'archived';
}

// 所有类型已经通过export语句导出
// 不需要额外的export type语句
