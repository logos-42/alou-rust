import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { requestMcpUiResource } from '@/services/mcpUiService'
import { ACTION_LABELS } from '@/hooks/useAgentChat'

/**
 * Hook for managing UI state and interactions
 */
export const useAgentUI = ({
  sessionId,
  viewportWidth: initialViewportWidth = 1440,
  isLeftSidebarCollapsed,
  isSidebarCollapsed,
}) => {
  const [isDarkMode, setIsDarkMode] = useState(false)
  const [isSidebarCollapsedLocal, setSidebarCollapsed] = useState(true)
  const [isLeftSidebarCollapsedLocal, setLeftSidebarCollapsed] = useState(false)
  const [isInteractionCollapsed, setInteractionCollapsed] = useState(true)
  const [isConversationVisible, setConversationVisible] = useState(false)
  const [viewportWidth, setViewportWidth] = useState(initialViewportWidth)
  const [interactionLogs, setInteractionLogs] = useState([])
  const [uiResource, setUiResource] = useState(null)
  const [isUiModalOpen, setUiModalOpen] = useState(false)
  const [showDiapPanel, setShowDiapPanel] = useState(true)

  const finalEventHandledRef = useRef(null)
  const contextEventsRef = useRef([])

  // Use external collapse states if provided, otherwise use local
  const effectiveSidebarCollapsed = isSidebarCollapsed ?? isSidebarCollapsedLocal
  const effectiveLeftSidebarCollapsed = isLeftSidebarCollapsed ?? isLeftSidebarCollapsedLocal

  const recordInteraction = useCallback((action, detail, label) => {
    const timestamp = Date.now()
    const entry = {
      id: `log_${timestamp}_${Math.random().toString(36).slice(2, 6)}`,
      action,
      label: label || ACTION_LABELS[action] || action,
      timestamp,
      detail,
    }

    setInteractionLogs((prev) => {
      const next = [entry, ...prev]
      return next.slice(0, 20)
    })

    contextEventsRef.current.push({ action, detail, timestamp })
    if (contextEventsRef.current.length > 50) {
      contextEventsRef.current.splice(0, contextEventsRef.current.length - 50)
    }

    if (typeof window !== 'undefined') {
      window.dispatchEvent(
        new CustomEvent('agent-context-event', { detail: { action, detail, timestamp } }),
      )
    }
  }, [])

  const openUiResource = useCallback(
    (payload, meta = {}) => {
      if (!payload || !payload.resource) return
      const embedded =
        payload.resource && payload.resource.uri
          ? { resource: payload.resource, metadata: payload.metadata }
          : payload
      setUiResource(embedded)
      setUiModalOpen(true)
      recordInteraction('trigger_mcp', {
        kind: 'ui_resource_open',
        uri: embedded.resource?.uri,
        metadata: payload.metadata,
        ...meta,
      })
    },
    [recordInteraction],
  )

  const fetchAndOpenUiResource = useCallback(
    async (target, params = {}, meta = {}) => {
      if (!target) {
        return
      }
      try {
        const payload = {
          session_id: sessionId,
          ...params,
        }
        const { resource, metadata } = await requestMcpUiResource(target, payload)
        if (!resource) {
          throw new Error('empty resource response')
        }
        openUiResource(
          { resource, metadata },
          {
            source: 'frontend',
            target,
            ...meta,
          },
        )
      } catch (error) {
        console.error('Failed to load MCP UI resource:', error)
        const errorMessage = error instanceof Error ? error.message : String(error)
        recordInteraction('trigger_mcp', {
          kind: 'ui_error',
          target,
          params,
          error: errorMessage,
        })
        setInteractionLogs((prev) => [
          {
            id: `mcp_error_${Date.now()}`,
            action: 'trigger_mcp',
            label: 'MCP UI 加载失败',
            timestamp: Date.now(),
            detail: { target, error: errorMessage },
          },
          ...prev,
        ])
      }
    },
    [openUiResource, recordInteraction, sessionId],
  )

  const closeUiResource = useCallback(() => {
    setUiModalOpen(false)
    setUiResource(null)
  }, [])

  const handleUiAction = useCallback(
    async (action) => {
      console.debug('MCP UI action', action)
      recordInteraction('trigger_mcp', {
        kind: 'ui_action',
        action,
      })
      if (action?.type === 'close') {
        closeUiResource()
      }
      return { type: 'success' }
    },
    [closeUiResource, recordInteraction],
  )

  const toggleSidebar = useCallback(() => {
    setSidebarCollapsed((prev) => {
      const next = !prev
      recordInteraction('toggle_sidebar', { collapsed: next })
      return next
    })
  }, [recordInteraction])

  const toggleInteractionPanel = useCallback(() => {
    setInteractionCollapsed((prev) => !prev)
  }, [])

  const toggleDarkMode = useCallback(() => {
    setIsDarkMode((prev) => {
      const next = !prev
      recordInteraction('toggle_theme', { theme: next ? 'dark' : 'light' })
      if (typeof localStorage !== 'undefined') {
        localStorage.setItem('alou-theme', next ? 'dark' : 'light')
      }
      return next
    })
  }, [recordInteraction])

  const toggleLeftSidebar = useCallback(() => {
    setLeftSidebarCollapsed((prev) => {
      const next = !prev
      recordInteraction('toggle_left_sidebar', { collapsed: next })
      return next
    })
  }, [recordInteraction])

  const openConversationPanel = useCallback(() => {
    setConversationVisible(true)
  }, [])

  const closeConversationPanel = useCallback(() => {
    setConversationVisible(false)
  }, [])

  const handleResize = useCallback(() => {
    if (typeof window === 'undefined') return
    setViewportWidth(window.innerWidth)
  }, [])

  // Console dock style calculation
  const consoleDockStyle = useMemo(() => {
    if (viewportWidth <= 1024) {
      return { margin: '0 1rem 0 1rem' }
    }
    const leftWidth = effectiveLeftSidebarCollapsed ? 84 : viewportWidth <= 1280 ? 240 : 300
    const rightWidth = effectiveSidebarCollapsed ? 80 : 340
    return {
      marginLeft: `${leftWidth + 24}px`,
      marginRight: `${rightWidth + 24}px`,
    }
  }, [viewportWidth, effectiveSidebarCollapsed, effectiveLeftSidebarCollapsed])

  // Initialize dark mode from localStorage or system preference
  useEffect(() => {
    if (typeof localStorage !== 'undefined') {
      const savedTheme = localStorage.getItem('alou-theme')
      if (savedTheme) {
        setIsDarkMode(savedTheme === 'dark')
      } else if (typeof window !== 'undefined') {
        setIsDarkMode(window.matchMedia('(prefers-color-scheme: dark)').matches)
      }
    }
  }, [])

  return {
    isDarkMode,
    setIsDarkMode,
    isSidebarCollapsed: effectiveSidebarCollapsed,
    setSidebarCollapsed,
    isLeftSidebarCollapsed: effectiveLeftSidebarCollapsed,
    setLeftSidebarCollapsed,
    isInteractionCollapsed,
    setInteractionCollapsed,
    isConversationVisible,
    setConversationVisible,
    viewportWidth,
    setViewportWidth,
    interactionLogs,
    setInteractionLogs,
    uiResource,
    setUiResource,
    isUiModalOpen,
    setUiModalOpen,
    showDiapPanel,
    setShowDiapPanel,
    finalEventHandledRef,
    contextEventsRef,
    recordInteraction,
    openUiResource,
    fetchAndOpenUiResource,
    closeUiResource,
    handleUiAction,
    toggleSidebar,
    toggleInteractionPanel,
    toggleDarkMode,
    toggleLeftSidebar,
    openConversationPanel,
    closeConversationPanel,
    handleResize,
    consoleDockStyle,
  }
}
