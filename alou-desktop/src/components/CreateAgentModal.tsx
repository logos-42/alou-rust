import { useMemo, useState, useEffect, useCallback } from 'react'
import agentAssetsService from '@/services/agentAssetsService'
import avatarService from '@/services/avatarService'
import agentService from '@/services/agentService'
// import agentDocumentService, { AgentDocuments } from '@/services/agentDocumentService'
import ipfsService from '@/services/ipfsService'
import { useI18n } from '@/hooks/useI18n'
import { setDiapIdentitySafe, hasDiapIdentitySafe } from '@/utils/diapIdentityManager'
import { setDiapIdentity } from '@/utils/memoryStorage'
import CloseIcon from '@/assets/关闭0.3.png'
import asyncDiapCreationService from '@/services/asyncDiapCreationService'
import CopyIcon from '@/assets/复制.png'
import AgentDocumentsViewer from './AgentDocumentsViewer'
import './CreateAgentModal.css'

const DEFAULT_IPFS_API = import.meta.env.VITE_IPFS_API_URL || 'http://127.0.0.1:5001'
const DEFAULT_IPFS_GATEWAY = import.meta.env.VITE_IPFS_GATEWAY_URL || 'http://127.0.0.1:8080'

function CreateAgentModal({ isOpen, onClose, onSubmit, sessionId, onEarlyChannel }) {
  const { t } = useI18n()
  const [name, setName] = useState('')
  const [roleDescription, setRoleDescription] = useState(t('agent.create.role.default'))
  const [avatarFile, setAvatarFile] = useState(null)
  const [avatarPreview, setAvatarPreview] = useState(null)
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

  // Dicebear 头像样式选择
  const [showDicebearStyles, setShowDicebearStyles] = useState(false)

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

      // TODO: 实现文档生成功能
      // 生成完整文档集（纯 AI 调用，不依赖 IPFS）
      // setDocumentGenerationProgress({ message: '生成 SOUL.md 性格与哲学...', stage: 'soul' })
      // const documents = await agentDocumentService.generateFullDocumentSet(userPrompt, {
      //   name: fallbackName,
      //   avatar: avatarPreview,
      //   emoji: '🤖',
      //   memoryConfig: {
      //     enableLongTerm: true,
      //     enableWorkingMemory: true,
      //     memoryLimit: 1000,
      //   },
      // })

      console.log('[CreateAgentModal] 文档生成功能暂时禁用')
      // setAgentDocuments(documents)
      setDocumentsGenerated(true)
      setDocumentGenerationProgress(null)

      // 尝试上传到 IPFS（可选，失败不影响创建）
      const documentCids = {}
      // try {
      //   setDocumentGenerationProgress({ message: '上传文档到 IPFS（可选）...', stage: 'uploading' })
      //   const isRunning = await ipfsService.isNodeRunning()
      //   if (isRunning) {
      //     documentCids = await agentDocumentService.uploadFullDocumentSet(documents)
      //     console.log('[CreateAgentModal] 文档已上传到 IPFS:', documentCids)
      //   }
      // } catch (ipfsErr) {
      //   console.warn('[CreateAgentModal] IPFS 上传失败，文档已本地保存:', ipfsErr.message)
      // } finally {
      //   setDocumentGenerationProgress(null)
      // }

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

  const handleSwitchDicebearStyle = (style) => {
    const seed = name.trim() || `agent_${Date.now()}`
    const newAvatar = avatarService.generateDicebearAvatar(seed, style)
    setAvatarPreview(newAvatar)
    setAvatarFile(null)
    setShowDicebearStyles(false)
  }

  const handleUseDefaultAvatar = () => {
    const newAvatar = avatarService.getFallbackAvatar()
    setAvatarPreview(newAvatar)
    setAvatarFile(null)
    setShowDicebearStyles(false)
  }

  const dicebearStyles = [
    { name: 'identicon', label: '🔷 Identicon', description: '几何图案' },
    { name: 'avataaars', label: '😊 Avataaars', description: '卡通头像' },
    { name: 'bottts', label: '🤖 Bottts', description: '机器人' },
    { name: 'lorelei', label: '🧚 Lorelei', description: '精灵风格' },
    { name: 'notionists', label: '📝 Notionists', description: '简约风格' },
    { name: 'fun-emoji', label: '😄 Fun Emoji', description: '表情符号' },
    { name: 'shapes', label: '🔶 Shapes', description: '抽象形状' },
  ]

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
      }
      // TODO: Implement getAllCids() method
      // else if (useDocumentBasedCreation && documentsGenerated && agentDocuments) {
      //   // 已生成文档，使用现有的 CID
      //   documentCids = agentDocuments.getAllCids()
      // }

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

      // 1. DIAP Identity 改为异步创建，不阻塞智能体创建流程
      // 智能体创建完成后，在后台异步创建 DIAP 身份
      const shouldCreateDiapAsync = ipfsAvailable
      console.log('[CreateAgentModal] DIAP 身份将异步创建:', shouldCreateDiapAsync)

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

      // 3. 构建完整的智能体数据（只调用一次onSubmit）
      const agentData = {
        name: fallbackName,
        roleDescription: finalRoleDescription,
        avatar_cid: avatarCid, // IPFS CID（有 IPFS 时使用）
        avatar_url: avatarCid ? null : avatarBase64, // 本地 base64 data URL（无 IPFS 时使用）
        // diapIdentity will be added asynchronously after creation
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
        hasDocuments: !!agentData.documentCids,
        sessionId,
        willCreateDiapAsync: shouldCreateDiapAsync
      })

      // 5. 提交完整的智能体信息（只调用一次）
      await onSubmit(agentData)

      // 6. 异步创建 DIAP 身份（不阻塞 UI）
      if (shouldCreateDiapAsync) {
        console.log('[CreateAgentModal] 启动异步 DIAP 身份创建...')
        // 静默启动后台 DIAP 身份创建，不需要处理错误（服务内部已处理）
        asyncDiapCreationService.startDiapCreation(
          sessionId,
          {
            name: fallbackName,
            roleDescription: finalRoleDescription,
            avatarCid: avatarCid,
            customPrompt: customPrompt
          },
          {
            ipfsApiUrl: DEFAULT_IPFS_API,
            ipfsGatewayUrl: DEFAULT_IPFS_GATEWAY
          }
        )
      }

      // 7. 关闭模态框
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
              <div className="agent-modal__avatar-actions">
                <label className="agent-modal__upload-button">
                  {t('agent.create.avatar.select')}
                  <input type="file" accept="image/*" onChange={handleAvatarChange} />
                </label>
                <button
                  type="button"
                  className="agent-modal__avatar-style-btn"
                  onClick={() => setShowDicebearStyles(!showDicebearStyles)}
                >
                  🎨 选择风格
                </button>
                <button
                  type="button"
                  className="agent-modal__avatar-default-btn"
                  onClick={handleUseDefaultAvatar}
                >
                  🔄 默认头像
                </button>
              </div>
            </div>
            {avatarUploadError && (
              <div className="agent-modal__avatar-warning">
                ⚠️ {avatarUploadError}
              </div>
            )}

            {/* Dicebear 风格选择器 */}
            {showDicebearStyles && (
              <div className="agent-modal__dicebear-styles">
                <div className="agent-modal__dicebear-header">
                  <span>选择头像风格</span>
                  <button
                    type="button"
                    className="agent-modal__dicebear-close"
                    onClick={() => setShowDicebearStyles(false)}
                  >
                    ✕
                  </button>
                </div>
                <div className="agent-modal__dicebear-grid">
                  {dicebearStyles.map((style) => (
                    <button
                      key={style.name}
                      type="button"
                      className="agent-modal__dicebear-style"
                      onClick={() => handleSwitchDicebearStyle(style.name)}
                    >
                      <div className="agent-modal__dicebear-preview">
                        <img
                          src={avatarService.generateDicebearAvatar(name.trim() || 'agent', style.name)}
                          alt={style.label}
                        />
                      </div>
                      <div className="agent-modal__dicebear-info">
                        <span className="agent-modal__dicebear-label">{style.label}</span>
                        <span className="agent-modal__dicebear-desc">{style.description}</span>
                      </div>
                    </button>
                  ))}
                </div>
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
