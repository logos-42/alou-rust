// ============================================
// Wallet Sync Service - 浏览器和桌面应用之间的钱包信息同步
// ============================================

/**
 * 检测是否在桌面环境
 */
function isDesktop() {
  if (typeof window === 'undefined') return false
  return typeof window.__TAURI__ !== 'undefined' || 
         typeof window.__TAURI_IPC__ !== 'undefined' ||
         (typeof import.meta !== 'undefined' && Boolean(import.meta.env?.TAURI_PLATFORM))
}

/**
 * 同步钱包信息到桌面应用
 * 在浏览器登录成功后调用此函数
 */
export async function syncWalletToDesktop({ address, chainId, walletType, token }) {
  try {
    // 如果在桌面环境中，不需要同步（已经在桌面应用中了）
    if (isDesktop()) {
      console.log('[WalletSync] Already in desktop environment, no sync needed')
      return { success: true, message: 'Already in desktop' }
    }

    // 方案1：尝试使用深度链接（如果桌面应用注册了协议）
    try {
      const deepLink = `alou://wallet-sync?address=${encodeURIComponent(address)}&chainId=${encodeURIComponent(chainId || '0x1')}&walletType=${encodeURIComponent(walletType || 'metamask')}&token=${encodeURIComponent(token || '')}`
      
      // 尝试打开深度链接
      const link = document.createElement('a')
      link.href = deepLink
      link.style.display = 'none'
      document.body.appendChild(link)
      link.click()
      document.body.removeChild(link)
      
      console.log('[WalletSync] Deep link sent:', deepLink)
      
      // 给桌面应用一些时间来处理
      await new Promise(resolve => setTimeout(resolve, 500))
      
      return { success: true, method: 'deep-link', message: 'Wallet info sent to desktop app' }
    } catch (deepLinkError) {
      console.warn('[WalletSync] Deep link failed:', deepLinkError)
    }

    // 方案2：通过 HTTP POST 请求发送到桌面应用的本地服务器（最可靠的方法）
    try {
      // 尝试常见的端口（桌面应用会在启动时选择一个可用端口）
      const ports = [1421, 1422, 1423, 8080, 8081]
      
      for (const port of ports) {
        try {
          const syncData = {
            address,
            chain_id: chainId || '0x1',
            wallet_type: walletType || 'metamask',
            token: token || '',
          }
          
          const response = await fetch(`http://localhost:${port}/wallet-sync`, {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json',
            },
            body: JSON.stringify(syncData),
          })
          
          if (response.ok) {
            const result = await response.json()
            console.log('[WalletSync] Wallet sync data sent to desktop app via HTTP:', port, result)
            
            return { 
              success: true, 
              method: 'http-post', 
              port,
              message: `Wallet info sent to desktop app on port ${port}` 
            }
          }
        } catch (httpError) {
          // 端口不可用，尝试下一个
          continue
        }
      }
      
      console.warn('[WalletSync] HTTP POST failed on all ports, trying Tauri API')
    } catch (httpError) {
      console.warn('[WalletSync] HTTP sync failed:', httpError)
    }

    // 方案3：尝试通过 Tauri API 写入文件（如果在桌面应用内打开）
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      
      const syncData = {
        address,
        chainId: chainId || '0x1',
        walletType: walletType || 'metamask',
        token: token || '',
        timestamp: Date.now(),
        source: 'browser',
      }
      
      await invoke('write_wallet_sync_data', { 
        data: JSON.stringify(syncData) 
      })
      
      console.log('[WalletSync] Wallet sync data written to temp file via Tauri')
      
      return { success: true, method: 'file-system-tauri', message: 'Wallet info written to temp file via Tauri' }
    } catch (tauriError) {
      console.warn('[WalletSync] Tauri API not available:', tauriError)
    }

    // 方案3：显示提示信息，引导用户打开桌面应用
    // 由于浏览器和桌面应用的 localStorage 是独立的，无法直接共享
    // 所以显示一个提示，告知用户需要在桌面应用中同步
    console.log('[WalletSync] Deep link and callback URL methods unavailable')
    console.log('[WalletSync] Wallet info:', { address, chainId, walletType })
    
    // 显示一个友好的提示（可以在浏览器中显示）
    if (typeof window !== 'undefined' && window.alert) {
      // 可选：显示一个提示（如果用户允许）
      // window.alert('钱包连接成功！请在桌面应用中手动同步钱包信息。')
    }
    
    return { 
      success: false, 
      method: 'none', 
      message: '无法自动同步到桌面应用，请手动在桌面应用中连接钱包或使用深度链接功能' 
    }
  } catch (error) {
    console.error('[WalletSync] Sync error:', error)
    return { 
      success: false, 
      error: error.message || 'Unknown error',
      message: 'Failed to sync wallet info to desktop app' 
    }
  }
}

/**
 * 从本地存储读取待同步的钱包信息（桌面应用调用）
 * 注意：由于浏览器和桌面应用的 localStorage 是独立的，此方法在当前实现中不会工作
 * 主要用于通过 URL 参数传递的情况
 */
export function getPendingWalletSync() {
  // 此方法在当前实现中不适用，因为浏览器和桌面应用的 localStorage 是独立的
  // 钱包信息主要通过深度链接或回调URL传递
  return null
}

/**
 * 清除待同步的钱包信息（桌面应用同步成功后调用）
 */
export function clearPendingWalletSync() {
  try {
    if (typeof window !== 'undefined' && window.localStorage) {
      localStorage.removeItem('wallet_sync_pending')
      localStorage.removeItem('wallet_sync_flag')
    }
  } catch (error) {
    console.error('[WalletSync] Error clearing pending sync:', error)
  }
}

/**
 * 监听来自浏览器的钱包同步（桌面应用调用）
 * 通过轮询文件系统和检查 URL 参数来实现
 */
export async function watchForWalletSync(onSync) {
  // 方案1：检查URL参数（如果通过回调URL或深度链接打开）
  if (typeof window !== 'undefined') {
    const params = new URLSearchParams(window.location.search)
    const address = params.get('address')
    const chainId = params.get('chainId')
    const walletType = params.get('walletType')
    const token = params.get('token')

    if (address) {
      // 从URL参数获取钱包信息
      const syncData = {
        address,
        chainId: chainId || '0x1',
        walletType: walletType || 'metamask',
        token: token || '',
        timestamp: Date.now(),
        source: 'browser-url',
      }

      onSync(syncData)
      return
    }
  }

  // 方案2：持续轮询文件系统（桌面应用使用）
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    
    // 检查文件的函数
    const checkFile = async () => {
      try {
        const data = await invoke('read_wallet_sync_data')
        if (data) {
          const syncData = JSON.parse(data)
          
          // 检查数据是否过期（5分钟内有效）
          const now = Date.now()
          const fiveMinutes = 5 * 60 * 1000
          if (now - syncData.timestamp <= fiveMinutes) {
            console.log('[WalletSync] Found wallet sync data in temp file:', syncData)
            onSync(syncData)
            return true
          } else {
            console.log('[WalletSync] Wallet sync data expired')
          }
        }
      } catch (error) {
        // 文件不存在或其他错误，继续轮询
        if (error && !error.includes('Failed to read sync data')) {
          console.warn('[WalletSync] Error reading sync file:', error)
        }
      }
      return false
    }
    
    // 立即检查一次
    if (await checkFile()) {
      console.log('[WalletSync] Wallet sync data found immediately')
      return
    }
    
    // 如果没有找到，持续轮询（直到找到数据或组件卸载）
    // 每500ms检查一次，更频繁的检查确保及时响应
    console.log('[WalletSync] Started continuous polling for wallet sync data in temp file')
    const intervalId = setInterval(async () => {
      const found = await checkFile()
      if (found) {
        clearInterval(intervalId)
        console.log('[WalletSync] Wallet sync data found, stopping poll')
      }
    }, 500) // 每500ms检查一次
    
    // 返回清理函数，让调用者可以在需要时停止轮询
    // 注意：由于这个函数是异步的，我们无法直接返回清理函数
    // 调用者需要通过 useEffect 的清理函数来处理
    console.log('[WalletSync] Polling interval set up, checking every 500ms')
  } catch (error) {
    console.warn('[WalletSync] File system polling not available:', error)
    console.log('[WalletSync] No wallet sync data found in URL parameters')
  }
}

