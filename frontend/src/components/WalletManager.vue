<template>
  <div class="wallet-manager" :class="{ 'dark-mode': isDarkMode }">
    <!-- 顶部导航栏 -->
    <nav class="top-nav">
      <div class="nav-content">
        <div class="logo-section">
          <div class="logo">💰</div>
          <h1 class="app-title">钱包管理</h1>
        </div>
        
        <div class="nav-controls">
          <button @click="toggleDarkMode" class="theme-toggle" title="切换主题">
            <span v-if="isDarkMode">🌞</span>
            <span v-else>🌙</span>
          </button>
          
          <button @click="goBack" class="back-btn" title="返回聊天">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
              <path d="M20,11V13H8L13.5,18.5L12.08,19.92L4.16,12L12.08,4.08L13.5,5.5L8,11H20Z"/>
            </svg>
            返回
          </button>
        </div>
      </div>
    </nav>

    <!-- 主内容区域 -->
    <div class="wallet-container">
      <div class="wallet-window">
        
        <!-- 钱包概览 -->
        <div class="wallet-overview">
          <h2>钱包概览</h2>
          <div class="wallet-stats">
            <div class="stat-card">
              <div class="stat-icon">🔗</div>
              <div class="stat-info">
                <div class="stat-label">当前网络</div>
                <div class="stat-value">{{ currentNetwork }}</div>
              </div>
            </div>
            <div class="stat-card">
              <div class="stat-icon">💳</div>
              <div class="stat-info">
                <div class="stat-label">钱包数量</div>
                <div class="stat-value">{{ walletCount }}</div>
              </div>
            </div>
            <div class="stat-card">
              <div class="stat-icon">⚡</div>
              <div class="stat-info">
                <div class="stat-label">连接状态</div>
                <div class="stat-value" :class="connectionStatus">{{ statusText }}</div>
              </div>
            </div>
          </div>
        </div>

        <!-- 钱包列表 -->
        <div class="wallet-list">
          <div class="section-header">
            <h3>我的钱包</h3>
          </div>
          
          <div v-if="wallets.length === 0" class="empty-state">
            <div class="empty-icon">💼</div>
            <h4>还没有钱包</h4>
            <p>请使用"添加私钥"功能来加载你的钱包</p>
          </div>
          
          <div v-else class="wallet-cards">
            <div 
              v-for="wallet in wallets" 
              :key="wallet.label"
              class="wallet-card"
              :class="{ active: wallet.isActive }"
            >
              <div class="wallet-header">
                <div class="wallet-info">
                  <div class="wallet-label">{{ wallet.label }}</div>
                  <div class="wallet-address">{{ formatAddress(wallet.address) }}</div>
                </div>
                <div class="wallet-actions">
                  <button 
                    @click="addPrivateKey" 
                    class="action-btn primary"
                  >
                    添加私钥
                  </button>
                  <button @click="removeWallet(wallet.label)" class="action-btn danger">
                    删除
                  </button>
                </div>
              </div>
              <div class="wallet-balance">
                <div class="balance-item">
                  <span class="balance-label">ETH</span>
                  <span class="balance-value">{{ wallet.ethBalance || '0.0' }}</span>
                </div>
                <div class="balance-item">
                  <span class="balance-label">USDC</span>
                  <span class="balance-value">{{ wallet.usdcBalance || '0.0' }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 网络设置 -->
        <div class="network-settings">
          <h3>网络设置</h3>
          <div class="network-selector">
            <label v-for="network in networks" :key="network.value" class="network-option">
              <input 
                type="radio" 
                :value="network.value" 
                v-model="currentNetwork"
                @change="switchNetwork"
              >
              <span class="network-info">
                <span class="network-name">{{ network.name }}</span>
                <span class="network-desc">{{ network.description }}</span>
              </span>
            </label>
          </div>
        </div>

        <!-- 操作日志 -->
        <div class="operation-log">
          <h3>操作日志</h3>
          <div class="log-list">
            <div v-if="logs.length === 0" class="empty-log">
              暂无操作记录
            </div>
            <div v-else>
              <div 
                v-for="log in logs" 
                :key="log.id"
                class="log-item"
                :class="log.type"
              >
                <div class="log-icon">
                  <span v-if="log.type === 'success'">✅</span>
                  <span v-else-if="log.type === 'error'">❌</span>
                  <span v-else>ℹ️</span>
                </div>
                <div class="log-content">
                  <div class="log-message">{{ log.message }}</div>
                  <div class="log-time">{{ formatTime(log.timestamp) }}</div>
                </div>
              </div>
            </div>
          </div>
        </div>

      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'

interface Wallet {
  label: string
  address: string
  isActive: boolean
  ethBalance?: string
  usdcBalance?: string
}

interface Log {
  id: string
  type: 'success' | 'error' | 'info'
  message: string
  timestamp: number
}

interface Network {
  value: string
  name: string
  description: string
}

// 响应式数据
const isDarkMode = ref(false)
const connectionStatus = ref<'connected' | 'disconnected' | 'error'>('disconnected')
const statusText = ref('连接中...')
const currentNetwork = ref('ethereum_sepolia')
const wallets = ref<Wallet[]>([])
const logs = ref<Log[]>([])

const networks: Network[] = [
  { value: 'ethereum_sepolia', name: 'Ethereum Sepolia', description: '以太坊测试网' },
  { value: 'base_sepolia', name: 'Base Sepolia', description: 'Base测试网' },
  { value: 'polygon_amoy', name: 'Polygon Amoy', description: 'Polygon测试网' },
  { value: 'ethereum_mainnet', name: 'Ethereum Mainnet', description: '以太坊主网' },
  { value: 'base_mainnet', name: 'Base Mainnet', description: 'Base主网' },
]

// 计算属性
const walletCount = computed(() => wallets.value.length)

// 生命周期
onMounted(() => {
  // 检查系统主题偏好
  const savedTheme = localStorage.getItem('alou-theme')
  if (savedTheme) {
    isDarkMode.value = savedTheme === 'dark'
  } else {
    isDarkMode.value = window.matchMedia('(prefers-color-scheme: dark)').matches
  }
  
  loadWallets()
  checkConnection()
})

// 方法
function goBack() {
  // 返回到聊天页面
  window.history.back()
}

function toggleDarkMode() {
  isDarkMode.value = !isDarkMode.value
  localStorage.setItem('alou-theme', isDarkMode.value ? 'dark' : 'light')
}

async function checkConnection() {
  try {
    const API_BASE_URL = import.meta.env.PROD ? '' : 'http://localhost:3001'
    const response = await fetch(`${API_BASE_URL}/api/health`)
    if (response.ok) {
      connectionStatus.value = 'connected'
      statusText.value = '已连接'
    } else {
      connectionStatus.value = 'error'
      statusText.value = `错误 ${response.status}`
    }
  } catch (error) {
    connectionStatus.value = 'disconnected'
    statusText.value = '连接失败'
  }
}

async function loadWallets() {
  // 这里应该调用MCP API获取钱包列表
  // 暂时使用模拟数据
  wallets.value = [
    {
      label: '主钱包',
      address: '0x308339a0C2fA14475EC42fbF0b8Fae239b293b52',
      isActive: true,
      ethBalance: '0.001751919051897896',
      usdcBalance: '0.0'
    }
  ]
}


async function switchWallet(label: string) {
  try {
    // 调用MCP API切换钱包
    const API_BASE_URL = import.meta.env.PROD ? '' : 'http://localhost:3001'
    const response = await fetch(`${API_BASE_URL}/api/chat`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        message: `切换钱包到 ${label}`,
        session_id: 'wallet_manager'
      })
    })
    
    if (response.ok) {
      // 更新活跃状态
      wallets.value.forEach(wallet => {
        wallet.isActive = wallet.label === label
      })
      addLog('success', `已切换到钱包: ${label}`)
    } else {
      addLog('error', '切换钱包失败')
    }
  } catch (error) {
    addLog('error', `切换钱包时出错: ${error}`)
  }
}

async function removeWallet(label: string) {
  if (!confirm(`确定要删除钱包 "${label}" 吗？`)) return
  
  try {
    // 调用MCP API删除钱包
    const API_BASE_URL = import.meta.env.PROD ? '' : 'http://localhost:3001'
    const response = await fetch(`${API_BASE_URL}/api/chat`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        message: `请使用remove_wallet工具删除标签为"${label}"的钱包`,
        session_id: 'wallet_manager'
      })
    })
    
    if (response.ok) {
      const result = await response.json()
      wallets.value = wallets.value.filter(wallet => wallet.label !== label)
      addLog('success', `已删除钱包: ${label}`)
    } else {
      addLog('error', '删除钱包失败')
    }
  } catch (error) {
    addLog('error', `删除钱包时出错: ${error}`)
  }
}

async function addPrivateKey() {
  const privateKey = prompt('请输入私钥（带0x前缀或不带都可以）:')
  if (!privateKey) return
  
  // 确保私钥格式正确（添加0x前缀）
  const formattedKey = privateKey.startsWith('0x') ? privateKey : `0x${privateKey}`
  
  try {
    // 调用后端API更新mcp.json中的私钥
    const API_BASE_URL = import.meta.env.PROD ? '' : 'http://localhost:3001'
    const response = await fetch(`${API_BASE_URL}/api/update-private-key`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        private_key: formattedKey
      })
    })
    
    if (response.ok) {
      const result = await response.json()
      addLog('success', `私钥已更新到mcp.json: ${formattedKey.slice(0, 10)}...`)
      
      // 重新启动MCP服务以加载新私钥
      await restartMcpService()
      
      // 重新加载钱包信息
      loadWallets()
    } else {
      addLog('error', '私钥更新失败')
    }
  } catch (error) {
    addLog('error', `设置私钥时出错: ${error}`)
  }
}

async function restartMcpService() {
  try {
    const API_BASE_URL = import.meta.env.PROD ? '' : 'http://localhost:3001'
    const response = await fetch(`${API_BASE_URL}/api/restart-mcp`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' }
    })
    
    if (response.ok) {
      addLog('success', 'MCP服务已重启，新私钥已生效')
    } else {
      addLog('error', 'MCP服务重启失败，请手动重启')
    }
  } catch (error) {
    addLog('error', '无法重启MCP服务，请手动重启')
  }
}

async function switchNetwork() {
  try {
    // 调用MCP API切换网络
    const API_BASE_URL = import.meta.env.PROD ? '' : 'http://localhost:3001'
    const response = await fetch(`${API_BASE_URL}/api/chat`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        message: `切换网络到 ${currentNetwork.value}`,
        session_id: 'wallet_manager'
      })
    })
    
    if (response.ok) {
      addLog('success', `已切换到网络: ${currentNetwork.value}`)
      loadWallets() // 重新加载钱包余额
    } else {
      addLog('error', '切换网络失败')
    }
  } catch (error) {
    addLog('error', `切换网络时出错: ${error}`)
  }
}

function formatAddress(address: string): string {
  if (address.length <= 10) return address
  return `${address.slice(0, 6)}...${address.slice(-4)}`
}

function formatTime(timestamp: number): string {
  return new Date(timestamp).toLocaleString('zh-CN')
}

function addLog(type: 'success' | 'error' | 'info', message: string) {
  logs.value.unshift({
    id: `log_${Date.now()}`,
    type,
    message,
    timestamp: Date.now()
  })
  
  // 限制日志数量
  if (logs.value.length > 50) {
    logs.value = logs.value.slice(0, 50)
  }
}
</script>

<style scoped>
/* CSS变量定义 */
.wallet-manager {
  --primary-color: #6366f1;
  --primary-hover: #5855eb;
  --secondary-color: #f1f5f9;
  --text-primary: #1e293b;
  --text-secondary: #64748b;
  --border-color: #e2e8f0;
  --background: #ffffff;
  --surface: #f8fafc;
  --success-color: #10b981;
  --warning-color: #f59e0b;
  --error-color: #ef4444;
  --shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
  --shadow-lg: 0 10px 15px -3px rgba(0, 0, 0, 0.1);
  
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: var(--surface);
  color: var(--text-primary);
  transition: all 0.3s ease;
}

/* Dark模式变量 */
.wallet-manager.dark-mode {
  --primary-color: #818cf8;
  --primary-hover: #6366f1;
  --secondary-color: #1e293b;
  --text-primary: #f1f5f9;
  --text-secondary: #94a3b8;
  --border-color: #334155;
  --background: #0f172a;
  --surface: #1e293b;
  --shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.3);
  --shadow-lg: 0 10px 15px -3px rgba(0, 0, 0, 0.3);
}

/* 顶部导航 */
.top-nav {
  background: var(--background);
  border-bottom: 1px solid var(--border-color);
  padding: 1rem 2rem;
  box-shadow: var(--shadow);
  z-index: 10;
}

.nav-content {
  max-width: 1200px;
  margin: 0 auto;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.logo-section {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.logo {
  font-size: 2rem;
  background: linear-gradient(135deg, var(--primary-color), #8b5cf6);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.app-title {
  font-size: 1.5rem;
  font-weight: 700;
  margin: 0;
  background: linear-gradient(135deg, var(--primary-color), #8b5cf6);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.nav-controls {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.theme-toggle, .back-btn {
  background: var(--secondary-color);
  border: none;
  border-radius: 0.75rem;
  padding: 0.75rem 1rem;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
  font-size: 0.875rem;
  font-weight: 500;
  transition: all 0.3s ease;
  color: var(--text-primary);
}

.theme-toggle:hover, .back-btn:hover {
  background: var(--border-color);
  transform: scale(1.05);
}

/* 主容器 */
.wallet-container {
  flex: 1;
  display: flex;
  justify-content: center;
  padding: 2rem;
  overflow-y: auto;
}

.wallet-window {
  width: 100%;
  max-width: 1000px;
  background: var(--background);
  border-radius: 1.5rem;
  box-shadow: var(--shadow-lg);
  padding: 2rem;
  display: flex;
  flex-direction: column;
  gap: 2rem;
}

/* 钱包概览 */
.wallet-overview h2 {
  margin: 0 0 1.5rem 0;
  font-size: 1.5rem;
  font-weight: 700;
}

.wallet-stats {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1rem;
}

.stat-card {
  background: var(--surface);
  border: 1px solid var(--border-color);
  border-radius: 1rem;
  padding: 1.5rem;
  display: flex;
  align-items: center;
  gap: 1rem;
}

.stat-icon {
  font-size: 2rem;
}

.stat-info {
  flex: 1;
}

.stat-label {
  font-size: 0.875rem;
  color: var(--text-secondary);
  margin-bottom: 0.25rem;
}

.stat-value {
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--text-primary);
}

.stat-value.connected {
  color: var(--success-color);
}

.stat-value.error {
  color: var(--error-color);
}

/* 钱包列表 */
.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1.5rem;
}

.section-header h3 {
  margin: 0;
  font-size: 1.25rem;
  font-weight: 600;
}

.create-btn {
  background: var(--primary-color);
  color: white;
  border: none;
  border-radius: 0.75rem;
  padding: 0.75rem 1rem;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
  font-size: 0.875rem;
  font-weight: 500;
  transition: all 0.3s ease;
}

.create-btn:hover {
  background: var(--primary-hover);
  transform: scale(1.05);
}

.empty-state {
  text-align: center;
  padding: 3rem 2rem;
  color: var(--text-secondary);
}

.empty-icon {
  font-size: 4rem;
  margin-bottom: 1rem;
}

.empty-state h4 {
  margin: 0 0 0.5rem 0;
  font-size: 1.25rem;
  color: var(--text-primary);
}

.empty-state p {
  margin: 0 0 2rem 0;
  line-height: 1.6;
}

.primary-btn {
  background: var(--primary-color);
  color: white;
  border: none;
  border-radius: 0.75rem;
  padding: 0.75rem 2rem;
  cursor: pointer;
  font-size: 1rem;
  font-weight: 500;
  transition: all 0.3s ease;
}

.primary-btn:hover {
  background: var(--primary-hover);
  transform: scale(1.05);
}

.wallet-cards {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.wallet-card {
  background: var(--surface);
  border: 1px solid var(--border-color);
  border-radius: 1rem;
  padding: 1.5rem;
  transition: all 0.3s ease;
}

.wallet-card.active {
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.1);
}

.wallet-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 1rem;
}

.wallet-info {
  flex: 1;
}

.wallet-label {
  font-size: 1.125rem;
  font-weight: 600;
  margin-bottom: 0.25rem;
}

.wallet-address {
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 0.875rem;
  color: var(--text-secondary);
}

.wallet-actions {
  display: flex;
  gap: 0.5rem;
}

.action-btn {
  background: transparent;
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  padding: 0.5rem 1rem;
  cursor: pointer;
  font-size: 0.875rem;
  transition: all 0.3s ease;
}

.action-btn:hover:not(:disabled) {
  background: var(--secondary-color);
  border-color: var(--primary-color);
  color: var(--primary-color);
}

.action-btn:disabled {
  background: var(--primary-color);
  color: white;
  border-color: var(--primary-color);
  cursor: not-allowed;
}

.action-btn.primary {
  background: var(--primary-color);
  color: white;
  border-color: var(--primary-color);
}

.action-btn.primary:hover {
  background: var(--primary-hover);
  border-color: var(--primary-hover);
}

.action-btn.danger:hover:not(:disabled) {
  border-color: var(--error-color);
  color: var(--error-color);
}

.wallet-balance {
  display: flex;
  gap: 2rem;
}

.balance-item {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.balance-label {
  font-size: 0.875rem;
  color: var(--text-secondary);
}

.balance-value {
  font-size: 1.125rem;
  font-weight: 600;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
}

/* 网络设置 */
.network-settings h3 {
  margin: 0 0 1.5rem 0;
  font-size: 1.25rem;
  font-weight: 600;
}

.network-selector {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.network-option {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 1rem;
  background: var(--surface);
  border: 1px solid var(--border-color);
  border-radius: 0.75rem;
  cursor: pointer;
  transition: all 0.3s ease;
}

.network-option:hover {
  border-color: var(--primary-color);
}

.network-option input[type="radio"] {
  margin: 0;
}

.network-option input[type="radio"]:checked + .network-info {
  color: var(--primary-color);
}

.network-info {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.network-name {
  font-weight: 500;
}

.network-desc {
  font-size: 0.875rem;
  color: var(--text-secondary);
}

/* 操作日志 */
.operation-log h3 {
  margin: 0 0 1.5rem 0;
  font-size: 1.25rem;
  font-weight: 600;
}

.log-list {
  max-height: 300px;
  overflow-y: auto;
}

.empty-log {
  text-align: center;
  padding: 2rem;
  color: var(--text-secondary);
}

.log-item {
  display: flex;
  align-items: flex-start;
  gap: 1rem;
  padding: 1rem;
  border-radius: 0.75rem;
  margin-bottom: 0.5rem;
  transition: all 0.3s ease;
}

.log-item.success {
  background: rgba(16, 185, 129, 0.1);
  border-left: 3px solid var(--success-color);
}

.log-item.error {
  background: rgba(239, 68, 68, 0.1);
  border-left: 3px solid var(--error-color);
}

.log-item.info {
  background: rgba(59, 130, 246, 0.1);
  border-left: 3px solid #3b82f6;
}

.log-icon {
  font-size: 1.25rem;
  flex-shrink: 0;
}

.log-content {
  flex: 1;
}

.log-message {
  font-weight: 500;
  margin-bottom: 0.25rem;
}

.log-time {
  font-size: 0.875rem;
  color: var(--text-secondary);
}

/* 响应式设计 */
@media (max-width: 768px) {
  .top-nav {
    padding: 1rem;
  }
  
  .nav-content {
    flex-direction: column;
    gap: 1rem;
  }
  
  .wallet-container {
    padding: 1rem;
  }
  
  .wallet-window {
    padding: 1.5rem;
  }
  
  .wallet-stats {
    grid-template-columns: 1fr;
  }
  
  .wallet-header {
    flex-direction: column;
    gap: 1rem;
  }
  
  .wallet-actions {
    align-self: stretch;
  }
  
  .wallet-balance {
    flex-direction: column;
    gap: 1rem;
  }
}
</style>
