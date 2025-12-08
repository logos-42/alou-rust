import { useCallback, useRef, useState } from 'react'
import { NODE_BOUNDARY } from '@/hooks/useAgentChat'

/**
 * Hook for managing agent node drag and drop functionality
 */
export const useAgentDrag = ({ canvasRef, recordInteraction, openConversationPanel }) => {
  const dragStateRef = useRef({ dragging: false, offsetX: 0, offsetY: 0, moved: false })
  const agentPositionRef = useRef({ x: 0, y: 0 })
  const [agentPosition, setAgentPosition] = useState({ x: 0, y: 0 })

  const updateAgentPosition = useCallback((next) => {
    agentPositionRef.current = next
    setAgentPosition(next)
  }, [])

  const clampPosition = useCallback(() => {
    const canvasElement = canvasRef.current?.getElement?.()
    if (!canvasElement) return
    const rect = canvasElement.getBoundingClientRect()
    const limitX = Math.max(rect.width / 2 - NODE_BOUNDARY, 0)
    const limitY = Math.max(rect.height / 2 - NODE_BOUNDARY, 0)
    const { x, y } = agentPositionRef.current
    const clamped = {
      x: Math.min(Math.max(x, -limitX), limitX),
      y: Math.min(Math.max(y, -limitY), limitY),
    }
    updateAgentPosition(clamped)
  }, [canvasRef, updateAgentPosition])

  const startDrag = useCallback(
    (event) => {
      const canvasElement = canvasRef.current?.getElement?.()
      if (!canvasElement) return

      dragStateRef.current.dragging = true
      dragStateRef.current.moved = false
      const rect = canvasElement.getBoundingClientRect()
      const centerX = rect.left + rect.width / 2
      const centerY = rect.top + rect.height / 2
      dragStateRef.current.offsetX = event.clientX - (centerX + agentPositionRef.current.x)
      dragStateRef.current.offsetY = event.clientY - (centerY + agentPositionRef.current.y)
      event.target?.setPointerCapture?.(event.pointerId)
      recordInteraction('agent_drag_start', { ...agentPositionRef.current })
    },
    [canvasRef, recordInteraction],
  )

  const onDrag = useCallback(
    (event) => {
      if (!dragStateRef.current.dragging) return
      const canvasElement = canvasRef.current?.getElement?.()
      if (!canvasElement) return

      const rect = canvasElement.getBoundingClientRect()
      const centerX = rect.left + rect.width / 2
      const centerY = rect.top + rect.height / 2

      const nextX = event.clientX - centerX - dragStateRef.current.offsetX
      const nextY = event.clientY - centerY - dragStateRef.current.offsetY

      const limitX = Math.max(rect.width / 2 - NODE_BOUNDARY, 0)
      const limitY = Math.max(rect.height / 2 - NODE_BOUNDARY, 0)

      const clamped = {
        x: Math.min(Math.max(nextX, -limitX), limitX),
        y: Math.min(Math.max(nextY, -limitY), limitY),
      }
      if (
        Math.abs(clamped.x - agentPositionRef.current.x) > 1 ||
        Math.abs(clamped.y - agentPositionRef.current.y) > 1
      ) {
        dragStateRef.current.moved = true
      }
      updateAgentPosition(clamped)
    },
    [canvasRef, updateAgentPosition],
  )

  const stopDrag = useCallback(
    (event) => {
      if (dragStateRef.current.dragging) {
        dragStateRef.current.dragging = false
        event.target?.releasePointerCapture?.(event.pointerId)
        recordInteraction('agent_drag_end', { ...agentPositionRef.current })
      }
    },
    [recordInteraction],
  )

  const handleGlobalPointerUp = useCallback(() => {
    dragStateRef.current.dragging = false
  }, [])

  const handleAgentActivate = useCallback(() => {
    if (dragStateRef.current?.moved) {
      dragStateRef.current.moved = false
      return
    }
    dragStateRef.current.moved = false
    openConversationPanel()
  }, [openConversationPanel])

  return {
    agentPosition,
    agentPositionRef,
    dragStateRef,
    updateAgentPosition,
    clampPosition,
    startDrag,
    onDrag,
    stopDrag,
    handleGlobalPointerUp,
    handleAgentActivate,
  }
}

