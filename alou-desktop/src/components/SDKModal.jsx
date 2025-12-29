import React, { useState } from 'react'
import { LspEditor, SpecManager } from './SDKComponents'
import CloseIcon from '@/assets/关闭0.3.png'
import './SDKModal.css'

/**
 * SDK 功能模态框
 */
const SDKModal = ({ isOpen, onClose, initialTab = 'lsp', isDarkMode = false }) => {
  const [activeTab, setActiveTab] = useState(initialTab)
  const [code, setCode] = useState(`// 示例JavaScript代码
function greet(name) {
  return \`Hello, \${name}!\`
}

const user = {
  name: 'Alou',
  age: 25,
}

console.log(greet(user.name))

// 定义一个类
class Calculator {
  constructor() {
    this.result = 0
  }

  add(num) {
    this.result += num
    return this
  }

  multiply(num) {
    this.result *= num
    return this
  }

  getResult() {
    return this.result
  }
}

const calc = new Calculator()
console.log(calc.add(10).multiply(2).getResult())
`)

  const [language, setLanguage] = useState('javascript')
  const [specType, setSpecType] = useState('product')

  const handleCodeChange = (newCode) => {
    setCode(newCode)
  }

  const handleLanguageChange = (newLanguage) => {
    setLanguage(newLanguage)
  }

  const handleSpecTypeChange = (newSpecType) => {
    setSpecType(newSpecType)
  }

  if (!isOpen) return null

  return (
    <div className="sdk-modal-overlay" onClick={onClose}>
      <div className={`sdk-modal ${isDarkMode ? 'dark' : ''}`} onClick={(e) => e.stopPropagation()}>
        <div className="sdk-modal__header">
          <div className="sdk-modal__tabs">
            <button
              className={`sdk-tab ${activeTab === 'lsp' ? 'sdk-tab--active' : ''}`}
              onClick={() => setActiveTab('lsp')}
            >
              LSP 编辑器
            </button>
            <button
              className={`sdk-tab ${activeTab === 'spec' ? 'sdk-tab--active' : ''}`}
              onClick={() => setActiveTab('spec')}
            >
              Spec 管理
            </button>
          </div>
          <button type="button" className="sdk-modal__close" onClick={onClose}>
            <img src={CloseIcon} alt="关闭" />
          </button>
        </div>

        <div className="sdk-modal__content">
          {activeTab === 'lsp' && (
            <div className="sdk-modal__lsp">
              <div className="lsp-toolbar">
                <label>
                  编程语言:
                  <select
                    value={language}
                    onChange={(e) => handleLanguageChange(e.target.value)}
                  >
                    <option value="javascript">JavaScript</option>
                    <option value="typescript">TypeScript</option>
                    <option value="python">Python</option>
                    <option value="rust">Rust</option>
                    <option value="go">Go</option>
                    <option value="java">Java</option>
                    <option value="csharp">C#</option>
                    <option value="cpp">C++</option>
                    <option value="html">HTML</option>
                    <option value="css">CSS</option>
                    <option value="json">JSON</option>
                    <option value="markdown">Markdown</option>
                  </select>
                </label>
              </div>

              <LspEditor
                code={code}
                language={language}
                onChange={handleCodeChange}
              />

              <div className="lsp-features">
                <h3>LSP 功能特性</h3>
                <ul>
                  <li>✓ 智能代码补全 (Ctrl + Space)</li>
                  <li>✓ 代码诊断 (错误、警告)</li>
                  <li>✓ 悬停信息 (鼠标悬停在代码上)</li>
                  <li>✓ 文档符号 (函数、类等)</li>
                  <li>✓ 代码格式化</li>
                  <li>✓ 跳转到定义</li>
                  <li>✓ 查找引用</li>
                  <li>✓ 支持多种编程语言</li>
                </ul>
              </div>
            </div>
          )}

          {activeTab === 'spec' && (
            <div className="sdk-modal__spec">
              <div className="spec-toolbar">
                <label>
                  规格类型:
                  <select
                    value={specType}
                    onChange={(e) => handleSpecTypeChange(e.target.value)}
                  >
                    <option value="product">产品规格</option>
                    <option value="technical">技术规格</option>
                    <option value="design">设计规格</option>
                    <option value="api">API规格</option>
                    <option value="user_story">用户故事</option>
                    <option value="tasks">任务规格</option>
                    <option value="structure">结构规格</option>
                  </select>
                </label>
              </div>

              <SpecManager specType={specType} />

              <div className="spec-features">
                <h3>Spec 管理功能</h3>
                <ul>
                  <li>✓ 创建规格文档</li>
                  <li>✓ 编辑规格内容</li>
                  <li>✓ 删除规格文档</li>
                  <li>✓ 规格文档验证</li>
                  <li>✓ 导出为 Markdown</li>
                  <li>✓ 导出为 HTML</li>
                  <li>✓ 从模板生成</li>
                  <li>✓ 按类型筛选</li>
                </ul>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

export default SDKModal
