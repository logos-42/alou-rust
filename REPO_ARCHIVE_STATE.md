# 仓库状态存档

## 基本信息

| 项目 | 内容 |
|------|------|
| 远程仓库 | `git@github.com:logos-42/alou-rust.git` |
| 当前分支 | `wasm` |
| 追踪状态 | 与 `origin/wasm` 同步 |
| 当前时间 | 2026-02-28 07:56 UTC |
| 当前版本 | v0.1.10-6-gaa02ea6 |

---

## 统计信息

| 指标 | 数值 |
|------|------|
| 总文件数 | 约 1,265 个 |
| 总提交数 | 226 个 |
| 代码总量 | 约 72 MB |

---

## 分支状态

### 本地分支
```
* wasm (当前分支)
```

### 远程分支
```
remotes/origin/HEAD -> origin/main
remotes/origin/main
remotes/origin/wasm
```

---

## 最近提交记录 (最新10条)

| 提交哈希 | 提交信息 |
|---------|----------|
| `aa02ea6` | fix: update useLocalIpfsGroupChatManager |
| `3acef22` | fix: update group chat manager hooks |
| `85fa4f4` | fix: P0/P1问题修复与群聊功能完善 |
| `a434cff` | docs: update repository archive state snapshot (v3) |
| `d046386` | feat: 群聊功能完善与通用组件增强 |
| `588a76c` | docs: update repository archive state snapshot (v2) |
| `109fcbf` | docs: add repository archive state snapshot |
| `a494a31` | fix: tauri.conf.json - disable updater placeholder, add macOS dmg+app targets |
| `07f0405` | feat: MVP 0.1.10 Release - AI 自进化文档系统 |
| `d7d994a` | feat: update API and add tool_api module |

---

## 工作区状态

### 已删除文件 (22个)
```
alou-desktop/scripts/test-ai-real.cjs
alou-desktop/scripts/test-ai-response.js
alou-desktop/scripts/test-autonomy-complete.cjs
alou-desktop/scripts/test-backend-connection.js
alou-desktop/scripts/test-claude-agent.js
alou-desktop/scripts/test-claude-request.json
alou-desktop/scripts/test-claude-request.result.json
alou-desktop/scripts/test-complete.cjs
alou-desktop/scripts/test-direct-api.js
alou-desktop/scripts/test-final-sdk-workflow.js
alou-desktop/scripts/test-fixed-agent.js
alou-desktop/scripts/test-full-loop.cjs
alou-desktop/scripts/test-group-chat.cjs
alou-desktop/scripts/test-local-backend.js
alou-desktop/scripts/test-model-conversion.js
alou-desktop/scripts/test-network.js
alou-desktop/scripts/test-plugin-system.cjs
alou-desktop/scripts/test-request-deepseek.json
alou-desktop/scripts/test-request-deepseek.result.json
alou-desktop/scripts/test-sdk-format.js
alou-desktop/scripts/test-tools-fix.js
alou-desktop/scripts/test-workers-connection.js
alou-desktop/scripts/test-workers-health.js
```

### 已修改但未暂存 (2个)
```
alou-desktop/src/hooks/useDiapGroupChat.ts
alou-desktop/src/hooks/useMultiAgentChat.ts
```

### 已暂存文件
无

### 未跟踪文件
无

---

## 项目结构

```
alou/
├── .cursor/            # Cursor IDE 配置
├── .vscode/            # VS Code 配置
├── alou-cli/           # CLI 工具 (Rust)
├── alou-desktop/       # 桌面应用 (Tauri + React)
│   ├── scripts/        # 测试脚本 (已清理)
│   ├── src/
│   │   ├── components/
│   │   │   ├── agent/          # 群聊组件
│   │   │   ├── common/         # 通用组件
│   │   │   ├── AgentChat/      # Agent聊天组件
│   │   │   └── [其他组件]
│   │   ├── hooks/              # 自定义Hooks
│   │   ├── services/           # 服务层
│   │   ├── types/              # TypeScript类型定义
│   │   ├── utils/              # 工具函数
│   │   ├── views/              # 页面视图
│   │   └── [其他文件]
│   └── [配置文件]
├── alou-edge/          # 边缘计算服务 (Rust)
└── [根目录配置文件]
```

---

## 近期开发重点

### 当前进行中

1. **测试脚本清理**
   - 删除了22个测试脚本文件
   - 清理临时测试文件和结果文件

2. **Hooks优化**
   - useDiapGroupChat.ts 修改中
   - useMultiAgentChat.ts 修改中

3. **已完成工作**
   - P0/P1问题修复 ✅
   - 群聊功能完善 ✅
   - 通用组件增强 ✅

---

## 历史归档

| 时间 | 提交 | 说明 |
|------|------|------|
| 2026-02-28 06:38 | 109fcbf | 首次存档 |
| 2026-02-28 06:57 | 588a76c | 第二次存档 |
| 2026-02-28 06:58 | d046386 | 群聊功能完善 |
| 2026-02-28 07:53 | a434cff | 第三次存档 |
| 2026-02-28 07:55 | 85fa4f4 | P0/P1修复 |
| 2026-02-28 07:55 | 3acef22 | hooks更新 |
| 2026-02-28 07:55 | aa02ea6 | manager更新 |

---

## 存档时间

- **存档创建**: 2026-02-28 07:56 UTC
- **存档者**: 系统自动存档
- **存档ID**: `archive-20260228-0756`

---

*此文件由自动存档脚本生成，用于记录仓库当前状态快照。*
