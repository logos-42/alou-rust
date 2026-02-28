#!/usr/bin/env python3
"""添加重试计数跟踪"""

# 读取文件
with open('/Users/apple/Downloads/alou/alou-desktop/src/components/AgentChat/hooks/useAsyncTaskPolling.ts', 'r') as f:
    content = f.read()

# 1. 添加 toolRetryCounts
old_line_152 = '  const pollingStatusByAgent = useRef<Record<string, PollingStatus>>({})'
new_lines = '''  const pollingStatusByAgent = useRef<Record<string, PollingStatus>>({})
  const toolRetryCounts = useRef<Record<string, number>>({})'''

content = content.replace(old_line_152, new_lines)

# 2. 修改调用 executeToolCallsAndSubmitResults 添加重试计数
old_call = '              await executeToolCallsAndSubmitResults(taskId, toolCalls, agentId)'
new_call = '''              // 获取当前重试次数
              const currentRetryCount = toolRetryCounts.current[taskId] || 0
              console.log(`[pollAsyncTask] 工具调用重试次数：${currentRetryCount}`)
              await executeToolCallsAndSubmitResults(taskId, toolCalls, agentId, currentRetryCount)
              
              // 增加重试计数
              toolRetryCounts.current[taskId] = currentRetryCount + 1'''

content = content.replace(old_call, new_call)

# 3. 任务完成时清理重试计数
old_cleanup = '''          delete pollingTimeoutsByAgent.current[agentId]
          delete pollingStatusByAgent.current[agentId]'''
          
new_cleanup = '''          delete pollingTimeoutsByAgent.current[agentId]
          delete pollingStatusByAgent.current[agentId]
          delete toolRetryCounts.current[taskId]'''

content = content.replace(old_cleanup, new_cleanup)

# 写入文件
with open('/Users/apple/Downloads/alou/alou-desktop/src/components/AgentChat/hooks/useAsyncTaskPolling.ts', 'w') as f:
    f.write(content)

print("文件已成功修改")
