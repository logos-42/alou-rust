# 🤖 Bot Gateway 使用指南

## 概述

Bot Gateway 是 Alou 内置的多平台 Bot 支持模块，允许用户通过即时通讯工具（Telegram、飞书等）远程调用 Alou 的工具和能力。

## 功能特性

- ✅ **多平台支持**: Telegram、飞书（更多平台即将支持）
- ✅ **统一管理**: 在一个界面配置所有平台 Bot
- ✅ **工具调用**: 远程执行 Alou 的所有工具（spec、filesystem、bash 等）
- ✅ **实时日志**: 查看 Bot 运行日志
- ✅ **权限控制**: 支持用户白名单和权限验证
- ✅ **热配置**: 配置修改后自动生效

---

## 快速开始

### 1. 打开 Bot Gateway 设置

在 Alou Desktop 应用中：
1. 点击设置图标 ⚙️
2. 选择 "Bot Gateway" 选项卡
3. 进入配置界面

### 2. 配置 Telegram Bot

#### 获取 Bot Token
1. 在 Telegram 中搜索并打开 [@BotFather](https://t.me/BotFather)
2. 发送 `/newbot` 创建新 Bot
3. 按照提示设置 Bot 名称和用户名
4. 复制 Bot Token（格式：`123456789:ABCdefGHIjklMNOpqrsTUVwxyz`）

#### 配置步骤
1. 在 Bot Gateway 设置中选择 "Telegram" 选项卡
2. 勾选 "启用 Telegram Bot"
3. 粘贴 Bot Token
4. 点击 "测试连接" 验证配置
5. 点击 "保存配置"

#### 使用 Bot
在 Telegram 中与你的 Bot 对话：
```
/help - 查看帮助
/status - 查看状态
/spec list - 列出规格文档
```

---

### 3. 配置飞书 Bot

#### 创建飞书应用
1. 访问 [飞书开放平台](https://open.feishu.cn/)
2. 登录企业账号
3. 点击 "创建应用"
4. 选择 "机器人" 应用类型
5. 在 "凭证与基础信息" 页面获取：
   - App ID
   - App Secret

#### 配置验证 Token
1. 在 "事件订阅" 页面
2. 设置验证 Token（自定义字符串）
3. 可选：设置加密 Key

#### 配置步骤
1. 在 Bot Gateway 设置中选择 "飞书" 选项卡
2. 勾选 "启用飞书 Bot"
3. 填写 App ID、App Secret、验证 Token
4. 点击 "测试连接" 验证配置
5. 点击 "保存配置"

#### 部署 Webhook
1. 确保 Alou Desktop 运行
2. 在飞书开放平台配置事件订阅
3. 请求 URL 设置为：`https://your-domain.com:8080/webhook/feishu`
4. 完成验证

---

### 4. 配置 Discord Bot

#### 创建 Discord 应用
1. 访问 [Discord Developer Portal](https://discord.com/developers/applications)
2. 点击 "New Application"
3. 在 "Bot" 页面点击 "Add Bot"
4. 复制 Bot Token

#### 邀请 Bot 到服务器
1. 在 "OAuth2" > "URL Generator" 页面
2. 选择 scopes: `bot`
3. 选择权限：`Send Messages`, `Read Message History` 等
4. 复制生成的 URL 并在浏览器打开
5. 选择服务器并授权

#### 配置步骤
1. 在 Bot Gateway 设置中选择 "Discord" 选项卡
2. 勾选 "启用 Discord Bot"
3. 粘贴 Bot Token
4. 点击 "测试连接" 验证配置
5. 点击 "保存配置"

#### 使用 Bot
在 Discord 中与你的 Bot 对话：
```
!help - 查看帮助
!status - 查看状态
!spec list - 列出规格文档
```

---

### 5. 配置 QQ Bot

#### 安装 OneBot 兼容框架
QQ Bot 需要运行 OneBot 兼容的机器人框架，如：
- [go-cqhttp](https://github.com/Mrs4s/go-cqhttp)
- [Lagrange.Core](https://github.com/LagrangeDev/Lagrange.Core)

#### 配置 go-cqhttp
1. 下载并运行 go-cqhttp
2. 配置 `config.yml` 启用 WebSocket 服务端
3. 设置访问令牌（可选）

```yaml
servers:
  - ws:
      address: 0.0.0.0:8080
      access-token: "your_token"
```

#### 配置步骤
1. 在 Bot Gateway 设置中选择 "QQ" 选项卡
2. 勾选 "启用 QQ Bot"
3. 填写 WebSocket URL（如：`ws://127.0.0.1:8080`）
4. 填写 Access Token（如果配置了）
5. 点击 "测试连接" 验证配置
6. 点击 "保存配置"

#### 使用 Bot
在 QQ 中发送消息给 Bot 或在群中使用：
```
/help - 查看帮助
/status - 查看状态
/spec list - 列出规格文档
```

---

## 可用命令

### 通用命令

| 命令 | 说明 | 示例 |
|------|------|------|
| `/help` | 显示帮助信息 | `/help` |
| `/status` | 查看服务状态 | `/status` |
| `/tools` | 列出可用工具 | `/tools` |

### Spec 工具命令

| 命令 | 说明 | 示例 |
|------|------|------|
| `/spec create <type>` | 创建规格文档 | `/spec create product` |
| `/spec get <id>` | 获取规格文档 | `/spec get abc123` |
| `/spec list [type]` | 列出规格文档 | `/spec list product` |
| `/spec validate <id>` | 验证规格文档 | `/spec validate abc123` |

---

## 配置说明

### 通用设置

- **启用 Bot Gateway**: 开启/关闭服务
- **监听端口**: HTTP 服务监听端口（默认：8080）
- **命令前缀**: 命令触发前缀（默认：`/`）

### Telegram 配置

- **Bot Token**: Telegram Bot 认证令牌
- **允许的用户 IDs**: 白名单用户 ID 列表（留空表示允许所有）
- **允许的聊天 IDs**: 白名单聊天 ID 列表（留空表示允许所有）
- **使用轮询模式**: 启用长轮询接收消息（无需 Webhook）

### 飞书配置

- **App ID**: 飞书应用 ID
- **App Secret**: 飞书应用密钥
- **验证 Token**: 事件订阅验证 Token
- **加密 Key**: 消息加密密钥（可选）
- **允许的用户 IDs**: 白名单用户列表
- **允许的租户 IDs**: 白名单租户列表

---

## 安全建议

1. **设置白名单**: 在允许的用户 IDs 中添加授权用户
2. **保护 Token**: 不要泄露 Bot Token 或 App Secret
3. **使用 HTTPS**: 公网部署时使用 HTTPS 加密
4. **定期更新密钥**: 定期更换 Token 和密钥
5. **监控日志**: 定期检查日志发现异常访问

---

## 故障排查

### Bot 无法连接

1. 检查 Alou Desktop 是否运行
2. 检查端口是否被占用
3. 查看日志中的错误信息
4. 测试网络连接

### 命令无响应

1. 确认命令格式正确
2. 检查用户是否在白名单中
3. 查看权限配置
4. 检查工具是否可用

### Webhook 验证失败（飞书）

1. 确认验证 Token 一致
2. 检查请求 URL 是否正确
3. 确保服务可公网访问
4. 查看签名验证日志

---

## 高级用法

### 自定义命令

可以通过修改配置文件添加自定义命令：

```toml
# ~/.alou/bot_gateway_config.toml

[[custom_commands]]
command = "deploy"
description = "部署应用"
script = "scripts/deploy.sh"
required_permission = "admin"
```

### 工具权限映射

```toml
[[tool_permissions]]
tool_id = "spec"
required_permission = "execute_spec"

[[tool_permissions]]
tool_id = "filesystem"
required_permission = "execute_file"
```

---

## API 接口

Bot Gateway 提供 HTTP API 供外部调用：

### 健康检查
```
GET /health
响应：OK
```

### 获取状态
```
GET /status
响应：{
  "running": true,
  "port": 8080,
  "platforms": ["telegram", "feishu"],
  "uptime_seconds": 3600
}
```

### 获取日志
```
GET /logs
响应：[
  {
    "timestamp": 1709798400,
    "level": "info",
    "message": "收到 Telegram 消息",
    "platform": "telegram"
  }
]
```

### 执行工具
```
POST /api/tools/execute
请求：{
  "tool_id": "spec",
  "args": {
    "operation": "list",
    "spec_type": "product"
  }
}
响应：{
  "success": true,
  "data": {...},
  "execution_time_ms": 150
}
```

---

## 开发指南

### 添加新平台适配器

1. 在 `adapters/` 目录创建新适配器文件
2. 实现平台特定的消息处理逻辑
3. 在 `config.rs` 添加配置结构
4. 在 `server.rs` 添加 webhook 路由
5. 更新前端配置界面

### 示例：添加 Discord 适配器

```rust
// adapters/discord.rs
use crate::bot_gateway::config::DiscordConfig;

pub struct DiscordAdapter {
    config: DiscordConfig,
    // ...
}

impl DiscordAdapter {
    pub fn new(config: DiscordConfig) -> Self {
        // ...
    }
    
    pub async fn send_message(&self, channel_id: &str, text: &str) -> Result<(), String> {
        // ...
    }
}
```

---

## 技术架构

```
┌─────────────────────────────────────────────────────────────┐
│                    Alou Desktop Application                  │
│  ┌────────────────────────────────────────────────────────┐ │
│  │              Bot Gateway 配置界面                       │ │
│  └────────────────────────────────────────────────────────┘ │
│                          │                                   │
│                          ▼                                   │
│  ┌────────────────────────────────────────────────────────┐ │
│  │           BotGatewayManager (Rust)                     │ │
│  │  - 配置管理  - 服务控制  - 日志管理                    │ │
│  └────────────────────────────────────────────────────────┘ │
│                          │                                   │
│                          ▼                                   │
│  ┌────────────────────────────────────────────────────────┐ │
│  │           HTTP Server (Axum)                           │ │
│  │  /webhook/telegram  /webhook/feishu  /api/tools/...    │ │
│  └────────────────────────────────────────────────────────┘ │
│                          │                                   │
│              ┌───────────┴───────────┐                      │
│              ▼                       ▼                      │
│  ┌─────────────────┐       ┌─────────────────┐             │
│  │ TelegramAdapter │       │  FeishuAdapter  │             │
│  └─────────────────┘       └─────────────────┘             │
└─────────────────────────────────────────────────────────────┘
```

---

## 相关文件

- **Rust 实现**: `alou-desktop/src-tauri/src/bot_gateway/`
- **前端组件**: `alou-desktop/src/components/BotGateway/`
- **配置文件**: `~/.alou/bot_gateway_config.toml`

---

## 支持的平台

| 平台 | 状态 | 功能 |
|------|------|------|
| Telegram | ✅ 已实现 | 消息接收/发送、命令解析、工具调用 |
| 飞书 | ✅ 已实现 | Webhook、事件订阅、消息回复 |
| Discord | ✅ 已实现 | Webhook、命令解析、消息回复 |
| QQ | ✅ 已实现 | OneBot 协议、私聊/群聊、命令解析 |

---

## 更新日志

### v0.1.0 (2026-03-07)
- ✨ 初始版本
- ✅ Telegram 适配器
- ✅ 飞书适配器
- ✅ 配置管理界面
- ✅ 日志查看器
- ✅ 工具调用集成

---

## 常见问题

**Q: Bot Gateway 需要一直运行吗？**
A: 是的，需要保持 Alou Desktop 运行才能接收消息。

**Q: 可以在多个平台同时使用吗？**
A: 可以，支持同时启用多个平台 Bot。

**Q: 如何限制某些用户的使用权限？**
A: 在配置中添加允许的用户 IDs 白名单。

**Q: 支持自定义命令吗？**
A: 当前支持内置命令，自定义命令功能开发中。

---

## 联系方式

如有问题或建议，请提交 Issue 或联系开发团队。
