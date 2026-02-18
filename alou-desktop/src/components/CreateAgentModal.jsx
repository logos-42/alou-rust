import { useMemo, useState, useEffect, useCallback } from 'react'
import agentAssetsService from '@/services/agentAssetsService'
import agentService from '@/services/agentService'
import agentDocumentService, { AgentDocuments, DocumentTypes } from '@/services/agentDocumentService'
import ipfsService from '@/services/ipfsService'
import { useI18n } from '@/hooks/useI18n'
import { setDiapIdentitySafe, hasDiapIdentitySafe } from '@/utils/diapIdentityManager'
import { setDiapIdentity } from '@/utils/memoryStorage'
import CloseIcon from '@/assets/关闭0.3.png'
import CopyIcon from '@/assets/复制.png'
import AgentDocumentsViewer from './AgentDocumentsViewer'
import './CreateAgentModal.css'

const DEFAULT_MCP_CODE = `{
  "ports": []
}`

const DEFAULT_IPFS_API = import.meta.env.VITE_IPFS_API_URL || 'http://127.0.0.1:5001'
const DEFAULT_IPFS_GATEWAY = import.meta.env.VITE_IPFS_GATEWAY_URL || 'http://127.0.0.1:8080'

function CreateAgentModal({ isOpen, onClose, onSubmit, sessionId, onEarlyChannel }) {
  const { t } = useI18n()
  const [name, setName] = useState('')
  const [roleDescription, setRoleDescription] = useState(t('agent.create.role.default'))
  const [avatarFile, setAvatarFile] = useState(null)
  const [avatarPreview, setAvatarPreview] = useState(null)
  const [mcpCode, setMcpCode] = useState(DEFAULT_MCP_CODE)
  const [mcpTools, setMcpTools] = useState([])
  const [mcpParseError, setMcpParseError] = useState(null)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState(null)
  const [avatarUploadError, setAvatarUploadError] = useState(null)

  // 文档化创建相关状态（默认开启，AI 生成文档集）
  const [useDocumentBasedCreation, setUseDocumentBasedCreation] = useState(true)
  const [isGeneratingDocuments, setIsGeneratingDocuments] = useState(false)
  const [documentsGenerated, setDocumentsGenerated] = useState(false)
  const [agentDocuments, setAgentDocuments] = useState(null)
  const [documentGenerationProgress, setDocumentGenerationProgress] = useState(null)
  const [showDocumentPreview, setShowDocumentPreview] = useState(false)

  // 解析 MCP 代码并提取工具名称
  const parseMcpCode = useCallback((code) => {
    try {
      setMcpParseError(null)

      // 移除注释、控制字符并修剪空白字符
      let cleanedCode = code
        .replace(/\/\/.*$/gm, '')  // 移除单行注释
        .replace(/\/\*[\s\S]*?\*\//g, '')  // 移除多行注释
        .trim()  // 移除首尾空白

      // 更彻底地移除控制字符
      cleanedCode = cleanedCode
        // 移除所有控制字符（除了换行、回车、制表符）
        .replace(/[\x00-\x08\x0B\x0C\x0E-\x1F\x7F]/g, '')
        // 移除零宽字符
        .replace(/[\u200B-\u200D\uFEFF]/g, '')
        // 修复可能被破坏的 URL（如果 :// 被分割）
        .replace(/"endpoint"\s*:\s*"([^"]*):\s*\/\/([^"]*)"/g, '"endpoint": "$1://$2"')

      if (!cleanedCode) {
        throw new Error('MCP 配置不能为空')
      }

      let config = null

      // 尝试多种解析方式
      try {
        // 首先尝试作为 JSON 解析
        config = JSON.parse(cleanedCode)
      } catch (jsonError) {
        console.log('JSON 解析失败:', jsonError.message)
        console.log('失败位置:', jsonError)

        // 如果 JSON 解析失败，尝试作为 JavaScript 对象解析
        try {
          // 确保代码以有效的 JavaScript 对象开始
          const jsCode = cleanedCode.trim()
          // 如果代码不以 { 开头，添加它
          const finalCode = jsCode.startsWith('{') ? jsCode : `{${jsCode}}`
          // 使用 Function 构造器来安全执行代码
          config = new Function('return ' + finalCode)()
          console.log('JavaScript 解析成功')
        } catch (jsError) {
          console.log('JavaScript 解析失败:', jsError.message)
          throw new Error(`解析失败: ${jsonError.message} (JSON) 或 ${jsError.message} (JS)`)
        }
      }

      if (!config || typeof config !== 'object') {
        throw new Error('配置必须是一个对象')
      }

      const ports = config.ports || []
      if (!Array.isArray(ports)) {
        throw new Error('ports 必须是一个数组')
      }

      // 提取工具名称（从 label 字段）
      const tools = ports
        .filter(port => port && port.label)
        .map(port => ({
          name: port.label,
          endpoint: port.endpoint || '',
          port: port.port || '',
          description: port.description || ''
        }))

      setMcpTools(tools)
      return { ports, tools }
    } catch (err) {
      setMcpParseError(err.message || '解析失败')
      setMcpTools([])
      return { ports: [], tools: [] }
    }
  }, [])

  // 当 MCP 代码改变时自动解析
  useEffect(() => {
    if (mcpCode.trim()) {
      parseMcpCode(mcpCode)
    } else {
      setMcpTools([])
      setMcpParseError(null)
    }
  }, [mcpCode, parseMcpCode])

  // 生成文档集合（不依赖 IPFS，AI 生成是核心，IPFS 上传是可选后台操作）
  const generateDocuments = useCallback(async () => {
    const fallbackName = name.trim() || 'agent'
    const finalRoleDescription = roleDescription.trim() || t('agent.create.role.default')

    setIsGeneratingDocuments(true)
    setDocumentGenerationProgress({ message: '正在生成智能体性格文档...', stage: 'soul' })
    setError(null)

    try {
      // 构建用户提示
      const userPrompt = `创建一个名为"${fallbackName}"的智能体，角色描述：${finalRoleDescription}`

      // 生成完整文档集（纯 AI 调用，不依赖 IPFS）
      setDocumentGenerationProgress({ message: '生成 SOUL.md 性格与哲学...', stage: 'soul' })
      const documents = await agentDocumentService.generateFullDocumentSet(userPrompt, {
        name: fallbackName,
        avatar: avatarPreview,
        emoji: '🤖',
        mcpTools: mcpTools.filter(tool => tool.name),
        memoryConfig: {
          enableLongTerm: true,
          enableWorkingMemory: true,
          memoryLimit: 1000,
        },
      })

      console.log('[CreateAgentModal] 文档生成完成:', Object.keys(documents))
      setAgentDocuments(documents)
      setDocumentsGenerated(true)
      setDocumentGenerationProgress(null)

      // 尝试上传到 IPFS（可选，失败不影响创建）
      let documentCids = {}
      try {
        setDocumentGenerationProgress({ message: '上传文档到 IPFS（可选）...', stage: 'uploading' })
        const isRunning = await ipfsService.isNodeRunning()
        if (isRunning) {
          documentCids = await agentDocumentService.uploadFullDocumentSet(documents)
          console.log('[CreateAgentModal] 文档已上传到 IPFS:', documentCids)
        }
      } catch (ipfsErr) {
        console.warn('[CreateAgentModal] IPFS 上传失败，文档已本地保存:', ipfsErr.message)
      } finally {
        setDocumentGenerationProgress(null)
      }

      setIsGeneratingDocuments(false)
      return documentCids

    } catch (err) {
      console.error('[CreateAgentModal] 文档生成失败:', err)
      setIsGeneratingDocuments(false)
      setDocumentGenerationProgress(null)
      setError(err?.message || '文档生成失败，请检查 API Key 配置')
      throw err
    }
  }, [name, roleDescription, avatarPreview, mcpTools, t])

  // 提前返回 null，但确保所有 hooks 都在条件之外定义
  if (!isOpen) {
    return null
  }

  const handleAvatarChange = (event) => {
    const file = event.target.files?.[0]
    if (!file) {
      setAvatarFile(null)
      setAvatarPreview(null)
      return
    }
    setAvatarFile(file)
    const reader = new FileReader()
    reader.onload = () => {
      if (typeof reader.result === 'string') {
        setAvatarPreview(reader.result)
      }
    }
    reader.readAsDataURL(file)
  }

  const handleInternalSubmit = async (event) => {
    event.preventDefault()
    setError(null)
    setIsLoading(true)

    const fallbackName = name.trim() || 'agent'
    const finalRoleDescription = roleDescription.trim() || t('agent.create.role.default')

    try {
      // 如果启用了文档化创建，先生成文档
      let documentCids = {}
      if (useDocumentBasedCreation && !documentsGenerated) {
        try {
          documentCids = await generateDocuments()
        } catch (docErr) {
          // 文档生成失败，询问用户是否继续
          const continueWithoutDocs = confirm(
            `文档生成失败：${docErr.message}\n\n是否继续创建智能体（不使用文档化配置）？`
          )
          if (!continueWithoutDocs) {
            setIsLoading(false)
            return
          }
        }
      } else if (useDocumentBasedCreation && documentsGenerated && agentDocuments) {
        // 已生成文档，使用现有的 CID
        documentCids = agentDocuments.getAllCids()
      }

      // 检查 IPFS 是否可用（可选，不阻塞创建）
      let ipfsAvailable = false
      try {
        const isRunning = await ipfsService.isNodeRunning()
        if (isRunning) {
          const apiReady = await ipfsService.waitForApiReady(3, 500) // 短暂等待
          ipfsAvailable = apiReady.success
        }
      } catch (ipfsErr) {
        console.warn('[CreateAgentModal] IPFS 不可用，将使用本地存储模式:', ipfsErr.message)
      }
      console.log(`[CreateAgentModal] IPFS 可用: ${ipfsAvailable}`)

      // 1. 尝试创建 DIAP Identity（需要 IPFS，失败则跳过）
      let diapIdentity = null
      if (ipfsAvailable) {
        try {
          console.log('[CreateAgentModal] 开始创建 DIAP Identity...')
          const { default: diapIntegrationService } = await import('../services/diapIntegrationService')
          const result = await diapIntegrationService.createCompleteDiapIdentity(sessionId, {
            agentName: fallbackName,
            agentDescription: finalRoleDescription,
          })
          diapIdentity = result.identity
          console.log('[CreateAgentModal] DIAP Identity 创建成功:', diapIdentity?.did)
        } catch (err) {
          console.error('[CreateAgentModal] DIAP Identity 创建失败:', err)
          console.warn('[CreateAgentModal] 将创建没有 DIAP Identity 的智能体')
        }
      } else {
        console.log('[CreateAgentModal] IPFS 不可用，跳过 DIAP Identity 创建')
      }

      // 2. 头像处理：IPFS 可用则上传，否则使用 base64 本地存储
      let avatarCid = null
      // 本地 base64 作为头像备用（即使没有 IPFS 也能显示头像）
      const avatarBase64 = avatarPreview || null

      if (avatarFile && ipfsAvailable) {
        try {
          console.log('[CreateAgentModal] 开始上传头像到 IPFS...')
          setAvatarUploadError(null)
          const uploaded = await agentAssetsService.uploadAvatar(avatarFile, { sessionId })
          avatarCid = uploaded?.cid || null
          console.log('[CreateAgentModal] 头像上传成功:', avatarCid)
        } catch (err) {
          console.error('[CreateAgentModal] 头像上传失败:', err)
          setAvatarUploadError(err.message || '头像上传失败，已使用本地存储')
        }
      } else if (avatarFile && !ipfsAvailable) {
        console.log('[CreateAgentModal] IPFS 不可用，头像将使用 base64 本地存储')
      }

      // 3. 解析 MCP 配置（IPFS 可用则上传，否则只本地保存）
      let mcpConfigCid = null
      let filteredPorts = []

      if (mcpCode.trim()) {
        try {
          console.log('[CreateAgentModal] 解析 MCP 配置...')
          const { ports } = parseMcpCode(mcpCode)
          filteredPorts = ports.filter((port) => port.label?.trim() || port.endpoint?.trim())

          if (filteredPorts.length > 0 && ipfsAvailable) {
            console.log('[CreateAgentModal] 开始上传 MCP 配置到 IPFS...')
            const uploadedConfig = await agentAssetsService.uploadMcpConfig(
              { ports: filteredPorts, generatedAt: Date.now() },
              { sessionId },
            )
            mcpConfigCid = uploadedConfig?.cid || null
            console.log('[CreateAgentModal] MCP 配置上传成功:', mcpConfigCid)
          } else if (filteredPorts.length > 0) {
            console.log('[CreateAgentModal] IPFS 不可用，MCP 配置将本地保存')
          }
        } catch (err) {
          console.error('[CreateAgentModal] MCP 配置处理失败:', err)
          // 不阻塞创建流程
        }
      }

      // 4. 提取文档系统提示词（用于增强智能体的深度）
      let customPrompt = null
      let documentsMap = null
      if (useDocumentBasedCreation && agentDocuments) {
        try {
          customPrompt = agentDocumentService.buildSystemPromptFromDocuments(agentDocuments)
          console.log('[CreateAgentModal] 已从文档集构建 customPrompt，长度:', customPrompt?.length)
          documentsMap = agentDocumentService.extractDocumentMap(agentDocuments)
          console.log('[CreateAgentModal] 已提取文档 map，文档数:', Object.keys(documentsMap).length)
        } catch (promptErr) {
          console.warn('[CreateAgentModal] 构建 customPrompt 失败，将忽略文档内容:', promptErr.message)
        }
      }

      // 4. 构建完整的智能体数据（只调用一次onSubmit）
      const agentData = {
        name: fallbackName,
        roleDescription: finalRoleDescription,
        avatar_cid: avatarCid, // IPFS CID（有 IPFS 时使用）
        avatar_url: avatarCid ? null : avatarBase64, // 本地 base64 data URL（无 IPFS 时使用）
        mcp_config_cid: mcpConfigCid, // 修复：使用下划线命名与agentStore保持一致
        mcp_ports: filteredPorts, // 修复：使用下划线命名与agentStore保持一致
        diapIdentity: diapIdentity ? {
          did: diapIdentity.did,
          cid: diapIdentity.cid,
          ipns: diapIdentity.ipns || '',
          public_key: diapIdentity.public_key
        } : null,
        sessionId,
        // 添加文档化配置
        useDocumentBasedCreation,
        documentCids: Object.keys(documentCids).length > 0 ? documentCids : null,
        // 添加文档系统提示词（AI 生成的 SOUL/IDENTITY/CAPABILITIES 等文档的组合）
        customPrompt,
        // 添加单独文档 map（用于 agent_document 工具的 read/update）
        documents: documentsMap,
        // 添加标识，表明这是完整的智能体数据
        isComplete: true,
      }

      console.log('[CreateAgentModal] 提交完整智能体数据:', {
        name: agentData.name,
        hasAvatar: !!avatarCid,
        hasMcp: !!mcpConfigCid,
        hasDiap: !!diapIdentity,
        hasDocuments: !!agentData.documentCids,
        sessionId
      })

      // 5. 提交完整的智能体信息（只调用一次）
      await onSubmit(agentData)

      // 6. 关闭模态框
      setIsLoading(false)
      onClose()

    } catch (err) {
      console.error('[CreateAgentModal] 创建智能体失败', err)
      setIsLoading(false)
      setError(err?.message || '创建失败，请稍后重试')
    }
  }


  return (
    <div className="agent-modal-backdrop">
      <div className="agent-modal">
        <div className="agent-modal__header">
          <div>
            <h2>{t('agent.create.title')}</h2>
            <p>{t('agent.create.subtitle')}</p>
          </div>
          <button type="button" onClick={onClose} className="agent-modal__close">
            <img src={CloseIcon} alt="关闭" />
          </button>
        </div>

        <form className="agent-modal__form" onSubmit={handleInternalSubmit}>
          <div className="agent-modal__field">
            <span>{t('agent.create.avatar.label')}</span>
            <div className="agent-modal__avatar-upload">
              {avatarPreview ? (
                <img src={avatarPreview} alt={t('agent.create.avatar.preview')} className="agent-modal__avatar-preview" />
              ) : (
                <div className="agent-modal__avatar-placeholder">{t('agent.create.avatar.preview')}</div>
              )}
              <label className="agent-modal__upload-button">
                {t('agent.create.avatar.select')}
                <input type="file" accept="image/*" onChange={handleAvatarChange} />
              </label>
            </div>
            {avatarUploadError && (
              <div className="agent-modal__avatar-warning">
                ⚠️ {avatarUploadError}
              </div>
            )}
          </div>

          <label className="agent-modal__field">
            <span>{t('agent.create.name.label')}</span>
            <input
              type="text"
              value={name}
              onChange={(event) => setName(event.target.value)}
              placeholder={t('agent.create.name.placeholder')}
            />
          </label>

          <label className="agent-modal__field">
            <span>{t('agent.create.role.label')}</span>
            <textarea
              value={roleDescription}
              onChange={(event) => setRoleDescription(event.target.value)}
              rows={4}
              placeholder={t('agent.create.role.placeholder')}
            />
          </label>

          {/* 文档化创建选项 */}
          <div className="agent-modal__field">
            <span>智能体配置方式</span>
            <div className="agent-modal__creation-mode">
              <label className="agent-modal__radio-label">
                <input
                  type="radio"
                  name="creationMode"
                  checked={useDocumentBasedCreation}
                  onChange={() => setUseDocumentBasedCreation(true)}
                />
                <div className="agent-modal__radio-content">
                  <strong>文档化配置（推荐）</strong>
                  <p>使用 AI 生成完整的文档集（SOUL.md, IDENTITY.md 等），创建更有深度的智能体</p>
                </div>
              </label>
              <label className="agent-modal__radio-label">
                <input
                  type="radio"
                  name="creationMode"
                  checked={!useDocumentBasedCreation}
                  onChange={() => setUseDocumentBasedCreation(false)}
                />
                <div className="agent-modal__radio-content">
                  <strong>传统配置</strong>
                  <p>使用简单的 JSON 配置，快速创建智能体</p>
                </div>
              </label>
            </div>
            {useDocumentBasedCreation && (
              <div className="agent-modal__document-actions">
                {!documentsGenerated ? (
                  <button
                    type="button"
                    className="agent-modal__generate-docs-btn"
                    onClick={generateDocuments}
                    disabled={isGeneratingDocuments || !name.trim()}
                  >
                    {isGeneratingDocuments ? (
                      <>
                        <span className="agent-modal__spinner"></span>
                        {documentGenerationProgress?.message || '生成中...'}
                      </>
                    ) : (
                      '生成智能体文档'
                    )}
                  </button>
                ) : (
                  <div className="agent-modal__docs-ready">
                    <span className="agent-modal__docs-ready-icon">✓</span>
                    文档已生成
                    <button
                      type="button"
                      className="agent-modal__preview-btn"
                      onClick={() => setShowDocumentPreview(!showDocumentPreview)}
                    >
                      {showDocumentPreview ? '隐藏' : '预览'}文档
                    </button>
                  </div>
                )}
              </div>
            )}
          </div>

          {/* 文档预览区域 */}
          {showDocumentPreview && agentDocuments && (
            <div className="agent-modal__document-preview">
              <AgentDocumentsViewer documents={agentDocuments} />
            </div>
          )}


          <div className="agent-modal__field">
            <span>{t('agent.create.mcp.label')}</span>
            <div className="agent-modal__mcp-hint">
              💡 <strong>重要提示：</strong>MCP工具是智能体的"能力"，没有工具配置的智能体将无法执行任何操作。
              请至少配置一个MCP工具端点，或使用默认配置。
            </div>
            <textarea
              className="agent-modal__code-editor"
              value={mcpCode}
              onChange={(event) => setMcpCode(event.target.value)}
              placeholder={DEFAULT_MCP_CODE}
              rows={12}
              spellCheck={false}
            />
            {mcpParseError && (
              <div className="agent-modal__mcp-error">{mcpParseError}</div>
            )}
            {mcpTools.length > 0 && (
              <div className="agent-modal__mcp-tools">
                <div className="agent-modal__mcp-tools-header">
                  <span>{t('agent.create.mcp.tools')} ({mcpTools.length})</span>
                </div>
                <div className="agent-modal__mcp-tools-list">
                  {mcpTools.map((tool, index) => (
                    <div key={index} className="agent-modal__mcp-tool-item">
                      <div className="agent-modal__mcp-tool-name">{tool.name}</div>
                      {tool.description && (
                        <div className="agent-modal__mcp-tool-desc">{tool.description}</div>
                      )}
                      {tool.endpoint && (
                        <div className="agent-modal__mcp-tool-endpoint">{tool.endpoint}</div>
                      )}
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>

          {error && <div className="agent-modal__error">{error}</div>}

          <div className="agent-modal__actions">
            <button type="button" className="ghost" onClick={onClose}>
              {t('common.cancel')}
            </button>
            <button type="submit" disabled={isLoading}>
              {isLoading ? t('agent.create.creating') : t('agent.create.submit')}
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}

export default CreateAgentModal
