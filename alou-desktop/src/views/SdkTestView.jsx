/**
 * SDK 功能测试页面
 * 用于测试 LSP、Spec和工作流功能
 */
import React, { useState } from 'react'
import { LspEditor, SpecManager } from '@/components/SDKComponents'
import SkillsManager from '@/components/SkillsManager'
import '@/components/SDKComponents.css'

function SdkTestPage() {
  const [activeTab, setActiveTab] = useState('lsp')

  const [code, setCode] = useState(`function example() {
  console.log("Hello, World!");
  return 42;
}

const data = {
  name: "Alou",
  version: "1.0.0"
};

console.log(data);`)

  const [language, setLanguage] = useState('javascript')
  const [specType, setSpecType] = useState('product')

  // 工作流相关状态
  const [apiKey, setApiKey] = useState('')
  const [agentInfo, setAgentInfo] = useState({
    name: 'Workflow Agent',
    role_description: '专门用于执行工作流任务的智能体',
    mode: 'agent'
  })

  // 工作流事件处理
  const handleWorkflowEvent = (event) => {
    console.log('[SdkTestPage] 工作流事件:', event)

    // 在对话框中显示工作流执行状态
    const eventMessage = getWorkflowEventMessage(event)
    if (eventMessage) {
      // 这里可以添加到对话历史中显示
      console.log('工作流消息:', eventMessage)
    }
  }

  const getWorkflowEventMessage = (event) => {
    switch (event.type) {
      case 'execution_started':
        return `🔄 开始执行工作流 ${event.workflowId}`
      case 'execution_completed':
        return `✅ 工作流执行完成`
      case 'execution_error':
        return `❌ 工作流执行失败: ${event.error}`
      case 'workflow_created':
        return `✨ 工作流创建成功`
      case 'step_retried':
        return `🔄 步骤重试完成: ${event.stepId}`
      default:
        return null
    }
  }

  return (
    <div style={{ padding: '2rem' }}>
      <h1>SDK 功能测试</h1>

      {/* 标签页导航 */}
      <div style={{ marginBottom: '2rem', borderBottom: '1px solid #ddd' }}>
        <button
          onClick={() => setActiveTab('lsp')}
          style={{
            padding: '10px 20px',
            marginRight: '10px',
            border: 'none',
            background: activeTab === 'lsp' ? '#007bff' : '#f8f9fa',
            color: activeTab === 'lsp' ? 'white' : '#333',
            cursor: 'pointer',
            borderRadius: '4px 4px 0 0'
          }}
        >
          LSP 编辑器
        </button>
        <button
          onClick={() => setActiveTab('spec')}
          style={{
            padding: '10px 20px',
            marginRight: '10px',
            border: 'none',
            background: activeTab === 'spec' ? '#007bff' : '#f8f9fa',
            color: activeTab === 'spec' ? 'white' : '#333',
            cursor: 'pointer',
            borderRadius: '4px 4px 0 0'
          }}
        >
          Spec 管理
        </button>
        <button
          onClick={() => setActiveTab('workflow')}
          style={{
            padding: '10px 20px',
            border: 'none',
            background: activeTab === 'workflow' ? '#007bff' : '#f8f9fa',
            color: activeTab === 'workflow' ? 'white' : '#333',
            cursor: 'pointer',
            borderRadius: '4px 4px 0 0'
          }}
        >
          Claude 工作流
        </button>
      </div>

      {activeTab === 'lsp' && (
        <div style={{ marginBottom: '2rem' }}>
          <h2>LSP 编辑器测试</h2>
          <div style={{ marginBottom: '1rem' }}>
            <label>语言: </label>
            <select value={language} onChange={(e) => setLanguage(e.target.value)}>
              <option value="javascript">JavaScript</option>
              <option value="typescript">TypeScript</option>
              <option value="python">Python</option>
              <option value="rust">Rust</option>
            </select>
          </div>
          <LspEditor
            code={code}
            language={language}
            onChange={setCode}
          />
        </div>
      )}

      {activeTab === 'spec' && (
        <div style={{ marginBottom: '2rem' }}>
          <h2>Spec 管理测试</h2>
          <div style={{ marginBottom: '1rem' }}>
            <label>类型: </label>
            <select value={specType} onChange={(e) => setSpecType(e.target.value)}>
              <option value="product">产品规格</option>
              <option value="technical">技术规格</option>
              <option value="design">设计规格</option>
            </select>
          </div>
          <SpecManager specType={specType} />
        </div>
      )}

      {activeTab === 'workflow' && (
        <div style={{ marginBottom: '2rem' }}>
          <h2>Claude Agent SDK 工作流测试</h2>

          {/* API Key 输入 */}
          <div style={{ marginBottom: '1rem' }}>
            <label style={{ display: 'block', marginBottom: '0.5rem' }}>
              Claude API Key:
            </label>
            <input
              type="password"
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              placeholder="输入你的Claude API Key"
              style={{
                width: '100%',
                padding: '8px',
                border: '1px solid #ddd',
                borderRadius: '4px',
                fontFamily: 'monospace'
              }}
            />
            <small style={{ color: '#666', display: 'block', marginTop: '0.25rem' }}>
              API Key 只会存储在本地内存中，不会持久化保存
            </small>
          </div>

          {/* 智能体信息配置 */}
          <div style={{ marginBottom: '1rem' }}>
            <label style={{ display: 'block', marginBottom: '0.5rem' }}>
              智能体配置:
            </label>
            <div style={{ display: 'flex', gap: '1rem', flexWrap: 'wrap' }}>
              <input
                type="text"
                value={agentInfo.name}
                onChange={(e) => setAgentInfo(prev => ({ ...prev, name: e.target.value }))}
                placeholder="智能体名称"
                style={{ padding: '8px', border: '1px solid #ddd', borderRadius: '4px', flex: 1, minWidth: '200px' }}
              />
              <select
                value={agentInfo.mode}
                onChange={(e) => setAgentInfo(prev => ({ ...prev, mode: e.target.value }))}
                style={{ padding: '8px', border: '1px solid #ddd', borderRadius: '4px' }}
              >
                <option value="agent">Agent 模式</option>
                <option value="alou">Alou 模式</option>
              </select>
            </div>
          </div>

          {/* 技能管理组件 */}
          <SkillsManager
            sessionId="sdk-test-session"
            agentInfo={agentInfo}
            onSkillEvent={handleWorkflowEvent}
            isDarkMode={false}
          />

          {/* 使用说明 */}
          <div style={{
            marginTop: '2rem',
            padding: '1rem',
            background: '#f8f9fa',
            borderRadius: '8px',
            border: '1px solid #e9ecef'
          }}>
            <h3>工作流功能说明</h3>
            <ul style={{ lineHeight: '1.6' }}>
              <li><strong>创建示例工作流：</strong>点击按钮创建一个演示用的4步骤工作流</li>
              <li><strong>执行工作流：</strong>选择工作流后点击"执行"按钮，使用Claude SDK本地执行</li>
              <li><strong>实时状态：</strong>工作流执行时会实时显示步骤状态和进度</li>
              <li><strong>步骤控制：</strong>可以暂停、恢复或重试失败的步骤</li>
              <li><strong>对话集成：</strong>工作流事件会显示在对话框中</li>
            </ul>
            <p style={{ color: '#6c757d', fontSize: '0.9em', marginTop: '1rem' }}>
              <strong>注意：</strong>此功能需要有效的Claude API Key，并且需要在Tauri后端实现相应的Rust命令处理函数。
            </p>
          </div>
        </div>
      )}
    </div>
  )
}

export default SdkTestPage
