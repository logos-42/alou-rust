import React, { useCallback, useEffect, useRef, useState } from 'react'
import './SplitView.css'

/**
 * SplitView - 可调整大小的分割视图组件
 * 支持水平分割（左右布局），中间有可拖拽的分割条
 */
const SplitView = ({
  left,
  right,
  defaultPosition = 50,
  minLeftWidth = 30,
  minRightWidth = 30,
  storageKey = 'split-view-position',
  onResize,
  className = '',
}) => {
  const containerRef = useRef(null)
  const splitterRef = useRef(null)
  const [position, setPosition] = useState(() => {
    // 从 localStorage 读取保存的位置
    if (typeof window !== 'undefined' && storageKey) {
      const saved = localStorage.getItem(storageKey)
      if (saved) {
        const parsed = parseFloat(saved)
        if (!isNaN(parsed) && parsed >= minLeftWidth && parsed <= 100 - minRightWidth) {
          return parsed
        }
      }
    }
    return defaultPosition
  })
  const [isDragging, setIsDragging] = useState(false)

  // 保存位置到 localStorage
  useEffect(() => {
    if (typeof window !== 'undefined' && storageKey) {
      localStorage.setItem(storageKey, position.toString())
    }
  }, [position, storageKey])

  // 处理分割条拖拽开始
  const handleSplitterMouseDown = useCallback((e) => {
    e.preventDefault()
    setIsDragging(true)
    if (splitterRef.current) {
      splitterRef.current.setPointerCapture(e.pointerId)
    }
  }, [])

  // 处理分割条拖拽
  const handleSplitterMouseMove = useCallback(
    (e) => {
      if (!isDragging || !containerRef.current) return

      const containerRect = containerRef.current.getBoundingClientRect()
      const newPosition = ((e.clientX - containerRect.left) / containerRect.width) * 100

      // 限制在最小和最大宽度之间
      const clamped = Math.max(minLeftWidth, Math.min(100 - minRightWidth, newPosition))
      setPosition(clamped)

      if (onResize) {
        onResize(clamped)
      }
    },
    [isDragging, minLeftWidth, minRightWidth, onResize],
  )

  // 处理分割条拖拽结束
  const handleSplitterMouseUp = useCallback(
    (e) => {
      if (isDragging) {
        setIsDragging(false)
        if (splitterRef.current) {
          splitterRef.current.releasePointerCapture(e.pointerId)
        }
      }
    },
    [isDragging],
  )

  // 全局鼠标事件监听
  useEffect(() => {
    if (isDragging) {
      const handleMouseMove = (e) => handleSplitterMouseMove(e)
      const handleMouseUp = (e) => handleSplitterMouseUp(e)

      window.addEventListener('pointermove', handleMouseMove)
      window.addEventListener('pointerup', handleMouseUp)

      return () => {
        window.removeEventListener('pointermove', handleMouseMove)
        window.removeEventListener('pointerup', handleMouseUp)
      }
    }
  }, [isDragging, handleSplitterMouseMove, handleSplitterMouseUp])

  return (
    <div ref={containerRef} className={`split-view ${className} ${isDragging ? 'dragging' : ''}`}>
      <div className="split-view-left" style={{ width: `${position}%` }}>
        {left}
      </div>
      <div
        ref={splitterRef}
        className="split-view-splitter"
        onPointerDown={handleSplitterMouseDown}
        role="separator"
        aria-orientation="vertical"
        aria-valuenow={position}
        aria-valuemin={minLeftWidth}
        aria-valuemax={100 - minRightWidth}
        tabIndex={0}
      >
        <div className="split-view-splitter-handle" />
      </div>
      <div className="split-view-right" style={{ width: `${100 - position}%` }}>
        {right}
      </div>
    </div>
  )
}

export default SplitView

