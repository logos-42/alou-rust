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

interface ImportMeta {
  readonly env: ImportMetaEnv
}

// 扩展全局 Window 接口
declare global {
  interface Window {
    __lastLoadChannelsError?: number
  }
}
