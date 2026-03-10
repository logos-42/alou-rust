// 这是 useAgentMessages.ts 的修改补丁
// 在文件开头添加导入
import { useAgentHookIntegration } from './useAgentHookIntegration'

// 在定义 sendMessage 之前，添加 Hook 初始化
// 找到现有的 hook 定义位置，添加:
/*
  const hookIntegration = useAgentHookIntegration({
    activeChannelId,
    sessionId,
    isLoading: isAgentLoading(activeChannelId),
  })
*/

// 然后修改 sendMessage 函数如下:

const sendMessage = useCallback(async () => {
  const text = currentMessage.trim()
  if (!text || isLoading) {
    return
  }

  // ── 检测 Agent 是否正在执行 ──
  const isExecuting = hookIntegration.isExecuting

  if (isExecuting && activeChannelId) {
    // Agent 正在执行，使用 Hook 注入指令
    console.log('[sendMessage] Agent 执行中，注入新指令:', text)
    
    try {
      // 判断优先级
      let priority: 'low' | 'medium' | 'high' | 'critical' = 'medium'
      if (text.includes('停止') || text.includes('取消') || text.includes('!') || text.includes('！')) {
        priority = 'critical'
      } else if (text.includes('请') || text.includes('尽快')) {
        priority = 'high'
      }

      // 注入指令
      const result = await hookIntegration.injectInstruction(text, priority)

      // 显示用户消息（作为指令）
      appendMessage({
        id: `user_instruction_${Date.now()}`,
        type: 'user',
        content: text,
        timestamp: Date.now(),
        source: 'user',
        metadata: {
          isInstruction: true,
          instructionId: result.instruction_id,
          priority,
        },
      })

      // 显示系统确认消息
      const priorityText = priority === 'critical' ? '🚨 紧急' : priority === 'high' ? '⚡ 高优先级' : '✅'
      appendMessage({
        id: `system_ack_${Date.now()}`,
        type: 'system',
        content: `${priorityText} 指令已发送给正在执行的 Agent: "${text}"`,
        timestamp: Date.now(),
        source: 'system',
      })

      // 如果是指令是"停止"或"取消"，显示额外提示
      if (priority === 'critical') {
        appendMessage({
          id: `system_stop_${Date.now()}`,
          type: 'system',
          content: '⏹️ Agent 将立即停止当前执行...',
          timestamp: Date.now(),
          source: 'system',
        })
      }
    } catch (error) {
      console.error('[sendMessage] 注入指令失败:', error)
      appendMessage({
        id: `system_err_${Date.now()}`,
        type: 'error',
        content: `❌ 指令发送失败：${(error as Error).message}`,
        timestamp: Date.now(),
        source: 'system',
      })
    }

    setCurrentMessage('')
    return
  }

  // ── 没有活动频道（未选中任何智能体）→ 进入对话式创建流程 ──
  if (!activeChannelId) {
    setCurrentMessage('')

    // 显示用户消息
    appendMessage({
      id: `user_${Date.now()}`,
      type: 'user',
      content: text,
      timestamp: Date.now(),
      source: 'user',
    })

    // 如果有自动创建回调，用 AI 解析描述 → 自动创建
    if (wrappedAutoCreateAgent) {
      appendMessage({
        id: `system_thinking_${Date.now()}`,
        type: 'system',
        content: '🤔 正在理解你的需求，准备创建智能体...',
        timestamp: Date.now(),
        source: 'system',
      })

      try {
        const agentInfo = await parseAgentCreationCommandWithAIDirect(text)
        console.log('[sendMessage] 解析的智能体信息:', agentInfo)

        appendMessage({
          id: `system_creating_${Date.now()}`,
          type: 'system',
          content: `🔄 正在创建智能体 **"${agentInfo.name}"**...`,
          timestamp: Date.now(),
          source: 'system',
        })

        await wrappedAutoCreateAgent(agentInfo as unknown as AgentInfo)

        appendMessage({
          id: `system_done_${Date.now()}`,
          type: 'assistant',
          content: `✅ 智能体 **"${agentInfo.name}"** 已创建！点击左侧频道开始对话。`,
          timestamp: Date.now(),
          source: 'system',
        })
      } catch (err) {
        appendMessage({
          id: `system_err_${Date.now()}`,
          type: 'assistant',
          content: `❌ 创建失败：${(err as Error).message || '未知错误'}`,
          timestamp: Date.now(),
          source: 'error',
        })
      }
    } else if (onCreateAgent) {
      appendMessage({
        id: `system_modal_${Date.now()}`,
        type: 'assistant',
        content: '📝 即将打开创建表单...',
        timestamp: Date.now(),
        source: 'system',
      })
      try {
        await onCreateAgent()
      } catch (e) {
        console.error('[sendMessage] 打开创建模态框失败:', e)
      }
    } else {
      appendMessage({
        id: `system_hint_${Date.now()}`,
        type: 'assistant',
        content: '👈 点击左侧 **"+"** 按钮创建你的第一个智能体，或者试试输入：\n\n> `创建一个擅长写代码的助手`',
        timestamp: Date.now(),
        source: 'system',
      })
    }

    return
  }

  // ── 正常流程：发送到选中的智能体 ──
  setCurrentMessage('')
  await sendMessageToAgent(activeChannelId, text, selectedAgent)
}, [
  activeChannelId,
  currentMessage,
  isLoading,
  selectedAgent,
  sendMessageToAgent,
  appendMessage,
  onAutoCreateAgent,
  onCreateAgent,
  parseAgentCreationCommandWithAIDirect,
  hookIntegration,  // 添加 Hook 依赖
])
