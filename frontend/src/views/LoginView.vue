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
      
      <h1 class="title">连接钱包</h1>
      <p class="subtitle">选择您的加密钱包以安全登录</p>

      <div v-if="error" class="error-message">
        <span class="error-icon">⚠️</span>
        {{ error }}
      </div>

      <div class="wallet-options">
        <!-- MetaMask -->
        <button 
          @click="connectMetaMask" 
          :disabled="isLoading"
          class="wallet-btn"
          :class="{ loading: isLoading && currentWallet === 'metamask' }"
        >
          <div class="wallet-icon">
            <img src="https://upload.wikimedia.org/wikipedia/commons/3/36/MetaMask_Fox.svg" alt="MetaMask" />
          </div>
          <div class="wallet-info">
            <div class="wallet-name">MetaMask</div>
            <div class="wallet-desc">
              {{ hasMetaMask ? '已安装' : '需要安装浏览器插件' }}
            </div>
          </div>
          <div class="wallet-arrow">
            <svg v-if="!isLoading || currentWallet !== 'metamask'" width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
              <path d="M8.59,16.58L13.17,12L8.59,7.41L10,6L16,12L10,18L8.59,16.58Z"/>
            </svg>
            <div v-else class="spinner"></div>
          </div>
        </button>

        <!-- WalletConnect -->
        <button 
          @click="connectWalletConnect" 
          :disabled="isLoading"
          class="wallet-btn"
          :class="{ loading: isLoading && currentWallet === 'walletconnect' }"
        >
          <div class="wallet-icon wallet-icon-walletconnect">
            <svg width="40" height="40" viewBox="0 0 300 185" fill="none">
              <path d="M61.439 36.256c48.91-47.888 128.212-47.888 177.123 0l5.886 5.764a6.041 6.041 0 010 8.67l-20.136 19.716a3.179 3.179 0 01-4.428 0l-8.101-7.931c-34.122-33.408-89.444-33.408-123.566 0l-8.675 8.494a3.179 3.179 0 01-4.428 0L54.978 51.253a6.041 6.041 0 010-8.67l6.461-6.327zm218.965 40.806l17.921 17.547a6.041 6.041 0 010 8.67l-80.81 79.122c-2.446 2.394-6.41 2.394-8.856 0l-57.354-56.155a1.59 1.59 0 00-2.214 0L91.737 182.4c-2.446 2.394-6.41 2.394-8.856 0L2.07 103.278a6.041 6.041 0 010-8.67l17.921-17.547c2.446-2.394 6.41-2.394 8.856 0l57.354 56.155a1.59 1.59 0 002.214 0l57.354-56.155c2.446-2.395 6.41-2.395 8.856 0l57.354 56.155a1.59 1.59 0 002.214 0l57.354-56.155c2.446-2.394 6.41-2.394 8.856 0z" fill="#3B99FC"/>
            </svg>
          </div>
          <div class="wallet-info">
            <div class="wallet-name">WalletConnect</div>
            <div class="wallet-desc">扫码连接移动钱包</div>
          </div>
          <div class="wallet-arrow">
            <svg v-if="!isLoading || currentWallet !== 'walletconnect'" width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
              <path d="M8.59,16.58L13.17,12L8.59,7.41L10,6L16,12L10,18L8.59,16.58Z"/>
            </svg>
            <div v-else class="spinner"></div>
          </div>
        </button>

        <!-- Coinbase Wallet -->
        <button 
          @click="connectCoinbase" 
          :disabled="isLoading"
          class="wallet-btn"
          :class="{ loading: isLoading && currentWallet === 'coinbase' }"
        >
          <div class="wallet-icon">
            <svg width="40" height="40" viewBox="0 0 1024 1024" fill="none">
              <rect width="1024" height="1024" rx="512" fill="#0052FF"/>
              <path fill-rule="evenodd" clip-rule="evenodd" d="M512 768c141.385 0 256-114.615 256-256S653.385 256 512 256 256 370.615 256 512s114.615 256 256 256zm-40-384h80c13.255 0 24 10.745 24 24v80h80c13.255 0 24 10.745 24 24v80c0 13.255-10.745 24-24 24h-80v80c0 13.255-10.745 24-24 24h-80c-13.255 0-24-10.745-24-24v-80h-80c-13.255 0-24-10.745-24-24v-80c0-13.255 10.745-24 24-24h80v-80c0-13.255 10.745-24 24-24z" fill="white"/>
            </svg>
          </div>
          <div class="wallet-info">
            <div class="wallet-name">Coinbase Wallet</div>
            <div class="wallet-desc">安全易用的加密钱包</div>
          </div>
          <div class="wallet-arrow">
            <svg v-if="!isLoading || currentWallet !== 'coinbase'" width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
              <path d="M8.59,16.58L13.17,12L8.59,7.41L10,6L16,12L10,18L8.59,16.58Z"/>
            </svg>
            <div v-else class="spinner"></div>
          </div>
        </button>
      </div>

      <div class="security-notice">
        <div class="notice-icon">🔒</div>
        <div class="notice-content">
          <h3>安全提示</h3>
          <ul>
            <li>我们不会存储您的私钥或助记词</li>
            <li>请确认您访问的是正确的网站</li>
            <li>不要与他人分享您的钱包信息</li>
          </ul>
        </div>
      </div>

      <div class="help-section">
        <p class="help-text">没有钱包？</p>
        <a href="https://metamask.io/download/" target="_blank" class="help-link">
          下载 MetaMask
        </a>
      </div>

      <p class="terms">
        连接钱包即表示您同意我们的<a href="/terms">服务条款</a>和<a href="/privacy">隐私政策</a>
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const authStore = useAuthStore()

const isLoading = ref(false)
const error = ref('')
const currentWallet = ref<'metamask' | 'walletconnect' | 'coinbase' | null>(null)
const hasMetaMask = ref(false)

// 检查是否安装了 MetaMask
onMounted(() => {
  hasMetaMask.value = typeof window !== 'undefined' && typeof (window as any).ethereum !== 'undefined'
})

function goBack() {
  router.push('/')
}

async function connectMetaMask() {
  try {
    isLoading.value = true
    currentWallet.value = 'metamask'
    error.value = ''

    console.log('🦊 开始连接 MetaMask...')

    // 检查是否在浏览器环境中
    if (typeof window === 'undefined') {
      throw new Error('请在浏览器中打开')
    }

    // 检查是否安装了以太坊提供者
    const { ethereum } = window as any
    
    if (!ethereum) {
      throw new Error('请先安装 MetaMask 浏览器插件')
    }

    console.log('✅ 检测到 MetaMask')

    // 如果有多个钱包，尝试选择 MetaMask
    if (ethereum.providers?.length) {
      const provider = ethereum.providers.find((p: any) => p.isMetaMask)
      if (provider) {
        console.log('🔄 切换到 MetaMask provider')
        await provider.request({ method: 'eth_requestAccounts' })
        
        const accounts = await provider.request({ method: 'eth_accounts' })
        const chainId = await provider.request({ method: 'eth_chainId' })
        
        if (!accounts || accounts.length === 0) {
          throw new Error('未能获取钱包地址')
        }

        console.log('✅ 成功获取账户:', accounts[0])

        await authStore.loginWithWeb3Wallet({
          address: accounts[0],
          chainId,
          walletType: 'metamask'
        })

        router.push('/')
        return
      }
    }

    // 单个钱包或默认情况
    console.log('📞 请求账户访问权限...')
    const accounts = await ethereum.request({ 
      method: 'eth_requestAccounts' 
    })
    
    if (!accounts || accounts.length === 0) {
      throw new Error('未能获取钱包地址')
    }

    console.log('✅ 成功获取账户:', accounts[0])

    const address = accounts[0]
    
    // 获取chainId
    const chainId = await ethereum.request({ method: 'eth_chainId' })
    console.log('✅ Chain ID:', chainId)

    // 使用 auth store 的钱包登录方法
    await authStore.loginWithWeb3Wallet({
      address,
      chainId,
      walletType: 'metamask'
    })

    console.log('🎉 登录成功！')
    
    // 登录成功后返回主页
    router.push('/')
  } catch (err: any) {
    console.error('❌ MetaMask 连接错误:', err)
    
    if (err.code === 4001) {
      error.value = '您拒绝了连接请求，请重试'
    } else if (err.code === -32002) {
      error.value = '请在 MetaMask 中确认连接请求（可能已有待处理的请求）'
    } else if (err.code === -32603) {
      error.value = 'MetaMask 内部错误，请刷新页面重试'
    } else {
      error.value = err.message || '连接 MetaMask 失败，请重试'
    }
  } finally {
    isLoading.value = false
    currentWallet.value = null
  }
}

async function connectWalletConnect() {
  try {
    isLoading.value = true
    currentWallet.value = 'walletconnect'
    error.value = ''

    // WalletConnect 需要额外的库，这里先显示提示
    error.value = 'WalletConnect 功能即将推出'
    
  } catch (err: any) {
    error.value = err.message || '连接失败'
  } finally {
    isLoading.value = false
    currentWallet.value = null
  }
}

async function connectCoinbase() {
  try {
    isLoading.value = true
    currentWallet.value = 'coinbase'
    error.value = ''

    // Coinbase Wallet 功能提示
    error.value = 'Coinbase Wallet 功能即将推出'
    
  } catch (err: any) {
    error.value = err.message || '连接失败'
  } finally {
    isLoading.value = false
    currentWallet.value = null
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
  border-radius: 24px;
  padding: 48px 40px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  max-width: 480px;
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
  background-color: #fef2f2;
  border: 1px solid #fecaca;
  color: #dc2626;
  padding: 14px 16px;
  border-radius: 12px;
  margin-bottom: 24px;
  font-size: 14px;
  display: flex;
  align-items: center;
  gap: 8px;
  animation: shake 0.5s ease;
}

@keyframes shake {
  0%, 100% { transform: translateX(0); }
  25% { transform: translateX(-10px); }
  75% { transform: translateX(10px); }
}

.error-icon {
  font-size: 18px;
  flex-shrink: 0;
}

.wallet-options {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 32px;
}

.wallet-btn {
  background: white;
  border: 2px solid #e5e7eb;
  border-radius: 16px;
  padding: 20px 24px;
  display: flex;
  align-items: center;
  gap: 16px;
  cursor: pointer;
  transition: all 0.3s ease;
  text-align: left;
}

.wallet-btn:hover:not(:disabled) {
  border-color: #667eea;
  background: #f9fafb;
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(102, 126, 234, 0.12);
}

.wallet-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.wallet-btn.loading {
  border-color: #667eea;
  background: #f9fafb;
}

.wallet-icon {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  border-radius: 12px;
  overflow: hidden;
}

.wallet-icon img {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.wallet-icon-walletconnect {
  background: #f9fafb;
}

.wallet-info {
  flex: 1;
  min-width: 0;
}

.wallet-name {
  font-size: 16px;
  font-weight: 600;
  color: #111827;
  margin-bottom: 4px;
}

.wallet-desc {
  font-size: 13px;
  color: #6b7280;
}

.wallet-arrow {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  color: #9ca3af;
}

.spinner {
  width: 20px;
  height: 20px;
  border: 2px solid #e5e7eb;
  border-top-color: #667eea;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
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

.help-section {
  text-align: center;
  margin-bottom: 24px;
  padding: 16px;
  background: #f9fafb;
  border-radius: 12px;
}

.help-text {
  font-size: 14px;
  color: #6b7280;
  margin: 0 0 8px 0;
}

.help-link {
  color: #667eea;
  font-size: 14px;
  font-weight: 500;
  text-decoration: none;
  transition: all 0.2s ease;
}

.help-link:hover {
  color: #5568d3;
  text-decoration: underline;
}

.terms {
  font-size: 12px;
  color: #999;
  line-height: 1.5;
  text-align: center;
  margin: 0;
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

