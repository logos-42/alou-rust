# 桌面版钱包连接方案设计文档

## 概述

本文档描述了Alou Desktop桌面应用的钱包连接方案。由于桌面应用无法直接使用浏览器的MetaMask插件，我们实现了以下连接方式：

1. **WalletConnect v2** - 通过二维码连接移动钱包
2. **本地钱包文件** - 使用私钥或助记词导入/创建钱包
3. **浏览器插件回退通路** - 当移动端没有钱包时，引导跳转浏览器安装/启用插件（MetaMask等）
4. **消息签名验证** - 验证钱包所有权

## 架构设计

### 1. 桌面钱包服务 (`desktopWalletService.js`)

核心服务类，提供以下功能：

- **WalletConnect连接**
  - 初始化WalletConnect Provider
  - 生成QR码URI
  - 监听连接事件

- **本地钱包管理**
  - 从私钥或助记词创建钱包
  - 创建新钱包
  - 使用ethers.js管理钱包

- **签名验证**
  - 消息签名
  - 签名验证（使用ethers.js和后端备用方案）

### 2. Tauri后端签名验证 (`main.rs`)

Rust后端提供签名验证API：

```rust
#[tauri::command]
async fn verify_wallet_signature(
    address: String,
    message: String,
    signature: String,
    chain: String,
) -> Result<bool, String>
```

### 3. UI组件

- **WalletConnectQR** - 显示二维码和连接状态
- **LocalWalletForm** - 本地钱包导入/创建表单
- **LoginView** - 更新后的登录页面，根据环境显示不同选项

## 浏览器插件回退通路

虽然桌面版主推 WalletConnect 与本地钱包，但我们保留了浏览器钱包（如 MetaMask 插件）的验证流程，确保以下场景可顺畅进入：

- 用户暂时没有移动端钱包可扫码
- 用户希望通过浏览器插件完成一次性验证再返回桌面版

具体策略：

1. **桌面端提示**：在显示 WalletConnect 二维码的同时提供“打开浏览器插件”入口，点击后调用 `@tauri-apps/api/shell` 打开默认浏览器并跳转至 Web 版登录页或插件安装页。
2. **浏览器端复用原有流程**：保持 `walletService` 的 MetaMask 登录逻辑不变，可继续使用已有的签名验证与登录接口。
3. **状态同步**：在浏览器完成钱包验证后，可通过共享账户体系（例如调用相同后端 API 或使用深链/回调链接）把登录态带回桌面应用。
4. **引导安装**：当检测到本地没有浏览器插件时，提示用户跳转到官方安装页面（MetaMask、OKX Wallet 等）。

这样即使用户手头只有 PC，也可以先在浏览器完成钱包连接，避免“必须有手机钱包”这一硬性限制。

## 连接流程

### WalletConnect流程

1. 用户点击"WalletConnect"按钮
2. 初始化WalletConnect Provider
3. 生成QR码URI并显示
4. 用户使用移动钱包扫码
5. 移动钱包确认连接
6. 桌面应用收到连接事件
7. 生成验证消息并请求签名
8. 验证签名
9. 保存连接信息并登录

### 本地钱包流程

1. 用户点击"本地钱包"按钮
2. 选择导入或创建
3. **导入模式**：
   - 输入私钥或助记词
   - 创建钱包实例
   - 生成验证消息并签名
   - 验证签名
   - 保存连接信息并登录
4. **创建模式**：
   - 生成新钱包
   - 显示助记词和私钥（用户必须保存）
   - 生成验证消息并签名
   - 验证签名
   - 保存连接信息并登录

## 安全考虑

### 1. 私钥保护
- 私钥仅在内存中存储
- 不持久化到本地文件（除非用户明确要求）
- 建议使用加密存储（未来增强）

### 2. 签名验证
- 每次连接都进行消息签名验证
- 验证消息包含地址和时间戳，防止重放攻击
- 双重验证：前端ethers.js + 后端备用

### 3. 助记词安全
- 创建新钱包时强制用户保存助记词
- 助记词仅在创建时显示一次
- 不在任何地方存储助记词

## 配置说明

### WalletConnect Project ID

1. 访问 [WalletConnect Cloud](https://cloud.walletconnect.com/)
2. 创建新项目或使用现有项目
3. 获取Project ID
4. 在 `desktopWalletService.js` 中替换 `YOUR_WALLETCONNECT_PROJECT_ID`

或者使用环境变量：

```javascript
const projectId = import.meta.env.VITE_WALLETCONNECT_PROJECT_ID || 'YOUR_WALLETCONNECT_PROJECT_ID'
```

在 `.env` 文件中添加：
```
VITE_WALLETCONNECT_PROJECT_ID=your_project_id_here
```

### RPC节点配置

默认使用公共RPC节点。可以通过环境变量配置：

```
VITE_ETH_RPC_URL=https://eth.llamarpc.com
```

## 依赖安装

```bash
cd alou-desktop
npm install
```

新添加的依赖：
- `@walletconnect/ethereum-provider` - WalletConnect v2支持
- `@walletconnect/types` - WalletConnect类型定义
- `ethers` - Ethereum库，用于钱包管理和签名
- `qrcode.react` - QR码生成组件

## 使用示例

### WalletConnect连接

```javascript
import { desktopWalletService } from '@/services/desktopWalletService'

// 连接WalletConnect
const result = await desktopWalletService.connectWalletConnect()
console.log('Connected address:', result.address)
```

### 本地钱包连接

```javascript
// 从私钥导入
const result = await desktopWalletService.connectLocalWallet(privateKey, false)

// 从助记词导入
const result = await desktopWalletService.connectLocalWallet(mnemonic, true)

// 创建新钱包
const walletData = await desktopWalletService.createNewWallet()
console.log('New wallet address:', walletData.address)
console.log('Mnemonic:', walletData.mnemonic)
console.log('Private key:', walletData.privateKey)
```

### 签名验证

```javascript
const message = desktopWalletService.generateVerificationMessage(address)
const signature = await desktopWalletService.signMessage(message)
const isValid = await desktopWalletService.verifySignature(address, message, signature)
```

## 环境检测

服务会自动检测是否在桌面环境：

```javascript
if (desktopWalletService.isDesktop()) {
  // 桌面环境，使用桌面钱包服务
} else {
  // 浏览器环境，使用浏览器钱包服务
}
```

## 未来增强

1. **加密存储** - 使用密码加密存储私钥
2. **硬件钱包支持** - 支持Ledger、Trezor等
3. **多钱包管理** - 支持同时管理多个钱包
4. **钱包备份恢复** - 安全备份和恢复功能
5. **交易历史** - 本地交易历史记录

## 注意事项

1. **WalletConnect Project ID** 必须在生产环境中配置
2. **私钥安全** - 始终提醒用户保护私钥和助记词
3. **网络切换** - 钱包连接后支持切换网络
4. **断开连接** - 提供明确的断开连接功能
5. **错误处理** - 完善的错误提示和处理机制

## 测试

### 测试WalletConnect

1. 在桌面应用中点击"WalletConnect"
2. 使用移动钱包（MetaMask、Trust Wallet等）扫描二维码
3. 在移动钱包中确认连接
4. 验证桌面应用成功连接

### 测试本地钱包

1. 导入测试私钥
2. 验证钱包地址正确
3. 测试签名功能
4. 验证签名验证通过

### 测试新钱包创建

1. 创建新钱包
2. 保存助记词和私钥
3. 使用助记词重新导入验证

