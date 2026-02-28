#!/usr/bin/env python3
"""修复工具调用无限循环问题"""

# 读取文件
with open('/Users/apple/Downloads/alou/alou-desktop/src/components/AgentChat/hooks/useAsyncTaskPolling.ts', 'r') as f:
    content = f.read()

# 1. 修改函数签名（已经修改过）
# 2. 添加 MAX_RETRY_COUNT 和日志
old_line_155 = '    console.log(`[pollAsyncTask] 执行 ${toolCalls.length} 个工具调用`, toolCalls)'
new_lines = '''    const MAX_RETRY_COUNT = 3 // 最大重试次数
    console.log(`[pollAsyncTask] 执行 ${toolCalls.length} 个工具调用 (重试次数：${retryCount})`, toolCalls)'''

content = content.replace(old_line_155, new_lines)

# 3. 添加失败检测和重试限制
old_failure_check = '''      console.log(`[pollAsyncTask] 提交工具结果到任务 ${taskId}`)
      await apiClient.post(`/ai-task/${taskId}/tool-result`, {'''

new_failure_check = '''      // 如果所有工具都失败且超过最大重试次数，不再重试
      if (hasFailure && retryCount >= MAX_RETRY_COUNT) {
        console.error(`[pollAsyncTask] 达到最大重试次数 (${MAX_RETRY_COUNT})，停止重试`)
        toolResults.push({
          tool: 'system',
          success: false,
          error: `工具执行失败，已达到最大重试次数 ${MAX_RETRY_COUNT}。请检查参数或尝试其他方法。`,
          arguments: {},
          timestamp: Date.now(),
        })
      }

      console.log(`[pollAsyncTask] 提交工具结果到任务 ${taskId}`)
      await apiClient.post(`/ai-task/${taskId}/tool-result`, {'''

content = content.replace(old_failure_check, new_failure_check)

# 4. 修改 for 循环添加 hasFailure 检测
old_for_loop = '''      const toolResults: ToolResult[] = []
      for (const toolCall of toolCalls) {'''

new_for_loop = '''      const toolResults: ToolResult[] = []
      let hasFailure = false
      
      for (const toolCall of toolCalls) {'''

content = content.replace(old_for_loop, new_for_loop)

# 5. 修改成功结果推送，添加失败检测
old_success_push = '''          toolResults.push({
            tool: toolCall.tool,
            success: toolResponse.success || false,
            result: toolResponse.data || toolResponse.output || '工具执行完成',
            error: toolResponse.error,
            arguments: normalizedArgs,
            timestamp: Date.now(),
            tool_call_id: toolCall.id,
          })'''

new_success_push = '''          const result = {
            tool: toolCall.tool,
            success: toolResponse.success || false,
            result: toolResponse.data || toolResponse.output || '工具执行完成',
            error: toolResponse.error,
            arguments: normalizedArgs,
            timestamp: Date.now(),
            tool_call_id: toolCall.id,
          }
          
          if (!result.success) {
            hasFailure = true
            console.error(`[pollAsyncTask] 工具执行失败：${toolCall.tool}`, result.error)
          }
          
          toolResults.push(result)'''

content = content.replace(old_success_push, new_success_push)

# 6. 修改 catch 块
old_catch = '''        } catch (toolError) {
          console.error(`[pollAsyncTask] 工具执行失败：${toolCall.tool}`, toolError)
          toolResults.push({
            tool: toolCall.tool,
            success: false,
            error: (toolError as Error).message,
            arguments: toolCall.arguments,
            timestamp: Date.now(),
          })
        }'''

new_catch = '''        } catch (toolError) {
          hasFailure = true
          console.error(`[pollAsyncTask] 工具执行异常：${toolCall.tool}`, toolError)
          toolResults.push({
            tool: toolCall.tool,
            success: false,
            error: (toolError as Error).message,
            arguments: toolCall.arguments,
            timestamp: Date.now(),
          })
        }'''

content = content.replace(old_catch, new_catch)

# 写入文件
with open('/Users/apple/Downloads/alou/alou-desktop/src/components/AgentChat/hooks/useAsyncTaskPolling.ts', 'w') as f:
    f.write(content)

print("文件已成功修改")
