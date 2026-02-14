# Alou CLI

Alou AI Agent 终端命令行工具 - 通过 npm 全局安装使用。

## 功能特性

- 🤖 **智能体管理** - 创建、列出、删除 AI 智能体
- 🔄 **自主循环** - 自动化任务执行（持续循环）
- 📋 **任务队列** - 任务添加、列表管理
- ⚙️ **配置管理** - API 密钥、模型配置

## 安装

```bash
# 全局安装
npm install -g alou-cli

# 或本地安装后运行 postinstall
npm install
npm run postinstall
```

## 使用方法

```bash
# 查看帮助
alou help

# 工具系统
alou tool list                     # 列出所有工具
alou tool exec bash '{"command":"ls -la"}'  # 执行工具
alou tool help <工具ID>             # 工具帮助

# 智能体管理
alou agent create <名称> [描述] [人设]  # 创建智能体
alou agent list                     # 列出智能体

# 群聊协作
alou chat create <名称> [创建者]     # 创建群聊
alou chat list                      # 列出群聊

# 自主循环
alou start                          # 启动（设置状态）
alou loop                           # 开始循环执行任务
alou stop                           # 停止
alou pause                          # 暂停
alou resume                         # 恢复
alou status                         # 状态
alou run                            # 执行单次任务

# 任务管理
alou task add <标题> [描述] [优先级]  # 添加任务
alou task list                      # 任务列表
```

## 可用工具

| 工具ID | 功能 |
|--------|------|
| `filesystem` | 文件系统操作 (读/写/复制/删除) |
| `search` | 代码和文件搜索 |
| `bash` | 终端命令执行 |
| `network` | 网络操作 (ping/curl/wget) |
| `system` | 系统信息和管理 |
| `agent_collaboration` | 智能体协作 |
| `agent_creator` | 智能体创建 |

## 开发

```bash
# 本地开发
npm install
npm run postinstall  # 编译 Rust

# 手动编译
npm run build

# 测试
./bin/alou help
```

## 要求

- Node.js >= 16.0.0
- Rust (安装 CLI 时自动检测)

## License

MIT
