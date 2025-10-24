# Implementation Plan

## Phase 1: 基础设施和环境配置

- [x] 1. 设置开发环境和项目结构




  - 配置 Rust WASM 编译工具链
  - 安装 worker-build 和 wrangler CLI
  - 创建 alou-edge 项目目录和模块结构
  - 配置 Cargo.toml 依赖（worker, serde, tokio 等）
  - _Requirements: 1.1_


- [x] 1.1 配置 Cloudflare 资源

  - 创建 D1 数据库实例
  - 创建 KV namespaces（SESSIONS, CACHE, NONCES）
  - 配置 wrangler.toml 绑定
  - 设置 Secrets（CLAUDE_API_KEY, JWT_SECRET, ETH_RPC_URL, SOLANA_RPC_URL）
  - _Requirements: 1.1, 2.1_

- [x] 1.2 设置数据库 schema


  - 编写 D1 迁移脚本（users 表）
  - 创建索引（wallet_address）
  - 执行初始化迁移
  - _Requirements: 2.1_


- [x] 1.3 实现存储层抽象

  - 实现 D1Database wrapper（query, execute 方法）
  - 实现 KvStore wrapper（get, put, delete 方法）
  - 实现错误处理（AloudError 枚举）
  - _Requirements: 2.2, 2.5_

## Phase 2: 会话管理

- [x] 2. 实现 SessionManager





  - 实现 create_session 方法（生成 UUID）
  - 实现 get_session 方法（从 KV 读取）
  - 实现 add_message 方法（追加消息到会话）
  - 实现 get_history 方法（返回消息列表）
  - _Requirements: 2.1, 2.2, 2.3_

- [x] 2.1 实现会话存储逻辑


  - 定义 Session 和 Message 数据结构
  - 实现 KV 序列化/反序列化（serde_json）
  - 实现会话 TTL（24 小时）
  - 限制每个会话最多 50 条消息
  - _Requirements: 2.2, 2.4_

- [x] 2.2 实现会话 API 端点


  - POST /api/session - 创建新会话
  - GET /api/session/:id - 获取会话历史
  - DELETE /api/session/:id - 清除会话
  - 添加 CORS 支持
  - _Requirements: 2.1, 2.3_

- [ ]* 2.3 编写会话管理测试
  - 测试会话创建
  - 测试消息添加
  - 测试会话过期
  - _Requirements: 2.4_

## Phase 3: MCP 客户端集成

- [x] 3. 实现 MCP 客户端




  - 实现 MCP 协议客户端（连接到 npx blockchain-payment-mcp）
  - 实现工具列表查询（tools/list）
  - 实现工具调用（tools/call）
  - 实现错误处理和重试
  - _Requirements: 3.1, 3.2_



- [x] 3.1 实现 MCP 连接管理




  - 实现 stdio 通信（与 npx 进程通信）
  - 实现 JSON-RPC 消息格式
  - 实现请求/响应匹配
  - 实现连接池（复用进程）
  - _Requirements: 3.1_
-

- [x] 3.2 实现 MCP 工具代理





  - 从 MCP 服务器获取工具列表
  - 将 MCP 工具暴露给 Claude Agent
  - 实现工具调用转发（Agent → MCP Server）
  - 实现结果转换（MCP → Agent）
  - _Requirements: 3.2, 3.3_

- [ ]* 3.3 编写 MCP 客户端测试
  - 测试连接建立
  - 测试工具列表查询
  - 测试工具调用
  - _Requirements: 3.5_

## Phase 4: Web3 钱包认证

- [x] 4. 实现签名验证器





  - 实现 Ethereum 签名验证（ethers-rs 或 web3）
  - 实现 Solana 签名验证（solana-sdk）
  - 实现 verify_signature 函数
  - _Requirements: 4.2, 4.4_

- [x] 4.1 实现 WalletAuthTool


  - 实现 McpTool trait for WalletAuthTool
  - 实现 generate_nonce 方法（存储到 KV，TTL 5 分钟）
  - 实现 verify_signature 方法
  - 实现 create_session 方法（生成 JWT token）
  - _Requirements: 4.1, 4.2, 4.3_

- [x] 4.2 实现 JWT token 管理


  - 实现 token 生成（使用 JWT_SECRET）
  - 实现 token 验证
  - 实现 token 解析（提取 wallet_address 和 chain）
  - Token 有效期 24 小时
  - _Requirements: 4.3, 4.5_

- [x] 4.3 实现钱包认证端点


  - GET /api/wallet/nonce/:address - 获取 nonce
  - POST /api/wallet/verify - 验证签名并返回 token
  - GET /api/wallet/me - 获取当前用户信息（需要 token）
  - _Requirements: 4.1, 4.2, 4.3_

- [ ]* 4.4 编写钱包认证测试
  - 测试 nonce 生成
  - 测试签名验证（Ethereum 和 Solana）
  - 测试 token 生命周期
  - _Requirements: 4.2_

## Phase 5: Claude Agent 集成


- [x] 5. 实现 Claude API 客户端




  - 使用边缘服务器支持的方式 调用 Claude API格式，但真实使用的是deepseek的API
  - 实现消息格式转换（Claude 格式）
  - 实现流式响应支持（SSE）
  - 实现错误处理和重试
  - _Requirements: 1.1, 1.5_

- [x] 5.1 实现 AgentCore


  - 实现 handle_message 方法
  - 集成 SessionManager（加载历史）
  - 集成 MCP 客户端（调用 MCP 工具）
  - 实现工具调用循环（Claude SDK → MCP Tool → Claude SDK）
  - _Requirements: 1.1, 1.2, 1.4_

- [x] 5.2 实现 Agent 上下文管理

  - 定义 AgentContext 结构
  - 从会话中提取 wallet_address 和 chain
  - 传递上下文到 MCP 工具调用
  - _Requirements: 1.3_


- [x] 5.3 实现 Agent API 端点

  - POST /api/agent/chat - 发送消息
  - POST /api/agent/stream - 流式对话（SSE）
  - 添加认证检查（可选）
  - _Requirements: 1.1, 1.2_

- [x] 5.4 编写 Agent 集成测试






  - 测试完整对话流程
  - 测试 MCP 工具调用
  - 测试多轮对话
  - _Requirements: 1.3_

## Phase 6: API 路由和主入口
-

- [x] 6. 实现 API 路由



  - 实现 Router 结构（路由表）
  - 实现路由匹配逻辑（path + method）
  - 集成所有端点（session, wallet, agent）
  - 实现 404 处理
  - _Requirements: 1.1_



- [x] 6.1 实现 Worker 主入口


  - 实现 fetch 函数（Worker 入口）
  - 初始化所有服务（SessionManager, MCP Client, AgentCore）
  - 启动 MCP 服务器连接
  - 实现全局错误处理


  - _Requirements: 1.1, 1.5_


- [x] 6.2 实现健康检查和监控






  - GET /api/health - 健康检查
  - GET /api/status - 服务状态
  - 实现请求日志记录
  - 实现性能指标收集
  - _Requirements: 1.5_


- [x] 6.3 编写端到端测试




  - 测试完整的用户流程（认证 → Agent 对话 → MCP 工具调用）
  - 测试错误处理


  - 测试性能指标
  - _Requirements: 1.1_


## Phase 7: 部署和优化

- [x] 7. 构建和部署
  - 配置 Cargo.toml 优化选项（opt-level = "z"）
  - 运行 cargo build --target wasm32-unknown-unknown --release
  - 创建 shim.mjs
  - 创建测试脚本（test-local.ps1, test-endpoints.ps1）
  - 编写 README.md 文档
  - _Requirements: 1.5_

- [x] 7.1 性能优化


  - 优化 WASM 包大小（移除未使用的依赖）
  - 实现 KV 缓存策略
  - 优化 MCP 通信（连接复用）
  - 实现请求合并
  - _Requirements: 1.5_

- [x] 7.2 文档和示例
  - 编写 README.md（包含 API 文档、部署指南、故障排查）
  - 编写 Agent 使用示例
  - 创建本地测试脚本
  - 创建端点测试脚本
  - _Requirements: 1.1_

- [ ]* 7.3 监控和告警
  - 配置 Cloudflare Analytics
  - 设置错误告警
  - 设置性能监控
  - _Requirements: 1.5_
