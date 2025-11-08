<template>
  <div class="wallet-manager" :class="{ 'dark-mode': isDarkMode }">
    <!-- 顶部导航栏 -->
    <nav class="top-nav">
      <div class="nav-content">
        <div class="logo-section">
          <div class="logo">💰</div>
          <h1 class="app-title">{{ t('walletManagement') }}</h1>
        </div>
        
        <div class="nav-controls">
          <button @click="toggleDarkMode" class="theme-toggle" :title="t('theme')">
            <span v-if="isDarkMode">🌞</span>
            <span v-else>🌙</span>
          </button>
          
          <button @click="goBack" class="back-btn">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
              <path d="M20,11V13H8L13.5,18.5L12.08,19.92L4.16,12L12.08,4.08L13.5,5.5L8,11H20Z"/>
            </svg>
            {{ t('back') }}
          </button>
        </div>
      </div>
    </nav>

    <!-- 主内容区域 -->
    <div class="wallet-container">
      <div class="wallet-content">
        
        <!-- 连接钱包 -->
        <WalletConnect
          v-if="!connectedWallet"
          @connect-metamask="connectMetaMask"
          @connect-walletconnect="connectWalletConnect"
        />

        <!-- 已连接钱包 -->
        <div v-else class="wallet-info-section">
          <WalletOverview
            :wallet="connectedWallet"
            :current-network="currentNetwork"
            :network-name="getNetworkName(currentNetwork)"
            :eth-price="ethPrice"
            @disconnect="disconnectWallet"
          />

          <NetworkSelector
            :networks="networks"
            :current-network="currentNetwork"
            @switch-network="switchToNetwork"
          />

          <TransactionList
            :transactions="transactions"
            :is-refreshing="isRefreshing"
            @refresh="refreshTransactions"
            @view-transaction="viewTransaction"
          />

          <AgentWallets :wallets="agentWallets" />

          <ContractWallet
            :contract-wallet="contractWallet"
            @create="createContractWallet"
            @deposit="depositToContract"
            @withdraw="withdrawFromContract"
            @manage="manageContract"
          />

          <SignatureModal
            :show="!!signatureRequest"
            :request="signatureRequest || defaultRequest"
            :is-signing="isSigning"
            @confirm="confirmSignature"
            @cancel="cancelSignature"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from '@/composables/useI18n'
import { blockchainService } from '@/services/blockchain.service'
import WalletConnect from './wallet/WalletConnect.vue'
import WalletOverview from './wallet/WalletOverview.vue'
import NetworkSelector from './wallet/NetworkSelector.vue'
import TransactionList from './wallet/TransactionList.vue'
import ContractWallet from './wallet/ContractWallet.vue'
import SignatureModal from './wallet/SignatureModal.vue'
import AgentWallets from './wallet/AgentWallets.vue'

interface ConnectedWallet {
  address: string
  ethBalance: string
  usdcBalance: string
}

interface Network {
  chainId: string
  name: string
  type: string
  icon: string
  rpcUrl: string
}

interface Transaction {
  hash: string
  type: 'send' | 'receive' | 'contract'
  from: string
  to: string
  value: string
  token: string
  timestamp: number
  status: 'confirmed' | 'pending' | 'failed'
}

interface ContractWallet {
  address: string
  balance: string
}

interface SignatureRequest {
  from: string
  to: string
  value: string
  token: string
  gasFee: string
  data?: string
}

const router = useRouter()
const { t } = useI18n()

// 响应式数据
const isDarkMode = ref(false)
const connectedWallet = ref<ConnectedWallet | null>(null)
const currentNetwork = ref('0x1')
const transactions = ref<Transaction[]>([])
const contractWallet = ref<ContractWallet | null>(null)
const signatureRequest = ref<SignatureRequest | null>(null)
const isSigning = ref(false)
const isRefreshing = ref(false)
const ethPrice = ref(2000)
const agentWallets = ref<any[]>([])

const defaultRequest: SignatureRequest = {
  from: '',
  to: '',
  value: '0',
  token: 'ETH',
  gasFee: '0'
}

const networks: Network[] = [
  { chainId: '0xaa36a7', name: 'Ethereum Sepolia', type: 'Testnet', icon: '🔷', rpcUrl: 'https://sepolia.infura.io/v3/' },
  { chainId: '0x14a34', name: 'Base Sepolia', type: 'Testnet', icon: '🔵', rpcUrl: 'https://sepolia.base.org' },
  { chainId: '0x13882', name: 'Polygon Amoy', type: 'Testnet', icon: '🟣', rpcUrl: 'https://rpc-amoy.polygon.technology' },
  { chainId: '0x1', name: 'Ethereum Mainnet', type: 'Mainnet', icon: '💎', rpcUrl: 'https://mainnet.infura.io/v3/' },
  { chainId: '0x2105', name: 'Base Mainnet', type: 'Mainnet', icon: '🔷', rpcUrl: 'https://mainnet.base.org' },
]

// 生命周期
onMounted(() => {
  const savedTheme = localStorage.getItem('alou-theme')
  isDarkMode.value = savedTheme === 'dark' || window.matchMedia('(prefers-color-scheme: dark)').matches
  
  checkWalletConnection()
  
  // Listen to network changes
  window.addEventListener('network-changed', handleNetworkChanged as EventListener)
  
  // Listen to wallet network changes from MetaMask
  if (typeof window.ethereum !== 'undefined') {
    window.ethereum.on('chainChanged', (chainId: string) => {
      currentNetwork.value = chainId
      localStorage.setItem('wallet_chain_id', chainId)
    })
  }
})

onUnmounted(() => {
  window.removeEventListener('network-changed', handleNetworkChanged as EventListener)
})

// 方法
function goBack() {
  router.push('/')
}

function toggleDarkMode() {
  isDarkMode.value = !isDarkMode.value
  localStorage.setItem('alou-theme', isDarkMode.value ? 'dark' : 'light')
}

async function connectMetaMask() {
  try {
    if (typeof window.ethereum === 'undefined') {
      alert(t('installMetaMask'))
      return
    }

    const accounts = await window.ethereum.request({ method: 'eth_requestAccounts' })
    const address = accounts[0]
    
    // Get real balance from blockchain
    const balanceInfo = await blockchainService.getBalance(address)
    
    connectedWallet.value = {
      address,
      ethBalance: balanceInfo?.balance || '0.0',
      usdcBalance: '0.0'
    }
    
    // Get current network
    const chainId = await window.ethereum.request({ method: 'eth_chainId' })
    currentNetwork.value = chainId
    
    localStorage.setItem('wallet_address', address)
    localStorage.setItem('wallet_chain_id', chainId)
    window.dispatchEvent(new CustomEvent('wallet-changed', { detail: { address } }))
    
    await loadTransactions()
    await loadAgentWallets()
    
  } catch (error) {
    console.error('Failed to connect MetaMask:', error)
    alert(t('connectionFailed'))
  }
}

async function connectWalletConnect() {
  alert(t('walletConnectComingSoon'))
}

function checkWalletConnection() {
  const savedAddress = localStorage.getItem('wallet_address')
  if (savedAddress && typeof window.ethereum !== 'undefined') {
    connectMetaMask()
  }
}

function disconnectWallet() {
  connectedWallet.value = null
  localStorage.removeItem('wallet_address')
  window.dispatchEvent(new CustomEvent('wallet-changed', { detail: { address: null } }))
}

async function switchToNetwork(network: Network) {
  if (!window.ethereum) return
  
  try {
    await window.ethereum.request({
      method: 'wallet_switchEthereumChain',
      params: [{ chainId: network.chainId }],
    })
    currentNetwork.value = network.chainId
  } catch (error: any) {
    if (error.code === 4902) {
      try {
        await window.ethereum!.request({
          method: 'wallet_addEthereumChain',
          params: [{
            chainId: network.chainId,
            chainName: network.name,
            rpcUrls: [network.rpcUrl],
          }],
        })
        currentNetwork.value = network.chainId
      } catch (addError) {
        console.error('Failed to add network:', addError)
      }
    }
  }
}

function getNetworkName(chainId: string): string {
  const network = networks.find(n => n.chainId === chainId)
  return network ? network.name : 'Unknown Network'
}

async function loadTransactions() {
  transactions.value = [
    {
      hash: '0x1234...5678',
      type: 'send',
      from: connectedWallet.value!.address,
      to: '0xabcd...efgh',
      value: '0.1',
      token: 'ETH',
      timestamp: Date.now() - 3600000,
      status: 'confirmed'
    },
    {
      hash: '0x8765...4321',
      type: 'receive',
      from: '0xijkl...mnop',
      to: connectedWallet.value!.address,
      value: '0.05',
      token: 'ETH',
      timestamp: Date.now() - 7200000,
      status: 'confirmed'
    }
  ]
}

async function refreshTransactions() {
  isRefreshing.value = true
  await loadTransactions()
  setTimeout(() => {
    isRefreshing.value = false
  }, 1000)
}

function viewTransaction(tx: Transaction) {
  window.open(`https://etherscan.io/tx/${tx.hash}`, '_blank')
}

async function createContractWallet() {
  try {
    contractWallet.value = {
      address: '0xContract...Wallet',
      balance: '0.0'
    }
    alert(t('contractWalletCreated'))
  } catch (error) {
    console.error('Failed to create contract wallet:', error)
    alert(t('createFailed'))
  }
}

async function depositToContract() {
  const amount = prompt(t('enterDepositAmount'))
  if (!amount) return
  
  signatureRequest.value = {
    from: connectedWallet.value!.address,
    to: contractWallet.value!.address,
    value: amount,
    token: 'ETH',
    gasFee: '0.001'
  }
}

async function withdrawFromContract() {
  const amount = prompt(t('enterWithdrawAmount'))
  if (!amount) return
  
  signatureRequest.value = {
    from: contractWallet.value!.address,
    to: connectedWallet.value!.address,
    value: amount,
    token: 'ETH',
    gasFee: '0.001'
  }
}

function manageContract() {
  alert(t('contractManagementComingSoon'))
}

async function confirmSignature() {
  if (!signatureRequest.value || !window.ethereum) return
  
  isSigning.value = true
  
  try {
    const txHash = await window.ethereum.request({
      method: 'eth_sendTransaction',
      params: [{
        from: signatureRequest.value.from,
        to: signatureRequest.value.to,
        value: '0x' + (parseFloat(signatureRequest.value.value) * 1e18).toString(16),
      }],
    })
    
    alert(t('transactionSent') + ': ' + txHash)
    signatureRequest.value = null
    
    await connectMetaMask()
    await loadTransactions()
    
  } catch (error) {
    console.error('Transaction failed:', error)
    alert(t('transactionFailed'))
  } finally {
    isSigning.value = false
  }
}

function cancelSignature() {
  signatureRequest.value = null
}

function handleNetworkChanged(event: CustomEvent<{ chainId: string; network: Network }>) {
  console.log('Network changed:', event.detail)
  currentNetwork.value = event.detail.chainId
  
  // Refresh balance after network change
  if (connectedWallet.value) {
    refreshWalletBalance()
  }
  
  const networkName = getNetworkName(event.detail.chainId)
  console.log(`Switched to ${networkName}`)
}

async function refreshWalletBalance() {
  if (!connectedWallet.value) return
  
  try {
    const balanceInfo = await blockchainService.getBalance(connectedWallet.value.address)
    if (balanceInfo) {
      connectedWallet.value.ethBalance = balanceInfo.balance
    }
  } catch (error) {
    console.error('Failed to refresh balance:', error)
  }
}

async function loadAgentWallets() {
  try {
    const sessionId = localStorage.getItem('session_id') || 'default'
    const wallets = await blockchainService.listAgentWallets(sessionId)
    agentWallets.value = wallets
    console.log('Loaded agent wallets:', wallets)
  } catch (error) {
    console.error('Failed to load agent wallets:', error)
  }
}
</script>

<style scoped>
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

.back-btn {
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

.back-btn:hover {
  background: var(--primary-hover);
  transform: scale(1.05);
}

.wallet-container {
  width: 100%;
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}

.wallet-content {
  max-width: 1200px;
  margin: 0 auto;
  padding: 2rem clamp(1rem, 3vw, 3rem);
}

.wallet-info-section {
  animation: fadeIn 0.5s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@media (max-width: 768px) {
  .wallet-content {
    padding: 1rem;
  }
  
  .app-title {
    font-size: 1.25rem;
  }
}
</style>
