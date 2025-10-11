<template>
  <div class="login-container">
    <div class="login-box">
      <button @click="goBack" class="close-btn" title="返回">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
          <path d="M19,6.41L17.59,5L12,10.59L6.41,5L5,6.41L10.59,12L5,17.59L6.41,19L12,13.41L17.59,19L19,17.59L13.41,12L19,6.41Z"/>
        </svg>
      </button>

      <div class="logo">
        <div class="logo-icon">💰</div>
      </div>
      
      <h1 class="title">钱包登录</h1>
      <p class="subtitle">使用您的加密钱包安全登录</p>

      <div v-if="error" class="error-message">
        {{ error }}
      </div>

      <div class="login-methods">
        <div class="method-tabs">
          <button 
            :class="['tab', { active: loginMethod === 'privateKey' }]"
            @click="loginMethod = 'privateKey'"
          >
            私钥登录
          </button>
          <button 
            :class="['tab', { active: loginMethod === 'mnemonic' }]"
            @click="loginMethod = 'mnemonic'"
          >
            助记词登录
          </button>
        </div>

        <div class="login-form">
          <div v-if="loginMethod === 'privateKey'" class="form-group">
            <label for="privateKey">私钥</label>
            <textarea
              id="privateKey"
              v-model="privateKey"
              placeholder="输入您的私钥（0x开头的64位十六进制字符串）"
              rows="3"
              :disabled="isLoading"
            ></textarea>
            <p class="hint">💡 您的私钥不会被上传到服务器</p>
          </div>

          <div v-else class="form-group">
            <label for="mnemonic">助记词</label>
            <textarea
              id="mnemonic"
              v-model="mnemonic"
              placeholder="输入您的助记词（12或24个单词，用空格分隔）"
              rows="4"
              :disabled="isLoading"
            ></textarea>
            <p class="hint">💡 您的助记词不会被上传到服务器</p>
          </div>

          <button 
            @click="handleLogin" 
            :disabled="isLoading || !canSubmit"
            class="login-button"
          >
            <span v-if="isLoading">登录中...</span>
            <span v-else>🔐 登录</span>
          </button>
        </div>
      </div>

      <div class="security-notice">
        <div class="notice-icon">🔒</div>
        <div class="notice-content">
          <h3>安全提示</h3>
          <ul>
            <li>请妥善保管您的私钥和助记词</li>
            <li>不要在公共网络环境下输入</li>
            <li>我们不会保存您的凭证信息</li>
          </ul>
        </div>
      </div>

      <p class="terms">
        登录即表示您同意我们的<a href="/terms">服务条款</a>和<a href="/privacy">隐私政策</a>
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const authStore = useAuthStore()

const isLoading = ref(false)
const error = ref('')
const loginMethod = ref<'privateKey' | 'mnemonic'>('privateKey')
const privateKey = ref('')
const mnemonic = ref('')

const canSubmit = computed(() => {
  if (loginMethod.value === 'privateKey') {
    return privateKey.value.trim().length > 0
  } else {
    const words = mnemonic.value.trim().split(/\s+/)
    return words.length === 12 || words.length === 24
  }
})

function goBack() {
  router.push('/')
}

async function handleLogin() {
  try {
    isLoading.value = true
    error.value = ''

    if (loginMethod.value === 'privateKey') {
      await authStore.loginWithWallet({ privateKey: privateKey.value.trim() })
    } else {
      await authStore.loginWithWallet({ mnemonic: mnemonic.value.trim() })
    }

    // 登录成功后返回主页
    router.push('/')
  } catch (err: any) {
    error.value = err.message || '登录失败，请检查您的凭证'
    isLoading.value = false
  }
}
</script>

<style scoped>
.login-container {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  padding: 20px;
}

.login-box {
  background: white;
  border-radius: 20px;
  padding: 48px 40px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  max-width: 550px;
  width: 100%;
  position: relative;
}

.close-btn {
  position: absolute;
  top: 20px;
  right: 20px;
  background: transparent;
  border: none;
  color: #999;
  cursor: pointer;
  padding: 8px;
  border-radius: 50%;
  transition: all 0.3s ease;
  display: flex;
  align-items: center;
  justify-content: center;
}

.close-btn:hover {
  background: #f5f5f5;
  color: #333;
}

.logo {
  text-align: center;
  margin-bottom: 24px;
}

.logo-icon {
  font-size: 64px;
  display: inline-block;
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

.title {
  font-size: 28px;
  font-weight: 700;
  color: #1a1a1a;
  margin-bottom: 8px;
  text-align: center;
}

.subtitle {
  font-size: 16px;
  color: #666;
  margin-bottom: 32px;
  text-align: center;
}

.error-message {
  background-color: #fee;
  border: 1px solid #fcc;
  color: #c33;
  padding: 12px;
  border-radius: 8px;
  margin-bottom: 20px;
  font-size: 14px;
}

.login-methods {
  margin-bottom: 32px;
}

.method-tabs {
  display: flex;
  gap: 8px;
  margin-bottom: 24px;
  background: #f5f5f5;
  padding: 4px;
  border-radius: 12px;
}

.tab {
  flex: 1;
  background: transparent;
  border: none;
  padding: 12px 20px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  color: #666;
  cursor: pointer;
  transition: all 0.3s ease;
}

.tab.active {
  background: white;
  color: #667eea;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.login-form {
  text-align: left;
}

.form-group {
  margin-bottom: 24px;
}

.form-group label {
  display: block;
  font-size: 14px;
  font-weight: 600;
  color: #333;
  margin-bottom: 8px;
}

.form-group textarea {
  width: 100%;
  padding: 12px;
  border: 2px solid #e0e0e0;
  border-radius: 8px;
  font-size: 14px;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  resize: vertical;
  transition: all 0.3s ease;
}

.form-group textarea:focus {
  outline: none;
  border-color: #667eea;
  box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
}

.form-group textarea:disabled {
  background: #f5f5f5;
  cursor: not-allowed;
}

.hint {
  font-size: 12px;
  color: #999;
  margin-top: 8px;
  margin-bottom: 0;
}

.login-button {
  width: 100%;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border: none;
  padding: 14px 24px;
  border-radius: 8px;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.3);
}

.login-button:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(102, 126, 234, 0.4);
}

.login-button:disabled {
  background: #ccc;
  cursor: not-allowed;
  transform: none;
  box-shadow: none;
}

.security-notice {
  background: #f0f7ff;
  border: 1px solid #d0e4ff;
  border-radius: 12px;
  padding: 16px;
  display: flex;
  gap: 12px;
  margin-bottom: 24px;
}

.notice-icon {
  font-size: 24px;
  flex-shrink: 0;
}

.notice-content {
  flex: 1;
}

.notice-content h3 {
  font-size: 14px;
  font-weight: 600;
  color: #1a1a1a;
  margin: 0 0 8px 0;
}

.notice-content ul {
  margin: 0;
  padding-left: 20px;
  font-size: 13px;
  color: #666;
  line-height: 1.6;
}

.notice-content li {
  margin-bottom: 4px;
}

.terms {
  font-size: 12px;
  color: #999;
  line-height: 1.5;
  text-align: center;
}

.terms a {
  color: #667eea;
  text-decoration: none;
}

.terms a:hover {
  text-decoration: underline;
}

@media (max-width: 600px) {
  .login-box {
    padding: 32px 24px;
  }

  .title {
    font-size: 24px;
  }

  .subtitle {
    font-size: 14px;
  }
}
</style>

