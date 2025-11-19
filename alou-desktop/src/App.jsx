import React, { useEffect } from 'react'
import AppRoutes from './routes/AppRoutes'
import ErrorBoundary from './components/ErrorBoundary'
import ipfsService from './services/ipfsService'

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
      <AppRoutes />
    </ErrorBoundary>
  )
}

export default App
