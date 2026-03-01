# 仓库存档状态快照

> 创建时间: 2026-03-01 14:27:55 UTC+8
> 分支: wasm
> 远程: origin (git@github.com:logos-42/alou-rust.git)

---

## 📊 当前仓库状态

### 分支信息
| 分支 | 状态 |
|------|------|
| wasm (当前) | 与 origin/wasm 同步 |
| main | remotes/origin/main |

### 最新提交 (Top 10)
```
05d2a4c Update: archive current changes
e9dd56b docs: add repository archive state snapshot (v10)
01afd19 feat: 智能体记忆系统与文档服务
5b5ff57 docs: add repository archive state snapshot (v9)
a0d9e44 fix: 工具参数修复与executor优化
3bcdafd docs: add repository archive state snapshot (v8)
5d33f0a chore: 清理临时文件并更新bash工具
1ca6980 docs: add repository archive state snapshot (v7)
963263c fix: 优化Tauri agent核心与streaming
9930f99 docs: add repository archive state snapshot (v6)
```

---

## 📝 未提交变更 (Working Directory Changes)

### 统计信息
- **文件变更**: 12 个文件
- **新增行数**: 668 行
- **删除行数**: 1295 行

### 详细变更清单

#### 已删除文件 (D)
| 文件 | 说明 |
|------|------|
| AGENT_MEMORY_SYSTEM.md | 智能体记忆系统文档 |
| COMPILATION_FIX_SUMMARY.md | 编译修复总结 |
| TOOL_PARAM_FIX_SUMMARY.md | 工具参数修复总结 |
| TOOL_SCHEMA_FIX_COMPLETE.md | 工具 schema 修复完成 |
| TOOL_SCHEMA_VERIFICATION.md | 工具 schema 验证 |
| add_normalize_function.py | Python 工具脚本 |
| fix_import.py | 导入修复脚本 |
| update_tool_handler.py | 工具处理器更新脚本 |

#### 已修改文件 (M)
| 文件 | 变更类型 |
|------|----------|
| alou-desktop/src-tauri/src/agent/executor.rs | 修改 |
| alou-desktop/src-tauri/src/tools/iroh_tool.rs | 修改 (+318 行) |
| alou-desktop/src-tauri/src/tools/pubsub_tool.rs | 修改 (+324 行) |
| alou-desktop/src/services/toolDescriptions.ts | 修改 |

---

## 🗂️ 项目结构概览

```
alou/
├── alou-cli/           # CLI 工具 (Rust)
├── alou-desktop/       # 桌面应用 (Tauri + TypeScript)
│   └── src/
│       └── services/   # 各种服务模块
├── alou-edge/          # Edge 计算 (Rust + Workers)
│   └── src/
│       ├── agent/      # Agent 核心逻辑
│       ├── durable_objects/  # Durable Objects
│       ├── mcp/       # MCP 协议实现
│       └── router/    # 路由处理
```

---

## 🔧 技术栈

| 模块 | 技术 |
|------|------|
| CLI | Rust, Cargo |
| Desktop | Tauri, TypeScript, React |
| Edge | Rust, Cloudflare Workers |
| 存储 | D1, KV, Durable Objects |
| 区块链 | 钱包集成, 交易处理 |

---

## 📌 备注

- 当前有未提交的变更，建议在推送前确认是否需要提交
- 已删除的文件主要是文档和临时脚本
- 主要变更是对 iroh_tool 和 pubsub_tool 的大幅修改
