# 钱包管理工具测试文档

## 功能概述

智能体现在可以主动管理用户的钱包，包括：
1. 列出支持的网络
2. 切换钱包网络
3. 查询当前网络
4. 获取钱包信息
5. 检查钱包余额

## 测试场景

### 1. 列出支持的网络

**用户输入：**
```
请列出所有支持的区块链网络
```

**预期智能体行为：**
- 调用 `wallet_manager` 工具，action 为 `list_networks`
- 返回包含以下网络的列表：
  - Ethereum Sepolia (测试网)
  - Base Sepolia (测试网)
  - Polygon Amoy (测试网)
  - Ethereum Mainnet (主网)
  - Base Mainnet (主网)
  - Polygon Mainnet (主网)

### 2. 切换到特定网络

**用户输入：**
```
请帮我切换到 Base Sepolia 测试网
```

**预期智能体行为：**
- 调用 `wallet_manager` 工具，action 为 `switch_network`
- 参数：`chainId: "0x14a34"`
- 前端自动执行网络切换
- 显示成功消息："✅ 已成功切换到 Base Sepolia (Testnet)"

### 3. 切换到主网

**用户输入：**
```
切换到以太坊主网
```

**预期智能体行为：**
- 调用 `wallet_manager` 工具，action 为 `switch_network`
- 参数：`chainId: "0x1"`
- 前端执行网络切换
- 显示成功消息

### 4. 查询当前网络

**用户输入：**
```
我现在在哪个网络？
```

**预期智能体行为：**
- 调用 `wallet_manager` 工具，action 为 `get_current_network`
- 返回当前网络的 chainId
- 智能体解释当前网络名称

### 5. 检查钱包余额

**用户输入：**
```
查看我的钱包余额
```

**预期智能体行为：**
- 调用 `wallet_manager` 工具，action 为 `check_balance`
- 前端查询 eth_getBalance
- 返回余额信息

## 技术实现

### 后端 (Rust)

**文件：** `alou-edge/src/mcp/tools/wallet_manager.rs`

新增的 MCP 工具：
- 工具名称：`wallet_manager`
- 支持的操作：
  - `list_networks`: 列出所有支持的网络
  - `switch_network`: 切换网络（需要 chainId 参数）
  - `get_current_network`: 获取当前网络
  - `get_wallet_info`: 获取钱包信息
  - `check_balance`: 检查余额

### 前端 (TypeScript/Vue)

**文件：** `frontend/src/services/wallet.service.ts`

新增的钱包服务：
- `switchNetwork()`: 切换网络
- `addNetwork()`: 添加新网络
- `getCurrentChainId()`: 获取当前链ID
- `getBalance()`: 获取余额
- `executeInstruction()`: 执行智能体指令

**文件：** `frontend/src/components/AgentChat.vue`

更新的聊天组件：
- `handleToolCalls()`: 处理工具调用
- 自动执行钱包操作指令
- 显示操作结果消息

## 支持的网络配置

| 网络名称 | Chain ID | 类型 | RPC URL |
|---------|----------|------|---------|
| Ethereum Sepolia | 0xaa36a7 | Testnet | https://sepolia.infura.io/v3/ |
| Base Sepolia | 0x14a34 | Testnet | https://sepolia.base.org |
| Polygon Amoy | 0x13882 | Testnet | https://rpc-amoy.polygon.technology |
| Ethereum Mainnet | 0x1 | Mainnet | https://mainnet.infura.io/v3/ |
| Base Mainnet | 0x2105 | Mainnet | https://mainnet.base.org |
| Polygon Mainnet | 0x89 | Mainnet | https://polygon-rpc.com |

## 使用示例

### 示例对话 1：切换网络

```
用户: 我想在 Base 测试网上测试一下
智能体: 好的，我来帮你切换到 Base Sepolia 测试网。
[调用 wallet_manager 工具]
系统: ✅ 已成功切换到 Base Sepolia (Testnet)
智能体: 已经成功切换到 Base Sepolia 测试网了，你现在可以在这个网络上进行测试了。
```

### 示例对话 2：查询网络

```
用户: 我现在在哪个链上？
智能体: 让我查看一下你当前的网络...
[调用 wallet_manager 工具]
智能体: 你当前连接的是 Ethereum Mainnet (主网)。
```

### 示例对话 3：多步骤操作

```
用户: 帮我切换到 Polygon 主网，然后查看余额
智能体: 好的，我先帮你切换到 Polygon 主网。
[调用 wallet_manager 切换网络]
系统: ✅ 已成功切换到 Polygon Mainnet (Mainnet)
智能体: 已切换到 Polygon 主网，现在查看你的余额...
[调用 wallet_manager 检查余额]
智能体: 你在 Polygon 主网上的余额是 0.523 MATIC。
```

## 注意事项

1. **用户授权**：所有钱包操作都需要用户在 MetaMask 中确认
2. **网络添加**：如果用户钱包中没有该网络，会自动提示添加
3. **错误处理**：如果操作失败，会显示友好的错误消息
4. **状态同步**：网络切换后会自动更新 localStorage 和触发事件

## 安全考虑

- 智能体只能发起操作请求，不能直接控制钱包
- 所有敏感操作都需要用户在钱包中确认
- 不存储私钥或助记词
- 只读取公开的钱包地址和余额信息
