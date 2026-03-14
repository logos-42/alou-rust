import React, { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import lspService from '../services/lspService'
import specService from '../services/specService'

/**
 * LSP组件 - 代码编辑器与语言服务集成
 */
export const LspEditor = ({ code, language, onChange }) => {
  const [completions, setCompletions] = useState([])
  const [diagnostics, setDiagnostics] = useState([])
  const [showCompletions, setShowCompletions] = useState(false)
  const [position, setPosition] = useState({ line: 0, character: 0 })
  const [supportedLanguages, setSupportedLanguages] = useState([])

  useEffect(() => {
    loadSupportedLanguages()
  }, [])

  const loadSupportedLanguages = async () => {
    try {
      const languages = await lspService.getSupportedLanguages()
      setSupportedLanguages(languages)
    } catch (error) {
      console.error('加载支持的语言失败:', error)
    }
  }

  const handleCodeChange = async (newCode) => {
    onChange(newCode)

    // 自动诊断
    try {
      const result = await lspService.getDiagnostics({
        code: newCode,
        language,
      })
      setDiagnostics(result.diagnostics)
    } catch (error) {
      console.error('代码诊断失败:', error)
    }
  }

  const handleKeyDown = async (e, lineIndex, charIndex) => {
    // Ctrl+Space 触发代码补全
    if (e.ctrlKey && e.code === 'Space') {
      e.preventDefault()
      try {
        const result = await lspService.getCompletions({
          code,
          language,
          position: { line: lineIndex, character: charIndex },
        })
        setCompletions(result.items)
        setShowCompletions(true)
      } catch (error) {
        console.error('代码补全失败:', error)
      }
    }
  }

  const selectCompletion = (item) => {
    const lines = code.split('\n')
    const currentLine = lines[position.line]
    const beforeCursor = currentLine.substring(0, position.character)
    const afterCursor = currentLine.substring(position.character)

    // 找到要替换的单词
    const wordMatch = beforeCursor.match(/([a-zA-Z_][a-zA-Z0-9_]*)$/)
    if (wordMatch) {
      const startPos = position.character - wordMatch[1].length
      const newLine = currentLine.substring(0, startPos) + item.insert_text + afterCursor
      lines[position.line] = newLine
      onChange(lines.join('\n'))
    }

    setShowCompletions(false)
  }

  return (
    <div className="lsp-editor">
      <div className="lsp-editor__toolbar">
        <select value={language} onChange={(e) => onChangeLanguage(e.target.value)}>
          <option value="">选择语言</option>
          {supportedLanguages.map((lang) => (
            <option key={lang} value={lang}>
              {lang}
            </option>
          ))}
        </select>
      </div>

      <div className="lsp-editor__container">
        <textarea
          value={code}
          onChange={(e) => handleCodeChange(e.target.value)}
          onKeyDown={(e) => handleKeyDown(e, position.line, position.character)}
          onClick={(e) => {
            const textarea = e.target
            const value = textarea.value.substring(0, textarea.selectionStart)
            const lines = value.split('\n')
            setPosition({
              line: lines.length - 1,
              character: lines[lines.length - 1].length,
            })
          }}
          className="lsp-editor__textarea"
          spellCheck={false}
        />

        {showCompletions && completions.length > 0 && (
          <div className="lsp-editor__completions">
            {completions.map((item, index) => (
              <div
                key={index}
                className="lsp-editor__completion-item"
                onClick={() => selectCompletion(item)}
              >
                <div className="completion-item__label">{item.label}</div>
                {item.detail && (
                  <div className="completion-item__detail">{item.detail}</div>
                )}
                {item.documentation && (
                  <div className="completion-item__documentation">
                    {item.documentation}
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>

      {diagnostics.length > 0 && (
        <div className="lsp-editor__diagnostics">
          <h4>诊断信息</h4>
          {diagnostics.map((diag, index) => (
            <div
              key={index}
              className={`diagnostic diagnostic--${diag.severity}`}
            >
              <span className="diagnostic__line">行 {diag.range.start.line + 1}:</span>
              <span className="diagnostic__message">{diag.message}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}

/**
 * Spec管理组件 - 规格文档管理
 */
export const SpecManager = ({ specType }) => {
  const [specs, setSpecs] = useState([])
  const [selectedSpec, setSelectedSpec] = useState(null)
  const [templates, setTemplates] = useState([])
  const [showCreateModal, setShowCreateModal] = useState(false)
  const [loading, setLoading] = useState(false)

  useEffect(() => {
    loadSpecs()
    loadTemplates()
  }, [specType])

  const loadSpecs = async () => {
    setLoading(true)
    try {
      const result = await specService.listSpecs({ specType })
      setSpecs(result)
    } catch (error) {
      console.error('加载规格列表失败:', error)
    } finally {
      setLoading(false)
    }
  }

  const loadTemplates = async () => {
    try {
      const result = await specService.getTemplates()
      setTemplates(result)
    } catch (error) {
      console.error('加载模板失败:', error)
    }
  }

  const createSpec = async (templateId, data) => {
    try {
      await specService.generateFromTemplate({
        templateId,
        specData: data,
      })
      setShowCreateModal(false)
      loadSpecs()
    } catch (error) {
      console.error('创建规格失败:', error)
      alert('创建失败: ' + error.message)
    }
  }

  const deleteSpec = async (specId) => {
    if (!confirm('确定要删除这个规格吗？')) return

    try {
      await specService.deleteSpec(specId)
      if (selectedSpec?.id === specId) {
        setSelectedSpec(null)
      }
      loadSpecs()
    } catch (error) {
      console.error('删除规格失败:', error)
      alert('删除失败: ' + error.message)
    }
  }

  const validateSpec = async (specId) => {
    try {
      const result = await specService.validateSpec({ specId })
      alert(
        `验证结果:\n${result.is_valid ? '✓ 有效' : '✗ 无效'}\n` +
        `错误: ${result.errors.length}\n` +
        `警告: ${result.warnings.length}\n` +
        `建议: ${result.suggestions.length}`
      )
    } catch (error) {
      console.error('验证规格失败:', error)
      alert('验证失败: ' + error.message)
    }
  }

  const exportSpec = async (specId, format) => {
    try {
      const result = await specService.exportSpec({
        specId,
        outputFormat: format,
      })
      alert(`导出成功:\n文件: ${result.output_path}\n格式: ${result.format}`)
    } catch (error) {
      console.error('导出规格失败:', error)
      alert('导出失败: ' + error.message)
    }
  }

  return (
    <div className="spec-manager">
      <div className="spec-manager__header">
        <h2>规格文档管理</h2>
        <button onClick={() => setShowCreateModal(true)}>
          + 创建规格
        </button>
      </div>

      <div className="spec-manager__content">
        <div className="spec-manager__list">
          {loading ? (
            <div>加载中...</div>
          ) : (
            <ul className="spec-list">
              {specs.map((spec) => (
                <li
                  key={spec.id}
                  className={`spec-item ${
                    selectedSpec?.id === spec.id ? 'spec-item--active' : ''
                  }`}
                  onClick={() => setSelectedSpec(spec)}
                >
                  <div className="spec-item__title">{spec.title}</div>
                  <div className="spec-item__meta">
                    <span className="spec-item__type">{spec.spec_type}</span>
                    <span className="spec-item__version">{spec.metadata.version}</span>
                    <span className="spec-item__status">{spec.metadata.status}</span>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </div>

        {selectedSpec && (
          <div className="spec-manager__detail">
            <div className="spec-detail__header">
              <h3>{selectedSpec.title}</h3>
              <div className="spec-detail__actions">
                <button onClick={() => validateSpec(selectedSpec.id)}>
                  验证
                </button>
                <button onClick={() => exportSpec(selectedSpec.id, 'md')}>
                  导出Markdown
                </button>
                <button onClick={() => exportSpec(selectedSpec.id, 'html')}>
                  导出HTML
                </button>
                <button onClick={() => deleteSpec(selectedSpec.id)}>
                  删除
                </button>
              </div>
            </div>
            <div className="spec-detail__content">
              <pre>{selectedSpec.content}</pre>
            </div>
          </div>
        )}
      </div>

      {showCreateModal && (
        <SpecCreateModal
          templates={templates}
          onClose={() => setShowCreateModal(false)}
          onCreate={createSpec}
        />
      )}
    </div>
  )
}

/**
 * Spec创建模态框
 */
const SpecCreateModal = ({ templates, onClose, onCreate }) => {
  const [selectedTemplate, setSelectedTemplate] = useState('')
  const [title, setTitle] = useState('')
  const [data, setData] = useState({})

  const handleSubmit = (e) => {
    e.preventDefault()
    onCreate(selectedTemplate, {
      title,
      ...data,
    })
  }

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal__header">
          <h3>创建规格文档</h3>
          <button onClick={onClose}>×</button>
        </div>
        <form onSubmit={handleSubmit} className="modal__body">
          <div className="form-group">
            <label>模板</label>
            <select
              value={selectedTemplate}
              onChange={(e) => setSelectedTemplate(e.target.value)}
              required
            >
              <option value="">选择模板</option>
              {templates.map((template) => (
                <option key={template.id} value={template.id}>
                  {template.name}
                </option>
              ))}
            </select>
          </div>

          <div className="form-group">
            <label>标题</label>
            <input
              type="text"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              required
              placeholder="输入规格标题"
            />
          </div>

          <div className="form-group">
            <label>数据（JSON）</label>
            <textarea
              value={JSON.stringify(data, null, 2)}
              onChange={(e) => {
                try {
                  setData(JSON.parse(e.target.value))
                } catch (err) {
                  console.error('JSON解析失败:', err)
                }
              }}
              placeholder='{"project_name": "示例项目", "target_users": "..."}'
            />
          </div>

          <div className="form-actions">
            <button type="button" onClick={onClose}>
              取消
            </button>
            <button type="submit">创建</button>
          </div>
        </form>
      </div>
    </div>
  )
}

export default { LspEditor, SpecManager }
