<template>
  <div class="chat-app" :class="{ 'dark-mode': isDarkMode }">
    <!-- 顶部导航栏 -->
    <nav class="top-nav">
      <div class="nav-content">
        <div class="logo-section">
          <div class="logo">🤖</div>
          <h1 class="app-title">Alou智能助手</h1>
        </div>
        
        <div class="nav-controls">
          <div class="status-badge" :class="connectionStatus">
            <div class="status-dot"></div>
            <span>{{ statusText }}</span>
          </div>
          
          <button @click="toggleDarkMode" class="theme-toggle" title="切换主题">
            <span v-if="isDarkMode">🌞</span>
            <span v-else>🌙</span>
          </button>
          
          <button 
            v-if="!authStore.isAuthenticated" 
            @click="goToLogin" 
            class="login-btn"
          >
            <span>🔐</span>
            <span>登录</span>
          </button>
          
          <div v-else class="user-menu">
            <button @click="toggleUserMenu" class="user-btn">
              <span>👤</span>
              <span>{{ authStore.userName }}</span>
            </button>
            <div v-if="showUserMenu" class="user-dropdown">
              <button @click="goToWallet" class="menu-item">
                <span>💰</span>
                <span>钱包管理</span>
              </button>
              <button @click="handleLogout" class="menu-item">
                <span>🚪</span>
                <span>退出登录</span>
              </button>
            </div>
          </div>
        </div>
      </div>
    </nav>

    <!-- 主聊天容器 -->
    <div class="chat-container">
      <!-- 消息区域 -->
      <div class="messages-area" ref="messagesContainer">
          <div v-if="messages.length === 0" class="welcome-screen">
            <div class="welcome-content">
              <div class="welcome-icon">🚀</div>
              <h2>欢迎使用Alou智能助手</h2>
              <p>我是区块链支付的AI助手，很高兴为您提供智能服务。</p>
              <div class="quick-actions">
                <button class="quick-action-btn" @click="sendQuickMessage('帮我查询钱包余额')">
                  <span class="action-icon">💰</span>
                  <span>查询钱包余额</span>
                </button>
                <button class="quick-action-btn" @click="sendQuickMessage('如何发送代币？')">
                  <span class="action-icon">📤</span>
                  <span>发送代币</span>
                </button>
                <button class="quick-action-btn" @click="sendQuickMessage('查看交易历史')">
                  <span class="action-icon">📊</span>
                  <span>交易历史</span>
                </button>
              </div>
            </div>
          </div>

          <div 
            v-for="message in messages" 
            :key="message.id"
            class="message-wrapper"
            :class="message.type"
          >
            <div class="message-bubble">
              <div class="message-content" v-html="formatMessage(message.content)"></div>
              <div class="message-footer">
                <span class="timestamp">{{ formatTime(message.timestamp) }}</span>
                <span v-if="message.source" class="source-tag">{{ formatSource(message.source) }}</span>
              </div>
            </div>
          </div>
          
          <!-- 加载指示器 -->
          <div v-if="isLoading" class="message-wrapper assistant">
            <div class="message-bubble loading">
              <div class="typing-animation">
                <div class="typing-dots">
                  <span></span>
                  <span></span>
                  <span></span>
                </div>
                <span class="typing-text">AI正在思考中...</span>
              </div>
            </div>
          </div>
      </div>

      <!-- 输入区域 - 固定在底部 -->
      <div class="input-area">
        <div class="input-container">
          <div class="input-wrapper">
            <textarea
              v-model="currentMessage"
              @keydown.enter.exact.prevent="sendMessage"
              @keydown.enter.shift.exact="newLine"
              @input="adjustTextareaHeight"
              placeholder="输入您的问题...（Enter发送，Shift+Enter换行）"
              ref="messageInput"
              class="message-input"
              rows="1"
            ></textarea>
            
            <div class="button-group">
              <button 
                @click="sendMessage" 
                :disabled="!currentMessage.trim() || isLoading"
                class="send-btn"
                title="发送消息"
              >
                <svg v-if="!isLoading" width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"/>
                </svg>
                <svg v-else width="20" height="20" viewBox="0 0 24 24" fill="currentColor" class="loading-icon">
                  <path d="M12,4V2A10,10 0 0,0 2,12H4A8,8 0 0,1 12,4Z"/>
                </svg>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, nextTick, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

interface Message {
  id: string
  type: 'user' | 'assistant'
  content: string
  timestamp: number
  source?: string
}

interface AgentResponse {
  response: string
  status: string
  timestamp: number
  session_id?: string
  source?: string
  error?: string
}

// 响应式数据
const messages = ref<Message[]>([])
const currentMessage = ref('')
const isLoading = ref(false)
const connectionStatus = ref<'connected' | 'disconnected' | 'error'>('disconnected')
const statusText = ref('连接中...')
const isDarkMode = ref(false)
const messagesContainer = ref<HTMLElement>()
const messageInput = ref<HTMLTextAreaElement>()
const router = useRouter()
const authStore = useAuthStore()
const showUserMenu = ref(false)

// API配置 - 适配云服务器
const API_BASE_URL = import.meta.env.PROD 
  ? '' // 生产环境使用相对路径（通过Nginx代理）
  : 'http://localhost:3001' // 开发环境直接连接后端
const sessionId = ref(`frontend_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`)

// 生命周期
onMounted(() => {
  // 检查系统主题偏好
  const savedTheme = localStorage.getItem('alou-theme')
  if (savedTheme) {
    isDarkMode.value = savedTheme === 'dark'
  } else {
    isDarkMode.value = window.matchMedia('(prefers-color-scheme: dark)').matches
  }
  
  checkConnection()
  addWelcomeMessage()
})

// 监听主题变化
watch(isDarkMode, (newValue) => {
  localStorage.setItem('alou-theme', newValue ? 'dark' : 'light')
})

// 方法
async function checkConnection() {
  try {
    const response = await fetch(`${API_BASE_URL}/api/health`)
    if (response.ok) {
      const health = await response.json()
      connectionStatus.value = 'connected'
      statusText.value = '已连接'
    } else {
      connectionStatus.value = 'error'
      statusText.value = `错误 ${response.status}`
    }
  } catch (error) {
    connectionStatus.value = 'disconnected'
    statusText.value = '连接失败'
    console.error('Connection check failed:', error)
  }
}

async function sendMessage() {
  if (!currentMessage.value.trim() || isLoading.value) return

  const userMessage: Message = {
    id: `user_${Date.now()}`,
    type: 'user',
    content: currentMessage.value.trim(),
    timestamp: Date.now()
  }

  messages.value.push(userMessage)
  const messageToSend = currentMessage.value.trim()
  currentMessage.value = ''
  isLoading.value = true

  await nextTick()
  scrollToBottom()

  try {
    const response = await fetch(`${API_BASE_URL}/api/chat`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        message: messageToSend,
        session_id: sessionId.value,
        context: {
          frontend: 'vue3',
          theme: isDarkMode.value ? 'dark' : 'light',
          timestamp: new Date().toISOString()
        }
      })
    })

    if (response.ok) {
      const agentData: AgentResponse = await response.json()
      
      const assistantMessage: Message = {
        id: `assistant_${Date.now()}`,
        type: 'assistant',
        content: agentData.response,
        timestamp: agentData.timestamp || Date.now(),
        source: agentData.source
      }

      messages.value.push(assistantMessage)
      
      if (agentData.session_id) {
        sessionId.value = agentData.session_id
      }
    } else {
      throw new Error(`HTTP ${response.status}`)
    }
  } catch (error) {
    const errorMessage: Message = {
      id: `error_${Date.now()}`,
      type: 'assistant',
      content: `❌ 抱歉，发生了错误：${error instanceof Error ? error.message : '未知错误'}\n\n请检查网络连接或稍后重试。`,
      timestamp: Date.now(),
      source: 'error'
    }
    messages.value.push(errorMessage)
  } finally {
    isLoading.value = false
    await nextTick()
    scrollToBottom()
    adjustTextareaHeight()
  }
}

function addWelcomeMessage() {
  const welcomeMessage: Message = {
    id: 'welcome',
    type: 'assistant',
    content: `👋 **欢迎使用Alou智能助手！💰 区块链支付** - 支持ETH和ERC-20代币\n•  完整的区块链功能\n\n💬 请输入您的问题，我会尽力帮助您！`,
    timestamp: Date.now(),
    source: 'system'
  }
  messages.value.push(welcomeMessage)
}


function newPage() {
  // 跳转到钱包管理页面
  router.push('/wallet')
}

function toggleDarkMode() {
  isDarkMode.value = !isDarkMode.value
}

function goToLogin() {
  router.push('/login')
}

function goToWallet() {
  showUserMenu.value = false
  router.push('/wallet')
}

function toggleUserMenu() {
  showUserMenu.value = !showUserMenu.value
}

async function handleLogout() {
  showUserMenu.value = false
  await authStore.logout()
  // 不需要跳转，因为主页不需要登录
}

function sendQuickMessage(message: string) {
  currentMessage.value = message
  sendMessage()
}

function newLine() {
  currentMessage.value += '\n'
}

function adjustTextareaHeight() {
  if (messageInput.value) {
    messageInput.value.style.height = 'auto'
    messageInput.value.style.height = Math.min(messageInput.value.scrollHeight, 120) + 'px'
  }
}

function formatMessage(content: string): string {
  return content
    .replace(/\n/g, '<br>')
    .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
    .replace(/\*(.*?)\*/g, '<em>$1</em>')
    .replace(/`(.*?)`/g, '<code>$1</code>')
    .replace(/•/g, '<span class="bullet">•</span>')
}

function formatTime(timestamp: number): string {
  return new Date(timestamp).toLocaleTimeString('zh-CN', { 
    hour: '2-digit', 
    minute: '2-digit' 
  })
}

function formatSource(source: string): string {
  const sourceMap: Record<string, string> = {
    'wasm-core': 'WASM',
    'edge-worker-proxy': 'Edge',
    'http-backend-fallback': 'Backend',
    'system': 'System',
    'test': 'Test',
    'error': 'Error'
  }
  return sourceMap[source] || source
}

function scrollToBottom() {
  if (messagesContainer.value) {
    setTimeout(() => {
      messagesContainer.value!.scrollTop = messagesContainer.value!.scrollHeight
    }, 100)
  }
}
</script>

<style scoped>
/* CSS变量定义 */
.chat-app {
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
  
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  width: 100vw;
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: var(--surface);
  color: var(--text-primary);
  transition: all 0.3s ease;
  overflow: hidden;
}

/* Dark模式变量 */
.chat-app.dark-mode {
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
  width: 100%;
  background: var(--background);
  border-bottom: 1px solid var(--border-color);
  padding: 1rem 0;
  box-shadow: var(--shadow);
  z-index: 10;
  flex-shrink: 0;
}

.nav-content {
  width: 100%;
  padding: 0 clamp(1rem, 3vw, 3rem);
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

.status-badge {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  border-radius: 2rem;
  font-size: 0.875rem;
  font-weight: 500;
  background: var(--secondary-color);
  color: var(--text-secondary);
}

.status-badge.connected {
  background: rgba(16, 185, 129, 0.1);
  color: var(--success-color);
}

.status-badge.error {
  background: rgba(239, 68, 68, 0.1);
  color: var(--error-color);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--text-secondary);
}

.status-badge.connected .status-dot {
  background: var(--success-color);
  animation: pulse 2s infinite;
}

.status-badge.error .status-dot {
  background: var(--error-color);
}

.theme-toggle {
  background: var(--secondary-color);
  border: none;
  border-radius: 50%;
  width: 44px;
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  font-size: 1.25rem;
  transition: all 0.3s ease;
}

.theme-toggle:hover {
  background: var(--border-color);
  transform: scale(1.05);
}

.login-btn {
  background: var(--primary-color);
  color: white;
  border: none;
  border-radius: 0.75rem;
  padding: 0.75rem 1.25rem;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
  font-size: 0.875rem;
  font-weight: 500;
  transition: all 0.3s ease;
}

.login-btn:hover {
  background: var(--primary-hover);
  transform: scale(1.05);
}

.user-menu {
  position: relative;
}

.user-btn {
  background: var(--secondary-color);
  border: 1px solid var(--border-color);
  border-radius: 0.75rem;
  padding: 0.75rem 1.25rem;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--text-primary);
  transition: all 0.3s ease;
}

.user-btn:hover {
  background: var(--border-color);
}

.user-dropdown {
  position: absolute;
  top: calc(100% + 0.5rem);
  right: 0;
  background: var(--background);
  border: 1px solid var(--border-color);
  border-radius: 0.75rem;
  box-shadow: var(--shadow-lg);
  min-width: 200px;
  z-index: 100;
  overflow: hidden;
}

.menu-item {
  width: 100%;
  background: transparent;
  border: none;
  padding: 0.875rem 1.25rem;
  display: flex;
  align-items: center;
  gap: 0.75rem;
  cursor: pointer;
  font-size: 0.875rem;
  color: var(--text-primary);
  transition: all 0.2s ease;
  text-align: left;
}

.menu-item:hover {
  background: var(--secondary-color);
}

.menu-item:not(:last-child) {
  border-bottom: 1px solid var(--border-color);
}

/* 聊天容器 */
.chat-container {
  width: 100%;
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--surface);
}

/* 消息区域 */
.messages-area {
  width: 100%;
  flex: 1;
  overflow-y: auto;
  scroll-behavior: smooth;
  display: flex;
  flex-direction: column;
}

.welcome-screen {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2rem;
}

.welcome-content {
  max-width: 800px;
  width: 100%;
  text-align: center;
  padding: 0 2rem;
}

.welcome-icon {
  font-size: 4rem;
  margin-bottom: 1.5rem;
  animation: float 3s ease-in-out infinite;
}

@keyframes float {
  0%, 100% {
    transform: translateY(0);
  }
  50% {
    transform: translateY(-10px);
  }
}

.welcome-content h2 {
  font-size: 2.5rem;
  font-weight: 700;
  margin: 0 0 1rem 0;
  background: linear-gradient(135deg, var(--primary-color), #8b5cf6);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.welcome-content p {
  font-size: 1.25rem;
  color: var(--text-secondary);
  margin: 0 0 2.5rem 0;
  line-height: 1.6;
}

.quick-actions {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1rem;
  margin-top: 2rem;
  max-width: 800px;
  width: 100%;
}

.quick-action-btn {
  background: var(--background);
  border: 1px solid var(--border-color);
  border-radius: 1rem;
  padding: 1.25rem 1.5rem;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.75rem;
  cursor: pointer;
  transition: all 0.3s ease;
  color: var(--text-primary);
  font-size: 0.95rem;
  font-weight: 500;
}

.quick-action-btn:hover {
  background: var(--secondary-color);
  border-color: var(--primary-color);
  transform: translateY(-2px);
  box-shadow: var(--shadow);
}

.action-icon {
  font-size: 2rem;
}

/* 消息样式 */
.message-wrapper {
  display: flex;
  animation: fadeInUp 0.3s ease;
  padding: 2rem 0;
  min-height: 100px;
}

.message-wrapper.user {
  background: var(--background);
  justify-content: center;
  border-bottom: 1px solid var(--border-color);
}

.message-wrapper.assistant {
  background: var(--surface);
  justify-content: center;
  border-bottom: 1px solid var(--border-color);
}

.message-wrapper:last-child {
  border-bottom: none;
}

.message-bubble {
  width: 100%;
  max-width: 100%;
  padding: 0 clamp(1rem, 5vw, 4rem);
  word-wrap: break-word;
}

.message-content {
  line-height: 1.8;
  font-size: 1rem;
  max-width: 1400px;
  margin: 0 auto;
}

.message-content :deep(strong) {
  font-weight: 600;
}

.message-content :deep(em) {
  font-style: italic;
  opacity: 0.9;
}

.message-content :deep(code) {
  background: rgba(0, 0, 0, 0.1);
  padding: 0.125rem 0.375rem;
  border-radius: 0.375rem;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 0.875rem;
}

.message-content :deep(.bullet) {
  color: var(--primary-color);
  font-weight: bold;
  margin-right: 0.25rem;
}

.message-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 0.75rem;
  font-size: 0.75rem;
  opacity: 0.7;
}

.source-tag {
  background: rgba(0, 0, 0, 0.1);
  padding: 0.125rem 0.5rem;
  border-radius: 1rem;
  font-size: 0.625rem;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

/* 加载动画 */
.message-bubble.loading {
  background: var(--secondary-color);
  color: var(--text-primary);
}

.typing-animation {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.typing-dots {
  display: flex;
  gap: 0.25rem;
}

.typing-dots span {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--primary-color);
  animation: typing 1.4s infinite ease-in-out;
}

.typing-dots span:nth-child(2) {
  animation-delay: 0.2s;
}

.typing-dots span:nth-child(3) {
  animation-delay: 0.4s;
}

.typing-text {
  color: var(--text-secondary);
  font-size: 0.875rem;
}

/* 输入区域 */
.input-area {
  width: 100%;
  background: var(--background);
  border-top: 1px solid var(--border-color);
  padding: 1.25rem 0;
  display: flex;
  justify-content: center;
  flex-shrink: 0;
}

.input-container {
  width: 100%;
  max-width: 100%;
  padding: 0 clamp(1rem, 5vw, 4rem);
}

.input-wrapper {
  display: flex;
  gap: 1rem;
  align-items: flex-end;
  background: var(--background);
  border: 2px solid var(--border-color);
  border-radius: 1.75rem;
  padding: 1rem 1.5rem;
  transition: all 0.3s ease;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.08);
  min-height: 64px;
}

.input-wrapper:focus-within {
  border-color: var(--primary-color);
  box-shadow: 0 4px 20px rgba(99, 102, 241, 0.15);
}

.message-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  resize: none;
  font-family: inherit;
  font-size: 1rem;
  line-height: 1.6;
  color: var(--text-primary);
  min-height: 48px;
  max-height: 200px;
  padding: 8px 0;
}

.message-input::placeholder {
  color: var(--text-secondary);
}

.button-group {
  display: flex;
  align-items: center;
}

.send-btn {
  background: var(--primary-color);
  color: white;
  border: none;
  border-radius: 0.875rem;
  width: 44px;
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s ease;
  flex-shrink: 0;
  margin-bottom: 2px;
}

.send-btn:hover:not(:disabled) {
  background: var(--primary-hover);
  transform: scale(1.05);
}

.send-btn:disabled {
  background: var(--text-secondary);
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
}

.loading-icon {
  animation: spin 1s linear infinite;
}

/* 输入选项 */
.input-options {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 1rem;
  gap: 1rem;
}

.option-toggle {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  cursor: pointer;
  user-select: none;
}

.option-toggle input[type="checkbox"] {
  display: none;
}

.toggle-slider {
  width: 44px;
  height: 24px;
  background: var(--border-color);
  border-radius: 12px;
  position: relative;
  transition: all 0.3s ease;
}

.toggle-slider::after {
  content: '';
  position: absolute;
  top: 2px;
  left: 2px;
  width: 20px;
  height: 20px;
  background: white;
  border-radius: 50%;
  transition: all 0.3s ease;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

.option-toggle input:checked + .toggle-slider {
  background: var(--primary-color);
}

.option-toggle input:checked + .toggle-slider::after {
  transform: translateX(20px);
}

.toggle-label {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--text-primary);
}

.option-btn {
  background: transparent;
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
  border-radius: 0.75rem;
  padding: 0.5rem 1rem;
  cursor: pointer;
  font-size: 0.875rem;
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  transition: all 0.3s ease;
}

.option-btn:hover {
  background: var(--secondary-color);
  border-color: var(--primary-color);
  color: var(--primary-color);
}

.test-btn {
  color: var(--success-color);
  border-color: var(--success-color);
}

.test-btn:hover {
  background: rgba(16, 185, 129, 0.1);
}

/* 动画 */
@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes typing {
  0%, 60%, 100% {
    transform: scale(1);
    opacity: 0.5;
  }
  30% {
    transform: scale(1.2);
    opacity: 1;
  }
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

/* 响应式设计 */
@media (max-width: 1200px) {
  .message-content {
    max-width: 100%;
  }
}

@media (max-width: 768px) {
  .top-nav {
    padding: 1rem;
  }
  
  .nav-content {
    gap: 0.75rem;
  }
  
  .logo-section {
    flex: 1;
  }
  
  .app-title {
    font-size: 1.25rem;
  }

  .nav-controls {
    gap: 0.5rem;
  }
  
  .status-badge span {
    display: none;
  }

  .login-btn span:last-child,
  .user-btn span:last-child {
    display: none;
  }
  
  .message-wrapper {
    padding: 1.5rem 0;
  }

  .message-bubble {
    padding: 0 1rem;
  }

  .message-content {
    font-size: 0.95rem;
  }
  
  .input-area {
    padding: 1rem;
  }

  .input-container {
    padding: 0 1rem;
  }

  .quick-actions {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 480px) {
  .welcome-content h2 {
    font-size: 1.5rem;
  }
  
  .welcome-content p {
    font-size: 1rem;
  }
  
  .feature-badges {
    flex-direction: column;
    align-items: center;
  }
  
  .message-bubble {
    max-width: 90%;
  }
}

/* 滚动条样式 */
.messages-area::-webkit-scrollbar {
  width: 6px;
}

.messages-area::-webkit-scrollbar-track {
  background: transparent;
}

.messages-area::-webkit-scrollbar-thumb {
  background: var(--border-color);
  border-radius: 3px;
}

.messages-area::-webkit-scrollbar-thumb:hover {
  background: var(--text-secondary);
}
</style>