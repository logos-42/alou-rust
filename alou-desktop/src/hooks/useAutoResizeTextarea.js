import { useCallback, useEffect, useRef, useState } from 'react'

/**
 * 自动调整高度的 Textarea Hook
 * 根据内容自动调整 textarea 的高度
 * 
 * @param {Object} options
 * @param {number} options.minRows - 最小行数
 * @param {number} options.maxRows - 最大行数
 * @param {number} options.lineHeight - 行高（像素）
 * @param {number} options.padding - 内边距（像素）
 * @param {boolean} options.autoFocus - 是否自动聚焦
 */
export const useAutoResizeTextarea = (options = {}) => {
  const {
    minRows = 1,
    maxRows = 5,
    lineHeight = 20,
    padding = 20,
    autoFocus = false,
  } = options

  const textareaRef = useRef(null)
  const [rows, setRows] = useState(minRows)
  const [isFocused, setIsFocused] = useState(false)

  // 计算最小和最大高度
  const minHeight = minRows * lineHeight + padding
  const maxHeight = maxRows * lineHeight + padding

  /**
   * 调整 textarea 高度
   */
  const resize = useCallback(() => {
    const textarea = textareaRef.current
    if (!textarea) return

    // 重置高度以获取正确的 scrollHeight
    textarea.style.height = 'auto'
    
    // 计算新的高度
    const scrollHeight = textarea.scrollHeight
    let newHeight = Math.max(minHeight, Math.min(scrollHeight, maxHeight))
    
    textarea.style.height = `${newHeight}px`
    
    // 计算行数
    const newRows = Math.min(
      maxRows,
      Math.max(minRows, Math.ceil((scrollHeight - padding) / lineHeight))
    )
    setRows(newRows)
  }, [minHeight, maxHeight, padding, lineHeight, maxRows, minRows])

  /**
   * 处理输入事件
   */
  const handleInput = useCallback((e) => {
    resize()
    return e.target.value
  }, [resize])

  /**
   * 处理聚焦事件
   */
  const handleFocus = useCallback(() => {
    setIsFocused(true)
  }, [])

  /**
   * 处理失焦事件
   */
  const handleBlur = useCallback(() => {
    setIsFocused(false)
  }, [])

  /**
   * 聚焦到 textarea
   */
  const focus = useCallback(() => {
    textareaRef.current?.focus()
  }, [])

  /**
   * 清空内容
   */
  const clear = useCallback(() => {
    if (textareaRef.current) {
      textareaRef.current.value = ''
      resize()
    }
  }, [resize])

  /**
   * 设置内容
   */
  const setValue = useCallback((value) => {
    if (textareaRef.current) {
      textareaRef.current.value = value
      resize()
    }
  }, [resize])

  // 初始调整
  useEffect(() => {
    resize()
  }, [resize])

  // 自动聚焦
  useEffect(() => {
    if (autoFocus) {
      focus()
    }
  }, [autoFocus, focus])

  // 监听窗口大小变化
  useEffect(() => {
    const handleResize = () => {
      resize()
    }

    window.addEventListener('resize', handleResize)
    return () => window.removeEventListener('resize', handleResize)
  }, [resize])

  return {
    textareaRef,
    rows,
    isFocused,
    minHeight,
    maxHeight,
    handleInput,
    handleFocus,
    handleBlur,
    resize,
    focus,
    clear,
    setValue,
  }
}

export default useAutoResizeTextarea
