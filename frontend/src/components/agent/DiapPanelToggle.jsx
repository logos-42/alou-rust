import React from 'react'
import DiapIdentityPanel from '@/components/agent/DiapIdentityPanel'
import DiapPanelIcon from '@/assets/方-收缩2.png'

function DiapPanelToggle({
  sessionId,
  isSidebarCollapsed,
  isDarkMode,
  showPanel,
  onToggle,
  onClosePanel,
}) {
  if (!sessionId) {
    return null
  }

  return (
    <>
      <button
        type="button"
        style={{
          position: 'fixed',
          top: '64px',
          right: isSidebarCollapsed ? '20px' : '320px',
          zIndex: 1000,
          border: 'none',
          background: 'transparent',
          padding: 0,
          cursor: 'pointer',
        }}
        aria-label={showPanel ? '隐藏 DIAP 面板' : '打开 DIAP 面板'}
        onClick={onToggle}
      >
        <img
          src={DiapPanelIcon}
          alt="DIAP 面板"
          style={{
            width: 32,
            height: 32,
            display: 'block',
            filter: isDarkMode ? 'invert(1)' : 'none',
          }}
        />
      </button>

      {showPanel && (
        <div
          className="diap-identity-overlay"
          style={{
            position: 'fixed',
            top: '160px',
            right: isSidebarCollapsed ? '20px' : '320px',
            zIndex: 999,
            maxWidth: '360px',
          }}
        >
          <DiapIdentityPanel sessionId={sessionId} onClose={onClosePanel} isDarkMode={isDarkMode} />
        </div>
      )}
    </>
  )
}

export default DiapPanelToggle

