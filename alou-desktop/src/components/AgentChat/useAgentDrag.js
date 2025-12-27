import { useCallback, useRef, useState } from 'react'
import { NODE_BOUNDARY } from '@/hooks/useAgentChat'

/**
 * Hook for managing agent node drag and drop functionality
 */
export const useAgentDrag = ({ canvasRef, recordInteraction, openConversationPanel, onAvatarClick }) => {
  const dragStateRef = useRef({ 
    dragging: false, 
    offsetX: 0, 
    offsetY: 0, 
    moved: false,
    pressStartTime: 0,
    pressStartX: 0,
    pressStartY: 0,
    isLongPress: false
  })
  const agentPositionRef = useRef({ x: 0, y: 0 })
  const [agentPosition, setAgentPosition] = useState({ x: 0, y: 0 })
  const longPressTimerRef = useRef(null)

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
      console.log('[useAgentDrag] startDrag called', event)
      
      // 只处理主按钮（左键），pointer 事件中 button 可能不存在
      if (event.button !== undefined && event.button !== 0) {
        console.log('[useAgentDrag] Wrong button, rejecting', event.button)
        return
      }
      
      const canvasElement = canvasRef.current?.getElement?.()
      if (!canvasElement) {
        return
      }

      // 记录按下时间和位置
      const pressStartTime = Date.now()
      dragStateRef.current.pressStartTime = pressStartTime
      dragStateRef.current.pressStartX = event.clientX
      dragStateRef.current.pressStartY = event.clientY
      dragStateRef.current.isLongPress = false
      dragStateRef.current.moved = false
      dragStateRef.current.dragging = false

      // 设置长按定时器（150ms）
      longPressTimerRef.current = setTimeout(() => {
        dragStateRef.current.isLongPress = true
        // 如果长按后还没有移动，开始拖动（允许长按拖动）
        if (!dragStateRef.current.dragging && !dragStateRef.current.moved) {
          dragStateRef.current.dragging = true
          recordInteraction('agent_drag_start', { ...agentPositionRef.current })
        }
      }, 150)

      const rect = canvasElement.getBoundingClientRect()
      const centerX = rect.left + rect.width / 2
      const centerY = rect.top + rect.height / 2
      dragStateRef.current.offsetX = event.clientX - (centerX + agentPositionRef.current.x)
      dragStateRef.current.offsetY = event.clientY - (centerY + agentPositionRef.current.y)
    },
    [canvasRef, recordInteraction],
  )

  const onDrag = useCallback(
    (event) => {
      console.log('[useAgentDrag] onDrag called', { pressStartTime: dragStateRef.current.pressStartTime, clientX: event.clientX, clientY: event.clientY })
      
      // 如果没有按下，直接返回
      if (dragStateRef.current.pressStartTime === 0) {
        console.log('[useAgentDrag] onDrag rejected - pressStartTime is 0')
        return
      }

      const canvasElement = canvasRef.current?.getElement?.()
      if (!canvasElement) {
        return
      }

      // 计算移动距离
      const moveDistance = Math.sqrt(
        Math.pow(event.clientX - dragStateRef.current.pressStartX, 2) +
        Math.pow(event.clientY - dragStateRef.current.pressStartY, 2)
      )

      // 如果移动距离超过5px，清除长按定时器并开始拖动
      if (moveDistance > 5) {
        // 清除长按定时器
        if (longPressTimerRef.current) {
          clearTimeout(longPressTimerRef.current)
          longPressTimerRef.current = null
        }

        // 如果还没有开始拖动，现在开始
        if (!dragStateRef.current.dragging) {
          dragStateRef.current.dragging = true
          recordInteraction('agent_drag_start', { ...agentPositionRef.current })
        }
      }

      // 如果已经在拖动中（无论是通过移动还是长按触发），执行拖动
      if (dragStateRef.current.dragging) {
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
      }
    },
    [canvasRef, updateAgentPosition, recordInteraction],
  )

  const stopDrag = useCallback(
    (event) => {
      // 清除长按定时器
      if (longPressTimerRef.current) {
        clearTimeout(longPressTimerRef.current)
        longPressTimerRef.current = null
      }

      const wasDragging = dragStateRef.current.dragging
      const hadMoved = dragStateRef.current.moved
      const pressStartTime = dragStateRef.current.pressStartTime

      if (wasDragging) {
        dragStateRef.current.dragging = false
        event.target?.releasePointerCapture?.(event.pointerId)
        recordInteraction('agent_drag_end', { ...agentPositionRef.current })
      }

      // 如果没有拖动且没有移动，可能是点击
      if (!wasDragging && !hadMoved && pressStartTime > 0) {
        const pressDuration = Date.now() - pressStartTime
        // 如果按下时间小于300ms且没有移动，认为是点击
        if (pressDuration < 300) {
          onAvatarClick?.()
        }
      }

      // 重置状态
      dragStateRef.current.moved = false
      dragStateRef.current.isLongPress = false
      dragStateRef.current.pressStartTime = 0
    },
    [recordInteraction, onAvatarClick],
  )

  const handleGlobalPointerUp = useCallback(() => {
    dragStateRef.current.dragging = false
  }, [])

  const handleAgentActivate = useCallback(() => {
    // 这个函数保留用于向后兼容，但点击逻辑现在在 stopDrag 中处理
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

