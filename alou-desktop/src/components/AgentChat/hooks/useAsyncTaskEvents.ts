/**
 * 异步任务事件监听 Hook
 * 
 * 监听 Rust 后端通过 Tauri 事件发送的异步任务状态更新
 * 包括：async:task:created, async:task:progress, async:task:completed
 */

import { useEffect, useRef, useCallback } from 'react'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

// 异步任务状态
export type AsyncTaskStatus = 'processing' | 'completed' | 'failed' | 'timeout' | 'cancelled' | 'pending'

// 异步任务数据
export interface AsyncTask {
  taskId: string
  toolName: string
  provider: string
  status: AsyncTaskStatus
  elapsed: number
  pollCount?: number
  progress?: number
  result?: any
  error?: string
  payload?: any
  details?: any
}

// Hook 参数
interface UseAsyncTaskEventsParams {
  agentId: string
  channelId: string
  onTaskCreated?: (task: AsyncTask) => void
  onTaskProgress?: (task: AsyncTask) => void
  onTaskCompleted?: (task: AsyncTask) => void
}

// Hook 返回值
interface UseAsyncTaskEventsReturn {
  tasks: Map<string, AsyncTask>
  getTask: (taskId: string) => AsyncTask | undefined
  cancelTask: (taskId: string) => void
}

export const useAsyncTaskEvents = ({
  agentId,
  channelId,
  onTaskCreated,
  onTaskProgress,
  onTaskCompleted,
}: UseAsyncTaskEventsParams): UseAsyncTaskEventsReturn => {
  // 使用 ref 存储任务状态，避免不必要的重渲染
  const tasksRef = useRef<Map<string, AsyncTask>>(new Map())
  const unlistenersRef = useRef<UnlistenFn[]>([])

  // 获取任务
  const getTask = useCallback((taskId: string): AsyncTask | undefined => {
    return tasksRef.current.get(taskId)
  }, [])

  // 取消任务（仅前端标记，不影响后端轮询）
  const cancelTask = useCallback((taskId: string) => {
    const task = tasksRef.current.get(taskId)
    if (task) {
      task.status = 'cancelled'
      tasksRef.current.set(taskId, task)
    }
  }, [])

  useEffect(() => {
    // 设置事件监听
    const setupListeners = async () => {
      // 监听任务创建事件
      const unlistenCreated = await listen('async:task:created', (event) => {
        const payload = event.payload as {
          task_id: string
          tool_name: string
          provider: string
          status: string
          payload?: any
        }

        console.log('[AsyncTaskEvents] 任务创建:', payload)

        const task: AsyncTask = {
          taskId: payload.task_id,
          toolName: payload.tool_name,
          provider: payload.provider,
          status: payload.status as AsyncTaskStatus,
          elapsed: 0,
          progress: 0,
          payload: payload.payload,
        }

        tasksRef.current.set(payload.task_id, task)
        onTaskCreated?.(task)
      })

      // 监听任务进度事件
      const unlistenProgress = await listen('async:task:progress', (event) => {
        const payload = event.payload as {
          task_id: string
          status: string
          elapsed: number
          poll_count?: number
          details?: any
        }

        console.log('[AsyncTaskEvents] 任务进度:', payload)

        const existingTask = tasksRef.current.get(payload.task_id)
        if (existingTask) {
          const updatedTask: AsyncTask = {
            ...existingTask,
            status: payload.status as AsyncTaskStatus,
            elapsed: payload.elapsed,
            pollCount: payload.poll_count,
            progress: payload.details?.progress || existingTask.progress,
            details: payload.details,
          }
          tasksRef.current.set(payload.task_id, updatedTask)
          onTaskProgress?.(updatedTask)
        }
      })

      // 监听任务完成事件
      const unlistenCompleted = await listen('async:task:completed', (event) => {
        const payload = event.payload as {
          task_id: string
          status: string
          result?: any
          error?: string
          elapsed?: number
        }

        console.log('[AsyncTaskEvents] 任务完成:', payload)

        const existingTask = tasksRef.current.get(payload.task_id)
        if (existingTask) {
          const completedTask: AsyncTask = {
            ...existingTask,
            status: payload.status as AsyncTaskStatus,
            result: payload.result,
            error: payload.error,
            elapsed: payload.elapsed || existingTask.elapsed,
            progress: payload.status === 'completed' ? 100 : existingTask.progress,
          }
          tasksRef.current.set(payload.task_id, completedTask)
          onTaskCompleted?.(completedTask)
        }
      })

      // 存储取消监听函数
      unlistenersRef.current = [
        unlistenCreated,
        unlistenProgress,
        unlistenCompleted,
      ]
    }

    setupListeners().catch((error) => {
      console.error('[AsyncTaskEvents] 设置事件监听失败:', error)
    })

    // 清理函数
    return () => {
      unlistenersRef.current.forEach((unlisten) => {
        try {
          unlisten()
        } catch (error) {
          console.error('[AsyncTaskEvents] 取消监听失败:', error)
        }
      })
      unlistenersRef.current = []
    }
  }, [agentId, channelId, onTaskCreated, onTaskProgress, onTaskCompleted])

  return {
    tasks: tasksRef.current,
    getTask,
    cancelTask,
  }
}

export default useAsyncTaskEvents
