/**
 * 轻量级 IndexedDB 封装 - 用于 Session 数据持久化
 *
 * 相比 localStorage 的优势：
 * - 容量大（通常 50MB+，可动态扩展）
 * - 支持结构化数据
 * - 异步操作，不阻塞主线程
 * - 支持索引和查询
 *
 * @module utils/sessionStorage
 */

const DB_NAME = 'alou_session_db'
const DB_VERSION = 1
const STORE_NAME = 'sessions'

/**
 * 打开数据库连接
 */
function openDB(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, DB_VERSION)

    request.onerror = () => {
      console.error('[SessionStorage] 打开数据库失败:', request.error)
      reject(request.error)
    }

    request.onsuccess = () => {
      resolve(request.result)
    }

    request.onupgradeneeded = (event) => {
      const db = (event.target as IDBOpenDBRequest).result
      
      // 创建 object store（如果不存在）
      if (!db.objectStoreNames.contains(STORE_NAME)) {
        const store = db.createObjectStore(STORE_NAME, { keyPath: 'key' })
        store.createIndex('sessionId', 'sessionId', { unique: false })
        store.createIndex('timestamp', 'timestamp', { unique: false })
      }
    }
  })
}

/**
 * Session 数据存储项
 */
export interface SessionStorageItem {
  key: string
  sessionId: string
  value: any
  timestamp: number
  expiresAt?: number
}

/**
 * 设置数据
 */
export async function set(
  key: string,
  value: any,
  options?: { sessionId?: string; expiresAt?: number }
): Promise<void> {
  try {
    const db = await openDB()
    
    return new Promise((resolve, reject) => {
      const transaction = db.transaction([STORE_NAME], 'readwrite')
      const store = transaction.objectStore(STORE_NAME)
      
      const item: SessionStorageItem = {
        key,
        sessionId: options?.sessionId || '',
        value,
        timestamp: Date.now(),
        expiresAt: options?.expiresAt,
      }
      
      const request = store.put(item)
      
      request.onsuccess = () => {
        resolve()
      }
      
      request.onerror = () => {
        console.error('[SessionStorage] 写入数据失败:', request.error)
        reject(request.error)
      }
    })
  } catch (error) {
    console.error('[SessionStorage] set 失败:', error)
    throw error
  }
}

/**
 * 获取数据
 */
export async function get<T = any>(key: string): Promise<T | null> {
  try {
    const db = await openDB()
    
    return new Promise((resolve, reject) => {
      const transaction = db.transaction([STORE_NAME], 'readonly')
      const store = transaction.objectStore(STORE_NAME)
      
      const request = store.get(key)
      
      request.onsuccess = () => {
        const item = request.result as SessionStorageItem | undefined
        
        if (!item) {
          resolve(null)
          return
        }
        
        // 检查是否过期
        if (item.expiresAt && Date.now() > item.expiresAt) {
          // 删除过期数据
          deleteByKey(key).catch(console.error)
          resolve(null)
          return
        }
        
        resolve(item.value as T)
      }
      
      request.onerror = () => {
        console.error('[SessionStorage] 读取数据失败:', request.error)
        reject(request.error)
      }
    })
  } catch (error) {
    console.error('[SessionStorage] get 失败:', error)
    throw error
  }
}

/**
 * 删除数据
 */
export async function deleteByKey(key: string): Promise<void> {
  try {
    const db = await openDB()
    
    return new Promise((resolve, reject) => {
      const transaction = db.transaction([STORE_NAME], 'readwrite')
      const store = transaction.objectStore(STORE_NAME)
      
      const request = store.delete(key)
      
      request.onsuccess = () => {
        resolve()
      }
      
      request.onerror = () => {
        console.error('[SessionStorage] 删除数据失败:', request.error)
        reject(request.error)
      }
    })
  } catch (error) {
    console.error('[SessionStorage] delete 失败:', error)
    throw error
  }
}

/**
 * 根据 sessionId 获取所有数据
 */
export async function getBySessionId(sessionId: string): Promise<any[]> {
  try {
    const db = await openDB()
    
    return new Promise((resolve, reject) => {
      const transaction = db.transaction([STORE_NAME], 'readonly')
      const store = transaction.objectStore(STORE_NAME)
      const index = store.index('sessionId')
      
      const request = index.getAll(sessionId)
      
      request.onsuccess = () => {
        const items = request.result as SessionStorageItem[]
        
        // 过滤过期数据
        const validItems = items.filter(item => {
          if (item.expiresAt && Date.now() > item.expiresAt) {
            deleteByKey(item.key).catch(console.error)
            return false
          }
          return true
        })
        
        resolve(validItems.map(item => item.value))
      }
      
      request.onerror = () => {
        console.error('[SessionStorage] 按 session 查询失败:', request.error)
        reject(request.error)
      }
    })
  } catch (error) {
    console.error('[SessionStorage] getBySessionId 失败:', error)
    throw error
  }
}

/**
 * 删除指定 session 的所有数据
 */
export async function deleteBySessionId(sessionId: string): Promise<void> {
  try {
    const db = await openDB()
    
    return new Promise((resolve, reject) => {
      const transaction = db.transaction([STORE_NAME], 'readwrite')
      const store = transaction.objectStore(STORE_NAME)
      const index = store.index('sessionId')
      
      const request = index.getAllKeys(sessionId)
      
      request.onsuccess = () => {
        const keys = request.result as string[]
        
        // 批量删除
        const deletePromises = keys.map(key => {
          return new Promise<void>((resolveDelete, rejectDelete) => {
            const deleteRequest = store.delete(key)
            deleteRequest.onsuccess = () => resolveDelete()
            deleteRequest.onerror = () => rejectDelete(deleteRequest.error)
          })
        })
        
        Promise.all(deletePromises)
          .then(() => resolve())
          .catch(reject)
      }
      
      request.onerror = () => {
        console.error('[SessionStorage] 按 session 删除失败:', request.error)
        reject(request.error)
      }
    })
  } catch (error) {
    console.error('[SessionStorage] deleteBySessionId 失败:', error)
    throw error
  }
}

/**
 * 获取所有 keys
 */
export async function getAllKeys(): Promise<string[]> {
  try {
    const db = await openDB()
    
    return new Promise((resolve, reject) => {
      const transaction = db.transaction([STORE_NAME], 'readonly')
      const store = transaction.objectStore(STORE_NAME)
      
      const request = store.getAllKeys()
      
      request.onsuccess = () => {
        resolve(request.result as string[])
      }
      
      request.onerror = () => {
        console.error('[SessionStorage] 获取所有 keys 失败:', request.error)
        reject(request.error)
      }
    })
  } catch (error) {
    console.error('[SessionStorage] getAllKeys 失败:', error)
    throw error
  }
}

/**
 * 清除所有数据
 */
export async function clear(): Promise<void> {
  try {
    const db = await openDB()
    
    return new Promise((resolve, reject) => {
      const transaction = db.transaction([STORE_NAME], 'readwrite')
      const store = transaction.objectStore(STORE_NAME)
      
      const request = store.clear()
      
      request.onsuccess = () => {
        resolve()
      }
      
      request.onerror = () => {
        console.error('[SessionStorage] 清除数据失败:', request.error)
        reject(request.error)
      }
    })
  } catch (error) {
    console.error('[SessionStorage] clear 失败:', error)
    throw error
  }
}

/**
 * 获取数据库使用统计
 */
export async function getStats(): Promise<{
  totalItems: number
  totalSize: number
  sessions: string[]
}> {
  try {
    const db = await openDB()
    
    return new Promise((resolve, reject) => {
      const transaction = db.transaction([STORE_NAME], 'readonly')
      const store = transaction.objectStore(STORE_NAME)
      
      const countRequest = store.count()
      const getAllRequest = store.getAll()
      
      countRequest.onsuccess = () => {
        getAllRequest.onsuccess = async () => {
          const items = getAllRequest.result as SessionStorageItem[]
          const sessions = [...new Set(items.map(item => item.sessionId).filter(Boolean))]
          
          // 估算大小（JSON 字符串长度）
          const totalSize = new TextEncoder().encode(
            JSON.stringify(items)
          ).length
          
          resolve({
            totalItems: countRequest.result,
            totalSize,
            sessions,
          })
        }
      }
      
      countRequest.onerror = () => {
        console.error('[SessionStorage] 获取统计失败:', countRequest.error)
        reject(countRequest.error)
      }
    })
  } catch (error) {
    console.error('[SessionStorage] getStats 失败:', error)
    throw error
  }
}

/**
 * 清理过期数据
 */
export async function cleanupExpired(): Promise<number> {
  try {
    const db = await openDB()
    
    return new Promise((resolve, reject) => {
      const transaction = db.transaction([STORE_NAME], 'readwrite')
      const store = transaction.objectStore(STORE_NAME)
      const index = store.index('timestamp')
      
      const request = index.getAll()
      let deletedCount = 0
      
      request.onsuccess = () => {
        const items = request.result as SessionStorageItem[]
        const now = Date.now()
        
        // 删除过期数据
        const deletePromises = items
          .filter(item => item.expiresAt && now > item.expiresAt)
          .map(item => {
            deletedCount++
            return new Promise<void>((resolveDelete, rejectDelete) => {
              const deleteRequest = store.delete(item.key)
              deleteRequest.onsuccess = () => resolveDelete()
              deleteRequest.onerror = () => rejectDelete(deleteRequest.error)
            })
          })
        
        Promise.all(deletePromises)
          .then(() => resolve(deletedCount))
          .catch(reject)
      }
      
      request.onerror = () => {
        console.error('[SessionStorage] 清理过期数据失败:', request.error)
        reject(request.error)
      }
    })
  } catch (error) {
    console.error('[SessionStorage] cleanupExpired 失败:', error)
    return 0
  }
}

/**
 * 检查 IndexedDB 是否可用
 */
export function isAvailable(): boolean {
  return typeof indexedDB !== 'undefined'
}
