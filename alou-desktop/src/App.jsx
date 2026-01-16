import React, { useEffect, useState } from 'react'
import AppRoutes from './routes/AppRoutes'
import ErrorBoundary from './components/ErrorBoundary'
import ToolPanel from './components/ToolPanel'
import ipfsService from './services/ipfsService'

const App = () => {
  const [showToolPanel, setShowToolPanel] = useState(false);

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

  // 监听键盘快捷键打开工具面板
  useEffect(() => {
    const handleKeyPress = (event) => {
      // Ctrl/Cmd + Shift + T 打开工具面板
      if ((event.ctrlKey || event.metaKey) && event.shiftKey && event.key === 'T') {
        event.preventDefault();
        setShowToolPanel(true);
      }

      // ESC 关闭工具面板
      if (event.key === 'Escape' && showToolPanel) {
        setShowToolPanel(false);
      }
    };

    document.addEventListener('keydown', handleKeyPress);
    return () => document.removeEventListener('keydown', handleKeyPress);
  }, [showToolPanel]);

  return (
    <ErrorBoundary>
      <div className="app-container">
        <AppRoutes />

        {/* 工具面板 */}
        {showToolPanel && (
          <ToolPanel onClose={() => setShowToolPanel(false)} />
        )}

        {/* 工具面板切换按钮 */}
        {!showToolPanel && (
          <button
            className="tool-panel-toggle"
            onClick={() => setShowToolPanel(true)}
            title="打开工具执行器 (Ctrl+Shift+T)"
          >
            🛠️
          </button>
        )}
      </div>

      <style jsx>{`
        .app-container {
          position: relative;
          width: 100%;
          height: 100vh;
        }

        .tool-panel-toggle {
          position: fixed;
          bottom: 20px;
          right: 20px;
          width: 60px;
          height: 60px;
          border-radius: 50%;
          background: #007bff;
          color: white;
          border: none;
          font-size: 24px;
          cursor: pointer;
          box-shadow: 0 4px 12px rgba(0,0,0,0.15);
          z-index: 999;
          transition: all 0.2s;
        }

        .tool-panel-toggle:hover {
          background: #0056b3;
          transform: scale(1.05);
        }

        .tool-panel-toggle:active {
          transform: scale(0.95);
        }
      `}</style>
    </ErrorBoundary>
  )
}

export default App