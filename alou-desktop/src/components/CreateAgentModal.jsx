import { useMemo, useState, useEffect, useCallback } from 'react'
import agentAssetsService from '@/services/agentAssetsService'
import agentService from '@/services/agentService'
import diapService from '@/services/diapService'
import ipfsService from '@/services/ipfsService'
import { useI18n } from '@/hooks/useI18n'
import CloseIcon from '@/assets/关闭0.3.png'
import './CreateAgentModal.css'

const DEFAULT_MCP_CODE = `{
  ports: [
    {
      label: 'web_search',
      endpoint: 'wss://mcp-server.example.com/web-search',
      port: 443,
      description: 'Web搜索工具，可以搜索最新信息'
    },
    {
      label: 'blockchain',
      endpoint: 'wss://mcp-server.example.com/blockchain',
      port: 443,
      description: '区块链工具，可以查询余额、发送交易等'
    },
    {
      label: 'file_system',
      endpoint: 'ws://localhost:8765',
      port: 8765,
      description: '本地文件系统工具'
    }
  ]
}`

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

  // 解析 MCP 代码并提取工具名称
  const parseMcpCode = useCallback((code) => {
    try {
      setMcpParseError(null)
      
      // 移除注释
      const cleanedCode = code.replace(/\/\/.*$/gm, '').replace(/\/\*[\s\S]*?\*\//g, '')
      
      // 尝试解析为 JavaScript 对象
      // 使用 Function 构造器来安全执行代码
      const config = new Function('return ' + cleanedCode)()
      
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
    const tempId = `temp_${Date.now()}`

    try {
      // 1. 立即显示频道（快速反馈，使用临时标识）
      if (onEarlyChannel) {
        const earlyMetadata = {
          display_name: fallbackName,
          name: fallbackName,
          role_description: finalRoleDescription,
          avatar_cid: null,
          avatar_url: avatarPreview, // 使用本地预览图
          agent_type: 'claude_agent_sdk',
          cid: tempId,
          did: null,
          ipns: null,
          diapIdentity: null,
          sessionId,
          status: 'creating', // 标记为创建中
        }
        console.log('[CreateAgentModal] 立即显示频道（临时）:', tempId)
        onEarlyChannel(earlyMetadata)
      }

      // 2. 立即关闭模态框，让用户看到频道
      setIsLoading(false)
      onClose()

      // 3. 在后台异步完成剩余工作
      void (async () => {
        try {
          // 检查 IPFS 节点
          let isRunning = await ipfsService.isNodeRunning()
          if (!isRunning) {
            const startResult = await ipfsService.startNode(true)
            if (!startResult.success) {
              console.error('[CreateAgentModal] IPFS 节点启动失败')
              return
            }
          }

          // 等待 IPFS API 就绪
          const apiReady = await ipfsService.waitForApiReady(15, 1000)
          if (!apiReady.success) {
            console.error('[CreateAgentModal] IPFS API 未就绪:', apiReady.error)
            return
          }

          // 上传头像（优先上传，完成后立即更新频道）
          let avatarCid = null
          if (avatarFile) {
            try {
              console.log('[CreateAgentModal] 后台上传头像...')
              const uploaded = await agentAssetsService.uploadAvatar(avatarFile, { sessionId })
              avatarCid = uploaded?.cid || null
              console.log('[CreateAgentModal] 头像上传成功:', avatarCid)
              
              // 头像上传成功后，立即更新频道显示
              if (avatarCid && onEarlyChannel) {
                const earlyMetadata = {
                  display_name: fallbackName,
                  name: fallbackName,
                  role_description: finalRoleDescription,
                  avatar_cid: avatarCid,
                  avatar_url: null, // 清除本地预览，使用 CID
                  agent_type: 'claude_agent_sdk',
                  cid: tempId,
                  did: null,
                  ipns: null,
                  diapIdentity: null,
                  sessionId,
                  status: 'creating',
                }
                console.log('[CreateAgentModal] 头像上传完成，立即更新频道:', tempId)
                onEarlyChannel(earlyMetadata)
              }
            } catch (err) {
              console.error('[CreateAgentModal] 头像上传失败:', err)
            }
          }

          // 解析并上传 MCP 配置
          const { ports: parsedPorts } = parseMcpCode(mcpCode)
          const filteredPorts = parsedPorts
            .filter((port) => port && (port.label?.trim() || port.endpoint?.trim()))
            .map((port) => ({
              label: port.label || '',
              endpoint: port.endpoint || '',
              port: port.port ? Number(port.port) : undefined,
              description: port.description || '',
              protocol: port.protocol || 'http',
            }))

          let mcpConfigCid = null
          if (filteredPorts.length > 0) {
            try {
              console.log('[CreateAgentModal] 后台上传 MCP 配置...')
              const uploadedConfig = await agentAssetsService.uploadMcpConfig(
                { ports: filteredPorts, generatedAt: Date.now() },
                { sessionId },
              )
              mcpConfigCid = uploadedConfig?.cid || null
              console.log('[CreateAgentModal] MCP 配置上传成功:', mcpConfigCid)
            } catch (err) {
              console.error('[CreateAgentModal] MCP 配置上传失败:', err)
            }
          }

          // 创建 DIAP Identity
          let diapIdentity = null
          try {
            console.log('[CreateAgentModal] 后台创建 DIAP Identity...')
            diapIdentity = await diapService.createLocalIdentity({
              name: fallbackName,
              description: finalRoleDescription,
              sessionId,
              avatarCid,
              mcpConfigCid,
            })
            console.log('[CreateAgentModal] DIAP Identity 创建成功:', diapIdentity?.did)

            // 保存到 localStorage
            if (diapIdentity && sessionId && typeof window !== 'undefined') {
              localStorage.setItem(
                `diap_identity_${sessionId}`,
                JSON.stringify({
                  ipns: diapIdentity.ipns,
                  did: diapIdentity.did,
                  cid: diapIdentity.cid,
                  public_key: diapIdentity.public_key,
                  created_at: Date.now(),
                })
              )
            }
          } catch (err) {
            console.error('[CreateAgentModal] DIAP Identity 创建失败:', err)
          }

          // 提交完整信息，更新频道
          console.log('[CreateAgentModal] 后台提交完整智能体信息...')
          await onSubmit({
            name: fallbackName,
            roleDescription: finalRoleDescription,
            avatarCid,
            mcpConfigCid,
            mcpPorts: filteredPorts,
            diapIdentity,
            tempId, // 传递临时 ID 用于匹配更新
          })
          console.log('[CreateAgentModal] 智能体创建完成！')

        } catch (err) {
          console.error('[CreateAgentModal] 后台创建失败:', err)
        }
      })()

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
