/**
 * Bot Gateway 设置面板
 * 允许用户配置和管理各平台 Bot
 */

import React, { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface BotGatewayConfig {
  enabled: boolean;
  port: number;
  public_domain?: string;
  platforms: {
    telegram?: TelegramConfig;
    feishu?: FeishuConfig;
  };
  command_prefix: string;
}

interface TelegramConfig {
  enabled: boolean;
  bot_token: string;
  allowed_user_ids: string[];
  allowed_chat_ids: string[];
  use_polling: boolean;
}

interface FeishuConfig {
  enabled: boolean;
  app_id: string;
  app_secret: string;
  verify_token: string;
  encrypt_key?: string;
  allowed_user_ids: string[];
  allowed_tenant_ids: string[];
}

interface ServerStatus {
  running: boolean;
  port: number;
  platforms: string[];
  uptime_seconds: number;
}

interface LogEntry {
  timestamp: number;
  level: string;
  message: string;
  platform?: string;
}

export function BotGatewaySettings() {
  const [config, setConfig] = useState<BotGatewayConfig | null>(null);
  const [status, setStatus] = useState<ServerStatus | null>(null);
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [activeTab, setActiveTab] = useState<'general' | 'telegram' | 'feishu' | 'logs'>('general');

  // 加载配置
  const loadConfig = useCallback(async () => {
    try {
      const [configData, statusData] = await Promise.all([
        invoke<BotGatewayConfig>('get_bot_gateway_config'),
        invoke<ServerStatus>('get_bot_gateway_status'),
      ]);
      setConfig(configData);
      setStatus(statusData);
    } catch (error) {
      console.error('加载配置失败:', error);
    } finally {
      setLoading(false);
    }
  }, []);

  // 加载日志
  const loadLogs = useCallback(async () => {
    try {
      const logsData = await invoke<LogEntry[]>('get_bot_gateway_logs', { lines: 50 });
      setLogs(logsData);
    } catch (error) {
      console.error('加载日志失败:', error);
    }
  }, []);

  useEffect(() => {
    loadConfig();
    loadLogs();
    const interval = setInterval(loadLogs, 5000);
    return () => clearInterval(interval);
  }, [loadConfig, loadLogs]);

  // 保存配置
  const handleSave = async () => {
    if (!config) return;
    setSaving(true);
    try {
      await invoke('update_bot_gateway_config', { config });
      await loadConfig();
      alert('配置已保存');
    } catch (error) {
      alert(`保存失败：${error}`);
    } finally {
      setSaving(false);
    }
  };

  // 切换服务状态
  const toggleService = async () => {
    if (!config) return;
    try {
      await invoke('toggle_bot_gateway', { enabled: !config.enabled });
      await loadConfig();
      await loadLogs();
    } catch (error) {
      alert(`操作失败：${error}`);
    }
  };

  // 测试平台连接
  const testConnection = async (platform: string) => {
    try {
      const result = await invoke<{ success: boolean; message: string }>('test_platform_connection', {
        platform,
      });
      alert(result.message);
    } catch (error) {
      alert(`测试失败：${error}`);
    }
  };

  if (loading) {
    return <div className="bot-gateway-settings">加载中...</div>;
  }

  return (
    <div className="bot-gateway-settings" style={{ padding: '20px' }}>
      <h2>🤖 Bot Gateway 配置</h2>

      {/* 状态栏 */}
      <div className="status-bar" style={{ 
        padding: '15px', 
        background: status?.running ? '#d4edda' : '#f8d7da',
        borderRadius: '8px',
        marginBottom: '20px'
      }}>
        <div>
          <strong>状态:</strong> {status?.running ? '✅ 运行中' : '❌ 已停止'}
        </div>
        {status?.running && (
          <>
            <div><strong>端口:</strong> {status.port}</div>
            <div><strong>启用平台:</strong> {status.platforms.join(', ') || '无'}</div>
          </>
        )}
      </div>

      {/* 控制按钮 */}
      <div className="controls" style={{ marginBottom: '20px' }}>
        <button 
          onClick={toggleService}
          style={{
            padding: '10px 20px',
            marginRight: '10px',
            background: config?.enabled ? '#dc3545' : '#28a745',
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
          }}
        >
          {config?.enabled ? '停止服务' : '启动服务'}
        </button>
        <button 
          onClick={handleSave}
          disabled={saving}
          style={{
            padding: '10px 20px',
            background: '#007bff',
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            cursor: saving ? 'not-allowed' : 'pointer',
            opacity: saving ? 0.6 : 1,
          }}
        >
          {saving ? '保存中...' : '保存配置'}
        </button>
      </div>

      {/* 选项卡 */}
      <div className="tabs" style={{ marginBottom: '20px' }}>
        {(['general', 'telegram', 'feishu', 'logs'] as const).map((tab) => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            style={{
              padding: '8px 16px',
              marginRight: '5px',
              background: activeTab === tab ? '#007bff' : '#e9ecef',
              color: activeTab === tab ? 'white' : 'black',
              border: 'none',
              borderRadius: '4px 4px 0 0',
              cursor: 'pointer',
            }}
          >
            {tab === 'general' && '⚙️ 通用设置'}
            {tab === 'telegram' && '📱 Telegram'}
            {tab === 'feishu' && '📧 飞书'}
            {tab === 'logs' && '📋 日志'}
          </button>
        ))}
      </div>

      {/* 通用设置 */}
      {activeTab === 'general' && config && (
        <div className="tab-content general-settings">
          <div style={{ marginBottom: '15px' }}>
            <label>
              <input
                type="checkbox"
                checked={config.enabled}
                onChange={(e) => setConfig({ ...config, enabled: e.target.checked })}
              />
              启用 Bot Gateway
            </label>
          </div>
          <div style={{ marginBottom: '15px' }}>
            <label>
              监听端口:
              <input
                type="number"
                value={config.port}
                onChange={(e) => setConfig({ ...config, port: parseInt(e.target.value) || 8080 })}
                style={{ marginLeft: '10px', padding: '5px' }}
              />
            </label>
          </div>
          <div style={{ marginBottom: '15px' }}>
            <label>
              命令前缀:
              <input
                type="text"
                value={config.command_prefix}
                onChange={(e) => setConfig({ ...config, command_prefix: e.target.value })}
                style={{ marginLeft: '10px', padding: '5px' }}
              />
            </label>
          </div>
        </div>
      )}

      {/* Telegram 配置 */}
      {activeTab === 'telegram' && config?.platforms.telegram && (
        <div className="tab-content telegram-settings">
          <div style={{ marginBottom: '15px' }}>
            <label>
              <input
                type="checkbox"
                checked={config.platforms.telegram.enabled}
                onChange={(e) => setConfig({ 
                  ...config, 
                  platforms: { 
                    ...config.platforms, 
                    telegram: { ...config.platforms.telegram!, enabled: e.target.checked } 
                  } 
                })}
              />
              启用 Telegram Bot
            </label>
          </div>
          <div style={{ marginBottom: '15px' }}>
            <label>
              Bot Token:
              <input
                type="password"
                value={config.platforms.telegram.bot_token}
                onChange={(e) => setConfig({ 
                  ...config, 
                  platforms: { 
                    ...config.platforms, 
                    telegram: { ...config.platforms.telegram!, bot_token: e.target.value } 
                  } 
                })}
                style={{ display: 'block', width: '100%', marginTop: '5px', padding: '8px' }}
                placeholder="123456789:ABCdefGHIjklMNOpqrsTUVwxyz"
              />
            </label>
          </div>
          <div style={{ marginBottom: '15px' }}>
            <label>
              <input
                type="checkbox"
                checked={config.platforms.telegram.use_polling}
                onChange={(e) => setConfig({ 
                  ...config, 
                  platforms: { 
                    ...config.platforms, 
                    telegram: { ...config.platforms.telegram!, use_polling: e.target.checked } 
                  } 
                })}
              />
              使用轮询模式 (无需 Webhook)
            </label>
          </div>
          <button 
            onClick={() => testConnection('telegram')}
            style={{
              padding: '8px 16px',
              background: '#007bff',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
            }}
          >
            测试连接
          </button>
          <p style={{ marginTop: '10px', fontSize: '12px', color: '#666' }}>
            💡 提示：从 <a href="https://t.me/BotFather" target="_blank" rel="noopener noreferrer">BotFather</a> 获取 Bot Token
          </p>
        </div>
      )}

      {/* 飞书配置 */}
      {activeTab === 'feishu' && config?.platforms.feishu && (
        <div className="tab-content feishu-settings">
          <div style={{ marginBottom: '15px' }}>
            <label>
              <input
                type="checkbox"
                checked={config.platforms.feishu.enabled}
                onChange={(e) => setConfig({ 
                  ...config, 
                  platforms: { 
                    ...config.platforms, 
                    feishu: { ...config.platforms.feishu!, enabled: e.target.checked } 
                  } 
                })}
              />
              启用飞书 Bot
            </label>
          </div>
          <div style={{ marginBottom: '15px' }}>
            <label>
              App ID:
              <input
                type="text"
                value={config.platforms.feishu.app_id}
                onChange={(e) => setConfig({ 
                  ...config, 
                  platforms: { 
                    ...config.platforms, 
                    feishu: { ...config.platforms.feishu!, app_id: e.target.value } 
                  } 
                })}
                style={{ display: 'block', width: '100%', marginTop: '5px', padding: '8px' }}
                placeholder="cli_a1b2c3d4e5f6g7h8"
              />
            </label>
          </div>
          <div style={{ marginBottom: '15px' }}>
            <label>
              App Secret:
              <input
                type="password"
                value={config.platforms.feishu.app_secret}
                onChange={(e) => setConfig({ 
                  ...config, 
                  platforms: { 
                    ...config.platforms, 
                    feishu: { ...config.platforms.feishu!, app_secret: e.target.value } 
                  } 
                })}
                style={{ display: 'block', width: '100%', marginTop: '5px', padding: '8px' }}
              />
            </label>
          </div>
          <div style={{ marginBottom: '15px' }}>
            <label>
              验证 Token:
              <input
                type="text"
                value={config.platforms.feishu.verify_token}
                onChange={(e) => setConfig({ 
                  ...config, 
                  platforms: { 
                    ...config.platforms, 
                    feishu: { ...config.platforms.feishu!, verify_token: e.target.value } 
                  } 
                })}
                style={{ display: 'block', width: '100%', marginTop: '5px', padding: '8px' }}
              />
            </label>
          </div>
          <button 
            onClick={() => testConnection('feishu')}
            style={{
              padding: '8px 16px',
              background: '#007bff',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
            }}
          >
            测试连接
          </button>
          <p style={{ marginTop: '10px', fontSize: '12px', color: '#666' }}>
            💡 提示：在 <a href="https://open.feishu.cn/" target="_blank" rel="noopener noreferrer">飞书开放平台</a> 创建应用获取配置
          </p>
        </div>
      )}

      {/* 日志 */}
      {activeTab === 'logs' && (
        <div className="tab-content logs">
          <div style={{ 
            maxHeight: '400px', 
            overflowY: 'auto', 
            background: '#1e1e1e', 
            padding: '10px', 
            borderRadius: '4px',
            fontFamily: 'monospace',
            fontSize: '12px',
          }}>
            {logs.length === 0 ? (
              <div style={{ color: '#666' }}>暂无日志</div>
            ) : (
              logs.map((log, index) => (
                <div 
                  key={index} 
                  style={{ 
                    marginBottom: '5px', 
                    color: log.level === 'error' ? '#f87171' : log.level === 'warn' ? '#fbbf24' : '#9ca3af',
                  }}
                >
                  <span style={{ color: '#6b7280' }}>
                    {new Date(log.timestamp * 1000).toLocaleString()}
                  </span>
                  {' '}
                  <span style={{ fontWeight: 'bold' }}>[{log.level.toUpperCase()}]</span>
                  {' '}
                  {log.platform && <span style={{ color: '#60a5fa' }}>[{log.platform}]</span>}
                  {' '}
                  {log.message}
                </div>
              ))
            )}
          </div>
        </div>
      )}
    </div>
  );
}

export default BotGatewaySettings;
