import { useCallback } from 'react'
import { walletService } from '@/services/walletService'

export const API_BASE_URL =
  import.meta.env.VITE_API_BASE_URL ||
  (import.meta.env.DEV ? '' : 'https://alou-edge.yuanjieliu65.workers.dev')

export const NODE_BOUNDARY = 140

// 频道接口
export interface Channel {
  id: string
  name: string
  status: 'online' | 'offline' | 'busy'
  statusLabel: string
  icon: string
  color: string
  updatedAt: number
  description?: string
}

// 交易接口
export interface Transaction {
  id: string
  hash: string
  direction: 'in' | 'out'
  amount: string
  token: string
  counterparty: string
  status: 'pending' | 'confirmed' | 'failed' | 'submitted' | 'success' | 'online'
  statusLabel: string
  timestamp: number
  raw?: Record<string, unknown>
}

// 工具调用结果接口
export interface ToolCallResult {
  [key: string]: unknown
}

// 工具调用接口
export interface ToolCall {
  id: string
  name: string
  result?: ToolCallResult
}

// 工具调用处理器返回结果
export interface ToolCallHandlerResult {
  tool_call_id: string
  role: 'tool'
  name: string
  content: string
  success: boolean
}

// UI资源接口
export interface UIResource {
  [key: string]: unknown
}

// 工具调用处理器配置
export interface ToolCallHandlerConfig {
  handleTransactionBuild: (result: ToolCallResult) => Promise<void>
  handleTransactionBroadcast: (result: ToolCallResult) => Promise<void>
  refreshWallet: () => Promise<void>
  recordInteraction: (type: string, data?: Record<string, unknown>) => void
  appendMessage: (message: { id: string; type: string; content: string; timestamp: number; source: string }) => void
  scrollToBottom: () => void
  openUiResource: (resource: UIResource, options?: { source?: string }) => void
}

export const defaultChannels: Channel[] = [
  {
    id: 'dev-relay',
    name: 'TRX Smart Contract Staking',
    status: 'online',
    statusLabel: '在线',
    icon: '⚡',
    color: 'linear-gradient(135deg,#6366f1,#8b5cf6)',
    updatedAt: Date.now() - 2 * 60 * 60 * 1000,
  },
  {
    id: 'eth-announce',
    name: 'ETH Contract Announcement',
    status: 'busy',
    statusLabel: '执行任务',
    icon: '⬡',
    color: 'linear-gradient(135deg,#0ea5e9,#2563eb)',
    updatedAt: Date.now() - 6 * 60 * 60 * 1000,
  },
  {
    id: 'firefly',
    name: 'Firefly Research',
    status: 'offline',
    statusLabel: '离线',
    icon: '🛰️',
    color: 'linear-gradient(135deg,#ec4899,#f97316)',
    updatedAt: Date.now() - 24 * 60 * 60 * 1000,
  },
  {
    id: 'wallet-ops',
    name: 'Wallet Operations',
    status: 'online',
    statusLabel: '在线',
    icon: '💼',
    color: 'linear-gradient(135deg,#14b8a6,#0ea5e9)',
    updatedAt: Date.now() - 30 * 60 * 1000,
  },
]

export const defaultTransactions: Transaction[] = [
  {
    id: 'tx-1',
    hash: 'tx-1',
    direction: 'out',
    amount: '0.42',
    token: 'ETH',
    counterparty: '0x1F345...ab91',
    status: 'confirmed',
    statusLabel: '已完成',
    timestamp: Date.now() - 4 * 60 * 60 * 1000,
  },
  {
    id: 'tx-2',
    hash: 'tx-2',
    direction: 'in',
    amount: '250',
    token: 'USDC',
    counterparty: '0x72ab...cc87',
    status: 'pending',
    statusLabel: '确认中',
    timestamp: Date.now() - 40 * 60 * 1000,
  },
  {
    id: 'tx-3',
    hash: 'tx-3',
    direction: 'out',
    amount: '1.2',
    token: 'ETH',
    counterparty: '0xbf12...9980',
    status: 'failed',
    statusLabel: '失败',
    timestamp: Date.now() - 3 * 24 * 60 * 60 * 1000,
  },
]

export const ensureMillis = (value: number | string | undefined): number => {
  if (!value || Number.isNaN(Number(value))) {
    return Date.now()
  }
  const numValue = typeof value === 'string' ? Number(value) : value
  if (numValue > 1_000_000_000_000) {
    return numValue
  }
  return numValue * 1000
}

const chainLabelMap: Record<string, string> = {
  ethereum: 'Ethereum Mainnet',
  eth: 'Ethereum Mainnet',
  eth_sepolia: 'Ethereum Sepolia',
  base: 'Base Mainnet',
  base_sepolia: 'Base Sepolia',
  polygon: 'Polygon Mainnet',
  polygon_amoy: 'Polygon Amoy',
  sol: 'Solana',
}

const chainIdToBackendMap: Record<string, string> = {
  '0x1': 'ethereum',
  '0x01': 'ethereum',
  '0xaa36a7': 'eth_sepolia',
  '0x14a34': 'base_sepolia',
  '0x2105': 'base',
}

export const mapChainLabel = (chain: string): string => 
  chainLabelMap[chain] || chain?.toUpperCase?.() || '未知网络'

export const mapChainIdToBackendChain = (chainId: string | null | undefined): string | null => {
  if (!chainId) return null
  const normalized = chainId.toLowerCase()
  return chainIdToBackendMap[normalized] || null
}

export const normalizeBackendChain = (chain: string | null | undefined): string | null => {
  if (!chain) return null
  const normalized = chain.toLowerCase()
  if (normalized === 'eth' || normalized === 'ethereum') {
    return 'ethereum'
  }
  if (normalized.includes('sepolia')) {
    return normalized.includes('base') ? 'base_sepolia' : 'eth_sepolia'
  }
  if (normalized.includes('base')) {
    return 'base'
  }
  if (normalized.includes('polygon')) {
    return normalized.includes('amoy') ? 'polygon_amoy' : 'polygon'
  }
  if (normalized === 'sol' || normalized === 'solana') {
    return 'sol'
  }
  if (normalized.includes('test')) {
    return normalized
  }
  return normalized
}

export const resolveBackendChain = ({ chainId, chain }: { chainId?: string; chain?: string } = {}): string | null => {
  const direct = normalizeBackendChain(chain)
  if (direct) {
    return direct
  }

  const fromChainId = mapChainIdToBackendChain(chainId)
  if (fromChainId) {
    return fromChainId
  }

  if (typeof window !== 'undefined') {
    const storedChainId = window.localStorage?.getItem?.('wallet_chain_id')
    const stored = mapChainIdToBackendChain(storedChainId)
    if (stored) {
      return stored
    }
  }

  return null
}

export const estimateFiatValue = (balance: string | number, token: string = 'ETH'): string => {
  const numeric = Number(balance)
  if (Number.isFinite(numeric)) {
    const usd = token === 'SOL' ? numeric * 150 : numeric * 3400
    return usd.toFixed(2)
  }
  return '--'
}

interface RawTransaction {
  hash?: string;
  id?: string;
  direction?: 'in' | 'out';
  type?: string;
  amount?: number | string;
  value?: number | string;
  quantity?: number | string;
  displayValue?: number | string;
  token?: string;
  counterparty?: string;
  to?: string;
  from?: string;
  status?: 'pending' | 'confirmed' | 'failed' | 'submitted' | 'success' | 'online';
  timestamp?: number;
  updatedAt?: number;
  [key: string]: unknown;
}

export const normalizeTransaction = (tx: RawTransaction, fallbackToken: string = 'ETH'): Transaction | null => {
  if (!tx) return null

  const hash = tx.hash || tx.id || `tx_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`

  const rawDirection = tx.direction || tx.type
  const direction = rawDirection === 'in' || rawDirection === 'receive' ? 'in' : 'out'

  const amountValue = tx.amount ?? tx.value ?? tx.quantity ?? tx.displayValue ?? '0'

  const token = tx.token || fallbackToken
  const counterparty = tx.counterparty || tx.to || tx.from || ''

  const status = tx.status || 'pending'
  const statusLabelMap: Record<string, string> = {
    pending: '确认中',
    submitted: '已提交',
    confirmed: '已完成',
    success: '已完成',
    failed: '失败',
    online: '在线',
  }

  return {
    id: hash,
    hash,
    direction,
    amount: typeof amountValue === 'number' ? amountValue.toString() : amountValue || '0',
    token,
    counterparty: counterparty || '未知地址',
    status,
    statusLabel: statusLabelMap[status] || status,
    timestamp: ensureMillis(tx.timestamp || tx.updatedAt || Date.now()),
    raw: tx,
  }
}

export const normalizeChannel = (channel: Partial<Channel>, index: number = 0): Channel | null => {
  if (!channel) return null
  const normalizedId = channel.id || `channel_${index}`
  return {
    id: normalizedId,
    name: channel.name || normalizedId,
    status: channel.status || 'online',
    statusLabel: channel.statusLabel || channel.status || '在线',
    icon: channel.icon || '💬',
    color: channel.color || 'linear-gradient(135deg,#6366f1,#8b5cf6)',
    updatedAt: ensureMillis(channel.updatedAt || Date.now()),
    description: channel.description,
  }
}

export const formatWeiHexToEth = (hexValue: string | number | undefined): string => {
  if (!hexValue) {
    return '0'
  }

  try {
    const normalized =
      typeof hexValue === 'string' && !hexValue.startsWith('0x') ? `0x${hexValue}` : hexValue
    const value = BigInt(normalized)
    const base = 10n ** 18n
    const whole = value / base
    const fraction = value % base

    if (fraction === 0n) {
      return whole.toString()
    }

    const fractionStr = fraction.toString().padStart(18, '0').replace(/0+$/, '')
    return `${whole.toString()}.${fractionStr.slice(0, 6)}`
  } catch (error) {
    console.error('formatWeiHexToEth error:', error)
    return typeof hexValue === 'string' ? hexValue : String(hexValue)
  }
}

export const ACTION_LABELS: Record<string, string> = {
  channel_selected: '切换频道',
  create_channel: '创建频道',
  toggle_theme: '主题切换',
  toggle_language: '语言切换',
  toggle_sidebar: '侧边栏',
  wallet_refresh: '刷新资产',
  wallet_event: '钱包变更',
  navigate_wallet: '打开钱包',
  navigate_login: '跳转登录',
  logout: '退出登录',
  agent_drag_start: '移动智能体',
  agent_drag_end: '智能体位置',
  trigger_mcp: '调用 MCP',
  user_message: '用户消息',
  wallet_instruction: '钱包指令',
  stream_event: '流式事件',
}


/**
 * 标准化工具调用参数格式
 * 确保参数符合 Rust 后端期望的格式
 */
function normalizeToolCallArguments(
  args: Record<string, unknown>,
  toolId: string
): Record<string, unknown> {
  if (!args || typeof args !== 'object') {
    return args;
  }

  const n = { ...args } as Record<string, unknown>;

  // ========== Bash Tool ==========
  if (toolId === 'bash') {
    n.operation = 'execute';
    
    // Shell 字段必须使用小写（serde 会自动转换为 Shell::Bash）
    if (!n.shell) {
      n.shell = 'bash';
    } else if (typeof n.shell === 'string') {
      const shellMap: Record<string, string> = {
        'bash': 'bash',
        'cmd': 'cmd',
        'powershell': 'powershell',
        'python': 'python',
        'node': 'node'
      };
      n.shell = shellMap[n.shell.toLowerCase()] || 'bash';
    }
    
    // 必须有 command 字段
    if (!n.command) {
      if (n.cmd) {
        n.command = n.cmd;
      } else if (n.script) {
        n.command = n.script;
      } else {
        throw new Error('Bash tool requires "command" argument');
      }
    }
    
    if (!n.timeout_seconds) {
      n.timeout_seconds = 30;
    }
    
    if (!n.environment || !Array.isArray(n.environment)) {
      n.environment = [];
    }
    
    if (n.working_dir === undefined) {
      n.working_dir = null;
    }
  }

  // ========== FileSystem Tool ==========
  else if (toolId === 'filesystem') {
    if (!n.operation) {
      n.operation = n.content ? 'write' : 'list';
    }
    if (!n.path && n.operation !== 'write') {
      n.path = '.';
    }
    if (n.operation === 'write' && n.create_dirs === undefined) {
      n.create_dirs = true;
    }
    if (n.operation === 'list' && n.recursive === undefined) {
      n.recursive = false;
    }
  }

  // ========== System Tool ==========
  else if (toolId === 'system') {
    if (!n.operation) {
      n.operation = 'info';
    }
  }

  // ========== Search Tool ==========
  else if (toolId === 'search') {
    if (!n.operation) {
      n.operation = n.query ? 'grep' : 'list';
    }
    if (!n.pattern && n.operation === 'grep') {
      n.pattern = n.query || '';
    }
    if (!n.directory) {
      n.directory = '.';
    }
    if (n.case_sensitive === undefined) {
      n.case_sensitive = false;
    }
  }

  // ========== UIControl Tool ==========
  else if (toolId === 'ui_control') {
    if (!n.action) {
      throw new Error('UI control tool requires "action" argument');
    }
  }

  // ========== TodoList Tool ==========
  else if (toolId === 'todolist') {
    if (!n.action) {
      n.action = 'list';
    }
  }

  // ========== Network Tool ==========
  else if (toolId === 'network') {
    if (!n.operation) {
      n.operation = 'get';
    }
    if (!n.url) {
      throw new Error('Network tool requires "url" argument');
    }
  }

  // ========== Browser Tool ==========
  else if (toolId === 'browser') {
    if (!n.action) {
      n.action = 'navigate';
    }
    if (!n.url) {
      throw new Error('Browser tool requires "url" argument');
    }
  }

  // ========== Iroh Tool ==========
  else if (toolId === 'iroh') {
    if (!n.action) {
      n.action = 'list';
    }
  }

  // ========== PubSub Tool ==========
  else if (toolId === 'pubsub') {
    if (!n.action) {
      n.action = 'subscribe';
    }
    if (!n.topic) {
      throw new Error('PubSub tool requires "topic" argument');
    }
  }

  // ========== MessagePassing Tool ==========
  else if (toolId === 'message_passing') {
    if (!n.action) {
      n.action = 'send';
    }
    if (!n.recipient) {
      throw new Error('MessagePassing tool requires "recipient" argument');
    }
    if (!n.content) {
      throw new Error('MessagePassing tool requires "content" argument');
    }
  }

  // ========== TaskQueue Tool ==========
  else if (toolId === 'task_queue') {
    if (!n.action) {
      n.action = 'list';
    }
  }

  // ========== Rollback Tool ==========
  else if (toolId === 'rollback') {
    if (!n.action) {
      n.action = 'create_snapshot';
    }
    if (!n.target_path) {
      throw new Error('Rollback tool requires "target_path" argument');
    }
  }

  // ========== GitHelper Tool ==========
  else if (toolId === 'git_helper') {
    if (!n.action) {
      n.action = 'status';
    }
    if (!n.repo_path) {
      n.repo_path = '.';
    }
  }

  // ========== AgentSkills/AgentCollaboration/AgentCreator/ToolCreation/Skills ==========
  else if (['agent_skills', 'agent_collaboration', 'agent_creator', 'tool_creation', 'skills', 'autonomous_executor'].includes(toolId)) {
    if (!n.action) {
      throw new Error(`${toolId} tool requires "action" argument`);
    }
  }

  // 其他工具保持原样
  return n;
}

export const useToolCallHandler = ({
  handleTransactionBuild,
  handleTransactionBroadcast,
  refreshWallet,
  recordInteraction,
  appendMessage,
  scrollToBottom,
  openUiResource,
}: ToolCallHandlerConfig) => {
  return useCallback(
    async (toolCalls: ToolCall[] = []): Promise<ToolCallHandlerResult[]> => {
      const results: ToolCallHandlerResult[] = [];
      
      for (const toolCall of toolCalls) {
        const result = toolCall.result || {};
        const toolResult: ToolCallHandlerResult = {
          tool_call_id: toolCall.id,
          role: 'tool',
          name: toolCall.name,
          content: '',
          success: true,
        };

        try {
          if (toolCall.name === 'build_transaction') {
            await handleTransactionBuild(result);
            toolResult.content = 'Transaction built successfully';
          } 
          else if (toolCall.name === 'broadcast_transaction') {
            await handleTransactionBroadcast(result);
            toolResult.content = 'Transaction broadcasted successfully';
          } 
          else if (toolCall.name === 'agent_wallet') {
            await refreshWallet();
            toolResult.content = 'Wallet refreshed successfully';
          } 
          else if (toolCall.name === 'wallet_manager' && result) {
            if (result.instruction) {
              if (result.action === 'switch_network' && result.network) {
                const success = await walletService.switchNetwork(result.network);
                if (success) {
                  recordInteraction('wallet_instruction', {
                    instruction: 'switch_network',
                    chainId: result.network.chainId,
                    name: result.network.name,
                  });
                  toolResult.content = `✅ 已成功切换到 ${result.network.name} (${result.network.type})`;
                  
                  appendMessage({
                    id: `system_${Date.now()}`,
                    type: 'assistant',
                    content: toolResult.content,
                    timestamp: Date.now(),
                    source: 'system',
                  });
                  scrollToBottom();
                }
              } else {
                await walletService.executeInstruction(result.instruction);
                recordInteraction('wallet_instruction', {
                  instruction: result.instruction?.method || 'unknown',
                });
                toolResult.content = 'Wallet instruction executed successfully';
              }
            }
          } 
          else if (toolCall.name === 'ui_resource' || toolCall.name === 'mcp_ui') {
            const { resource, resources } = result || {};
            if (Array.isArray(resources) && resources.length > 0) {
              openUiResource(resources[0], { source: toolCall.name });
            } else if (resource) {
              openUiResource(resource, { source: toolCall.name });
            }
            toolResult.content = 'UI resource opened successfully';
          } 
          else {
            toolResult.content = JSON.stringify(result) || 'Tool executed successfully';
          }
        } catch (error: unknown) {
          console.error(`Tool call ${toolCall.name} failed:`, error);
          toolResult.content = `Error: ${(error instanceof Error ? error.message : 'Unknown error')}`;
          toolResult.success = false;
          
          appendMessage({
            id: `error_${Date.now()}`,
            type: 'assistant',
            content: `❌ ${toolCall.name} 操作失败：${error instanceof Error ? error.message : '未知错误'}`,
            timestamp: Date.now(),
            source: 'error',
          });
          scrollToBottom();
        }
        
        results.push(toolResult);
      }
      
      return results;
    },
    [
      appendMessage,
      handleTransactionBroadcast,
      handleTransactionBuild,
      openUiResource,
      recordInteraction,
      refreshWallet,
      scrollToBottom,
    ],
  );
}


