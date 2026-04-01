/**
 * Simple theme hook for standalone views (login, auth callback, etc.)
 * Reads from the same localStorage key as useAgentUI
 */
import { useState, useEffect, useCallback } from 'react'

export function useTheme() {
  const [isDarkMode, setIsDarkMode] = useState(() => {
    if (typeof localStorage === 'undefined') return true
    const saved = localStorage.getItem('alou-theme')
    if (saved) return saved === 'dark'
    if (typeof window !== 'undefined') {
      return window.matchMedia('(prefers-color-scheme: dark)').matches
    }
    return true
  })

  useEffect(() => {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('alou-theme', isDarkMode ? 'dark' : 'light')
    }
  }, [isDarkMode])

  const toggleTheme = useCallback(() => {
    setIsDarkMode((prev) => !prev)
  }, [])

  return { isDarkMode, toggleTheme }
}
