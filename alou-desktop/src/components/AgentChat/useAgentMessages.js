import { useCallback, useState, useMemo, useEffect, useRef } from 'react'
import apiClient from '@/services/api'
import agentService from '@/services/agentService'
import useAgentStore from '@/stores/agentStore'

/**
 * Hook for managing messages and conversation
 * 按频道分开存储消息，支持 IPFS 持久化
 * 支持多智能体独立执行空间
 */
export const useAgentMessages = ({
  sessionId,
  setSessionId,
  activeChain,
  activeChannelId,
  selectedAgent,
  isSessionReady,
  createSession,
  setSessionReady,
  recordInteraction,
  handleToolCalls,
  conversationOverlayRef,
  consoleDockRef,
  contextEventsRef,
  currentMode = 'agent', // 当前模式：'agent' 或 'alou'
  onRateLimitExceeded, // 回调函数：当遇到 429 错误时调用
  onCreateAgent, // 新增：创建智能体的回调函数
  onAutoCreateAgent, // 新增：自动创建智能体的回调函数
}) => {
  // 按频道存储消息：Map<channelId, Message[]>
  const [messagesByChannel, setMessagesByChannel] = useState({})
  const [currentMessage, setCurrentMessage] = useState('')
  // 全局 loading 状态（向后兼容）
  const [isLoading, setIsLoading] = useState(false)
  // 按智能体存储 loading 状态：Map<channelId, boolean>
  const [loadingByAgent, setLoadingByAgent] = useState({})
  // 按智能体存储 session：Map<channelId, sessionId>
  const [sessionsByAgent, setSessionsByAgent] = useState({})
  const [isConversationVisible, setConversationVisible] = useState(false)
  
  // 跟踪已保存过的消息数量，避免重复保存
  const savedMessageCountRef = useRef({})
  
  // 按智能体存储 AbortController：Map<channelId, AbortController>
  const abortControllersByAgent = useRef({})
  
  // 获取 agentStore 方法
  const updateAgent = useAgentStore((state) => state.updateAgent)
  
  // 获取指定智能体的 loading 状态
  const isAgentLoading = useCallback((agentId) => {
    return loadingByAgent[agentId] || false
  }, [loadingByAgent])
  
  // 设置指定智能体的 loading 状态
  const setAgentLoading = useCallback((agentId, loading) => {
    setLoadingByAgent(prev => ({ ...prev, [agentId]: loading }))
    // 同时更新全局 loading（如果是当前活动智能体）
    if (agentId === activeChannelId) {
      setIsLoading(loading)
    }
  }, [activeChannelId])

  // 当前频道的消息
  const messages = useMemo(() => {
    return messagesByChannel[activeChannelId] || []
  }, [messagesByChannel, activeChannelId])

  // 添加消息到指定频道
  const appendMessage = useCallback(
    (message, channelId = activeChannelId) => {
      if (!channelId) {
        console.warn('[useAgentMessages] 无法添加消息：没有活动频道')
        return
      }
      
      setMessagesByChannel((prev) => {
        const channelMessages = prev[channelId] || []
        return {
          ...prev,
          [channelId]: [...channelMessages, message],
        }
      })
      
      if (!isConversationVisible) {
        setConversationVisible(true)
      }
    },
    [activeChannelId, isConversationVisible],
  )

  // 设置指定频道的所有消息（用于从 IPFS 加载）
  const setMessagesForChannel = useCallback((channelId, messages) => {
    if (!channelId) return
    
    setMessagesByChannel((prev) => ({
      ...prev,
      [channelId]: messages,
    }))
    
    // 更新已保存计数
    savedMessageCountRef.current[channelId] = messages.length
  }, [])

  // 清空指定频道的消息
  const clearMessagesForChannel = useCallback((channelId) => {
    if (!channelId) return
    
    setMessagesByChannel((prev) => {
      const newMap = { ...prev }
      delete newMap[channelId]
      return newMap
    })
    
    // 重置保存计数
    delete savedMessageCountRef.current[channelId]
  }, [])

  const scrollToBottom = useCallback(() => {
    requestAnimationFrame(() => {
      conversationOverlayRef.current?.scrollToBottom?.()
    })
  }, [conversationOverlayRef])

  // 保存消息到 IPFS
  const saveMessagesToIpfs = useCallback(async (channelId, agentId) => {
    const channelMessages = messagesByChannel[channelId] || []
    const savedCount = savedMessageCountRef.current[channelId] || 0
    
    // 如果没有新消息，跳过保存
    if (channelMessages.length === 0 || channelMessages.length <= savedCount) {
      return null
    }
    
    try {
      console.log(`[useAgentMessages] 保存 ${channelMessages.length} 条消息到 IPFS，频道: ${channelId}`)
      
      const cid = await agentService.uploadMessagesToIpfs(channelMessages, agentId)
      
      // 更新 agentStore 中的 messages_cid
      if (cid && agentId) {
        updateAgent(agentId, { messages_cid: cid })
        console.log(`[useAgentMessages] 消息已保存到 IPFS，CID: ${cid}`)
      }
      
      // 更新已保存计数
      savedMessageCountRef.current[channelId] = channelMessages.length
      
      return cid
    } catch (error) {
      console.error('[useAgentMessages] 保存消息到 IPFS 失败:', error)
      return null
    }
  }, [messagesByChannel, updateAgent])

  // 从 IPFS 加载消息
  const loadMessagesFromIpfs = useCallback(async (channelId, messagesCid) => {
    if (!channelId || !messagesCid) return false
    
    try {
      console.log(`[useAgentMessages] 从 IPFS 加载消息，CID: ${messagesCid}`)
      
      const data = await agentService.loadMessagesFromIpfs(messagesCid)
      
      if (data && data.messages && Array.isArray(data.messages)) {
        setMessagesForChannel(channelId, data.messages)
        console.log(`[useAgentMessages] 已加载 ${data.messages.length} 条消息`)
        return true
      }
      
      return false
    } catch (error) {
      console.error('[useAgentMessages] 从 IPFS 加载消息失败:', error)
      return false
    }
  }, [setMessagesForChannel])

  /**
   * 发送消息到指定智能体
   * 支持多智能体独立执行空间
   * 自动检测是否需要集群行动
   */
  const sendMessageToAgent = useCallback(async (targetAgentId, text, targetAgent = null) => {
    if (!text?.trim() || !targetAgentId) {
      console.warn('[useAgentMessages] 无法发送消息：缺少文本或目标智能体')
      return
    }

    // 检查该智能体是否正在执行
    if (loadingByAgent[targetAgentId]) {
      console.log('[useAgentMessages] 智能体正在执行中，跳过:', targetAgentId)
      return
    }

    const userMessage = {
      id: `user_${Date.now()}_${targetAgentId}`,
      type: 'user',
      content: text.trim(),
      timestamp: Date.now(),
    }

    appendMessage(userMessage, targetAgentId)
    setAgentLoading(targetAgentId, true)
    recordInteraction('user_message', { content: text, agentId: targetAgentId })
    scrollToBottom()

    const contextSnapshot = contextEventsRef.current.splice(0, contextEventsRef.current.length)

    // 直接调用后端 API 进行单智能体聊天
    // 注意：多智能体协作（集群行动）只在用户主动创建邀请时创建（见 useAgentInvite.js）
    try {
      const walletAddress =
        typeof window !== 'undefined' ? localStorage.getItem('wallet_address') : null

      // 获取或创建该智能体的 session
      let agentSessionId = sessionsByAgent[targetAgentId] || sessionId
      
      // 如果没有有效的 session，创建一个新的
      if (!agentSessionId || agentSessionId.startsWith('frontend_')) {
        try {
          console.log('[useAgentMessages] 为智能体创建新会话:', targetAgentId)
          await createSession()
          setSessionReady(true)
          await new Promise((resolve) => setTimeout(resolve, 100))
          agentSessionId = sessionId
        } catch (sessionErr) {
          console.error('[useAgentMessages] 会话创建失败:', sessionErr)
          appendMessage({
            id: `error_${Date.now()}`,
            type: 'assistant',
            content: '❌ 无法连接到后端服务，请检查网络连接或稍后重试。',
            timestamp: Date.now(),
            source: 'error',
          }, targetAgentId)
          setAgentLoading(targetAgentId, false)
          return
        }
      }

      // 保存该智能体的 session
      setSessionsByAgent(prev => ({ ...prev, [targetAgentId]: agentSessionId }))

      console.log('[useAgentMessages] 发送消息，sessionId:', agentSessionId, '智能体:', targetAgentId)

      // 创建 AbortController 用于终止请求
      const abortController = new AbortController()
      abortControllersByAgent.current[targetAgentId] = abortController

      const agentInfo = targetAgent || selectedAgent
      
      const data = await apiClient
        .post('/agent/chat', {
          session_id: agentSessionId,
          message: text.trim(),
          wallet_address: walletAddress || undefined,
          chain: activeChain || undefined,
          context_events: contextSnapshot,
          agent_id: targetAgentId,
          agent_info: agentInfo ? {
            name: agentInfo.display_name || agentInfo.name,
            role_description: agentInfo.role_description,
            custom_prompt: agentInfo.customPrompt,
            mode: currentMode, // 传递当前模式
          } : undefined,
        }, {
          signal: abortController.signal, // 添加 abort signal 用于终止请求
        })
        .then((response) => response.data)

      if (data.tool_calls) {
        await handleToolCalls(data.tool_calls)
      }

      const assistantMessage = {
        id: `assistant_${Date.now()}_${targetAgentId}`,
        type: 'assistant',
        content: data.content || data.response || '收到响应',
        timestamp: data.timestamp || Date.now(),
        source: data.source || 'alou-edge',
        agentId: targetAgentId,
      }
      appendMessage(assistantMessage, targetAgentId)

      if (data.session_id) {
        setSessionsByAgent(prev => ({ ...prev, [targetAgentId]: data.session_id }))
        if (targetAgentId === activeChannelId) {
          setSessionId(data.session_id)
        }
      }
    } catch (error) {
      // 如果是用户主动取消，不显示错误消息
      if (error.name === 'AbortError' || error.name === 'CanceledError') {
        console.log('[useAgentMessages] 用户取消了智能体执行:', targetAgentId)
        appendMessage({
          id: `cancel_${Date.now()}`,
          type: 'assistant',
          content: '⏹️ 执行已终止',
          timestamp: Date.now(),
          source: 'cancel',
        }, targetAgentId)
        return
      }
      
      const errorMessage = error instanceof Error ? error.message : '未知错误'
      const statusCode = error?.response?.status
      
      // 尝试从响应中提取详细错误信息
      let detailedError = errorMessage
      if (error?.response?.data) {
        const errorData = error.response.data
        if (errorData.error) {
          detailedError = errorData.error
        } else if (typeof errorData === 'string') {
          detailedError = errorData
        }
      }
      
      console.error('[useAgentMessages] 后端 API 错误:', {
        status: statusCode,
        message: errorMessage,
        detailedError,
        response: error?.response?.data,
      })
      
      // 处理 429 错误（限额超限）
      if (statusCode === 429) {
        const errorData = error?.response?.data
        const remainingRequests = errorData?.remaining_requests ?? 0
        const resetTime = errorData?.reset_time ?? null
        
        // 调用回调函数显示弹窗
        if (onRateLimitExceeded) {
          onRateLimitExceeded({
            remainingRequests,
            resetTime,
          })
        }
        
        // 不添加错误消息到对话中，因为已经有弹窗了
        return
      }
      
      let friendlyMessage = `❌ 抱歉，发生了错误：${detailedError}`
      if (statusCode === 404) {
        friendlyMessage = '❌ 会话已过期，请刷新页面重试。'
        setSessionReady(false)
      } else if (statusCode === 500) {
        friendlyMessage = `❌ 服务器内部错误：${detailedError}\n\n请检查后端服务是否正常运行，或查看控制台获取更多信息。`
      } else if (!error?.response) {
        friendlyMessage = '❌ 无法连接到服务器，请检查网络连接。'
      }
      
      appendMessage({
        id: `error_${Date.now()}`,
        type: 'assistant',
        content: friendlyMessage,
        timestamp: Date.now(),
        source: 'error',
      }, targetAgentId)
    } finally {
      // 清理 AbortController
      delete abortControllersByAgent.current[targetAgentId]
      setAgentLoading(targetAgentId, false)
      scrollToBottom()
      consoleDockRef.current?.adjustInputHeight?.()
    }
  }, [
    activeChain,
    activeChannelId,
    appendMessage,
    consoleDockRef,
    contextEventsRef,
    createSession,
    handleToolCalls,
    loadingByAgent,
    onRateLimitExceeded,
    recordInteraction,
    scrollToBottom,
    selectedAgent,
    sessionId,
    sessionsByAgent,
    setAgentLoading,
    setSessionId,
    setSessionReady,
  ])

  // 使用规则解析用户指令（作为fallback）
  const parseAgentCreationCommandWithRules = useCallback(async (text) => {
    console.log('[useAgentMessages] 直接调用AI解析指令:', text)
    
    try {
      // 移除创建命令关键词
      const createKeywords = ['创建智能体', '新建智能体', 'create agent', 'new agent', '/create', '/new']
      let cleanedText = text.toLowerCase().trim()
      for (const keyword of createKeywords) {
        cleanedText = cleanedText.replace(keyword.toLowerCase(), '').trim()
      }
      
      // 如果指令为空，使用默认值
      if (!cleanedText) {
        return {
          name: `智能体_${Date.now().toString().slice(-6)}`,
          roleDescription: '这是一个自动创建的智能体，可以帮助您处理各种任务。',
          isDefault: true
        }
      }
      
      // 尝试使用现有的AI服务解析指令
      // 首先检查是否有可用的API key
      const apiKey = typeof window !== 'undefined' ? localStorage.getItem('claude_api_key') : null
      
      if (apiKey) {
        try {
          // 导入agentService
          const agentService = (await import('@/services/agentService')).default
          
          // 构建AI提示词
          const aiPrompt = `用户想要创建一个智能体，指令是："${cleanedText}"
          
请解析这个指令并生成智能体信息：
1. 智能体名字（2-6个中文字符，有创意且相关）
2. 角色描述（100-200字，描述智能体的职责和能力）

请以JSON格式返回，格式如下：
{
  "name": "智能体名字",
  "roleDescription": "详细的角色描述"
}

只返回JSON，不要其他内容。`
          
          console.log('[useAgentMessages] 调用AI服务解析指令...')
          
          // 调用AI服务
          const result = await agentService.queryClaudeAgentDirect({
            apiKey,
            prompt: aiPrompt,
            systemPrompt: '你是一个智能体创建助手，负责解析用户指令并生成智能体信息。',
            model: 'claude-3-haiku-20240307', // 使用更快的模型
            maxTokens: 500,
            temperature: 0.7,
          })
          
          console.log('[useAgentMessages] AI解析结果:', result)
          
          // 尝试解析返回的JSON
          try {
            const parsedResult = JSON.parse(result.content)
            if (parsedResult.name && parsedResult.roleDescription) {
              return {
                name: parsedResult.name,
                roleDescription: parsedResult.roleDescription,
                isDefault: false
              }
            }
          } catch (jsonError) {
            console.warn('[useAgentMessages] AI返回的不是有效JSON，尝试提取信息:', jsonError)
            // 尝试从文本中提取信息
            const lines = result.content.split('\n')
            let name = null
            let roleDescription = null
            
            for (const line of lines) {
              const trimmedLine = line.trim()
              if (trimmedLine.includes('名字') || trimmedLine.includes('name') || trimmedLine.includes('：')) {
                const nameMatch = trimmedLine.match(/[：:]\s*(.+)/)
                if (nameMatch && nameMatch[1]) {
                  name = nameMatch[1].trim().replace(/["']/g, '')
                }
              }
              if (trimmedLine.length > 20 && !trimmedLine.includes('{') && !trimmedLine.includes('}')) {
                roleDescription = trimmedLine
              }
            }
            
            if (name && roleDescription) {
              return {
                name,
                roleDescription,
                isDefault: false
              }
            }
          }
        } catch (aiError) {
          console.warn('[useAgentMessages] AI服务调用失败，使用规则解析:', aiError)
        }
      }
      
      // AI解析失败或没有API key，使用规则解析
      console.log('[useAgentMessages] 使用规则解析指令')
      return parseAgentCreationCommandWithRulesFallback(text)
      
    } catch (error) {
      console.error('[useAgentMessages] 解析指令失败:', error)
      // 出错时使用规则解析
      return parseAgentCreationCommandWithRulesFallback(text)
    }
  }, [])

  // 使用规则解析用户指令（作为fallback）
  const parseAgentCreationCommandWithRulesFallback = useCallback((text) => {
    const lowerText = text.toLowerCase().trim()
    
    // 移除创建命令关键词
    const createKeywords = ['创建智能体', '新建智能体', 'create agent', 'new agent', '/create', '/new']
    let cleanedText = lowerText
    for (const keyword of createKeywords) {
      cleanedText = cleanedText.replace(keyword.toLowerCase(), '').trim()
    }
    
    // 如果指令为空，使用默认值
    if (!cleanedText) {
      return {
        name: `智能体_${Date.now().toString().slice(-6)}`,
        roleDescription: '这是一个自动创建的智能体，可以帮助您处理各种任务。',
        isDefault: true
      }
    }
    
    // 尝试从指令中提取信息
    let name = null
    let roleDescription = cleanedText
    
    // 模式1：包含"为"、"叫做"、"名为"（中文）
    const chinesePatterns = [
      { pattern: /(?:为|叫做|名为)[：:]\s*([^，,。.\n]+)/, group: 1 },
      { pattern: /(?:为|叫做|名为)\s+([^，,。.\n]+)/, group: 1 },
      { pattern: /([^，,。.\n]+?)(?:为|叫做|名为)/, group: 1 }
    ]
    
    // 模式2：包含"named"、"called"、"as"（英文）
    const englishPatterns = [
      { pattern: /(?:named|called|as)[：:]\s*([^，,.\n]+)/, group: 1 },
      { pattern: /(?:named|called|as)\s+([^，,.\n]+)/, group: 1 },
      { pattern: /([^，,.\n]+?)(?:named|called|as)/, group: 1 }
    ]
    
    // 模式3：包含"角色是"、"功能是"、"用于"（描述性）
    const descriptionPatterns = [
      { pattern: /(?:角色是|功能是|用于)[：:]\s*([^，,。.\n]+)/, group: 1 },
      { pattern: /(?:角色是|功能是|用于)\s+([^，,。.\n]+)/, group: 1 }
    ]
    
    // 尝试所有模式
    const allPatterns = [...chinesePatterns, ...englishPatterns, ...descriptionPatterns]
    
    for (const patternInfo of allPatterns) {
      const match = cleanedText.match(patternInfo.pattern)
      if (match && match[patternInfo.group]) {
        const extracted = match[patternInfo.group].trim()
        
        // 如果是名字模式
        if (patternInfo.pattern.source.includes('为') || 
            patternInfo.pattern.source.includes('叫做') || 
            patternInfo.pattern.source.includes('名为') ||
            patternInfo.pattern.source.includes('named') ||
            patternInfo.pattern.source.includes('called') ||
            patternInfo.pattern.source.includes('as')) {
          name = extracted
          roleDescription = cleanedText.replace(match[0], '').trim()
          break
        } 
        // 如果是描述模式
        else if (patternInfo.pattern.source.includes('角色是') || 
                 patternInfo.pattern.source.includes('功能是') || 
                 patternInfo.pattern.source.includes('用于')) {
          roleDescription = extracted
          // 从原始文本中移除描述部分，剩下的可能是名字
          const remaining = cleanedText.replace(match[0], '').trim()
          if (remaining && remaining.length > 0 && remaining.length <= 20) {
            name = remaining
          }
          break
        }
      }
    }
    
    // 如果没有提取到名字，使用指令作为角色描述，生成默认名字
    if (!name) {
      name = `智能体_${Date.now().toString().slice(-6)}`
      // 如果指令较短且没有空格，直接作为名字
      if (cleanedText.length <= 20 && !cleanedText.includes(' ')) {
        name = cleanedText
        roleDescription = '这是一个自动创建的智能体，可以帮助您处理各种任务。'
      }
    }
    
    // 清理角色描述
    if (!roleDescription || roleDescription.length < 5) {
      roleDescription = '这是一个自动创建的智能体，可以帮助您处理各种任务。'
    } else if (roleDescription.length > 200) {
      // 截断过长的描述
      roleDescription = roleDescription.substring(0, 197) + '...'
    }
    
    // 生成更友好的名字
    const friendlyName = name
      .replace(/[^a-zA-Z0-9\u4e00-\u9fa5]/g, ' ') // 替换特殊字符为空格
      .split(' ')
      .filter(word => word.length > 0)
      .map(word => word.charAt(0).toUpperCase() + word.slice(1))
      .join(' ')
      .trim()
    
    return {
      name: friendlyName || `智能体_${Date.now().toString().slice(-6)}`,
      roleDescription,
      isDefault: false
    }
  }, [])

  // 向后兼容的 sendMessage（发送到当前活动智能体）
  const sendMessage = useCallback(async () => {
    const text = currentMessage.trim()
    if (!text || isLoading) {
      return
    }

    // 如果没有活动频道，检查是否为创建智能体的命令
    if (!activeChannelId) {
      const createCommands = ['创建智能体', '新建智能体', 'create agent', 'new agent', '/create', '/new']
      const isCreateCommand = createCommands.some(cmd => 
        text.toLowerCase().includes(cmd.toLowerCase())
      )
      
      if (isCreateCommand) {
        console.log('[useAgentMessages] 检测到创建智能体命令:', text)
        setCurrentMessage('')
        
        // 解析指令并生成智能体信息
        const agentInfo = await parseAgentCreationCommandWithRules(text)
        console.log('[useAgentMessages] 解析的智能体信息:', agentInfo)
        
        // 显示创建中的消息
        appendMessage({
          id: `system_${Date.now()}`,
          type: 'assistant',
          content: `🔄 正在创建智能体 "${agentInfo.name}"...`,
          timestamp: Date.now(),
          source: 'system',
        }, 'system')
        
        // 调用自动创建智能体的逻辑
        if (onAutoCreateAgent) {
          try {
            // 调用自动创建智能体回调
            await onAutoCreateAgent(agentInfo)
            console.log('[useAgentMessages] 智能体自动创建命令已处理')
            
            // 显示创建成功消息
            appendMessage({
              id: `system_${Date.now()}_success`,
              type: 'assistant',
              content: `✅ 智能体 "${agentInfo.name}" 创建成功！已添加到频道栏。`,
              timestamp: Date.now(),
              source: 'system',
            }, 'system')
          } catch (error) {
            console.error('[useAgentMessages] 自动创建智能体失败:', error)
            // 显示错误消息
            appendMessage({
              id: `system_${Date.now()}_error`,
              type: 'assistant',
              content: `❌ 创建智能体失败: ${error.message || '未知错误'}`,
              timestamp: Date.now(),
              source: 'system',
            }, 'system')
          }
        } else if (onCreateAgent) {
          // 如果没有自动创建回调，但有创建回调，则打开模态框
          try {
            await onCreateAgent()
            console.log('[useAgentMessages] 智能体创建命令已处理（打开模态框）')
          } catch (error) {
            console.error('[useAgentMessages] 打开创建模态框失败:', error)
          }
        } else {
          // 如果没有提供任何创建回调，显示提示
          setTimeout(() => {
            appendMessage({
              id: `system_${Date.now()}_2`,
              type: 'assistant',
              content: '⚠️ 智能体创建功能需要从左侧"+"按钮启动。请点击左侧的"+"按钮创建智能体。',
              timestamp: Date.now(),
              source: 'system',
            }, 'system')
          }, 1000)
        }
        
        return
      }
      
      console.warn('[useAgentMessages] 无法发送消息：没有活动频道')
      return
    }

    setCurrentMessage('')
    await sendMessageToAgent(activeChannelId, text, selectedAgent)
  }, [
    activeChannelId,
    currentMessage,
    isLoading,
    selectedAgent,
    sendMessageToAgent,
    appendMessage,
    onCreateAgent,
    onAutoCreateAgent,
    parseAgentCreationCommandWithRules,
  ])

  const openConversationPanel = useCallback(() => {
    setConversationVisible(true)
    scrollToBottom()
  }, [scrollToBottom])

  const closeConversationPanel = useCallback(() => {
    setConversationVisible(false)
  }, [])

  // 当关闭对话面板或切换频道时，保存消息到 IPFS
  const previousChannelRef = useRef(activeChannelId)
  useEffect(() => {
    const prevChannel = previousChannelRef.current
    
    // 如果频道发生变化，保存之前频道的消息
    if (prevChannel && prevChannel !== activeChannelId) {
      const prevMessages = messagesByChannel[prevChannel] || []
      const savedCount = savedMessageCountRef.current[prevChannel] || 0
      
      if (prevMessages.length > savedCount) {
        // 异步保存，不阻塞
        const agentId = prevChannel
        saveMessagesToIpfs(prevChannel, agentId).catch(err => {
          console.error('[useAgentMessages] 切换频道时保存消息失败:', err)
        })
      }
    }
    
    previousChannelRef.current = activeChannelId
  }, [activeChannelId, messagesByChannel, saveMessagesToIpfs])

  /**
   * 终止指定智能体的执行
   */
  const cancelAgentExecution = useCallback((agentId) => {
    const controller = abortControllersByAgent.current[agentId]
    if (controller) {
      console.log('[useAgentMessages] 终止智能体执行:', agentId)
      controller.abort()
      delete abortControllersByAgent.current[agentId]
      setAgentLoading(agentId, false)
      
      // 添加终止消息
      appendMessage({
        id: `cancel_${Date.now()}`,
        type: 'assistant',
        content: '⏹️ 执行已终止',
        timestamp: Date.now(),
        source: 'cancel',
      }, agentId)
    }
  }, [appendMessage, setAgentLoading])

  return {
    messages,
    messagesByChannel,
    setMessages: (msgs) => {
      if (activeChannelId) {
        setMessagesForChannel(activeChannelId, msgs)
      }
    },
    setMessagesForChannel,
    clearMessagesForChannel,
    currentMessage,
    setCurrentMessage,
    isLoading,
    setIsLoading,
    // 多智能体独立执行空间
    loadingByAgent,
    isAgentLoading,
    setAgentLoading,
    sessionsByAgent,
    isConversationVisible,
    setConversationVisible,
    appendMessage,
    scrollToBottom,
    sendMessage,
    sendMessageToAgent, // 新增：发送到指定智能体
    cancelAgentExecution, // 新增：终止智能体执行
    openConversationPanel,
    closeConversationPanel,
    saveMessagesToIpfs,
    loadMessagesFromIpfs,
  }
}
