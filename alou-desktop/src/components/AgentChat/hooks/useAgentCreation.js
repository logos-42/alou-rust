import { useCallback } from 'react'
import agentService from '@/services/agentService'

/**
 * Hook for managing agent creation logic
 */
export const useAgentCreation = ({
  appendMessage,
}) => {

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
  const parseAgentCreationCommandWithAIWrapper = useCallback(async (text) => {
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

  return {
    parseAgentCreationCommandWithAIDirect,
    parseAgentCreationCommandWithAI: parseAgentCreationCommandWithAIWrapper,
    parseAgentCreationCommandWithRules,
  }
}