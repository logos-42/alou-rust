/**
 * Secure storage utility for default private key
 * Uses Tauri secure storage in desktop app, localStorage fallback for browser
 */

import { invoke } from '@tauri-apps/api/core'

const DEFAULT_KEY_STORAGE_KEY = 'alou_default_wallet_key'
const DEFAULT_KEY_ADDRESS_KEY = 'alou_default_wallet_address'

/**
 * Check if running in Tauri desktop environment
 */
function isTauri(): boolean {
  if (typeof window === 'undefined') {
    return false
  }
  return (
    typeof window.__TAURI__ !== 'undefined' ||
    typeof window.__TAURI_IPC__ !== 'undefined' ||
    (typeof import.meta !== 'undefined' && Boolean(import.meta.env?.TAURI_PLATFORM))
  )
}

/**
 * Save default private key securely
 * In Tauri: uses secure storage
 * In browser: uses localStorage (development only)
 */
export async function saveDefaultPrivateKey(privateKey: string): Promise<void> {
  try {
    if (isTauri()) {
      // Use Tauri secure storage
      await invoke('save_secure_storage', {
        key: DEFAULT_KEY_STORAGE_KEY,
        value: privateKey,
      })
    } else {
      // Browser fallback (development only)
      localStorage.setItem(DEFAULT_KEY_STORAGE_KEY, privateKey)
    }
    console.log('[SecureStorage] Default private key saved securely')
  } catch (error) {
    console.error('[SecureStorage] Failed to save default private key:', error)
    throw error
  }
}

/**
 * Get default private key
 * Returns null if not set
 */
export async function getDefaultPrivateKey(): Promise<string | null> {
  try {
    if (isTauri()) {
      // Use Tauri secure storage
      const result = await invoke('get_secure_storage', {
        key: DEFAULT_KEY_STORAGE_KEY,
      })
      return result as string | null
    } else {
      // Browser fallback
      const stored = localStorage.getItem(DEFAULT_KEY_STORAGE_KEY)
      return stored
    }
  } catch (error) {
    console.error('[SecureStorage] Failed to get default private key:', error)
    return null
  }
}

/**
 * Save wallet address associated with default key
 */
export async function saveDefaultWalletAddress(address: string): Promise<void> {
  try {
    if (isTauri()) {
      await invoke('save_secure_storage', {
        key: DEFAULT_KEY_ADDRESS_KEY,
        value: address,
      })
    } else {
      localStorage.setItem(DEFAULT_KEY_ADDRESS_KEY, address)
    }
  } catch (error) {
    console.error('[SecureStorage] Failed to save wallet address:', error)
  }
}

/**
 * Get wallet address associated with default key
 */
export async function getDefaultWalletAddress(): Promise<string | null> {
  try {
    if (isTauri()) {
      const result = await invoke('get_secure_storage', {
        key: DEFAULT_KEY_ADDRESS_KEY,
      })
      return result as string | null
    } else {
      return localStorage.getItem(DEFAULT_KEY_ADDRESS_KEY)
    }
  } catch (error) {
    console.error('[SecureStorage] Failed to get wallet address:', error)
    return null
  }
}

/**
 * Check if default private key is set
 */
export async function hasDefaultPrivateKey(): Promise<boolean> {
  try {
    if (isTauri()) {
      const result = await invoke('has_secure_storage', {
        key: DEFAULT_KEY_STORAGE_KEY,
      })
      return result as boolean
    } else {
      const stored = localStorage.getItem(DEFAULT_KEY_STORAGE_KEY)
      return stored !== null && stored.length > 0
    }
  } catch (error) {
    console.error('[SecureStorage] Failed to check default private key:', error)
    return false
  }
}

/**
 * Clear default private key
 */
export async function clearDefaultPrivateKey(): Promise<void> {
  try {
    if (isTauri()) {
      await invoke('delete_secure_storage', {
        key: DEFAULT_KEY_STORAGE_KEY,
      })
      await invoke('delete_secure_storage', {
        key: DEFAULT_KEY_ADDRESS_KEY,
      })
    } else {
      localStorage.removeItem(DEFAULT_KEY_STORAGE_KEY)
      localStorage.removeItem(DEFAULT_KEY_ADDRESS_KEY)
    }
    console.log('[SecureStorage] Default private key cleared')
  } catch (error) {
    console.error('[SecureStorage] Failed to clear default private key:', error)
  }
}

/**
 * Mask private key for display (shows first 6 and last 4 characters)
 * Example: 0x1234...5678
 */
export function maskPrivateKey(privateKey: string): string {
  if (!privateKey || privateKey.length < 10) {
    return '***'
  }
  
  // Remove 0x prefix if present for masking
  const keyWithoutPrefix = privateKey.startsWith('0x') ? privateKey.slice(2) : privateKey
  
  if (keyWithoutPrefix.length < 10) {
    return '***'
  }
  
  const prefix = keyWithoutPrefix.slice(0, 6)
  const suffix = keyWithoutPrefix.slice(-4)
  
  return `0x${prefix}...${suffix}`
}
