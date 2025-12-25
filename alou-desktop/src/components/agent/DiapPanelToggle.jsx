import React, { useEffect, useRef } from 'react'
import DiapIdentityPanel from '@/components/agent/DiapIdentityPanel'
import DiapPanelIcon from '@/assets/设置0.3.png'
import './DiapPanelToggle.css'

function DiapPanelToggle({
  sessionId,
  selectedAgent,
  isSidebarCollapsed,
  isDarkMode,
  showPanel,
  onToggle,
  onClosePanel,
}) {
  if (!sessionId) {
    return null
  }

  const toggleButtonRef = useRef(null)
  const panelRef = useRef(null)

  useEffect(() => {
    if (!showPanel) {
      return undefined
    }

    const handlePointerDown = (event) => {
      const panelEl = panelRef.current
      const toggleEl = toggleButtonRef.current
      if (
        panelEl &&
        !panelEl.contains(event.target) &&
        toggleEl &&
        !toggleEl.contains(event.target)
      ) {
        onClosePanel?.()
      }
    }

    const handleKeyDown = (event) => {
      if (event.key === 'Escape') {
        onClosePanel?.()
      }
    }

    document.addEventListener('mousedown', handlePointerDown)
    document.addEventListener('keydown', handleKeyDown)

    return () => {
      document.removeEventListener('mousedown', handlePointerDown)
      document.removeEventListener('keydown', handleKeyDown)
    }
  }, [showPanel, onClosePanel])

  return (
    <>
      <button
        type="button"
        ref={toggleButtonRef}
        className="diap-toggle-btn"
        style={{
          position: 'fixed',
          top: '64px',
          right: '20px', // 完全固定位置，不受侧边栏状态和偏移影响
          zIndex: 1000,
        }}
        aria-label={showPanel ? '隐藏 DIAP 面板' : '打开 DIAP 面板'}
        onClick={onToggle}
        data-dark-mode={isDarkMode}
      >
        <img
          src={DiapPanelIcon}
          alt="DIAP 面板"
          style={{
            width: 20,
            height: 20,
            display: 'block',
            filter: isDarkMode ? 'brightness(0) invert(1)' : 'brightness(0) invert(0.4)',
          }}
        />
      </button>

      {showPanel && (
        <div
          ref={panelRef}
          className="diap-identity-overlay"
          style={{
            position: 'fixed',
            top: '100px',
            right: '20px', // 完全固定位置，不受侧边栏状态和偏移影响
            zIndex: 999,
            maxWidth: '360px',
          }}
        >
          <DiapIdentityPanel 
            sessionId={sessionId} 
            selectedAgent={selectedAgent}
            onClose={onClosePanel} 
            isDarkMode={isDarkMode} 
          />
        </div>
      )}
    </>
  )
}

export default DiapPanelToggle

