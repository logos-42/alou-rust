/**
 * 智能体文档化创建示例
 *
 * 演示如何使用文档化系统创建智能体
 */

import agentDocumentService, { AgentDocuments, DocumentTypes } from '../src/services/agentDocumentService'

interface AgentResult {
  documents: AgentDocuments
  documentCids: Record<string, string>
  systemPrompt: string
}

interface CreateAgentConfig {
  prompt: string
  name: string
  emoji: string
  mcpTools?: Array<{
    name: string
    description?: string
    endpoint?: string
    port?: string
  }>
  memoryConfig?: {
    enableLongTerm: boolean
    enableWorkingMemory: boolean
    memoryLimit: number
  }
}

interface BatchResult {
  name: string
  documentCids?: Record<string, string>
  systemPromptLength?: number
  success: boolean
  error?: string
}

/**
 * 示例 1: 创建代码审查专家
 */
async function createCodeReviewer(): Promise<AgentResult> {
  console.log('=== 示例 1: 创建代码审查专家 ===')

  const userPrompt = '创建一个代码审查专家，擅长 JavaScript 和 TypeScript，专注于安全性和性能优化'

  const documents = await agentDocumentService.generateFullDocumentSet(userPrompt, {
    name: '代码审查员',
    avatar: null,
    emoji: '🔍',
    mcpTools: [
      { name: 'linter', description: '代码语法检查' },
      { name: 'formatter', description: '代码格式化' },
    ],
    memoryConfig: {
      enableLongTerm: true,
      enableWorkingMemory: true,
      memoryLimit: 1000,
    },
  })

  console.log('生成的文档:')
  console.log('- SOUL.md:', documents.soul?.content.slice(0, 100) + '...')
  console.log('- IDENTITY.md:', documents.identity?.content.slice(0, 100) + '...')
  console.log('- CAPABILITIES.md:', documents.capabilities?.content.slice(0, 100) + '...')

  // 上传到 IPFS
  const documentCids = await agentDocumentService.uploadFullDocumentSet(documents)
  console.log('\n文档已上传到 IPFS:', documentCids)

  // 构建系统提示词
  const systemPrompt = agentDocumentService.buildSystemPromptFromDocuments(documents)
  console.log('\n系统提示词长度:', systemPrompt.length)
  console.log('系统提示词预览:', systemPrompt.slice(0, 200) + '...')

  return { documents, documentCids, systemPrompt }
}

/**
 * 示例 2: 创建创意写作助手
 */
async function createCreativeWriter(): Promise<AgentResult> {
  console.log('=== 示例 2: 创建创意写作助手 ===')

  const userPrompt = '创建一个创意写作助手，擅长小说创作和故事讲述，风格温暖且富有想象力'

  const documents = await agentDocumentService.generateFullDocumentSet(userPrompt, {
    name: '创意作家',
    avatar: null,
    emoji: '✍️',
    mcpTools: [
      { name: 'dictionary', description: '词典查询' },
      { name: 'thesaurus', description: '同义词词典' },
    ],
    memoryConfig: {
      enableLongTerm: true,
      enableWorkingMemory: true,
      memoryLimit: 2000,
    },
  })

  const documentCids = await agentDocumentService.uploadFullDocumentSet(documents)
  const systemPrompt = agentDocumentService.buildSystemPromptFromDocuments(documents)

  return { documents, documentCids, systemPrompt }
}

/**
 * 示例 3: 创建数据分析师
 */
async function createDataAnalyst(): Promise<AgentResult> {
  console.log('=== 示例 3: 创建数据分析师 ===')

  const userPrompt = '创建一个数据分析师，擅长数据可视化和统计分析，能够从数据中发现洞察'

  const documents = await agentDocumentService.generateFullDocumentSet(userPrompt, {
    name: '数据分析师',
    avatar: null,
    emoji: '📊',
    mcpTools: [
      { name: 'chart', description: '图表生成' },
      { name: 'statistics', description: '统计分析' },
      { name: 'database', description: '数据库查询' },
    ],
    memoryConfig: {
      enableLongTerm: true,
      enableWorkingMemory: true,
      memoryLimit: 1500,
    },
  })

  const documentCids = await agentDocumentService.uploadFullDocumentSet(documents)
  const systemPrompt = agentDocumentService.buildSystemPromptFromDocuments(documents)

  return { documents, documentCids, systemPrompt }
}

/**
 * 示例 4: 单独生成特定文档
 */
async function generateSingleDocument(): Promise<{ soulDoc: any; capabilitiesDoc: any }> {
  console.log('=== 示例 4: 单独生成特定文档 ===')

  // 只生成 SOUL 文档
  const soulDoc = await agentDocumentService.generateSoulDocument(
    '创建一个严谨的科学研究助手，专注于物理学和数学'
  )
  console.log('SOUL.md:\n', soulDoc.content)

  // 只生成 CAPABILITIES 文档
  const capabilitiesDoc = await agentDocumentService.generateCapabilitiesDocument(
    '创建一个全栈开发助手',
    [
      { name: 'code', description: '代码生成' },
      { name: 'test', description: '测试生成' },
      { name: 'deploy', description: '部署自动化' },
    ]
  )
  console.log('\nCAPABILITIES.md:\n', capabilitiesDoc.content)

  return { soulDoc, capabilitiesDoc }
}

/**
 * 示例 5: 从 IPFS 加载文档
 */
async function loadDocumentsFromIpfs(documentCids: Record<string, string>): Promise<AgentDocuments> {
  console.log('=== 示例 5: 从 IPFS 加载文档 ===')

  const documents = await AgentDocuments.fromCids(documentCids)
  console.log('加载的文档类型:', Object.keys(documents).filter(k => k !== 'createdAt' && k !== 'version'))

  // 构建系统提示词
  const systemPrompt = agentDocumentService.buildSystemPromptFromDocuments(documents)
  console.log('系统提示词长度:', systemPrompt.length)

  return documents
}

/**
 * 示例 6: 批量创建智能体
 */
async function createMultipleAgents(): Promise<BatchResult[]> {
  console.log('=== 示例 6: 批量创建智能体 ===')

  const agentConfigs: CreateAgentConfig[] = [
    {
      prompt: '创建一个 Python 专家',
      name: 'Python 导师',
      emoji: '🐍',
      mcpTools: [],
      memoryConfig: {
        enableLongTerm: true,
        enableWorkingMemory: true,
        memoryLimit: 1000,
      },
    },
    {
      prompt: '创建一个 Rust 专家',
      name: 'Rust 大师',
      emoji: '🦀',
      mcpTools: [],
      memoryConfig: {
        enableLongTerm: true,
        enableWorkingMemory: true,
        memoryLimit: 1000,
      },
    },
    {
      prompt: '创建一个 Go 专家',
      name: 'Go 开发者',
      emoji: '🔷',
      mcpTools: [],
      memoryConfig: {
        enableLongTerm: true,
        enableWorkingMemory: true,
        memoryLimit: 1000,
      },
    },
  ]

  const results: BatchResult[] = []

  for (const config of agentConfigs) {
    console.log(`\n创建智能体: ${config.name}`)

    try {
      const documents = await agentDocumentService.generateFullDocumentSet(config.prompt, {
        name: config.name,
        emoji: config.emoji,
        mcpTools: config.mcpTools,
        memoryConfig: config.memoryConfig,
      })

      const documentCids = await agentDocumentService.uploadFullDocumentSet(documents)
      const systemPrompt = agentDocumentService.buildSystemPromptFromDocuments(documents)

      results.push({
        name: config.name,
        documentCids,
        systemPromptLength: systemPrompt.length,
        success: true,
      })

      console.log(`✓ ${config.name} 创建成功`)
    } catch (error: any) {
      console.error(`✗ ${config.name} 创建失败:`, error.message)
      results.push({
        name: config.name,
        success: false,
        error: error.message,
      })
    }
  }

  console.log('\n批量创建结果:')
  results.forEach(result => {
    console.log(`- ${result.name}: ${result.success ? '成功' : '失败'}`)
  })

  return results
}

/**
 * 示例 7: 自定义文档编辑
 */
async function editDocument(documentCids: Record<string, string>): Promise<Record<string, string>> {
  console.log('=== 示例 7: 自定义文档编辑 ===')

  // 加载文档
  const documents = await AgentDocuments.fromCids(documentCids)

  // 修改 SOUL 文档
  const originalSoul = documents.soul?.content || ''
  console.log('原始 SOUL.md 长度:', originalSoul.length)

  // 添加自定义内容
  const customAddition = '\n\n## Custom Rules\n\n- 遵循用户的特定偏好\n- 保持简洁和高效\n'
  const modifiedSoul = originalSoul + customAddition

  console.log('修改后的 SOUL.md 长度:', modifiedSoul.length)

  // 重新上传
  const newCid = await agentDocumentService.uploadDocumentToIpfs({
    content: modifiedSoul,
    type: DocumentTypes.SOUL,
    metadata: documents.soul?.metadata || { generatedAt: Date.now() },
  })

  console.log('新 SOUL.md CID:', newCid)

  // 更新 CID 映射
  const updatedCids = {
    ...documentCids,
    soul: newCid,
  }

  return updatedCids
}

/**
 * 主函数：运行所有示例
 */
async function main(): Promise<void> {
  console.log('智能体文档化创建示例程序\n')

  try {
    // 运行示例 1
    const result1 = await createCodeReviewer()
    console.log('\n示例 1 完成')

    // 运行示例 5（从示例 1 的结果加载）
    await loadDocumentsFromIpfs(result1.documentCids)
    console.log('\n示例 5 完成')

    // 运行示例 4
    await generateSingleDocument()
    console.log('\n示例 4 完成')

    // 运行示例 2
    const result2 = await createCreativeWriter()
    console.log('\n示例 2 完成')

    // 运行示例 7（编辑示例 2 的文档）
    await editDocument(result2.documentCids)
    console.log('\n示例 7 完成')

    // 运行示例 6（批量创建）
    await createMultipleAgents()
    console.log('\n示例 6 完成')

    console.log('\n所有示例运行完成!')
  } catch (error) {
    console.error('运行示例时出错:', error)
  }
}

// 导出示例函数
export {
  createCodeReviewer,
  createCreativeWriter,
  createDataAnalyst,
  generateSingleDocument,
  loadDocumentsFromIpfs,
  createMultipleAgents,
  editDocument,
  main,
}

// 导出类型
export type {
  AgentResult,
  CreateAgentConfig,
  BatchResult,
}
