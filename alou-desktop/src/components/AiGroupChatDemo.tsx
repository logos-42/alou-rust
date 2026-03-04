/**
 * AI自主创建群聊演示组件
 * 展示AI如何自主决策并创建PubSub群聊
 */

import React, { useState } from 'react';
import enhancedToolService from '@/services/enhancedToolService';

const AiGroupChatDemo: React.FC = () => {
  const [isLoading, setIsLoading] = useState(false);
  const [result, setResult] = useState<any>(null);
  const [error, setError] = useState<string | null>(null);
  const [taskDescription, setTaskDescription] = useState('开发一个完整的Web应用，需要代码、测试、部署、文档等多个AI协作');

  const handleAiCreateGroup = async () => {
    setIsLoading(true);
    setError(null);
    setResult(null);

    try {
      console.log('🤖 AI开始分析任务并决定是否创建群聊...');
      
      // AI自主创建群聊
      const result = await enhancedToolService.demoAiCreateGroup();
      
      setResult(result);
      
      if (result.success) {
        console.log('🎉 AI成功创建群聊！', result.data);
      } else {
        console.error('❌ AI创建群聊失败:', result.error);
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : '未知错误';
      setError(errorMessage);
      console.error('演示失败:', err);
    } finally {
      setIsLoading(false);
    }
  };

  const handleTestGroupChatTools = async () => {
    setIsLoading(true);
    setError(null);
    setResult(null);

    try {
      console.log('🧪 测试群聊工具...');
      
      const results = await enhancedToolService.testGroupChatTools();
      
      setResult({ type: 'test_results', results });
      
      console.log('测试完成:', results);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : '未知错误';
      setError(errorMessage);
      console.error('测试失败:', err);
    } finally {
      setIsLoading(false);
    }
  };

  const handleCreateSpecificGroup = async () => {
    setIsLoading(true);
    setError(null);
    setResult(null);

    try {
      console.log('📝 创建特定目的的群聊...');
      
      const result = await enhancedToolService.executeToolEnhanced(
        'group_chat_ai_create',
        { purpose: taskDescription },
        {}
      );
      
      setResult(result);
      
      if (result.success) {
        console.log('群聊创建成功:', result.data);
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : '未知错误';
      setError(errorMessage);
      console.error('创建失败:', err);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div style={{
      padding: '20px',
      maxWidth: '800px',
      margin: '0 auto',
      fontFamily: 'system-ui, -apple-system, sans-serif'
    }}>
      <h1 style={{ color: '#333', borderBottom: '2px solid #4CAF50', paddingBottom: '10px' }}>
        🤖 AI自主创建群聊演示
      </h1>
      
      <div style={{
        backgroundColor: '#f5f5f5',
        padding: '20px',
        borderRadius: '8px',
        marginBottom: '20px'
      }}>
        <h3 style={{ color: '#555', marginTop: 0 }}>🎯 演示目标</h3>
        <p>展示AI智能体如何自主决策并创建PubSub群聊，用于多智能体协作。</p>
        
        <h4 style={{ color: '#666' }}>核心功能：</h4>
        <ul>
          <li>AI分析任务复杂度，决定是否需要创建群聊</li>
          <li>AI自主创建PubSub群聊（无需钱包验证）</li>
          <li>AI发送欢迎消息并管理群聊</li>
          <li>支持多种群聊操作（创建、加入、发送消息、列表）</li>
        </ul>
      </div>

      <div style={{
        backgroundColor: '#e8f5e9',
        padding: '20px',
        borderRadius: '8px',
        marginBottom: '20px'
      }}>
        <h3 style={{ color: '#2e7d32', marginTop: 0 }}>🧠 AI决策流程</h3>
        <ol>
          <li><strong>任务分析</strong>：评估是否需要多智能体协作</li>
          <li><strong>需求评估</strong>：判断是否需要实时沟通和协调</li>
          <li><strong>目的确定</strong>：明确群聊的主要功能</li>
          <li><strong>自主创建</strong>：调用群聊创建工具</li>
          <li><strong>协作管理</strong>：邀请AI、发送消息、跟踪进度</li>
        </ol>
      </div>

      <div style={{
        backgroundColor: '#e3f2fd',
        padding: '20px',
        borderRadius: '8px',
        marginBottom: '20px'
      }}>
        <h3 style={{ color: '#1565c0', marginTop: 0 }}>🚀 演示操作</h3>
        
        <div style={{ marginBottom: '15px' }}>
          <label style={{ display: 'block', marginBottom: '5px', fontWeight: 'bold' }}>
            任务描述：
          </label>
          <textarea
            value={taskDescription}
            onChange={(e) => setTaskDescription(e.target.value)}
            style={{
              width: '100%',
              height: '80px',
              padding: '10px',
              borderRadius: '4px',
              border: '1px solid #ccc',
              fontSize: '14px',
              resize: 'vertical'
            }}
          />
          <small style={{ color: '#666' }}>
            描述一个需要多AI协作的复杂任务，AI会分析并决定是否创建群聊
          </small>
        </div>

        <div style={{ display: 'flex', gap: '10px', flexWrap: 'wrap' }}>
          <button
            onClick={handleAiCreateGroup}
            disabled={isLoading}
            style={{
              padding: '10px 20px',
              backgroundColor: '#4CAF50',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: isLoading ? 'not-allowed' : 'pointer',
              fontSize: '14px',
              fontWeight: 'bold'
            }}
          >
            {isLoading ? '🤖 AI思考中...' : '🤖 AI自主创建群聊'}
          </button>

          <button
            onClick={handleCreateSpecificGroup}
            disabled={isLoading}
            style={{
              padding: '10px 20px',
              backgroundColor: '#2196F3',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: isLoading ? 'not-allowed' : 'pointer',
              fontSize: '14px'
            }}
          >
            📝 创建特定群聊
          </button>

          <button
            onClick={handleTestGroupChatTools}
            disabled={isLoading}
            style={{
              padding: '10px 20px',
              backgroundColor: '#FF9800',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: isLoading ? 'not-allowed' : 'pointer',
              fontSize: '14px'
            }}
          >
            🧪 测试群聊工具
          </button>
        </div>
      </div>

      {isLoading && (
        <div style={{
          padding: '20px',
          textAlign: 'center',
          backgroundColor: '#fff3cd',
          borderRadius: '8px',
          marginBottom: '20px'
        }}>
          <div style={{ fontSize: '24px', marginBottom: '10px' }}>⏳</div>
          <p style={{ margin: 0, color: '#856404' }}>
            AI正在分析任务并决策... 请稍候
          </p>
        </div>
      )}

      {error && (
        <div style={{
          padding: '15px',
          backgroundColor: '#f8d7da',
          color: '#721c24',
          borderRadius: '8px',
          marginBottom: '20px'
        }}>
          <h4 style={{ marginTop: 0 }}>❌ 错误</h4>
          <pre style={{
            backgroundColor: 'rgba(0,0,0,0.05)',
            padding: '10px',
            borderRadius: '4px',
            overflow: 'auto',
            fontSize: '12px'
          }}>
            {error}
          </pre>
        </div>
      )}

      {result && (
        <div style={{
          padding: '20px',
          backgroundColor: '#d4edda',
          color: '#155724',
          borderRadius: '8px'
        }}>
          <h4 style={{ marginTop: 0 }}>✅ 执行结果</h4>
          
          {result.type === 'test_results' ? (
            <div>
              <p><strong>测试完成！</strong> 共执行 {result.results.length} 个操作：</p>
              <ul>
                {result.results.map((r: any, i: number) => (
                  <li key={i}>
                    操作 {i + 1}: {r.success ? '✅ 成功' : '❌ 失败'} - {r.output || r.error}
                  </li>
                ))}
              </ul>
            </div>
          ) : result.success ? (
            <div>
              <p><strong>🎉 成功！</strong> {result.output}</p>
              {result.data && (
                <div>
                  <p><strong>群聊数据：</strong></p>
                  <pre style={{
                    backgroundColor: 'rgba(0,0,0,0.05)',
                    padding: '10px',
                    borderRadius: '4px',
                    overflow: 'auto',
                    fontSize: '12px'
                  }}>
                    {JSON.stringify(result.data, null, 2)}
                  </pre>
                </div>
              )}
            </div>
          ) : (
            <div>
              <p><strong>❌ 失败：</strong> {result.error || result.output}</p>
            </div>
          )}
        </div>
      )}

      <div style={{
        marginTop: '30px',
        padding: '15px',
        backgroundColor: '#f8f9fa',
        borderRadius: '8px',
        fontSize: '14px',
        color: '#6c757d'
      }}>
        <h4 style={{ marginTop: 0, color: '#495057' }}>💡 技术说明</h4>
        <ul>
          <li><strong>无需钱包验证</strong>：群聊创建基于IPFS PubSub，不需要Web3钱包</li>
          <li><strong>AI自主决策</strong>：AI分析任务复杂度，自主决定是否创建群聊</li>
          <li><strong>工具集成</strong>：群聊创建已集成到工具系统，AI可以直接调用</li>
          <li><strong>实时协作</strong>：支持多AI实时沟通和任务协调</li>
          <li><strong>向后兼容</strong>：与现有群聊系统完全兼容</li>
        </ul>
      </div>
    </div>
  );
};

export default AiGroupChatDemo;