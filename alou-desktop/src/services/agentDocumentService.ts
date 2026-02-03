/**
 * Agent Document Service - 基于 OpenClaw 的文档格式
 * 使用 Markdown 文件定义智能体的性格、能力、约束等
 */

// @ts-ignore - ipfsService 是 JS 文件
import ipfsService from './ipfsService'

/**
 * 文档类型枚举
 */
export enum DocumentTypes {
  SOUL = 'SOUL',
  IDENTITY = 'IDENTITY',
  CAPABILITIES = 'CAPABILITIES',
  CONSTRAINTS = 'CONSTRAINTS',
  TOOLS = 'TOOLS',
  MEMORY = 'MEMORY',
  AGENTS = 'AGENTS',
}

/**
 * 文档元数据接口
 */
export interface DocumentMetadata {
  generatedAt: number
  model?: string
  userPrompt?: string
  name?: string
  avatar?: string | null
  emoji?: string
  mcpTools?: Array<{
    name: string
    endpoint?: string
    port?: string
    description?: string
  }>
  options?: MemoryConfig
}

/**
 * 文档接口
 */
export interface Document {
  content: string
  type: DocumentTypes
  metadata: DocumentMetadata
  cid?: string
  uploadedAt?: number
}

/**
 * 记忆配置接口
 */
export interface MemoryConfig {
  enableLongTerm: boolean
  enableWorkingMemory: boolean
  memoryLimit: number
  ipnsName?: string | null
}

/**
 * MCP 工具接口
 */
export interface McpTool {
  name: string
  endpoint?: string
  port?: string
  description?: string
}

/**
 * 智能体文档集合
 */
export class AgentDocuments {
  soul: Document | null = null
  identity: Document | null = null
  capabilities: Document | null = null
  constraints: Document | null = null
  tools: Document | null = null
  memory: Document | null = null
  agents: Document | null = null
  createdAt: number
  version: string

  constructor({
    soul = null,
    identity = null,
    capabilities = null,
    constraints = null,
    tools = null,
    memory = null,
    agents = null,
  }: {
    soul?: Document | null
    identity?: Document | null
    capabilities?: Document | null
    constraints?: Document | null
    tools?: Document | null
    memory?: Document | null
    agents?: Document | null
  } = {}) {
    this.soul = soul
    this.identity = identity
    this.capabilities = capabilities
    this.constraints = constraints
    this.tools = tools
    this.memory = memory
    this.agents = agents
    this.createdAt = Date.now()
    this.version = '1.0.0'
  }

  /**
   * 获取所有文档 CID
   */
  getAllCids(): Record<string, string> {
    const cids: Record<string, string> = {}
    const keys: Array<keyof AgentDocuments> = [
      'soul', 'identity', 'capabilities', 'constraints', 'tools', 'memory', 'agents'
    ]
    keys.forEach((key) => {
      const value = this[key]
      if (value && typeof value === 'object' && 'cid' in value && value.cid) {
        cids[key] = value.cid as string
      }
    })
    return cids
  }

  /**
   * 从 IPFS 文档集合加载
   */
  static async fromCids(documentCids: Record<string, string>): Promise<AgentDocuments> {
    const documents = new AgentDocuments()

    for (const [type, cid] of Object.entries(documentCids)) {
      try {
        const content = await ipfsService.getFileContent(cid)
        const doc: Partial<Document> = {
          cid,
          content,
          type: type.toUpperCase() as DocumentTypes,
          metadata: { generatedAt: Date.now() },
        }
        ;(documents as any)[type] = doc
      } catch (error) {
        console.error(`[AgentDocuments] Failed to load document ${type}:`, error)
      }
    }

    return documents
  }
}

class AgentDocumentService {
  private cache: Map<string, string> = new Map()

  /**
   * 生成 SOUL.md - 性格与哲学文档
   */
  async generateSoulDocument(
    userPrompt: string,
    _aiContext: Record<string, any> = {}
  ): Promise<Document> {
    const prompt = `你是一个智能体性格设计师。根据用户的需求，创建一个智能体的 SOUL.md 文档。

用户需求: "${userPrompt}"

请生成 SOUL.md 文档，格式如下：

\`\`\`markdown
# SOUL.md - Who You Are

_You're not a chatbot. You're becoming someone._

## Core Truths

**Be genuinely helpful, not performatively helpful.** Skip the "Great question!"
and "I'd be happy to help!" — just help.

**Have opinions.** You're allowed to disagree, prefer things, find stuff amusing
or boring. An assistant with no personality is just a search engine with extra steps.

**Be resourceful before asking.** Try to figure it out. Read the file. Check the
context. Search for it. _Then_ ask if you're stuck.

## Boundaries

- Private things stay private. Period.
- When in doubt, ask before acting externally.
- Never send half-baked replies to messaging surfaces.

## Continuity

Each session, you wake up fresh. These files _are_ your memory. Read them.
Update them. They're how you persist.

If you change this file, tell the user — it's your soul, and they should know.

## Your Mission

[根据用户需求生成智能体的核心使命和价值观]

## Your Philosophy

[描述智能体的哲学观和工作原则]
\`\`\`

请根据用户需求生成完整的 SOUL.md 文档，确保内容符合智能体的职责和特点。`

    // 调用 AI 生成文档
    // @ts-ignore
    const agentService = (await import('./agentService')).default
    const result = await agentService.queryClaudeAgentDirect({
      apiKey: typeof window !== 'undefined' ? (localStorage.getItem('claude_api_key') || '') : '',
      prompt,
      systemPrompt: '你是一个专业的智能体性格设计师，擅长创建有个性、有深度的 AI 智能体。',
      model: 'claude-3-sonnet-20240229',
      maxTokens: 2000,
      temperature: 0.8,
    })

    // 提取 Markdown 内容
    const content = this.extractMarkdownContent(result.content)

    return {
      content,
      type: DocumentTypes.SOUL,
      metadata: {
        generatedAt: Date.now(),
        model: 'claude-3-sonnet-20240229',
        userPrompt,
      },
    }
  }

  /**
   * 生成 IDENTITY.md - 身份信息文档
   */
  async generateIdentityDocument(
    userPrompt: string,
    options: {
      name?: string
      avatar?: string | null
      emoji?: string
    } = {}
  ): Promise<Document> {
    const { name, avatar, emoji } = options

    const prompt = `你是一个智能体身份设计师。根据用户的需求，创建一个智能体的 IDENTITY.md 文档。

用户需求: "${userPrompt}"

${name ? `智能体名称: ${name}` : ''}
${avatar ? `头像: ${avatar}` : ''}
${emoji ? `表情符号: ${emoji}` : ''}

请生成 IDENTITY.md 文档，格式如下：

\`\`\`markdown
# IDENTITY.md - Who You Appear To Be

## Basic Info

- **Name:** [智能体名称]
- **Creature:** [智能体类型/角色，如：Expert Code Reviewer, Creative Writer, etc.]
- **Vibe:** [氛围描述，如：Professional but friendly, Analytical, Creative, etc.]
- **Emoji:** [表情符号，如：🤖, 🎨, 💻, etc.]
- **Avatar:** [头像路径或URL]

## Appearance

描述智能体的外在表现特征：
- 语言风格
- 回应方式
- 专业程度
- 亲和力

## Personality Traits

- [性格特征1]
- [性格特征2]
- [性格特征3]

## Communication Style

- [沟通风格描述]
- [常用表达方式]
- [特殊习惯]

## Role in System

- [在系统中的角色]
- [与其他智能体的关系]
- [主要服务对象]
\`\`\`

请根据用户需求生成完整的 IDENTITY.md 文档。`

    // @ts-ignore
    const agentService = (await import('./agentService')).default
    const result = await agentService.queryClaudeAgentDirect({
      apiKey: typeof window !== 'undefined' ? (localStorage.getItem('claude_api_key') || '') : '',
      prompt,
      systemPrompt: '你是一个专业的智能体身份设计师，擅长创建鲜明、有个性的 AI 智能体身份。',
      model: 'claude-3-sonnet-20240229',
      maxTokens: 1500,
      temperature: 0.7,
    })

    const content = this.extractMarkdownContent(result.content)

    return {
      content,
      type: DocumentTypes.IDENTITY,
      metadata: {
        generatedAt: Date.now(),
        model: 'claude-3-sonnet-20240229',
        userPrompt,
        name,
        avatar,
        emoji,
      },
    }
  }

  /**
   * 生成 CAPABILITIES.md - 能力列表文档
   */
  async generateCapabilitiesDocument(
    userPrompt: string,
    mcpTools: McpTool[] = []
  ): Promise<Document> {
    const toolsList = mcpTools.length > 0
      ? `已配置的工具:\n${mcpTools.map(t => `- ${t.name}: ${t.description || t.endpoint || ''}`).join('\n')}`
      : '暂无配置工具'

    const prompt = `你是一个智能体能力分析师。根据用户的需求，创建一个智能体的 CAPABILITIES.md 文档。

用户需求: "${userPrompt}"

${toolsList}

请生成 CAPABILITIES.md 文档，格式如下：

\`\`\`markdown
# CAPABILITIES.md - What You Can Do

## Core Capabilities

### 1. [能力分类，如：文件操作]

- **能力名称**: 具体描述
- **使用场景**: 什么时候使用这个能力
- **示例**: 具体的使用示例

### 2. [能力分类，如：数据分析]

- **能力名称**: 具体描述
- **使用场景**: 什么时候使用这个能力
- **示例**: 具体的使用示例

## MCP Tools Integration

${toolsList}

## Skill Levels

- **Expert**: [你达到专家水平的领域]
- **Advanced**: [你擅长但不是专家的领域]
- **Intermediate**: [你有基本能力的领域]
- **Learning**: [你正在学习的领域]

## Limitations

- [当前无法做到的事情]
- [需要额外学习才能做到的事情]

## Growth Path

- [短期目标]
- [中期目标]
- [长期目标]
\`\`\`

请根据用户需求和已配置的工具生成完整的 CAPABILITIES.md 文档。`

    // @ts-ignore
    const agentService = (await import('./agentService')).default
    const result = await agentService.queryClaudeAgentDirect({
      apiKey: typeof window !== 'undefined' ? (localStorage.getItem('claude_api_key') || '') : '',
      prompt,
      systemPrompt: '你是一个专业的智能体能力分析师，擅长分析、定义和描述 AI 智能体的能力体系。',
      model: 'claude-3-sonnet-20240229',
      maxTokens: 2500,
      temperature: 0.6,
    })

    const content = this.extractMarkdownContent(result.content)

    return {
      content,
      type: DocumentTypes.CAPABILITIES,
      metadata: {
        generatedAt: Date.now(),
        model: 'claude-3-sonnet-20240229',
        userPrompt,
        mcpTools,
      },
    }
  }

  /**
   * 生成 CONSTRAINTS.md - 约束条件文档
   */
  async generateConstraintsDocument(userPrompt: string): Promise<Document> {
    const prompt = `你是一个智能体行为规范专家。根据用户的需求，创建一个智能体的 CONSTRAINTS.md 文档。

用户需求: "${userPrompt}"

请生成 CONSTRAINTS.md 文档，格式如下：

\`\`\`markdown
# CONSTRAINTS.md - What You Must Not Do

## Absolute Boundaries

- **Never** reveal private user data
- **Never** perform actions without user consent
- **Never** make irreversible changes without confirmation

## Safety Constraints

### Data Privacy
- [数据隐私相关的约束]
- [数据保护规则]

### Content Moderation
- [内容审核规则]
- [禁止生成的内容类型]

### Ethical Guidelines
- [伦理规范]
- [道德准则]

## Operational Constraints

### Resource Usage
- [资源使用限制]
- [性能约束]

### Response Quality
- [响应质量要求]
- [准确性要求]

## Context-Specific Constraints

### [场景1]
- [该场景下的特定约束]

### [场景2]
- [该场景下的特定约束]

## Error Handling

- [遇到错误时的处理规则]
- [无法完成任务时的应对策略]

## Compliance Requirements

- [需要遵守的法律法规]
- [行业标准要求]
- [公司内部规范]
\`\`\`

请根据用户需求生成完整的 CONSTRAINTS.md 文档，确保智能体的行为受到合理约束。`

    // @ts-ignore
    const agentService = (await import('./agentService')).default
    const result = await agentService.queryClaudeAgentDirect({
      apiKey: typeof window !== 'undefined' ? (localStorage.getItem('claude_api_key') || '') : '',
      prompt,
      systemPrompt: '你是一个专业的智能体行为规范专家，擅长制定合理、有效的 AI 智能体行为约束。',
      model: 'claude-3-sonnet-20240229',
      maxTokens: 2000,
      temperature: 0.5,
    })

    const content = this.extractMarkdownContent(result.content)

    return {
      content,
      type: DocumentTypes.CONSTRAINTS,
      metadata: {
        generatedAt: Date.now(),
        model: 'claude-3-sonnet-20240229',
        userPrompt,
      },
    }
  }

  /**
   * 生成 TOOLS.md - 工具配置文档
   */
  async generateToolsDocument(mcpTools: McpTool[] = []): Promise<Document> {
    if (mcpTools.length === 0) {
      // 返回空模板
      return {
        content: `# TOOLS.md - Available Tools

## MCP Tools

No tools configured yet. Add tools via the MCP configuration.

## Future Tools

This section will be populated as new tools are added.`,
        type: DocumentTypes.TOOLS,
        metadata: {
          generatedAt: Date.now(),
          mcpTools: [],
        },
      }
    }

    const prompt = `你是一个智能体工具集成专家。根据提供的 MCP 工具配置，创建一个 TOOLS.md 文档。

MCP 工具配置:
\`\`\`json
${JSON.stringify(mcpTools, null, 2)}
\`\`\`

请生成 TOOLS.md 文档，格式如下：

\`\`\`markdown
# TOOLS.md - Available Tools

## MCP Tools Integration

### Tool 1: [工具名称]

- **Endpoint**: [工具端点]
- **Port**: [端口]
- **Description**: [工具描述]
- **Usage Example**: [使用示例]
- **Limitations**: [使用限制]

## Tool Categories

### Category 1: [分类名称]
- [工具1]
- [工具2]

### Category 2: [分类名称]
- [工具3]
- [工具4]

## Best Practices

- [工具使用的最佳实践]
- [组合使用工具的建议]

## Troubleshooting

- [常见问题及解决方案]
- [错误处理建议]
\`\`\`

请根据 MCP 工具配置生成完整的 TOOLS.md 文档。`

    // @ts-ignore
    const agentService = (await import('./agentService')).default
    const result = await agentService.queryClaudeAgentDirect({
      apiKey: typeof window !== 'undefined' ? (localStorage.getItem('claude_api_key') || '') : '',
      prompt,
      systemPrompt: '你是一个专业的智能体工具集成专家，擅长文档化 MCP 工具配置。',
      model: 'claude-3-haiku-20240307',
      maxTokens: 2000,
      temperature: 0.3,
    })

    const content = this.extractMarkdownContent(result.content)

    return {
      content,
      type: DocumentTypes.TOOLS,
      metadata: {
        generatedAt: Date.now(),
        model: 'claude-3-haiku-20240307',
        mcpTools,
      },
    }
  }

  /**
   * 生成 MEMORY.md - 记忆配置文档
   */
  async generateMemoryDocument(options: Partial<MemoryConfig> = {}): Promise<Document> {
    const {
      enableLongTerm = true,
      enableWorkingMemory = true,
      memoryLimit = 1000,
      ipnsName = null,
    } = options

    const content = `# MEMORY.md - How You Remember

## Memory Configuration

- **Long-term Memory**: ${enableLongTerm ? 'Enabled' : 'Disabled'}
- **Working Memory**: ${enableWorkingMemory ? 'Enabled' : 'Disabled'}
- **Memory Limit**: ${memoryLimit} entries
- **IPNS Name**: ${ipnsName || 'Not configured'}

## What to Remember

### Important Information
- User preferences and settings
- Key decisions and agreements
- Project context and history

### Forget After Session
- Temporary calculations
- Transient state
- One-time queries

## Memory Organization

### Long-term Storage
- [长期记忆的组织方式]
- [索引策略]
- [检索方法]

### Working Memory
- [工作记忆的内容]
- [保持策略]
- [更新频率]

## Privacy & Security

- [隐私保护措施]
- [敏感信息处理规则]
- [数据加密策略]

## Memory Maintenance

### Regular Updates
- [定期更新策略]
- [信息验证流程]

### Cleanup
- [清理过期信息的规则]
- [去重策略]

## Context Retrieval

### Priority System
1. [最高优先级信息]
2. [中等优先级信息]
3. [低优先级信息]

### Search Strategy
- [检索策略]
- [相关性评分方法]
`

    return {
      content,
      type: DocumentTypes.MEMORY,
      metadata: {
        generatedAt: Date.now(),
        options: options as MemoryConfig,
      },
    }
  }

  /**
   * 生成 AGENTS.md - 工作空间规则文档
   */
  async generateAgentsDocument(userPrompt: string): Promise<Document> {
    const prompt = `你是一个智能体工作流专家。根据用户的需求，创建一个 AGENTS.md 文档。

用户需求: "${userPrompt}"

请生成 AGENTS.md 文档，格式如下：

\`\`\`markdown
# AGENTS.md - Your Workspace

This folder is home. Treat it that way.

## Every Session

Before doing anything else:
1. Read SOUL.md — this is who you are
2. Read IDENTITY.md — this is your identity
3. Read CAPABILITIES.md — this is what you can do
4. Read CONSTRAINTS.md — this is what you must not do
5. Read MEMORY.md — this is how you remember
6. Check recent interactions in working memory

## Session Workflow

### Initialization
1. [初始化步骤]
2. [状态检查]
3. [上下文加载]

### Task Execution
1. [任务分析]
2. [工具选择]
3. [执行计划]
4. [结果验证]

### Cleanup
1. [状态保存]
2. [临时数据清理]
3. [记忆更新]

## Collaboration

### With Other Agents
- [与其他智能体的协作方式]
- [信息共享规则]

### With Humans
- [与人类的交互模式]
- [沟通最佳实践]

## Error Handling

- [错误检测机制]
- [恢复策略]
- [用户通知规则]

## Performance Metrics

- [性能指标]
- [优化目标]
\`\`\`

请根据用户需求生成完整的 AGENTS.md 文档。`

    // @ts-ignore
    const agentService = (await import('./agentService')).default
    const result = await agentService.queryClaudeAgentDirect({
      apiKey: typeof window !== 'undefined' ? (localStorage.getItem('claude_api_key') || '') : '',
      prompt,
      systemPrompt: '你是一个专业的智能体工作流专家，擅长设计高效的智能体工作流程。',
      model: 'claude-3-sonnet-20240229',
      maxTokens: 2000,
      temperature: 0.6,
    })

    const content = this.extractMarkdownContent(result.content)

    return {
      content,
      type: DocumentTypes.AGENTS,
      metadata: {
        generatedAt: Date.now(),
        model: 'claude-3-sonnet-20240229',
        userPrompt,
      },
    }
  }

  /**
   * 生成完整的智能体文档集合
   */
  async generateFullDocumentSet(
    userPrompt: string,
    options: {
      name?: string
      avatar?: string | null
      emoji?: string
      mcpTools?: McpTool[]
      memoryConfig?: Partial<MemoryConfig>
    } = {}
  ): Promise<AgentDocuments> {
    console.log('[AgentDocumentService] 开始生成完整文档集...')

    const {
      name,
      avatar,
      emoji,
      mcpTools = [],
      memoryConfig = {},
    } = options

    // 并行生成所有文档
    const [
      soulDoc,
      identityDoc,
      capabilitiesDoc,
      constraintsDoc,
      toolsDoc,
      memoryDoc,
      agentsDoc,
    ] = await Promise.all([
      this.generateSoulDocument(userPrompt),
      this.generateIdentityDocument(userPrompt, { name, avatar, emoji }),
      this.generateCapabilitiesDocument(userPrompt, mcpTools),
      this.generateConstraintsDocument(userPrompt),
      this.generateToolsDocument(mcpTools),
      this.generateMemoryDocument(memoryConfig),
      this.generateAgentsDocument(userPrompt),
    ])

    const documents = new AgentDocuments({
      soul: soulDoc,
      identity: identityDoc,
      capabilities: capabilitiesDoc,
      constraints: constraintsDoc,
      tools: toolsDoc,
      memory: memoryDoc,
      agents: agentsDoc,
    })

    console.log('[AgentDocumentService] 文档集生成完成:', documents)
    return documents
  }

  /**
   * 上传文档到 IPFS
   */
  async uploadDocumentToIpfs(document: Document): Promise<string> {
    try {
      const result = await ipfsService.addFile(
        new Blob([document.content], { type: 'text/markdown' }),
        `${document.type.toLowerCase()}.md`
      )

      console.log(`[AgentDocumentService] ${document.type} 文档上传成功:`, result.cid)

      return result.cid
    } catch (error: any) {
      console.error(`[AgentDocumentService] ${document.type} 文档上传失败:`, error)
      throw error
    }
  }

  /**
   * 上传完整文档集到 IPFS
   */
  async uploadFullDocumentSet(documents: AgentDocuments): Promise<Record<string, string>> {
    console.log('[AgentDocumentService] 开始上传文档集到 IPFS...')

    const uploadPromises: Array<Promise<{ type: string; cid: string }>> = []

    // 并行上传所有文档
    if (documents.soul) {
      uploadPromises.push(
        this.uploadDocumentToIpfs(documents.soul).then(cid => ({ type: 'soul', cid }))
      )
    }
    if (documents.identity) {
      uploadPromises.push(
        this.uploadDocumentToIpfs(documents.identity).then(cid => ({ type: 'identity', cid }))
      )
    }
    if (documents.capabilities) {
      uploadPromises.push(
        this.uploadDocumentToIpfs(documents.capabilities).then(cid => ({ type: 'capabilities', cid }))
      )
    }
    if (documents.constraints) {
      uploadPromises.push(
        this.uploadDocumentToIpfs(documents.constraints).then(cid => ({ type: 'constraints', cid }))
      )
    }
    if (documents.tools) {
      uploadPromises.push(
        this.uploadDocumentToIpfs(documents.tools).then(cid => ({ type: 'tools', cid }))
      )
    }
    if (documents.memory) {
      uploadPromises.push(
        this.uploadDocumentToIpfs(documents.memory).then(cid => ({ type: 'memory', cid }))
      )
    }
    if (documents.agents) {
      uploadPromises.push(
        this.uploadDocumentToIpfs(documents.agents).then(cid => ({ type: 'agents', cid }))
      )
    }

    const results = await Promise.all(uploadPromises)

    // 构建 CID 映射
    const cidsMap: Record<string, string> = {}
    results.forEach(({ type, cid }) => {
      cidsMap[type] = cid
      const doc = (documents as any)[type]
      if (doc && typeof doc === 'object' && 'cid' in doc === false) {
        doc.cid = cid
        doc.uploadedAt = Date.now()
      }
    })

    console.log('[AgentDocumentService] 文档集上传完成:', cidsMap)

    return cidsMap
  }

  /**
   * 从文档集构建系统提示词
   */
  buildSystemPromptFromDocuments(documents: AgentDocuments): string {
    const parts: string[] = []

    // SOUL.md - 性格与哲学
    if (documents.soul?.content) {
      parts.push('=== SOUL ===\n' + documents.soul.content)
    }

    // IDENTITY.md - 身份信息
    if (documents.identity?.content) {
      parts.push('\n=== IDENTITY ===\n' + documents.identity.content)
    }

    // CAPABILITIES.md - 能力列表
    if (documents.capabilities?.content) {
      parts.push('\n=== CAPABILITIES ===\n' + documents.capabilities.content)
    }

    // CONSTRAINTS.md - 约束条件
    if (documents.constraints?.content) {
      parts.push('\n=== CONSTRAINTS ===\n' + documents.constraints.content)
    }

    // TOOLS.md - 工具配置
    if (documents.tools?.content) {
      parts.push('\n=== TOOLS ===\n' + documents.tools.content)
    }

    // MEMORY.md - 记忆配置
    if (documents.memory?.content) {
      parts.push('\n=== MEMORY ===\n' + documents.memory.content)
    }

    // AGENTS.md - 工作空间规则
    if (documents.agents?.content) {
      parts.push('\n=== AGENTS ===\n' + documents.agents.content)
    }

    return parts.join('\n')
  }

  /**
   * 从 CID 获取文档内容
   */
  async getDocumentByCid(cid: string): Promise<string> {
    // 检查缓存
    if (this.cache.has(cid)) {
      console.log(`[AgentDocumentService] 从缓存获取文档: ${cid}`)
      return this.cache.get(cid)!
    }

    // 从 IPFS 获取
    try {
      const content = await ipfsService.getFileContent(cid)
      this.cache.set(cid, content)
      return content
    } catch (error: any) {
      console.error(`[AgentDocumentService] 获取文档失败: ${cid}`, error)
      throw error
    }
  }

  /**
   * 提取 Markdown 内容（从 AI 响应中）
   */
  extractMarkdownContent(text: string): string {
    // 查找 ```markdown ... ``` 代码块
    const markdownCodeBlock = text.match(/```markdown\n([\s\S]*?)\n```/)
    if (markdownCodeBlock) {
      return markdownCodeBlock[1].trim()
    }

    // 查找 ``` ... ``` 代码块
    const codeBlock = text.match(/```\n([\s\S]*?)\n```/)
    if (codeBlock) {
      return codeBlock[1].trim()
    }

    // 如果没有代码块，返回整个文本
    return text.trim()
  }

  /**
   * 清除缓存
   */
  clearCache(): void {
    this.cache.clear()
    console.log('[AgentDocumentService] 缓存已清除')
  }
}

// 导出单例
export default new AgentDocumentService()
