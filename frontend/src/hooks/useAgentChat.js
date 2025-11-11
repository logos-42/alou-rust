import { useCallback } from 'react'
import { walletService } from '@/services/walletService'

export const API_BASE_URL =
  import.meta.env.VITE_API_BASE_URL ||
  (import.meta.env.DEV ? 'http://localhost:8787' : 'https://alou-edge.yuanjieliu65.workers.dev')

export const NODE_BOUNDARY = 140

export const defaultChannels = [
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

export const defaultTransactions = [
  {
    id: 'tx-1',
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
    direction: 'out',
    amount: '1.2',
    token: 'ETH',
    counterparty: '0xbf12...9980',
    status: 'failed',
    statusLabel: '失败',
    timestamp: Date.now() - 3 * 24 * 60 * 60 * 1000,
  },
]

export const ensureMillis = (value) => {
  if (!value || Number.isNaN(Number(value))) {
    return Date.now()
  }
  if (value > 1_000_000_000_000) {
    return value
  }
  return value * 1000
}

const chainLabelMap = {
  ethereum: 'Ethereum Mainnet',
  eth: 'Ethereum Mainnet',
  eth_sepolia: 'Ethereum Sepolia',
  base: 'Base Mainnet',
  base_sepolia: 'Base Sepolia',
  polygon: 'Polygon Mainnet',
  polygon_amoy: 'Polygon Amoy',
  sol: 'Solana',
}

const chainIdToBackendMap = {
  '0x1': 'ethereum',
  '0x01': 'ethereum',
  '0xaa36a7': 'eth_sepolia',
  '0x14a34': 'base_sepolia',
  '0x2105': 'base',
}

export const mapChainLabel = (chain) => chainLabelMap[chain] || chain?.toUpperCase?.() || '未知网络'

export const mapChainIdToBackendChain = (chainId) => {
  if (!chainId) return null
  const normalized = chainId.toLowerCase()
  return chainIdToBackendMap[normalized] || null
}

export const normalizeBackendChain = (chain) => {
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

export const resolveBackendChain = ({ chainId, chain } = {}) => {
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

export const estimateFiatValue = (balance, token = 'ETH') => {
  const numeric = Number(balance)
  if (Number.isFinite(numeric)) {
    const usd = token === 'SOL' ? numeric * 150 : numeric * 3400
    return usd.toFixed(2)
  }
  return '--'
}

export const normalizeTransaction = (tx, fallbackToken = 'ETH') => {
  if (!tx) return null

  const hash = tx.hash || tx.id || `tx_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`

  const rawDirection = tx.direction || tx.type
  const direction = rawDirection === 'in' || rawDirection === 'receive' ? 'in' : 'out'

  const amountValue = tx.amount ?? tx.value ?? tx.quantity ?? tx.displayValue ?? '0'

  const token = tx.token || fallbackToken
  const counterparty = tx.counterparty || tx.to || tx.from || ''

  const status = tx.status || 'pending'
  const statusLabelMap = {
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

export const normalizeChannel = (channel, index = 0) => {
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

export const formatWeiHexToEth = (hexValue) => {
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

export const ACTION_LABELS = {
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

export const useToolCallHandler = ({
  handleTransactionBuild,
  handleTransactionBroadcast,
  refreshWallet,
  recordInteraction,
  appendMessage,
  scrollToBottom,
  openUiResource,
}) => {
  return useCallback(
    async (toolCalls = []) => {
      for (const toolCall of toolCalls) {
        const result = toolCall.result

        if (toolCall.name === 'build_transaction') {
          await handleTransactionBuild(result)
          continue
        }

        if (toolCall.name === 'broadcast_transaction') {
          await handleTransactionBroadcast(result)
          continue
        }

        if (toolCall.name === 'agent_wallet') {
          await refreshWallet()
          continue
        }

        if (toolCall.name === 'wallet_manager' && result) {
          if (result.instruction) {
            try {
              if (result.action === 'switch_network' && result.network) {
                const success = await walletService.switchNetwork(result.network)
                if (success) {
                  recordInteraction('wallet_instruction', {
                    instruction: 'switch_network',
                    chainId: result.network.chainId,
                    name: result.network.name,
                  })
                  appendMessage({
                    id: `system_${Date.now()}`,
                    type: 'assistant',
                    content: `✅ 已成功切换到 ${result.network.name} (${result.network.type})`,
                    timestamp: Date.now(),
                    source: 'system',
                  })
                  scrollToBottom()
                }
              } else {
                await walletService.executeInstruction(result.instruction)
                recordInteraction('wallet_instruction', {
                  instruction: result.instruction?.method || 'unknown',
                })
              }
            } catch (error) {
              console.error('Failed to execute wallet instruction:', error)
              appendMessage({
                id: `error_${Date.now()}`,
                type: 'assistant',
                content: `❌ 钱包操作失败：${error instanceof Error ? error.message : '未知错误'}`,
                timestamp: Date.now(),
                source: 'error',
              })
              scrollToBottom()
            }
          }
        }

        if (toolCall.name === 'ui_resource' || toolCall.name === 'mcp_ui') {
          const { resource, resources } = result || {}
          if (Array.isArray(resources) && resources.length > 0) {
            openUiResource(resources[0], { source: toolCall.name })
          } else if (resource) {
            openUiResource(resource, { source: toolCall.name })
          }
        }
      }
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
  )
}
