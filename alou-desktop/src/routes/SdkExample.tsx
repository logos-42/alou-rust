import React, { useState } from 'react'
import { LspEditor, SpecManager } from '../components/SDKComponents'
import '../components/SDKComponents.css'

/**
 * SdkExamplePage 组件
 * SDK 示例页面，展示 LSP 编辑器和 Spec 管理功能
 */
const SdkExamplePage: React.FC = () => {
  const [activeTab, setActiveTab] = useState<string>('lsp')
  const [code, setCode] = useState<string>(`// 示例JavaScript代码
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

  const [language, setLanguage] = useState<string>('javascript')
  const [specType, setSpecType] = useState<string>('product')

  const handleCodeChange = (newCode: string) => {
    setCode(newCode)
  }

  const handleLanguageChange = (newLanguage: string) => {
    setLanguage(newLanguage)
  }

  const handleSpecTypeChange = (newSpecType: string) => {
    setSpecType(newSpecType)
  }

  return (
    <div className="sdk-example-page">
      <div className="sdk-example-page__header">
        <h1>SDK 功能示例</h1>
        <p>展示桌面版 SDK 的 LSP 和 Spec 功能</p>
      </div>

      <div className="sdk-example-page__tabs">
        <button
          className={`tab ${activeTab === 'lsp' ? 'tab--active' : ''}`}
          onClick={() => setActiveTab('lsp')}
        >
          LSP 编辑器
        </button>
        <button
          className={`tab ${activeTab === 'spec' ? 'tab--active' : ''}`}
          onClick={() => setActiveTab('spec')}
        >
          Spec 管理
        </button>
      </div>

      <div className="sdk-example-page__content">
        {activeTab === 'lsp' && (
          <div className="sdk-example-page__lsp">
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
          <div className="sdk-example-page__spec">
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
  )
}

export default SdkExamplePage
