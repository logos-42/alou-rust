import { useEffect } from 'react'
import AppRoutes from './routes/AppRoutes'
import ErrorBoundary from './components/ErrorBoundary'
import ipfsService from './services/ipfsService'
import '@/utils/diapIdentityCleanupTool' // 加载DIAP身份清理工具
import '@/utils/avatarProtectionTool' // 加载头像保护工具
import '@/utils/agentCreationDiagnosticTool' // 加载智能体创建诊断工具

const App = () => {
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

    ensureIpfs()

    return () => {
      cancelled = true
    }
  }, [])

  return (
    <ErrorBoundary>
      <div className="app-container">
        <AppRoutes />
      </div>

      <style>{`
        .app-container {
          position: relative;
          width: 100%;
          height: 100vh;
        }
      `}</style>
    </ErrorBoundary>
  )
}

export default App