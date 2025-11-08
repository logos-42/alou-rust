<template>
  <div class="callback-container">
    <div class="callback-box">
      <div v-if="isLoading" class="loading">
        <div class="spinner"></div>
        <p>正在登录...</p>
      </div>

      <div v-else-if="error" class="error">
        <div class="error-icon">❌</div>
        <h2>登录失败</h2>
        <p>{{ error }}</p>
        <button @click="goToLogin" class="btn-primary">返回登录</button>
      </div>

      <div v-else class="success">
        <div class="success-icon">✓</div>
        <h2>登录成功</h2>
        <p>正在跳转...</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const route = useRoute()
const router = useRouter()
const authStore = useAuthStore()

const isLoading = ref(true)
const error = ref('')

onMounted(async () => {
  try {
    const code = route.query.code as string
    const state = route.query.state as string

    if (!code || !state) {
      throw new Error('缺少必要的认证参数')
    }

    // Handle OAuth callback
    await authStore.handleGoogleCallback(code, state)

    // Success - redirect to home
    setTimeout(() => {
      router.push('/')
    }, 1000)
  } catch (err: any) {
    error.value = err.message || '认证失败，请重试'
    isLoading.value = false
  }
})

function goToLogin() {
  router.push('/login')
}
</script>

<style scoped>
.callback-container {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

.callback-box {
  background: white;
  border-radius: 16px;
  padding: 48px 40px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  max-width: 400px;
  width: 100%;
  text-align: center;
}

.loading {
  padding: 20px;
}

.spinner {
  width: 48px;
  height: 48px;
  border: 4px solid #f3f3f3;
  border-top: 4px solid #667eea;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin: 0 auto 20px;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.loading p {
  color: #666;
  font-size: 16px;
}

.error {
  padding: 20px;
}

.error-icon {
  font-size: 48px;
  margin-bottom: 16px;
}

.error h2 {
  color: #c33;
  margin-bottom: 12px;
}

.error p {
  color: #666;
  margin-bottom: 24px;
}

.success {
  padding: 20px;
}

.success-icon {
  width: 64px;
  height: 64px;
  background-color: #4caf50;
  color: white;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 32px;
  margin: 0 auto 16px;
  font-weight: bold;
}

.success h2 {
  color: #4caf50;
  margin-bottom: 12px;
}

.success p {
  color: #666;
}

.btn-primary {
  background-color: #667eea;
  color: white;
  border: none;
  padding: 12px 32px;
  border-radius: 8px;
  font-size: 16px;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.3s;
}

.btn-primary:hover {
  background-color: #5568d3;
}
</style>

