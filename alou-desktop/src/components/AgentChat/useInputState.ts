import { useState, useCallback, useRef, useEffect } from 'react'

/**
 * 独立的输入框状态 Hook
 * 
 * 目的：将输入框状态从消息状态中分离出来
 * 这样输入时不会触发消息列表重新渲染
 */
export const useInputState = (initialValue = '') => {
  const [value, setValue] = useState(initialValue)
  const valueRef = useRef(value)

  // 保持 ref 同步
  useEffect(() => {
    valueRef.current = value
  }, [value])

  const onChange = useCallback((newValue: string | ((prev: string) => string)) => {
    setValue(prev => typeof newValue === 'function' ? newValue(prev) : newValue)
  }, [])

  const clear = useCallback(() => {
    setValue('')
  }, [])

  return {
    value,
    onChange,
    clear,
    valueRef,
  }
}

export default useInputState
