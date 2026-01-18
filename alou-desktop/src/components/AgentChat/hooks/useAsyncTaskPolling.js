import { useCallback, useRef } from 'react'
import apiClient from '@/services/api'
import LoadingIcon from '@/assets/加载0.2.png'

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
   * 执行工具调用并提交结果
   */
  const executeToolCallsAndSubmitResults = useCallback(async (taskId, toolCalls, agentId) => {
    console.log(`[pollAsyncTask] 执行 ${toolCalls.length} 个工具调用`, toolCalls)

    try {
      // 执行所有工具调用
      const toolResults = []
      for (const toolCall of toolCalls) {
        try {
          console.log(`[pollAsyncTask] 执行工具: ${toolCall.tool}`, toolCall.arguments)

          // 调用实际的工具执行API
          const toolResponse = await apiClient.post('/mcp/execute-tool', {
            tool_id: toolCall.tool,
            args: toolCall.arguments,
            session_id: `task_${taskId}`,
            timeout_seconds: 30,
          })

          const toolResult = {
            tool: toolCall.tool,
            success: toolResponse.data.success || false,
            result: toolResponse.data.data || toolResponse.data.result || '工具执行完成',
            error: toolResponse.data.error,
            arguments: toolCall.arguments,
            timestamp: Date.now(),
            tool_call_id: toolCall.id, // 使用正确的tool_call_id
          }

          toolResults.push(toolResult)
        } catch (toolError) {
          console.error(`[pollAsyncTask] 工具执行失败: ${toolCall.tool}`, toolError)
          toolResults.push({
            tool: toolCall.tool,
            success: false,
            error: toolError.message,
            arguments: toolCall.arguments,
            timestamp: Date.now(),
          })
        }
      }

      // 提交工具结果
      console.log(`[pollAsyncTask] 提交工具结果到任务 ${taskId}`)
      await apiClient.post(`/ai-task/${taskId}/tool-result`, {
        results: toolResults,
        timestamp: Date.now(),
      })

      console.log(`[pollAsyncTask] 工具结果已提交`)
    } catch (error) {
      console.error(`[pollAsyncTask] 提交工具结果失败:`, error)
      throw error
    }
  }, [])

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
        const response = await apiClient.get(`/ai-task/${taskId}/status`)
        const result = response.data

        // TaskStatusResponse 没有 success 字段，直接使用 status
        const { status, progress = 0, current_step, error, result: taskResult } = result
        const taskResponse = taskResult?.response

        console.log(`[pollAsyncTask] 第 ${++pollCount} 次轮询 - 状态: ${status}, 进度: ${progress}%, 步骤: ${current_step}, 错误: ${error}`)

        // 检查是否有待处理的工具调用（需要从后端获取）
        if (status === 'processing') {
           try {
             // 尝试获取待处理的工具调用
             const toolCallsResponse = await apiClient.get(`/ai-task/${taskId}/pending-tools`)
             const toolCalls = toolCallsResponse.data?.toolCalls || []

             if (toolCalls.length > 0) {
               console.log(`[pollAsyncTask] 发现 ${toolCalls.length} 个待处理工具调用:`, toolCalls)

               // 执行工具调用并提交结果
               await executeToolCallsAndSubmitResults(taskId, toolCalls, agentId)

               // 工具结果已提交，继续轮询等待AI继续处理
               return
             }
           } catch (toolError) {
             // 如果获取工具调用失败，继续正常轮询
             console.log(`[pollAsyncTask] 获取待处理工具调用失败，继续轮询:`, toolError.message)
           }
         }

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
            let content = `<img src="${LoadingIcon}" alt="加载中" class="loading-icon" />`
            if (current_step) {
              content += ` ${current_step}`
            } else {
              content += ' 任务正在执行中'
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
        console.log(`[POLL_DEBUG] === 开始处理状态 ===`)
        console.log(`[POLL_DEBUG] 任务ID: ${taskId}`)
        console.log(`[POLL_DEBUG] 状态: "${status}" (类型: ${typeof status})`)
        console.log(`[POLL_DEBUG] 进度: ${progress}`)
        console.log(`[POLL_DEBUG] 轮询计数: ${pollCount}`)
        console.log(`[POLL_DEBUG] AgentID: ${agentId}`)

        switch (status) {
           case 'queued':
           case 'pending':
           case 'processing':
           case 'running':
             // 继续轮询
             console.log(`[POLL_DEBUG] 状态 "${status}" 继续轮询`)
             
             // 更新加载消息（如果存在）
             if (progressMessageId) {
               setMessagesByChannel((prev) => {
                 const channelMessages = prev[agentId] || []
                 const messageIndex = channelMessages.findIndex(msg => msg.id === progressMessageId)
                 
                 if (messageIndex !== -1 && current_step) {
                   const updatedMessages = [...channelMessages]
                   const progressMessage = { ...updatedMessages[messageIndex] }
                   
                   // 更新加载提示
                   progressMessage.content = `🔄 ${current_step}${progress > 0 ? ` (${Math.round(progress * 100)}%)` : ''}`
                   
                   updatedMessages[messageIndex] = progressMessage
                   return {
                     ...prev,
                     [agentId]: updatedMessages,
                   }
                 }
                 
                 return prev
               })
             }
             break

           case 'completed':
             console.log(`[POLL_DEBUG] 🎯 检测到完成状态，执行清理逻辑`)

             // 任务完成，清理轮询
             console.log(`[POLL_DEBUG] 清理定时器...`)
             clearInterval(intervalId)

             console.log(`[POLL_DEBUG] 删除轮询状态...`)
             delete pollingIntervalsByAgent.current[agentId]
             delete pollingStatusByAgent.current[agentId]

             console.log(`[POLL_DEBUG] ✅ 任务完成: ${taskId}`)

            // 如果没有进度消息ID，直接添加AI响应消息
            if (progressMessageId && taskResponse) {
              setMessagesByChannel((prev) => {
                const channelMessages = prev[agentId] || []
                const messageIndex = channelMessages.findIndex(msg => msg.id === progressMessageId)

                if (messageIndex !== -1) {
                  const updatedMessages = [...channelMessages]
                  const completedMessage = { ...updatedMessages[messageIndex] }

                  // 直接用AI响应替换进度消息内容
                  completedMessage.content = String(taskResponse)
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
            } else if (taskResponse) {
              // 没有进度消息，直接添加新消息
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
            console.log(`[POLL_DEBUG] ❌ 检测到失败状态，执行清理逻辑`)
            // 任务失败，清理轮询
            console.log(`[POLL_DEBUG] 清理定时器...`)
            clearInterval(intervalId)

            console.log(`[POLL_DEBUG] 删除轮询状态...`)
            delete pollingIntervalsByAgent.current[agentId]
            delete pollingStatusByAgent.current[agentId]

            console.error(`[pollAsyncTask] 任务失败: ${taskId}, 错误: ${error}`)
            console.log(`[POLL_DEBUG] ❌ 任务失败: ${taskId}`)

            // 如果没有进度消息ID，直接添加失败消息
            if (progressMessageId) {
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
            } else {
              // 没有进度消息，直接添加错误消息
              appendMessage({
                id: `error_${Date.now()}_${agentId}`,
                type: 'assistant',
                content: `❌ 任务执行失败: ${error || '未知错误'}`,
                timestamp: Date.now(),
                source: 'error',
                agentId: agentId,
              }, agentId)
            }

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

        // 如果没有进度消息ID，直接添加错误消息
        if (progressMessageId) {
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
        } else {
          // 没有进度消息，直接添加错误消息
          appendMessage({
            id: `error_${Date.now()}_${agentId}`,
            type: 'assistant',
            content: `❌ 任务执行出错: ${error.message}`,
            timestamp: Date.now(),
            source: 'error',
            agentId: agentId,
          }, agentId)
        }

        setAgentLoading(agentId, false)
        scrollToBottom()
      }
    }, interval)

    // 保存定时器ID
    pollingIntervalsByAgent.current[agentId] = intervalId

    // 立即执行第一次查询（通过缩短第一次延迟来实现）
    const immediatePoll = setTimeout(async () => {
      try {
        // 手动执行第一次轮询
        const response = await apiClient.get(`/ai-task/${taskId}/status`)
        const result = response.data

        if (result.success) {
          const { status, progress = 0, current_step, error, result: taskResult } = result
          console.log(`[pollAsyncTask] 首次轮询 - 状态: ${status}, 进度: ${progress}%`)
        }
      } catch (error) {
        console.error(`[pollAsyncTask] 首次轮询失败:`, error)
      }
    }, 100)
    
    // 保存immediate timeout引用，避免重复创建
    pollingIntervalsByAgent.current[`${agentId}_immediate`] = immediatePoll
  }, [appendMessage, scrollToBottom, setAgentLoading, setMessagesByChannel, executeToolCallsAndSubmitResults])

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