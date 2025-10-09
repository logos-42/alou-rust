<template>
  <div class="login-container">
    <div class="login-box">
      <div class="logo">
        <img src="@/assets/logo.svg" alt="Alou Logo" />
      </div>
      
      <h1 class="title">欢迎使用 Alou 智能助手</h1>
      <p class="subtitle">您的个人AI工作流助理</p>

      <div v-if="error" class="error-message">
        {{ error }}
      </div>

      <GoogleLoginButton 
        @click="handleLogin" 
        :loading="isLoading" 
      />

      <p class="terms">
        登录即表示您同意我们的<a href="/terms">服务条款</a>和<a href="/privacy">隐私政策</a>
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import GoogleLoginButton from '@/components/GoogleLoginButton.vue'

const router = useRouter()
const authStore = useAuthStore()

const isLoading = ref(false)
const error = ref('')

async function handleLogin() {
  try {
    isLoading.value = true
    error.value = ''
    await authStore.loginWithGoogle()
  } catch (err: any) {
    error.value = err.message || '登录失败，请重试'
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
  border-radius: 16px;
  padding: 48px 40px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  max-width: 400px;
  width: 100%;
  text-align: center;
}

.logo {
  margin-bottom: 24px;
}

.logo img {
  width: 64px;
  height: 64px;
}

.title {
  font-size: 24px;
  font-weight: 600;
  color: #1a1a1a;
  margin-bottom: 8px;
}

.subtitle {
  font-size: 16px;
  color: #666;
  margin-bottom: 32px;
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

.terms {
  margin-top: 24px;
  font-size: 12px;
  color: #999;
  line-height: 1.5;
}

.terms a {
  color: #667eea;
  text-decoration: none;
}

.terms a:hover {
  text-decoration: underline;
}
</style>

