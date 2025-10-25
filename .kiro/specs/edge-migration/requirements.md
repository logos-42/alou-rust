# Requirements Document

## Introduction

在 Cloudflare 边缘网络上构建基于 Claude Agent SDK 的 Web3 AI Agent，通过 MCP 协议实现钱包认证和链上操作。

## Glossary

- **Edge Network**: Cloudflare 全球 300+ 节点的边缘计算网络
- **WASM**: WebAssembly，Rust 编译目标格式
- **D1**: Cloudflare 的 SQLite 边缘数据库
- **KV**: Cloudflare 的键值存储
- **MCP**: Model Context Protocol，AI Agent 工具调用协议
- **Claude Agent SDK**: Anthropic 提供的 Agent 开发框架
- **Web3 Wallet**: 加密钱包（MetaMask、Phantom 等）

## Requirements

### Requirement 1: Claude Agent 核心集成

**User Story:** 作为开发者，我希望在 Cloudflare Worker 中集成 Claude Agent SDK，以便构建智能 AI Agent。

#### Acceptance Criteria

1. WHEN Agent 接收用户消息时，THE Agent_System SHALL 调用 Claude API 生成响应
2. WHEN Agent 需要调用工具时，THE Agent_System SHALL 通过 MCP 协议调用对应工具
3. WHEN 对话进行时，THE Agent_System SHALL 在 KV 中维护会话上下文
4. WHERE 工具调用失败，THE Agent_System SHALL 返回错误信息并继续对话
5. WHILE Agent 运行时，THE Agent_System SHALL 在 50ms CPU 时间内完成处理

### Requirement 2: 会话和状态管理

**User Story:** 作为用户，我希望系统能够记住我的对话历史，以便进行连续的多轮对话。

#### Acceptance Criteria

1. WHEN 新会话创建时，THE Session_Manager SHALL 生成唯一的 session_id
2. WHEN 消息发送时，THE Session_Manager SHALL 将消息存储到 KV
3. WHEN 查询历史时，THE Session_Manager SHALL 返回完整的对话记录
4. WHERE 会话超过 24 小时，THE Session_Manager SHALL 自动清理过期会话
5. WHILE 会话活跃时，THE Session_Manager SHALL 维护用户状态和钱包信息

### Requirement 3: MCP 工具系统

**User Story:** 作为 Agent，我希望通过 MCP 协议调用各种工具，以便完成链上操作和数据查询。

#### Acceptance Criteria

1. WHEN Agent 需要调用工具时，THE MCP_System SHALL 解析工具名称和参数
2. WHEN 工具执行时，THE MCP_System SHALL 在 500ms 内返回结果
3. WHEN 工具列表被查询时，THE MCP_System SHALL 返回所有可用工具的描述
4. WHERE 工具调用失败，THE MCP_System SHALL 返回详细的错误信息
5. WHILE 工具执行时，THE MCP_System SHALL 记录调用日志到 D1

### Requirement 4: Web3 钱包认证

**User Story:** 作为 Web3 用户，我希望使用加密钱包（MetaMask、Phantom 等）登录，以便无需传统的用户名密码。

#### Acceptance Criteria

1. WHEN 用户请求认证 nonce 时，THE Auth_System SHALL 生成唯一的随机字符串并存储到 KV
2. WHEN 用户提交钱包签名时，THE Auth_System SHALL 在 50ms 内验证签名的有效性
3. WHEN 签名验证成功时，THE Auth_System SHALL 生成 JWT token 并创建会话
4. WHERE 支持多链钱包，THE Auth_System SHALL 正确验证 Ethereum 和 Solana 签名
5. WHILE 会话活跃时，THE Auth_System SHALL 在 KV 中维护钱包地址和会话的映射关系

### Requirement 5: MCP 工具 - 链上查询

**User Story:** 作为用户，我希望 Agent 能够查询我的链上资产和交易历史。

#### Acceptance Criteria

1. WHEN 查询钱包余额时，THE Query_Tool SHALL 返回所有代币的余额
2. WHEN 查询 NFT 持有时，THE Query_Tool SHALL 返回 NFT 列表及元数据
3. WHEN 查询交易历史时，THE Query_Tool SHALL 返回最近的交易记录
4. WHERE 需要实时价格，THE Query_Tool SHALL 从 RPC 或 API 获取价格数据
5. WHILE 查询进行时，THE Query_Tool SHALL 使用 KV 缓存减少 RPC 调用

### Requirement 6: MCP 工具 - 交易构造

**User Story:** 作为用户，我希望 Agent 能够帮我构造区块链交易。

#### Acceptance Criteria

1. WHEN 用户描述交易意图时，THE Transaction_Tool SHALL 解析意图并构造交易
2. WHEN 构造交易时，THE Transaction_Tool SHALL 查询 gas 价格并设置合理参数
3. WHEN 交易构造完成时，THE Transaction_Tool SHALL 返回未签名的交易数据
4. WHERE 需要合约交互，THE Transaction_Tool SHALL 编码合约调用数据
5. WHILE 构造交易时，THE Transaction_Tool SHALL 返回人类可读的交易摘要

### Requirement 7: MCP 工具 - 交易广播

**User Story:** 作为用户，我希望 Agent 能够广播我签名后的交易到区块链网络。

#### Acceptance Criteria

1. WHEN 接收到已签名交易时，THE Broadcast_Tool SHALL 验证签名的有效性
2. WHEN 验证通过后，THE Broadcast_Tool SHALL 将交易广播到对应的区块链网络
3. WHEN 广播成功时，THE Broadcast_Tool SHALL 返回交易哈希
4. WHERE 广播失败，THE Broadcast_Tool SHALL 返回详细的错误信息
5. WHILE 交易待确认时，THE Broadcast_Tool SHALL 提供状态查询功能

### Requirement 8: MCP 工具 - 合约交互

**User Story:** 作为用户，我希望 Agent 能够帮我与智能合约交互。

#### Acceptance Criteria

1. WHEN 用户请求合约交互时，THE Contract_Tool SHALL 解析合约 ABI
2. WHEN 构造合约调用时，THE Contract_Tool SHALL 编码函数参数
3. WHEN 模拟交易时，THE Contract_Tool SHALL 使用 eth_call 预测结果
4. WHERE 合约未验证，THE Contract_Tool SHALL 警告用户潜在风险
5. WHILE 交互进行时，THE Contract_Tool SHALL 提供交易的详细解释


