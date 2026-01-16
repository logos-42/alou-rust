/**
 * 工具面板组件 - 提供工具执行界面
 */

import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import toolService from '../services/toolService';

const ToolPanel = ({ onClose }) => {
  const [tools, setTools] = useState([]);
  const [selectedTool, setSelectedTool] = useState(null);
  const [toolArgs, setToolArgs] = useState('');
  const [executionResult, setExecutionResult] = useState(null);
  const [isExecuting, setIsExecuting] = useState(false);
  const [executionMode, setExecutionMode] = useState('auto'); // 'auto', 'local', 'remote'

  // 加载工具列表
  useEffect(() => {
    loadTools();
  }, []);

  const loadTools = async () => {
    try {
      const toolList = await toolService.getToolList();
      setTools(toolList);
    } catch (error) {
      console.error('Failed to load tools:', error);
      // 回退到本地工具列表
      setTools([
        { id: 'filesystem', name: 'File System', category: 'FileSystem', availableModes: ['local'] },
        { id: 'search', name: 'Search', category: 'Search', availableModes: ['local'] },
        { id: 'bash', name: 'Bash Shell', category: 'Terminal', availableModes: ['local'] },
        { id: 'plan', name: 'Task Planning', category: 'Planning', availableModes: ['local'] },
        { id: 'skills', name: 'Skills', category: 'Skills', availableModes: ['local'] },
      ]);
    }
  };

  const handleToolSelect = (tool) => {
    setSelectedTool(tool);
    setExecutionResult(null);

    // 设置默认参数示例
    const defaultArgs = getDefaultArgsForTool(tool.id);
    setToolArgs(JSON.stringify(defaultArgs, null, 2));
  };

  const getDefaultArgsForTool = (toolId) => {
    switch (toolId) {
      case 'filesystem':
        return {
          operation: 'list',
          path: '.',
          recursive: false
        };
      case 'search':
        return {
          type: 'grep',
          pattern: 'function',
          path: '.'
        };
      case 'bash':
        return {
          command: 'echo "Hello from tool execution!"'
        };
      case 'plan':
        return {
          action: 'list_plans'
        };
      case 'skills':
        return {
          action: 'list_skills'
        };
      default:
        return {};
    }
  };

  const handleExecute = async () => {
    if (!selectedTool) return;

    setIsExecuting(true);
    setExecutionResult(null);

    try {
      let args;
      try {
        args = JSON.parse(toolArgs);
      } catch (error) {
        setExecutionResult({
          success: false,
          error: 'Invalid JSON arguments',
          timestamp: Date.now()
        });
        return;
      }

      const options = {
        preferLocal: executionMode === 'auto' || executionMode === 'local',
        timeout: 30000,
        allowAsync: executionMode === 'remote' || executionMode === 'auto',
      };

      console.log(`Executing tool: ${selectedTool.id}`, { args, options });

      const result = await toolService.executeTool(selectedTool.id, args, options);

      setExecutionResult({
        ...result,
        timestamp: Date.now()
      });

    } catch (error) {
      console.error('Tool execution failed:', error);
      setExecutionResult({
        success: false,
        error: error.message || 'Tool execution failed',
        timestamp: Date.now(),
        executionMode: 'failed'
      });
    } finally {
      setIsExecuting(false);
    }
  };

  const getExecutionModeText = (mode) => {
    switch (mode) {
      case 'local': return '本地执行';
      case 'remote': return '远程执行';
      default: return '智能路由';
    }
  };

  return (
    <div className="tool-panel">
      <div className="tool-panel-header">
        <h3>工具执行器</h3>
        <button onClick={onClose} className="close-button">×</button>
      </div>

      <div className="tool-panel-content">
        {/* 工具选择 */}
        <div className="tool-selection">
          <h4>选择工具</h4>
          <div className="tool-grid">
            {tools.map(tool => (
              <div
                key={tool.id}
                className={`tool-card ${selectedTool?.id === tool.id ? 'selected' : ''}`}
                onClick={() => handleToolSelect(tool)}
              >
                <h5>{tool.name}</h5>
                <p>{tool.description}</p>
                <div className="tool-meta">
                  <span className="category">{tool.category}</span>
                  <div className="modes">
                    {tool.availableModes?.map(mode => (
                      <span key={mode} className={`mode mode-${mode}`}>
                        {getExecutionModeText(mode)}
                      </span>
                    ))}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* 参数配置 */}
        {selectedTool && (
          <div className="tool-config">
            <h4>执行配置</h4>

            <div className="config-row">
              <label>执行模式:</label>
              <select
                value={executionMode}
                onChange={(e) => setExecutionMode(e.target.value)}
              >
                <option value="auto">智能路由（推荐）</option>
                <option value="local">强制本地执行</option>
                <option value="remote">强制远程执行</option>
              </select>
            </div>

            <div className="config-row">
              <label>工具参数 (JSON):</label>
              <textarea
                value={toolArgs}
                onChange={(e) => setToolArgs(e.target.value)}
                placeholder="输入工具参数 JSON"
                rows={8}
              />
            </div>

            <button
              onClick={handleExecute}
              disabled={isExecuting}
              className="execute-button"
            >
              {isExecuting ? '执行中...' : `执行 ${selectedTool.name}`}
            </button>
          </div>
        )}

        {/* 执行结果 */}
        {executionResult && (
          <div className="execution-result">
            <h4>执行结果</h4>

            <div className="result-header">
              <span className={`status ${executionResult.success ? 'success' : 'error'}`}>
                {executionResult.success ? '✓ 成功' : '✗ 失败'}
              </span>
              <span className="execution-mode">
                执行模式: {getExecutionModeText(executionResult.executionMode)}
              </span>
              {executionResult.executionTimeMs && (
                <span className="execution-time">
                  执行时间: {executionResult.executionTimeMs}ms
                </span>
              )}
            </div>

            {executionResult.error ? (
              <div className="error-result">
                <h5>错误信息</h5>
                <pre>{executionResult.error}</pre>
              </div>
            ) : (
              <div className="success-result">
                {executionResult.output && (
                  <div className="output">
                    <h5>输出信息</h5>
                    <pre>{executionResult.output}</pre>
                  </div>
                )}

                {executionResult.data && (
                  <div className="data">
                    <h5>返回数据</h5>
                    <pre>{JSON.stringify(executionResult.data, null, 2)}</pre>
                  </div>
                )}

                {executionResult.warnings && executionResult.warnings.length > 0 && (
                  <div className="warnings">
                    <h5>警告信息</h5>
                    <ul>
                      {executionResult.warnings.map((warning, index) => (
                        <li key={index}>{warning}</li>
                      ))}
                    </ul>
                  </div>
                )}
              </div>
            )}
          </div>
        )}
      </div>

      <style jsx>{`
        .tool-panel {
          position: fixed;
          top: 0;
          right: 0;
          width: 600px;
          height: 100vh;
          background: white;
          box-shadow: -2px 0 8px rgba(0,0,0,0.1);
          z-index: 1000;
          display: flex;
          flex-direction: column;
        }

        .tool-panel-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 16px;
          border-bottom: 1px solid #e0e0e0;
          background: #f8f9fa;
        }

        .close-button {
          background: none;
          border: none;
          font-size: 24px;
          cursor: pointer;
          padding: 0;
          width: 32px;
          height: 32px;
          display: flex;
          align-items: center;
          justify-content: center;
          border-radius: 4px;
        }

        .close-button:hover {
          background: #e9ecef;
        }

        .tool-panel-content {
          flex: 1;
          overflow-y: auto;
          padding: 16px;
        }

        .tool-selection h4,
        .tool-config h4,
        .execution-result h4 {
          margin: 0 0 16px 0;
          color: #333;
        }

        .tool-grid {
          display: grid;
          grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
          gap: 12px;
        }

        .tool-card {
          border: 2px solid #e0e0e0;
          border-radius: 8px;
          padding: 16px;
          cursor: pointer;
          transition: all 0.2s;
        }

        .tool-card:hover {
          border-color: #007bff;
          background: #f8f9ff;
        }

        .tool-card.selected {
          border-color: #007bff;
          background: #e7f3ff;
        }

        .tool-card h5 {
          margin: 0 0 8px 0;
          color: #333;
        }

        .tool-card p {
          margin: 0 0 12px 0;
          color: #666;
          font-size: 14px;
          line-height: 1.4;
        }

        .tool-meta {
          display: flex;
          justify-content: space-between;
          align-items: center;
        }

        .category {
          background: #e9ecef;
          padding: 4px 8px;
          border-radius: 4px;
          font-size: 12px;
          color: #495057;
        }

        .modes {
          display: flex;
          gap: 4px;
        }

        .mode {
          padding: 2px 6px;
          border-radius: 3px;
          font-size: 11px;
          font-weight: 500;
        }

        .mode-local {
          background: #d4edda;
          color: #155724;
        }

        .mode-remote {
          background: #cce5ff;
          color: #004085;
        }

        .tool-config {
          margin-top: 24px;
          padding: 16px;
          background: #f8f9fa;
          border-radius: 8px;
        }

        .config-row {
          margin-bottom: 16px;
        }

        .config-row label {
          display: block;
          margin-bottom: 8px;
          font-weight: 500;
          color: #333;
        }

        .config-row select,
        .config-row textarea {
          width: 100%;
          padding: 8px 12px;
          border: 1px solid #ddd;
          border-radius: 4px;
          font-family: monospace;
        }

        .config-row textarea {
          resize: vertical;
        }

        .execute-button {
          background: #007bff;
          color: white;
          border: none;
          padding: 12px 24px;
          border-radius: 6px;
          cursor: pointer;
          font-weight: 500;
          width: 100%;
        }

        .execute-button:hover:not(:disabled) {
          background: #0056b3;
        }

        .execute-button:disabled {
          background: #6c757d;
          cursor: not-allowed;
        }

        .execution-result {
          margin-top: 24px;
          padding: 16px;
          border-radius: 8px;
          border: 2px solid #e0e0e0;
        }

        .result-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 16px;
          flex-wrap: wrap;
          gap: 8px;
        }

        .status {
          padding: 4px 12px;
          border-radius: 4px;
          font-weight: 500;
        }

        .status.success {
          background: #d4edda;
          color: #155724;
        }

        .status.error {
          background: #f8d7da;
          color: #721c24;
        }

        .execution-mode,
        .execution-time {
          font-size: 14px;
          color: #666;
        }

        .error-result,
        .success-result,
        .output,
        .data,
        .warnings {
          margin-bottom: 16px;
        }

        .error-result h5,
        .success-result h5,
        .output h5,
        .data h5,
        .warnings h5 {
          margin: 0 0 8px 0;
          color: #333;
          font-size: 16px;
        }

        .error-result pre,
        .output pre,
        .data pre {
          background: #f8f9fa;
          padding: 12px;
          border-radius: 4px;
          overflow-x: auto;
          font-size: 14px;
          line-height: 1.4;
        }

        .data pre {
          background: #f1f3f4;
        }

        .warnings ul {
          background: #fff3cd;
          padding: 12px;
          border-radius: 4px;
          margin: 0;
        }

        .warnings li {
          color: #856404;
          margin-bottom: 4px;
        }

        .warnings li:last-child {
          margin-bottom: 0;
        }
      `}</style>
    </div>
  );
};

export default ToolPanel;