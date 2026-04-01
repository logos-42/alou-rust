/**
 * Secure storage utility for default private key
 * Uses Tauri secure storage in desktop app, AES-256-GCM for browser
 */

import { invoke } from '@tauri-apps/api/core'

const DEFAULT_KEY_STORAGE_KEY = 'alou_default_wallet_key'
const DEFAULT_KEY_ADDRESS_KEY = 'alou_default_wallet_address'
const DEFAULT_MNEMONIC_KEY = 'alou_default_wallet_mnemonic'
const ENCRYPTION_SALT_KEY = 'alou_encryption_salt'
const WALLET_LIST_KEY = 'alou_wallet_list'

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
 * Derive AES-GCM key from browser fingerprint
 * Uses Web Crypto API PBKDF2 for key derivation
 */
async function getBrowserEncryptionKey(): Promise<CryptoKey> {
  // Generate or retrieve salt
  let salt = localStorage.getItem(ENCRYPTION_SALT_KEY)
  if (!salt) {
    const saltBytes = crypto.getRandomValues(new Uint8Array(16))
    salt = btoa(String.fromCharCode(...saltBytes))
    localStorage.setItem(ENCRYPTION_SALT_KEY, salt)
  }

  // Create machine-specific key material from navigator properties
  const keyMaterial = [
    navigator.userAgent,
    navigator.language,
    navigator.platform,
    screen.width,
    screen.height,
    new Date().getTimezoneOffset(),
    'alou-browser-encryption-v1',
  ].join('|')

  const encoder = new TextEncoder()
  const keyData = encoder.encode(keyMaterial)
  const saltData = Uint8Array.from(atob(salt), (c) => c.charCodeAt(0))

  // Import key material
  const baseKey = await crypto.subtle.importKey(
    'raw',
    keyData,
    'PBKDF2',
    false,
    ['deriveKey']
  )

  // Derive AES-GCM key using PBKDF2
  return crypto.subtle.deriveKey(
    {
      name: 'PBKDF2',
      salt: saltData,
      iterations: 100000,
      hash: 'SHA-256',
    },
    baseKey,
    { name: 'AES-GCM', length: 256 },
    false,
    ['encrypt', 'decrypt']
  )
}

/**
 * Encrypt data using AES-256-GCM (browser)
 * Returns base64 encoded string with IV prepended
 */
async function browserEncrypt(plaintext: string): Promise<string> {
  const key = await getBrowserEncryptionKey()
  const encoder = new TextEncoder()
  const data = encoder.encode(plaintext)

  const iv = crypto.getRandomValues(new Uint8Array(12))
  const encrypted = await crypto.subtle.encrypt(
    { name: 'AES-GCM', iv },
    key,
    data
  )

  // Combine IV + ciphertext and encode as base64
  const result = new Uint8Array(iv.length + encrypted.byteLength)
  result.set(iv)
  result.set(new Uint8Array(encrypted), iv.length)

  return btoa(String.fromCharCode(...result))
}

/**
 * Decrypt data using AES-256-GCM (browser)
 * Expects base64 encoded string with IV prepended
 */
async function browserDecrypt(ciphertext: string): Promise<string | null> {
  try {
    const key = await getBrowserEncryptionKey()

    // Decode base64
    const data = Uint8Array.from(atob(ciphertext), (c) => c.charCodeAt(0))

    if (data.length < 13) {
      return null
    }

    const iv = data.slice(0, 12)
    const encrypted = data.slice(12)

    const decrypted = await crypto.subtle.decrypt(
      { name: 'AES-GCM', iv },
      key,
      encrypted
    )

    const decoder = new TextDecoder()
    return decoder.decode(decrypted)
  } catch {
    return null
  }
}

/**
 * Save default private key securely
 * In Tauri: uses secure storage with AES-256-GCM
 * In browser: uses AES-256-GCM via Web Crypto API
 */
export async function saveDefaultPrivateKey(privateKey: string): Promise<void> {
  try {
    if (isTauri()) {
      await invoke('save_secure_storage', {
        key: DEFAULT_KEY_STORAGE_KEY,
        value: privateKey,
      })
    } else {
      const encrypted = await browserEncrypt(privateKey)
      localStorage.setItem(DEFAULT_KEY_STORAGE_KEY, encrypted)
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
      const result = await invoke('get_secure_storage', {
        key: DEFAULT_KEY_STORAGE_KEY,
      })
      return result as string | null
    } else {
      const stored = localStorage.getItem(DEFAULT_KEY_STORAGE_KEY)
      if (!stored) return null

      // Try AES-GCM decryption first
      const decrypted = await browserDecrypt(stored)
      if (decrypted) return decrypted

      // Backward compatibility: return plain text if decryption fails
      // (for data stored before encryption was implemented)
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
      const encrypted = await browserEncrypt(address)
      localStorage.setItem(DEFAULT_KEY_ADDRESS_KEY, encrypted)
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
      const stored = localStorage.getItem(DEFAULT_KEY_ADDRESS_KEY)
      if (!stored) return null

      const decrypted = await browserDecrypt(stored)
      if (decrypted) return decrypted

      return stored
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
      localStorage.removeItem(ENCRYPTION_SALT_KEY)
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

  const keyWithoutPrefix = privateKey.startsWith('0x') ? privateKey.slice(2) : privateKey

  if (keyWithoutPrefix.length < 10) {
    return '***'
  }

  const prefix = keyWithoutPrefix.slice(0, 6)
  const suffix = keyWithoutPrefix.slice(-4)

  return `0x${prefix}...${suffix}`
}

/**
 * Save mnemonic phrase securely (encrypted)
 */
export async function saveMnemonic(mnemonic: string): Promise<void> {
  try {
    if (isTauri()) {
      await invoke('save_secure_storage', {
        key: DEFAULT_MNEMONIC_KEY,
        value: mnemonic,
      })
    } else {
      const encrypted = await browserEncrypt(mnemonic)
      localStorage.setItem(DEFAULT_MNEMONIC_KEY, encrypted)
    }
  } catch (error) {
    console.error('[SecureStorage] Failed to save mnemonic:', error)
    throw error
  }
}

/**
 * Get stored mnemonic phrase
 */
export async function getMnemonic(): Promise<string | null> {
  try {
    if (isTauri()) {
      const result = await invoke('get_secure_storage', {
        key: DEFAULT_MNEMONIC_KEY,
      })
      return result as string | null
    } else {
      const stored = localStorage.getItem(DEFAULT_MNEMONIC_KEY)
      if (!stored) return null

      const decrypted = await browserDecrypt(stored)
      if (decrypted) return decrypted

      return stored
    }
  } catch (error) {
    console.error('[SecureStorage] Failed to get mnemonic:', error)
    return null
  }
}

/**
 * Clear stored mnemonic
 */
export async function clearMnemonic(): Promise<void> {
  try {
    if (isTauri()) {
      await invoke('delete_secure_storage', {
        key: DEFAULT_MNEMONIC_KEY,
      })
    } else {
      localStorage.removeItem(DEFAULT_MNEMONIC_KEY)
    }
  } catch (error) {
    console.error('[SecureStorage] Failed to clear mnemonic:', error)
  }
}

export interface StoredWallet {
  id: string
  name: string
  chain: 'ethereum' | 'solana'
  address: string
  createdAt: string
}

/**
 * Save wallet list (metadata only, not keys)
 */
export async function saveWalletList(wallets: StoredWallet[]): Promise<void> {
  try {
    const data = JSON.stringify(wallets)
    if (isTauri()) {
      await invoke('save_secure_storage', {
        key: WALLET_LIST_KEY,
        value: data,
      })
    } else {
      const encrypted = await browserEncrypt(data)
      localStorage.setItem(WALLET_LIST_KEY, encrypted)
    }
  } catch (error) {
    console.error('[SecureStorage] Failed to save wallet list:', error)
  }
}

/**
 * Get stored wallet list
 */
export async function getWalletList(): Promise<StoredWallet[]> {
  try {
    let data: string | null
    if (isTauri()) {
      const result = await invoke('get_secure_storage', {
        key: WALLET_LIST_KEY,
      })
      data = result as string | null
    } else {
      const stored = localStorage.getItem(WALLET_LIST_KEY)
      if (!stored) return []
      const decrypted = await browserDecrypt(stored)
      data = decrypted || stored
    }

    if (!data) return []
    return JSON.parse(data)
  } catch {
    return []
  }
}

/**
 * Add wallet to the list
 */
export async function addWalletToList(wallet: StoredWallet): Promise<void> {
  const list = await getWalletList()
  const exists = list.find((w) => w.address === wallet.address)
  if (exists) {
    Object.assign(exists, wallet)
  } else {
    list.push(wallet)
  }
  await saveWalletList(list)
}

/**
 * Remove wallet from the list
 */
export async function removeWalletFromList(address: string): Promise<void> {
  const list = await getWalletList()
  const filtered = list.filter((w) => w.address !== address)
  await saveWalletList(filtered)
}
