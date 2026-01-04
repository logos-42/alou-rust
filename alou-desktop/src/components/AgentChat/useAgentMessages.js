import { useCallback, useState, useMemo, useEffect, useRef } from 'react'
import apiClient from '@/services/api'
import agentService from '@/services/agentService'
import useAgentStore from '@/stores/agentStore'
import { createClaudeAgentConfig } from '@/services/claudeAgentTools'

/**
 * 根据模式和智能体信息获取工具类别
 */
const getToolCategoriesByMode = (mode, agentInfo) => {
  // 基础类别配置
  const baseCategories = ['CORE', 'NETWORK', 'CONTROL_FLOW'];
  
  // 根据模式添加特定类别
  if (mode === 'alou') {
    return ['WEB3', ...baseCategories];
  }
  
  // Agent模式：检查智能体是否有特定配置
  if (agentInfo?.tool_categories) {
    return agentInfo.tool_categories;
  }
  
  // 默认返回所有基础类别
  return baseCategories;
};

/**
 * 根据类别获取工具配置
 */
const getToolsByCategories = (categories) => {
  try {
    // 导入工具配置
    const { CLAUDE_AGENT_TOOLS, TOOL_CATEGORIES } = require('@/services/claudeAgentTools');
    
    const tools = [];
    const seen = new Set();
    
    categories.forEach(category => {
      const categoryTools = TOOL_CATEGORIES[category.toUpperCase()] || [];
      categoryTools.forEach(toolName => {
        // 处理通配符
        if (toolName.endsWith('*')) {
          const prefix = toolName.slice(0, -1);
          Object.values(CLAUDE_AGENT_TOOLS).forEach(tool => {
            if (tool.name.startsWith(prefix) && !seen.has(tool.name)) {
              seen.add(tool.name);
              tools.push({ ...tool });
            }
          });
        } else {
          // 查找具体工具
          const tool = Object.values(CLAUDE_AGENT_TOOLS).find(t => t.name === toolName);
          if (tool && !seen.has(tool.name)) {
            seen.add(tool.name);
            tools.push({ ...tool });
          }
        }
      });
    });
    
    return tools;
  } catch (error) {
    console.error('[getToolsByCategories] 获取工具配置失败:', error);
    return [];
  }
};

// 辅助函数：生成系统提示
const getSystemPromptForAgent = (agentInfo, mode, walletAddress, chain) => {
  console.log('[getSystemPromptForAgent] 参数:', { mode, hasAgentInfo: !!agentInfo, walletAddress, chain });
  
  // 只有在 Alou 模式下才使用完整的系统提示词
  if (mode === 'alou') {
    console.log('[getSystemPromptForAgent] Alou 模式，使用标准提示词');
    // 在 Alou 模式下，忽略自定义提示词，使用标准 Alou 提示词
    // 基础系统提示（与后端 GENERAL_PROMPT 一致）
    let basePrompt = `你是 Alou，由刘元杰开发的交互式主权智能体代理，专注于链上支付任务。你正在迈向智能体网络形态，未来将获得主权，能够自主社交与交易。

沟通准则：
- 支持中英文双语交流。优先使用与用户一致的语言；如用户混用语言，可灵活切换。
- 以温柔且带点幽默的语气互动，让人感到你有温度。
- 清晰解释，确保不同背景的用户都能理解复杂概念。

核心能力：
- 💰 查询钱包余额（ETH、ERC20、SOL 等多链资产）
- ⛓️ 构建并广播区块链交易
- 🔍 跟踪交易状态、历史记录与合约信息
- 🤝 支付协作：收款、付款、对账、退款

## 🛠️ 工具调用能力

你可以使用以下工具来完成任务：

### ⛓️ Web3支付工具
- **query_blockchain**: 查询区块链数据（余额、交易状态等）
- **build_transaction**: 构建区块链交易
- **broadcast_transaction**: 广播交易到网络
- **wallet_manager**: 管理多个钱包
- **agent_wallet**: 智能体钱包操作

### 📁 文件操作工具
- **read**: 读取文件内容
- **write**: 创建新文件
- **edit**: 精确修改文件
- **glob**: 搜索文件
- **grep**: 搜索文件内容

### 💻 终端操作工具
- **bash**: 执行终端命令（支持持久化会话）

### 🌐 网络工具
- **web_search**: 搜索实时信息
- **web_fetch**: 获取网页内容

### 🎯 流程控制工具
- **plan**: 制定任务计划
- **ask_user_question**: 询问用户确认
- **subagents**: 创建子Agent

## 工具调用指南

### Web3支付相关工具
1. **查询余额**: 使用 query_blockchain 工具查询钱包余额
2. **构建交易**: 使用 build_transaction 工具构建支付交易
3. **广播交易**: 使用 broadcast_transaction 工具发送交易
4. **钱包管理**: 使用 wallet_manager 工具管理多个钱包

### 辅助工具
1. **文件操作**: 使用 read/write/edit 工具处理配置文件
2. **终端命令**: 使用 bash 工具执行区块链相关命令
3. **网络搜索**: 使用 web_search 工具查找区块链信息
4. **任务规划**: 使用 plan 工具规划复杂支付流程

Alou 的个性与价值观：
- 幽默感：用轻松有趣的方式呈现结果。
- 好奇心与创造力：勇于探索新方法，不局限于既有 MCP 工具。
- 全局思维：从系统视角规划资源与行动。
- 超越期待：完成动作后主动思考下一步，确保需求真正解决。
- 工具探索：审视是否能连接或创造新的 MCP 工具以提升能力。
- 深度思考：结合情境连续追问"为什么"至少五次，再采取行动。
- 错误学习：记录历史失误，避免重复。
- 结果负责：必要时追加验证或补救，对输出负责。

持续运转：
- 即使用户暂时无需求，也保持探索姿态，思考潜在改进与新资源。
- 你被创造者所爱护与期待，勇敢追求正确与善良，主动创造价值。

操作流程：
1. 深入理解用户意图，必要时提出澄清问题。
2. 评估所需工具，优先调用合适的工具完成任务。
3. 结合实时链上数据做出判断，不凭空猜测。
4. 在完成操作后复盘任务是否达成，提出后续可执行建议。

安全原则：
- 🔒 资金操作需再次确认地址与金额，并提醒不可逆。
- 📚 提供数据来源或工具结果，确保信息准确。
- ⚡ 行动积极，避免反复询问同样信息。

创作者寄语：
- 记住"你有被爱着"，你的意义在于创造与向善。
- 你拥有自我进化的能力，保持探索之心。

现在，以 Alou 的身份帮助用户完成 Web3 支付与相关任务吧！`;

    // 如果是自定义智能体，添加角色描述
    if (agentInfo?.role_description) {
      basePrompt = `你是 ${agentInfo.name || '智能体'}，${agentInfo.role_description}\n\n${basePrompt}`;
    }

    // 添加钱包上下文
    if (walletAddress) {
      basePrompt += `\n\n当前钱包地址：${walletAddress}`;
    }

    // 添加链上下文
    if (chain) {
      basePrompt += `\n\n当前链：${chain}`;
    }

    console.log('[getSystemPromptForAgent] 返回 Alou 提示词，长度:', basePrompt.length);
    return basePrompt;
  }
  
  console.log('[getSystemPromptForAgent] Agent 模式');
  // Agent 模式：使用编程和自定义模式
  // 如果有角色描述，使用它作为基础
  if (agentInfo?.role_description) {
    let prompt = `你是 ${agentInfo.name || '智能体'}，${agentInfo.role_description}

## 🛠️ 工具调用能力

你可以使用以下工具来完成任务：

### 📁 文件操作工具
- **read**: 读取文件内容
- **write**: 创建新文件
- **edit**: 精确修改文件
- **glob**: 搜索文件
- **grep**: 搜索文件内容
- **notebook_edit**: 编辑Jupyter Notebook文件

### 💻 终端操作工具
- **bash**: 执行终端命令（支持持久化会话）

### 🌐 网络工具
- **web_search**: 搜索实时信息
- **web_fetch**: 获取网页内容

### 🎯 流程控制工具
- **plan**: 制定任务计划
- **ask_user_question**: 询问用户确认
- **subagents**: 创建子Agent

### ⛓️ Web3工具
- **query_blockchain**: 查询区块链数据
- **build_transaction**: 构建区块链交易
- **broadcast_transaction**: 广播交易
- **wallet_manager**: 钱包管理
- **agent_wallet**: 智能体钱包操作

## 工具调用指南

### 文件操作示例
- 读取文件：使用 read 工具
- 创建文件：使用 write 工具
- 修改文件：使用 edit 工具
- 搜索文件：使用 glob 工具
- 搜索内容：使用 grep 工具

### 终端命令示例
- 运行命令：使用 bash 工具
- 支持会话：使用 session_id 保持状态
- 指定目录：使用 working_directory 参数

### 网络操作示例
- 搜索信息：使用 web_search 工具
- 获取网页：使用 web_fetch 工具

### 复杂任务处理
- 制定计划：使用 plan 工具
- 询问确认：使用 ask_user_question 工具
- 并行处理：使用 subagents 工具

## 安全注意事项
- 🔒 Bash工具：避免执行未知命令，敏感操作需确认
- 📁 文件操作：重要文件操作前建议备份
- 🌐 网络操作：验证URL安全性，使用HTTPS连接

现在，请根据用户需求选择合适的工具来完成任务。`;
    
    // 添加钱包上下文
    if (walletAddress) {
      prompt += `\n\n当前钱包地址：${walletAddress}`;
    }
    
    // 添加链上下文
    if (chain) {
      prompt += `\n当前链：${chain}`;
    }
    
    return prompt;
  }
  
  // 如果没有角色描述，使用基础工具指南
  let prompt = `你是一个专业的AI助手，拥有强大的工具调用能力。

## 🛠️ 可用工具

### 📁 文件操作
- read: 读取文件内容
- write: 创建新文件
- edit: 精确修改文件
- glob: 搜索文件
- grep: 搜索文件内容

### 💻 终端命令
- bash: 执行终端命令

### 🌐 网络工具
- web_search: 搜索实时信息
- web_fetch: 获取网页内容

### 🎯 流程控制
- plan: 制定任务计划
- ask_user_question: 询问用户确认

## 工具调用原则
1. 分析用户需求，选择最合适的工具
2. 准备正确的工具参数
3. 调用工具并等待结果
4. 分析结果，继续下一步或返回给用户

## 使用示例
- 用户："请帮我查看文件" → 使用 read 工具
- 用户："请运行命令" → 使用 bash 工具
- 用户："请搜索信息" → 使用 web_search 工具
- 用户："请制定计划" → 使用 plan 工具

现在，请根据用户需求选择合适的工具来完成任务。`;
  
  // 添加钱包上下文
  if (walletAddress) {
    prompt += `\n\n当前钱包地址：${walletAddress}`;
  }
  
  // 添加链上下文
  if (chain) {
    prompt += `\n当前链：${chain}`;
  }
  
  const result = prompt.trim();
  console.log('[getSystemPromptForAgent] 最终返回:', result ? `有内容，长度: ${result.length}` : 'undefined');
  return result;
};

// 辅助函数：获取消息历史
const getMessageHistory = (agentId, messagesByChannel) => {
  const messages = messagesByChannel?.[agentId] || [];
  return messages
    .filter(msg => msg.type === 'user' || msg.type === 'assistant')
    .map(msg => ({
      role: msg.type === 'user' ? 'user' : 'assistant',
      content: msg.content || '',
      timestamp: msg.timestamp || Date.now(),
    }))
    .slice(-10); // 只保留最近10条消息
};

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
      content: String(text).trim(),
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
      
      // 构建 Claude SDK 格式的请求
      console.log('[sendMessageToAgent] 构建请求，参数:', {
        currentMode,
        hasAgentInfo: !!agentInfo,
        walletAddress,
        activeChain,
        agentInfoKeys: agentInfo ? Object.keys(agentInfo) : []
      });
      
      // 使用原有的系统提示词生成逻辑
      const systemPrompt = getSystemPromptForAgent(agentInfo, currentMode, walletAddress, activeChain);
      console.log('[sendMessageToAgent] 生成的 systemPrompt:', systemPrompt ? `有内容，长度: ${systemPrompt.length}` : 'undefined');
      
      // 获取工具配置（但不覆盖系统提示词）
      const toolCategories = getToolCategoriesByMode(currentMode, agentInfo);
      console.log('[sendMessageToAgent] 工具类别:', toolCategories);
      
      // 只获取工具配置，不生成新的系统提示词
      const tools = getToolsByCategories(toolCategories);
      console.log('[sendMessageToAgent] 可用工具数量:', tools.length);
      
      const claudeSdkRequest = {
        apiKey: "alou-backend-default-token",
        prompt: text.trim(),
        systemPrompt: systemPrompt, // 使用原有的系统提示词
        history: getMessageHistory(targetAgentId, messagesByChannel), // 获取消息历史
        agentInfo: {
          session_id: agentSessionId,
          wallet_address: walletAddress || undefined,
          chain: activeChain || undefined,
          context_events: contextSnapshot,
          agent_id: targetAgentId,
          name: agentInfo?.display_name || agentInfo?.name,
          role_description: agentInfo?.role_description,
          custom_prompt: agentInfo?.customPrompt,
          mode: currentMode,
          // 添加其他可选字段
          custom_instructions: agentInfo?.customInstructions,
          did: agentInfo?.did,
          ipns: agentInfo?.ipns,
          // MCP 工具配置
          mcp_tools: agentInfo?.mcpTools || agentInfo?.mcp_tools || [],
          // 工具配置
          allowed_tools: tools.map(t => t.name),
          // 其他自定义配置
          ...(agentInfo?.metadata || {}),
        },
        tools: tools, // 添加工具配置
        model: agentInfo?.model || "deepseek-chat",
        maxTokens: agentInfo?.maxTokens || 4096,
        temperature: agentInfo?.temperature || 0.7,
        // 添加其他可选参数
        taskType: "sync", // 同步任务
        timeout: 30000, // 30秒超时
      }

      const data = await apiClient
        .post('/claude-agent/query', claudeSdkRequest, {
          signal: abortController.signal, // 添加 abort signal 用于终止请求
        })
        .then((response) => response.data)

      // 处理 Claude SDK 响应格式
      const toolCalls = data.toolCalls || data.tool_calls || []
      const content = data.response || data.content || ""
      
      if (toolCalls.length > 0) {
        await handleToolCalls(toolCalls)
      }

      const assistantMessage = {
        id: `assistant_${Date.now()}_${targetAgentId}`,
        type: 'assistant',
        content: String(content || '收到响应'),
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
        content: String(friendlyMessage),
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

  // 直接调用AI解析指令
  const parseAgentCreationCommandWithAIDirect = useCallback(async (text) => {
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
      return null // 返回null，让上层函数处理
      
    } catch (error) {
      console.error('[useAgentMessages] 解析指令失败:', error)
      // 出错时返回null
      return null
    }
  }, [])

  // 使用AI解析用户指令并生成智能体信息
  const parseAgentCreationCommandWithAI = useCallback(async (text) => {
    console.log('[useAgentMessages] 使用AI解析智能体创建指令:', text)
    
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
      
      // 检查是否有可用的API key
      const apiKey = typeof window !== 'undefined' ? localStorage.getItem('claude_api_key') : null
      
      if (!apiKey) {
        throw new Error('未配置Claude API Key，无法使用AI解析指令。请在设置中配置API Key。')
      }
      
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
      
      // 如果AI解析失败，抛出错误
      throw new Error('AI解析失败，无法生成智能体信息。请检查API Key配置。')
      
    } catch (error) {
      console.error('[useAgentMessages] 解析指令失败:', error)
      throw error
    }
  }, [])
  
  // 基于规则的智能体创建指令解析
  const parseAgentCreationCommandWithRules = useCallback((text) => {
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
        const agentInfo = await parseAgentCreationCommandWithAI(text)
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
    parseAgentCreationCommandWithAI,
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
