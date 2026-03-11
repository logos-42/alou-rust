import { useEffect } from 'react'
import AppRoutes from './routes/AppRoutes'
import ErrorBoundary from './components/ErrorBoundary'
import { SessionProvider } from '@/context/SessionContext'
import ipfsService from './services/ipfsService'
import '@/utils/diapIdentityCleanupTool' // 加载 DIAP 身份清理工具
import '@/utils/avatarProtectionTool' // 加载头像保护工具
import '@/utils/agentCreationDiagnosticTool' // 加载智能体创建诊断工具

const AppContent = () => {
  useEffect(() => {
    let cancelled = false

    const ensureIpfs = async () => {
      try {
        const result = await ipfsService.startNode(true)
        if (!cancelled && result?.success) {
          console.log('[IPFS] 节点已自动启动')
        }
      } catch (error) {
        console.warn('[IPFS] 自动启动失败', error)
      }
    }

    const autoFixAvatars = async () => {
      // 等待一小段时间，确保 store 已经 hydration
      await new Promise(resolve => setTimeout(resolve, 500))
      try {
        // @ts-ignore - 工具已加载到 window
        if (window.AlouAvatarProtection) {
          // @ts-ignore
          const fixedCount = await window.AlouAvatarProtection.autoFixAvatarUrls()
          console.log(`[App] 启动时自动修复了 ${fixedCount} 个头像 URL`)
        }
      } catch (error) {
        console.warn('[App] 自动修复头像失败', error)
      }
    }

    ensureIpfs()
    autoFixAvatars()

    return () => {
      cancelled = true
    }
  }, [])

  return (
    <div className="app-container">
      <AppRoutes />

      <style>{`
        .app-container {
          position: relative;
          width: 100%;
          height: 100vh;
        }
      `}</style>
    </div>
  )
}

const App = () => {
  return (
    <ErrorBoundary>
      <SessionProvider autoInit={true} defaultConfig={{ debug: false }}>
        <AppContent />
      </SessionProvider>
    </ErrorBoundary>
  )
}

export default App
