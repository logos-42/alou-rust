export interface AgentProfile {
  name: string
  role: string
  avatar: string
}

export interface Channel {
  id: string
  name: string
  status: 'online' | 'offline' | 'busy'
  statusLabel: string
  icon: string
  color: string
  updatedAt: number
}

export interface TransactionItem {
  id: string
  direction: 'in' | 'out'
  amount: string
  token: string
  counterparty: string
  status: 'pending' | 'confirmed' | 'failed'
  statusLabel: string
  timestamp: number
}

export interface WalletSnapshot {
  address: string
  chainId: string
  balance: string
  balanceFiat: string
  networkLabel: string
}

export interface Message {
  id: string
  type: 'user' | 'assistant'
  content: string
  timestamp: number
  source?: string
}

export interface InteractionLog {
  id: string
  action: string
  label: string
  timestamp: number
  detail?: Record<string, unknown>
}

export interface ContextEvent {
  action: string
  detail?: Record<string, unknown>
  timestamp: number
}

