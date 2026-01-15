/**
 * useAlouWorkflow - Alou工作流Hook
 * 
 * 提供三种工作流模式：
 * 1. Interactive - 交互式模式，逐步引导 AI 完成任务
 * 2. Auto - 自动模式，自动分析并执行任务
 * 3. Parallel - 并行模式，拆分任务并行处理
 * 
 * 不依赖外部 CLI，直接使用现有的 AgentChat 工具调用
 */

import { useState, useCallback, useEffect, useRef } from 'react'
import { WorkflowService, WorkflowMode } from '@/services/workflowService'
import { generateInteractiveSystemPrompt, generateAutoSystemPrompt, generateParallelSystemPrompt } from '../utils/workflowPrompts'
import { analyzeTaskComplexity } from '../utils/workflowUtils'

export function useAlouWorkflow({
  sendMessage,
  handleToolCalls,
  appendMessage,
  scrollToBottom,
  options = {}
}) {
  const [isInitialized, setIsInitialized] = useState(false)
  const [currentMode, setCurrentMode] = useState(WorkflowMode.SMART)
  const [isProcessing, setIsProcessing] = useState(false)
  const [progress, setProgress] = useState(null)
  
  // 使用 ref 存储工作流服务，避免重复创建
  const workflowServiceRef = useRef(null)

  // 初始化工作流服务
  useEffect(() => {
    if (sendMessage && handleToolCalls && appendMessage && !workflowServiceRef.current) {
      workflowServiceRef.current = new WorkflowService({
        sendMessage,
        handleToolCalls,
        appendMessage,
        options: {
          verbose: false,
          ...options
        }
      })
      setIsInitialized(true)
      console.log('[useAlouWorkflow] 工作流服务初始化完成')
    }
  }, [sendMessage, handleToolCalls, appendMessage, options])

  /**
   * 切换工作流模式
   */
  const switchMode = useCallback((mode) => {
    setCurrentMode(mode)
    console.log('[useAlouWorkflow] 切换模式:', mode)
  }, [])

  /**
   * 获取当前系统提示词
   */
  const getSystemPrompt = useCallback(() => {
    switch (currentMode) {
      case WorkflowMode.INTERACTIVE:
        return generateInteractiveSystemPrompt()
      case WorkflowMode.AUTO:
        return generateAutoSystemPrompt()
      case WorkflowMode.PARALLEL:
        return generateParallelSystemPrompt()
      default:
        return '你是一个专业的AI助手'
    }
  }, [currentMode])

  /**
   * Interactive模式 - 交互式任务执行
   */
  const executeInteractive = useCallback(async (task, workflowOptions = {}) => {
    if (!workflowServiceRef.current || !isInitialized) {
      throw new Error('工作流服务未初始化')
    }

    setIsProcessing(true)
    setCurrentMode(WorkflowMode.INTERACTIVE)
    setProgress({ mode: WorkflowMode.INTERACTIVE, stage: 'starting' })

    try {
      const result = await workflowServiceRef.current.interactiveWorkflow(task, {
        ...workflowOptions,
        onProgress: (p) => {
          setProgress({ ...p, mode: WorkflowMode.INTERACTIVE })
          scrollToBottom?.()
        }
      })

      setProgress(null)
      return result
    } catch (error) {
      console.error('[useAlouWorkflow] Interactive模式执行失败:', error)
      setProgress({ error: error.message, mode: WorkflowMode.INTERACTIVE })
      throw error
    } finally {
      setIsProcessing(false)
    }
  }, [isInitialized, scrollToBottom])

  /**
   * Auto模式 - 自动任务执行
   */
  const executeAuto = useCallback(async (task, tools = [], workflowOptions = {}) => {
    if (!workflowServiceRef.current || !isInitialized) {
      throw new Error('工作流服务未初始化')
    }

    setIsProcessing(true)
    setCurrentMode(WorkflowMode.AUTO)
    setProgress({ mode: WorkflowMode.AUTO, stage: 'starting' })

    try {
      const result = await workflowServiceRef.current.autoWorkflow(task, {
        ...workflowOptions,
        tools,
        onProgress: (p) => {
          setProgress({ ...p, mode: WorkflowMode.AUTO })
          scrollToBottom?.()
        }
      })

      setProgress(null)
      return result
    } catch (error) {
      console.error('[useAlouWorkflow] Auto模式执行失败:', error)
      setProgress({ error: error.message, mode: WorkflowMode.AUTO })
      throw error
    } finally {
      setIsProcessing(false)
    }
  }, [isInitialized, scrollToBottom])

  /**
   * Parallel模式 - 并行任务执行
   */
  const executeParallel = useCallback(async (tasks, workflowOptions = {}) => {
    if (!workflowServiceRef.current || !isInitialized) {
      throw new Error('工作流服务未初始化')
    }

    setIsProcessing(true)
    setCurrentMode(WorkflowMode.PARALLEL)
    setProgress({ mode: WorkflowMode.PARALLEL, stage: 'starting' })

    try {
      const result = await workflowServiceRef.current.parallelWorkflow(tasks, {
        ...workflowOptions,
        onProgress: (p) => {
          setProgress({ ...p, mode: WorkflowMode.PARALLEL })
          scrollToBottom?.()
        }
      })

      setProgress(null)
      return result
    } catch (error) {
      console.error('[useAlouWorkflow] Parallel模式执行失败:', error)
      setProgress({ error: error.message, mode: WorkflowMode.PARALLEL })
      throw error
    } finally {
      setIsProcessing(false)
    }
  }, [isInitialized, scrollToBottom])

  /**
   * Smart模式 - 智能选择最佳模式
   */
  const executeSmart = useCallback(async (task, workflowOptions = {}) => {
    if (!workflowServiceRef.current || !isInitialized) {
      throw new Error('工作流服务未初始化')
    }

    setIsProcessing(true)

    // 分析任务复杂度
    const complexity = analyzeTaskComplexity(task)
    
    // 自动选择模式
    let selectedMode
    if (complexity.complexity > 0.8) {
      selectedMode = WorkflowMode.PARALLEL
    } else if (complexity.hasSubtasks || complexity.needsPlanning) {
      selectedMode = WorkflowMode.INTERACTIVE
    } else {
      selectedMode = WorkflowMode.AUTO
    }

    setCurrentMode(selectedMode)
    setProgress({ mode: selectedMode, stage: 'analyzing', message: `自动选择模式: ${selectedMode}` })
    console.log('[useAlouWorkflow] 智能选择模式:', selectedMode, complexity)

    try {
      let result
      if (selectedMode === WorkflowMode.PARALLEL) {
        // 将单个任务转换为数组
        result = await workflowServiceRef.current.parallelWorkflow([task], {
          ...workflowOptions,
          onProgress: (p) => {
            setProgress({ ...p, mode: WorkflowMode.PARALLEL })
            scrollToBottom?.()
          }
        })
      } else if (selectedMode === WorkflowMode.INTERACTIVE) {
        result = await workflowServiceRef.current.interactiveWorkflow(task, {
          ...workflowOptions,
          onProgress: (p) => {
            setProgress({ ...p, mode: WorkflowMode.INTERACTIVE })
            scrollToBottom?.()
          }
        })
      } else {
        result = await workflowServiceRef.current.autoWorkflow(task, {
          ...workflowOptions,
          onProgress: (p) => {
            setProgress({ ...p, mode: WorkflowMode.AUTO })
            scrollToBottom?.()
          }
        })
      }

      setProgress(null)
      return { ...result, selectedMode, complexity }
    } catch (error) {
      console.error('[useAlouWorkflow] Smart模式执行失败:', error)
      setProgress({ error: error.message, mode: selectedMode })
      throw error
    } finally {
      setIsProcessing(false)
    }
  }, [isInitialized, scrollToBottom])

  /**
   * 通用任务执行（根据当前模式执行）
   */
  const executeTask = useCallback(async (task, ...args) => {
    switch (currentMode) {
      case WorkflowMode.INTERACTIVE:
        return await executeInteractive(task, ...args)
      case WorkflowMode.AUTO:
        return await executeAuto(task, ...args)
      case WorkflowMode.PARALLEL:
        return await executeParallel(task, ...args)
      case WorkflowMode.SMART:
        return await executeSmart(task, ...args)
      default:
        return await executeSmart(task, ...args)
    }
  }, [currentMode, executeInteractive, executeAuto, executeParallel, executeSmart])

  /**
   * 取消当前任务
   */
  const cancelTask = useCallback(() => {
    if (workflowServiceRef.current) {
      workflowServiceRef.current.cancel()
      setProgress({ cancelled: true, message: '任务已取消' })
      setIsProcessing(false)
    }
  }, [])

  /**
   * 获取工作流服务状态
   */
  const getServiceStatus = useCallback(() => {
    return {
      isInitialized,
      isProcessing: workflowServiceRef.current?.isBusy() || false,
      currentMode
    }
  }, [isInitialized, currentMode])

  return {
    // 状态
    isInitialized,
    isProcessing,
    currentMode,
    progress,
    
    // 模式常量
    WorkflowMode,
    
    // 模式切换
    switchMode,
    getSystemPrompt,
    
    // 执行方法
    executeTask,
    executeInteractive,
    executeAuto,
    executeParallel,
    executeSmart,
    
    // 控制方法
    cancelTask,
    getServiceStatus,
  }
}
