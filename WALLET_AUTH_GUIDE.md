# 钱包认证使用指南

## 🦊 MetaMask 连接问题排查

如果 MetaMask 弹窗没有出现，请按以下步骤检查：

### 1. 检查 MetaMask 是否已安装
打开浏览器控制台（F12），输入：
```javascript
console.log('MetaMask installed:', typeof window.ethereum !== 'undefined')
console.log('Provider:', window.ethereum)
```

### 2. 手动测试 MetaMask 连接
在浏览器控制台输入：
```javascript
window.ethereum.request({ method: 'eth_requestAccounts' })
  .then(accounts => console.log('✅ 连接成功:', accounts))
  .catch(err => console.error('❌ 连接失败:', err))
```

### 3. 常见问题

#### 问题 A: MetaMask 已锁定
- **现象**：弹窗不出现或立即消失
- **解决**：点击浏览器工具栏的 MetaMask 图标，解锁钱包

#### 问题 B: 有待处理的连接请求
- **现象**：错误代码 -32002
- **解决**：打开 MetaMask，处理所有待处理的请求，或刷新页面重试

#### 问题 C: 多个钱包插件冲突
- **现象**：弹出其他钱包而不是 MetaMask
- **解决**：临时禁用其他钱包插件，或在 MetaMask 设置中启用"优先作为默认钱包"

#### 问题 D: 浏览器权限被阻止
- **现象**：完全没有弹窗
- **解决**：检查浏览器是否阻止了弹出窗口，允许此网站的弹窗

## 🔐 后端钱包验证API

### 1. 获取 Nonce（可选，用于更安全的验证）

**端点**: `POST /api/auth/wallet/nonce`

**请求**:
```json
{
  "address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"
}
```

**响应**:
```json
{
  "nonce": "Sign this message to authenticate with Alou Pay:\n\nNonce: ...",
  "message": "Please sign this message..."
}
```

### 2. 验证签名并登录（未来实现）

**端点**: `POST /api/auth/wallet/verify`

**请求**:
```json
{
  "address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
  "signature": "0x...",
  "message": "Sign this message to authenticate..."
}
```

**响应**:
```json
{
  "access_token": "eyJ...",
  "refresh_token": "eyJ...",
  "expires_in": 3600,
  "user": {
    "id": "...",
    "email": "0x742d...@wallet.local",
    "name": "0x742d...0bEb"
  }
}
```

## 🚀 当前实现状态

### ✅ 已完成
- MetaMask 检测和连接
- 获取钱包地址和 Chain ID
- 前端状态管理
- 后端 API 结构

### 🔄 进行中
- 前端暂时使用简化的登录（无签名）
- 后端签名验证逻辑已准备好

### 📋 待完成
- 集成完整的签名流程
- 使用 ethers-rs 进行真实的签名验证
- 支持 WalletConnect 和 Coinbase Wallet
- 多链支持和网络切换

## 💻 开发者说明

### 前端简化流程（当前）
```typescript
// 1. 连接 MetaMask
const accounts = await ethereum.request({ method: 'eth_requestAccounts' })
const address = accounts[0]
const chainId = await ethereum.request({ method: 'eth_chainId' })

// 2. 直接登录（存储到 localStorage）
await authStore.loginWithWeb3Wallet({ address, chainId, walletType: 'metamask' })
```

### 完整安全流程（推荐用于生产）
```typescript
// 1. 获取 nonce
const { nonce } = await fetch('/api/auth/wallet/nonce', {
  method: 'POST',
  body: JSON.stringify({ address })
})

// 2. 使用 MetaMask 签名
const signature = await ethereum.request({
  method: 'personal_sign',
  params: [nonce, address]
})

// 3. 验证签名并获取 token
const { access_token } = await fetch('/api/auth/wallet/verify', {
  method: 'POST',
  body: JSON.stringify({ address, signature, message: nonce })
})
```

## 🔧 调试技巧

### 启用详细日志
前端已添加 console.log，查看浏览器控制台：
- 🦊 开始连接 MetaMask...
- ✅ 检测到 MetaMask
- 📞 请求账户访问权限...
- ✅ 成功获取账户: 0x...
- 🎉 登录成功！

### 检查 MetaMask 状态
```javascript
// 检查是否已连接
ethereum.request({ method: 'eth_accounts' })
  .then(accounts => console.log('已连接账户:', accounts))

// 检查当前网络
ethereum.request({ method: 'eth_chainId' })
  .then(chainId => console.log('当前链ID:', chainId))
```

## 📚 相关资源
- [MetaMask 文档](https://docs.metamask.io/)
- [EIP-1193: Ethereum Provider API](https://eips.ethereum.org/EIPS/eip-1193)
- [ethers-rs](https://github.com/gakonst/ethers-rs)

