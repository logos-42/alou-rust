import { useMemo, useState, useEffect, useCallback } from 'react'
import agentAssetsService from '@/services/agentAssetsService'
import agentService from '@/services/agentService'
import ipfsService from '@/services/ipfsService'
import { useI18n } from '@/hooks/useI18n'
import { setDiapIdentitySafe, hasDiapIdentitySafe } from '@/utils/diapIdentityManager'
import { setDiapIdentity } from '@/utils/memoryStorage'
import CloseIcon from '@/assets/关闭0.3.png'
import CopyIcon from '@/assets/复制.png'
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
      // 检查 IPFS 节点是否运行
      let isRunning = await ipfsService.isNodeRunning()
      if (!isRunning) {
        const startResult = await ipfsService.startNode(true)
        if (!startResult.success) {
          setError(t('agent.create.error.ipfsNotRunning'))
          setIsLoading(false)
          return
        }
        console.log('[IPFS] 节点已自动启动，等待 API 就绪...')
      }

      // 等待 IPFS API 完全就绪（最多等待 15 秒）
      const apiReady = await ipfsService.waitForApiReady(15, 1000)
      if (!apiReady.success) {
        setError(
          apiReady.error ||
            'IPFS API 未就绪。请确保 IPFS 节点正常运行，然后重试。'
        )
        setIsLoading(false)
        return
      }
      console.log(`[IPFS] API 已就绪 (尝试 ${apiReady.attempts} 次)`)

      // 1. 创建 DIAP Identity（核心步骤）
      let diapIdentity = null
      try {
        console.log('[CreateAgentModal] 开始创建 DIAP Identity...')
        
        // 使用新的DiapIntegrationService
        const { default: diapIntegrationService } = await import('../services/diapIntegrationService')
        
        const result = await diapIntegrationService.createCompleteDiapIdentity(sessionId, {
          agentName: fallbackName,
          agentDescription: finalRoleDescription,
        })
        
        diapIdentity = result.identity
        console.log('[CreateAgentModal] DIAP Identity 创建成功:', diapIdentity?.did)
        
      } catch (err) {
        console.error('[CreateAgentModal] DIAP Identity 创建失败:', err)
        // DIAP 创建失败时显示错误但继续（允许用户创建没有 DIAP 的智能体）
        console.warn('[CreateAgentModal] 将创建没有 DIAP Identity 的智能体')
      }

      // 2. 上传头像
      let avatarCid = null
      if (avatarFile) {
        try {
          console.log('[CreateAgentModal] 开始上传头像...')
          const uploaded = await agentAssetsService.uploadAvatar(avatarFile, { sessionId })
          avatarCid = uploaded?.cid || null
          console.log('[CreateAgentModal] 头像上传成功:', avatarCid)
        } catch (err) {
          console.error('[CreateAgentModal] 头像上传失败:', err)
          // 头像上传失败不阻塞创建
        }
      }

      // 3. 解析并上传 MCP 配置
      let mcpConfigCid = null
      let filteredPorts = []
      
      if (mcpCode.trim()) {
        try {
          console.log('[CreateAgentModal] 解析 MCP 配置...')
          const { ports } = parseMcpCode(mcpCode)
          filteredPorts = ports.filter((port) => port.label?.trim() || port.endpoint?.trim())
          
          if (filteredPorts.length > 0) {
            console.log('[CreateAgentModal] 开始上传 MCP 配置...')
            const uploadedConfig = await agentAssetsService.uploadMcpConfig(
              {
                ports: filteredPorts,
                generatedAt: Date.now(),
              },
              { sessionId },
            )
            mcpConfigCid = uploadedConfig?.cid || null
            console.log('[CreateAgentModal] MCP 配置上传成功:', mcpConfigCid)
          }
        } catch (err) {
          console.error('[CreateAgentModal] MCP 配置处理失败:', err)
          // 不阻塞创建流程
        }
      }

      // 4. 构建完整的智能体数据（只调用一次onSubmit）
      const agentData = {
        name: fallbackName,
        roleDescription: finalRoleDescription,
        avatar_cid: avatarCid, // 修复：使用下划线命名与agentStore保持一致
        mcp_config_cid: mcpConfigCid, // 修复：使用下划线命名与agentStore保持一致
        mcp_ports: filteredPorts, // 修复：使用下划线命名与agentStore保持一致
        diapIdentity: diapIdentity ? {
          did: diapIdentity.did,
          cid: diapIdentity.cid,
          ipns: diapIdentity.ipns || '',
          public_key: diapIdentity.public_key
        } : null,
        sessionId,
        // 添加标识，表明这是完整的智能体数据
        isComplete: true,
      }

      console.log('[CreateAgentModal] 提交完整智能体数据:', {
        name: agentData.name,
        hasAvatar: !!avatarCid,
        hasMcp: !!mcpConfigCid,
        hasDiap: !!diapIdentity,
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
