import { useCallback, useRef } from 'react'
import apiClient from '@/services/api'

/**
 * Hook for managing asynchronous task polling
 */
export const useAsyncTaskPolling = ({
  appendMessage,
  scrollToBottom,
  setAgentLoading,
  setMessagesByChannel,
}) => {
  // 按智能体存储轮询定时器：Map<agentId, intervalId>
  const pollingIntervalsByAgent = useRef({})

  // 按智能体存储轮询状态：Map<agentId, { taskId, messageId, startTime }>
  const pollingStatusByAgent = useRef({})

  /**
   * 轮询异步任务状态
   */
  const pollAsyncTask = useCallback((taskId, progressMessageId, agentId, options = {}) => {
    const {
      interval = 2000, // 2秒轮询一次
      timeout = 120000, // 120秒超时
    } = options

    // 清理该智能体之前的轮询
    const existingInterval = pollingIntervalsByAgent.current[agentId]
    if (existingInterval) {
      clearInterval(existingInterval)
      delete pollingIntervalsByAgent.current[agentId]
    }

    const startTime = Date.now()
    let pollCount = 0

    // 记录轮询状态
    pollingStatusByAgent.current[agentId] = {
      taskId,
      messageId: progressMessageId,
      startTime,
    }

    console.log(`[pollAsyncTask] 开始轮询任务 ${taskId}，智能体: ${agentId}`)

    const intervalId = setInterval(async () => {
      try {
        // 检查超时
        if (Date.now() - startTime > timeout) {
          throw new Error('任务执行超时')
        }

        // 查询任务状态
        const response = await apiClient.get(`/api/tasks/${taskId}`)
        const result = response.data

        if (!result.success) {
          throw new Error(result.error || '获取任务状态失败')
        }

        const { status, progress = 0, current_step, error, result: taskResult } = result
        const taskResponse = taskResult?.response

        console.log(`[pollAsyncTask] 第 ${++pollCount} 次轮询 - 状态: ${status}, 进度: ${progress}%`)

        // 更新进度消息
        setMessagesByChannel((prev) => {
          const channelMessages = prev[agentId] || []
          const messageIndex = channelMessages.findIndex(msg => msg.id === progressMessageId)

          if (messageIndex !== -1) {
            const updatedMessages = [...channelMessages]
            const progressMessage = { ...updatedMessages[messageIndex] }

            // 更新进度信息
            progressMessage.progress = progress
            progressMessage.status = status
            progressMessage.currentStep = current_step

            // 根据状态更新内容
            let content = '🔄 任务正在执行中'
            if (current_step) {
              content += `: ${current_step}`
            }
            if (progress > 0) {
              content += ` (${Math.round(progress * 100)}%)`
            }
            progressMessage.content = content

            updatedMessages[messageIndex] = progressMessage
            return {
              ...prev,
              [agentId]: updatedMessages,
            }
          }

          return prev
        })

        // 根据状态处理
        switch (status) {
          case 'pending':
          case 'processing':
          case 'running':
            // 继续轮询
            break

          case 'completed':
            // 任务完成，清理轮询
            clearInterval(intervalId)
            delete pollingIntervalsByAgent.current[agentId]
            delete pollingStatusByAgent.current[agentId]

            console.log(`[pollAsyncTask] 任务完成: ${taskId}`)

            // 更新消息为完成状态
            setMessagesByChannel((prev) => {
              const channelMessages = prev[agentId] || []
              const messageIndex = channelMessages.findIndex(msg => msg.id === progressMessageId)

              if (messageIndex !== -1) {
                const updatedMessages = [...channelMessages]
                const completedMessage = { ...updatedMessages[messageIndex] }

                completedMessage.content = '✅ 任务执行完成'
                completedMessage.progress = 1
                completedMessage.status = 'completed'

                updatedMessages[messageIndex] = completedMessage
                return {
                  ...prev,
                  [agentId]: updatedMessages,
                }
              }

              return prev
            })

            // 如果有响应内容，添加新消息
            if (taskResponse) {
              const responseMessage = {
                id: `assistant_${Date.now()}_${agentId}`,
                type: 'assistant',
                content: String(taskResponse),
                timestamp: Date.now(),
                source: 'alou-edge',
                agentId: agentId,
              }
              appendMessage(responseMessage, agentId)
            }

            setAgentLoading(agentId, false)
            scrollToBottom()
            break

          case 'failed':
            // 任务失败，清理轮询
            clearInterval(intervalId)
            delete pollingIntervalsByAgent.current[agentId]
            delete pollingStatusByAgent.current[agentId]

            console.error(`[pollAsyncTask] 任务失败: ${taskId}, 错误: ${error}`)

            // 更新消息为失败状态
            setMessagesByChannel((prev) => {
              const channelMessages = prev[agentId] || []
              const messageIndex = channelMessages.findIndex(msg => msg.id === progressMessageId)

              if (messageIndex !== -1) {
                const updatedMessages = [...channelMessages]
                const failedMessage = { ...updatedMessages[messageIndex] }

                failedMessage.content = `❌ 任务执行失败: ${error || '未知错误'}`
                failedMessage.progress = 1
                failedMessage.status = 'failed'
                failedMessage.error = error

                updatedMessages[messageIndex] = failedMessage
                return {
                  ...prev,
                  [agentId]: updatedMessages,
                }
              }

              return prev
            })

            setAgentLoading(agentId, false)
            scrollToBottom()
            break

          default:
            console.warn(`[pollAsyncTask] 未知任务状态: ${status}`)
        }
      } catch (error) {
        console.error(`[pollAsyncTask] 轮询失败:`, error)

        // 清理轮询
        clearInterval(intervalId)
        delete pollingIntervalsByAgent.current[agentId]
        delete pollingStatusByAgent.current[agentId]

        // 更新消息为错误状态
        setMessagesByChannel((prev) => {
          const channelMessages = prev[agentId] || []
          const messageIndex = channelMessages.findIndex(msg => msg.id === progressMessageId)

          if (messageIndex !== -1) {
            const updatedMessages = [...channelMessages]
            const errorMessage = { ...updatedMessages[messageIndex] }

            errorMessage.content = `❌ 任务执行出错: ${error.message}`
            errorMessage.progress = 1
            errorMessage.status = 'error'
            errorMessage.error = error.message

            updatedMessages[messageIndex] = errorMessage
            return {
              ...prev,
              [agentId]: updatedMessages,
            }
          }

          return prev
        })

        setAgentLoading(agentId, false)
        scrollToBottom()
      }
    }, interval)

    // 保存定时器ID
    pollingIntervalsByAgent.current[agentId] = intervalId

    // 立即执行第一次查询
    setTimeout(() => {
      // 触发第一次轮询
      intervalId._immediate = true
    }, 100)
  }, [appendMessage, scrollToBottom, setAgentLoading, setMessagesByChannel])

  /**
   * 终止指定智能体的轮询
   */
  const cancelPolling = useCallback((agentId) => {
    const intervalId = pollingIntervalsByAgent.current[agentId]
    if (intervalId) {
      console.log('[useAsyncTaskPolling] 终止智能体轮询:', agentId)
      clearInterval(intervalId)
      delete pollingIntervalsByAgent.current[agentId]
      delete pollingStatusByAgent.current[agentId]
    }
  }, [])

  return {
    pollAsyncTask,
    cancelPolling,
  }
}