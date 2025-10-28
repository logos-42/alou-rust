<template>
  <div class="chat-app" :class="{ 'dark-mode': isDarkMode }">
    <ChatHeader
      :connection-status="connectionStatus"
      :is-dark-mode="isDarkMode"
      :is-authenticated="authStore.isAuthenticated"
      :user-name="authStore.userName"
      @toggle-theme="toggleDarkMode"
      @go-to-login="goToLogin"
      @go-to-wallet="goToWallet"
      @logout="handleLogout"
    />

    <div class="chat-container">
      <div class="messages-area" ref="messagesContainer">
        <WelcomeScreen
          :show="showWelcomeScreen"
          @quick-message="sendQuickMessage"
        />

        <MessageList
          :messages="messages"
          :is-loading="isLoading"
          ref="messageListRef"
        />
      </div>

      <SettingsPanel
        :show="showSettings"
        :language="currentLanguage"
        @toggle="toggleSettings"
        @change-language="changeLanguage"
      />

      <ChatInput
        v-model="currentMessage"
        :is-loading="isLoading"
        @send="sendMessage"
        @new-line="newLine"
        ref="chatInputRef"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useI18n } from '@/composables/useI18n'
import { walletService, type WalletInstruction } from '@/services/wallet.service'
import ChatHeader from './ChatHeader.vue'
import WelcomeScreen from './WelcomeScreen.vue'
import MessageList from './MessageList.vue'
import ChatInput from './ChatInput.vue'
import SettingsPanel from './SettingsPanel.vue'

interface Message {
  id: string
  type: 'user' | 'assistant'
  content: string
  timestamp: number
  source?: string
}

interface AgentResponse {
  content?: string
  response?: string
  status?: string
  timestamp?: number
  session_id?: string
  source?: string
  error?: string
  tool_calls?: Array<{
    id: string
    name: string
    result: any
  }>
  wallet_instructions?: WalletInstruction[]
}

// 响应式数据
const messages = ref<Message[]>([])
const currentMessage = ref('')
const isLoading = ref(false)
const connectionStatus = ref<'connected' | 'disconnected' | 'error'>('disconnected')
const isDarkMode = ref(false)
const messagesContainer = ref<HTMLElement>()
const messageListRef = ref()
const chatInputRef = ref()
const router = useRouter()
const authStore = useAuthStore()
const { currentLanguage, setLanguage, initLanguage } = useI18n()
const showSettings = ref(false)
const showWelcomeScreen = ref(true)

// API配置
const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || (import.meta.env.DEV ? 'http://localhost:8787' : 'https://alou-edge.yuanjieliu65.workers.dev')
const sessionId = ref(`frontend_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`)
const isSessionReady = ref(false)

// 生命周期
onMounted(async () => {
  const savedTheme = localStorage.getItem('alou-theme')
  if (savedTheme) {
    isDarkMode.value = savedTheme === 'dark'
  } else {
    isDarkMode.value = window.matchMedia('(prefers-color-scheme: dark)').matches
  }
  
  initLanguage()
  await checkConnection()
  await createSession()
  isSessionReady.value = true
  
  window.addEventListener('wallet-changed', handleWalletChanged)
})

onUnmounted(() => {
  window.removeEventListener('wallet-changed', handleWalletChanged as EventListener)
})

watch(isDarkMode, (newValue) => {
  localStorage.setItem('alou-theme', newValue ? 'dark' : 'light')
})

// 方法
const handleWalletChanged = async (event: Event) => {
  const customEvent = event as CustomEvent<{ address: string }>
  console.log('Wallet changed, recreating session...', customEvent.detail.address)
  
  messages.value = []
  showWelcomeScreen.value = true
  
  await createSession()
  isSessionReady.value = true
}

/**
 * Handle tool calls from agent response
 */
async function handleToolCalls(toolCalls: Array<{ id: string; name: string; result: any }>) {
  for (const toolCall of toolCalls) {
    if (toolCall.name === 'wallet_manager' && toolCall.result) {
      const result = toolCall.result
      
      // Check if there's a wallet instruction to execute
      if (result.instruction) {
        try {
          console.log('Executing wallet instruction:', result.instruction)
          
          if (result.action === 'switch_network' && result.network) {
            // Execute network switch
            const success = await walletService.switchNetwork(result.network)
            
            if (success) {
              // Add system message about network switch
              const systemMessage: Message = {
                id: `system_${Date.now()}`,
                type: 'assistant',
                content: `✅ 已成功切换到 ${result.network.name} (${result.network.type})`,
                timestamp: Date.now(),
                source: 'system'
              }
              messages.value.push(systemMessage)
            }
          } else {
            // Execute other wallet instructions
            await walletService.executeInstruction(result.instruction)
          }
        } catch (error) {
          console.error('Failed to execute wallet instruction:', error)
          
          // Add error message
          const errorMessage: Message = {
            id: `error_${Date.now()}`,
            type: 'assistant',
            content: `❌ 钱包操作失败：${error instanceof Error ? error.message : '未知错误'}`,
            timestamp: Date.now(),
            source: 'error'
          }
          messages.value.push(errorMessage)
        }
      }
    }
  }
}

async function createSession() {
  try {
    const walletAddress = localStorage.getItem('wallet_address')
    
    const response = await fetch(`${API_BASE_URL}/api/session`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        wallet_address: walletAddress || undefined
      })
    })
    if (response.ok) {
      const data = await response.json()
      sessionId.value = data.session_id
      console.log('Session created:', sessionId.value)
    }
  } catch (error) {
    console.error('Failed to create session:', error)
  }
}

async function checkConnection() {
  try {
    const response = await fetch(`${API_BASE_URL}/api/health`)
    if (response.ok) {
      connectionStatus.value = 'connected'
    } else {
      connectionStatus.value = 'error'
    }
  } catch (error) {
    connectionStatus.value = 'disconnected'
    console.error('Connection check failed:', error)
  }
}

async function sendMessage() {
  if (!currentMessage.value.trim() || isLoading.value) return
  
  if (!isSessionReady.value) {
    await createSession()
    isSessionReady.value = true
  }
  
  if (showWelcomeScreen.value) {
    showWelcomeScreen.value = false
    await new Promise(resolve => setTimeout(resolve, 400))
  }

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
    const walletAddress = localStorage.getItem('wallet_address')
    
    const response = await fetch(`${API_BASE_URL}/api/agent/chat`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        session_id: sessionId.value,
        message: messageToSend,
        wallet_address: walletAddress || undefined
      })
    })

    if (response.ok) {
      const agentData: AgentResponse = await response.json()
      
      // Handle wallet instructions from agent
      if (agentData.tool_calls) {
        await handleToolCalls(agentData.tool_calls)
      }
      
      const assistantMessage: Message = {
        id: `assistant_${Date.now()}`,
        type: 'assistant',
        content: agentData.content || agentData.response || '收到响应',
        timestamp: agentData.timestamp || Date.now(),
        source: agentData.source || 'alou-edge'
      }

      messages.value.push(assistantMessage)
      
      if (agentData.session_id) {
        sessionId.value = agentData.session_id
      }
    } else {
      const errorData = await response.json().catch(() => ({ error: '未知错误' }))
      throw new Error(`HTTP ${response.status}: ${errorData.error || response.statusText}`)
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
    if (chatInputRef.value) {
      chatInputRef.value.adjustHeight()
    }
  }
}

async function sendQuickMessage(message: string) {
  if (!isSessionReady.value) {
    await createSession()
    isSessionReady.value = true
  }
  
  if (showWelcomeScreen.value) {
    showWelcomeScreen.value = false
    await new Promise(resolve => setTimeout(resolve, 400))
  }
  
  currentMessage.value = message
  await sendMessage()
}

function newLine() {
  currentMessage.value += '\n'
}

function toggleDarkMode() {
  isDarkMode.value = !isDarkMode.value
}

function toggleSettings() {
  showSettings.value = !showSettings.value
}

function changeLanguage(lang: string) {
  setLanguage(lang as 'zh' | 'en')
}

function goToLogin() {
  router.push('/login')
}

function goToWallet() {
  router.push('/wallet')
}

async function handleLogout() {
  await authStore.logout()
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

.chat-container {
  width: 100%;
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--surface);
}

.messages-area {
  width: 100%;
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  scroll-behavior: smooth;
}
</style>
