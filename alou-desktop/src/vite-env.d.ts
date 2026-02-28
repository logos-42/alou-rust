/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_API_BASE_URL: string
  readonly VITE_IPFS_API_URL: string
  readonly VITE_IPFS_GATEWAY_URL: string
  readonly VITE_AI_PROVIDER: string
  readonly VITE_AI_MODEL: string
  readonly VITE_ETH_RPC_URL: string
  readonly VITE_SOL_RPC_URL: string
  readonly VITE_TAURI_PLATFORM: string
  readonly VITE_DEFAULT_CHAIN_ID: string
  readonly VITE_WALLET_NETWORKS: string
  readonly NODE_ENV: string
  readonly DEV: boolean
  readonly PROD: boolean
  readonly [key: string]: string | boolean
}

// eslint-disable-next-line @typescript-eslint/no-unused-vars
interface ImportMeta {
  readonly env: ImportMetaEnv
}

// 扩展全局 Window 接口
declare global {
  interface Window {
    __lastLoadChannelsError?: number
    // Tauri APIs
    __TAURI__?: Record<string, unknown>
    __TAURI_IPC__?: Record<string, unknown>
    // Ethereum Provider (MetaMask etc.)
    ethereum?: {
      request: (args: { method: string; params?: unknown[] }) => Promise<unknown>
      on: (event: string, callback: (...args: unknown[]) => void) => void
      removeListener: (event: string, callback: (...args: unknown[]) => void) => void
      isMetaMask?: boolean
      selectedAddress?: string
      chainId?: string
    }
    // Custom memory storage
    AlouMemoryStorage?: {
      get: (key: string) => string | null
      set: (key: string, value: string) => void
      remove: (key: string) => void
      keys: () => string[]
      clear: () => void
    }
  }
}

// 确保文件被视为模块
export {}
