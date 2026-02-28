# 仓库状态存档

## 基本信息

| 项目 | 内容 |
|------|------|
| 远程仓库 | `git@github.com:logos-42/alou-rust.git` |
| 当前分支 | `wasm` |
| 追踪状态 | 与 `origin/wasm` 同步 |
| 当前时间 | 2026-02-28 06:56 UTC |
| 当前版本 | v0.1.10-2-g109fcbf |

---

## 统计信息

| 指标 | 数值 |
|------|------|
| 总文件数 | 1,267 个 |
| 总提交数 | 220 个 |
| 代码总量 | 约 72.2 MB |

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
| `109fcbf` | docs: add repository archive state snapshot |
| `a494a31` | fix: tauri.conf.json - disable updater placeholder, add macOS dmg+app targets |
| `07f0405` | feat: MVP 0.1.10 Release - AI 自进化文档系统 |
| `d7d994a` | feat: update API and add tool_api module |
| `4d3290b` | feat(cli): 添加 cmd_run 函数和 npm 包配置 |
| `2f65210` | feat: MVP 0.1.10 - 修复群聊按钮、完善CLI和配置 |
| `fc8bf2b` | feat(cli): 完善CLI功能 - agent模块、API配置、存档管理 |
| `aefc80d` | feat: CLI添加run命令支持长时间自主循环 |
| `68b82ce` | fix: 修复CLI agent模块的类型导入问题 |
| `93a6e24` | feat: MVP 0.1.10 - 群聊功能完善、CLI增强 |

---

## 工作区状态

### 已修改但未暂存 (12个)
```
README.md
alou-desktop/package-lock.json
alou-desktop/package.json
alou-desktop/src/components/AgentChat/useGroupChatButton.jsx
alou-desktop/src/components/AgentChat/useGroupChatRemoteControl.ts
alou-desktop/src/components/agent/GroupChatMessage.jsx
alou-desktop/src/components/agent/GroupChatPanel.css
alou-desktop/src/components/agent/GroupChatPanel.jsx
alou-desktop/src/hooks/useLocalIpfsGroupChat.ts
alou-desktop/src/services/agentCoordinatorService.ts
alou-desktop/src/services/localIpfsGroupChatService.ts
alou-desktop/src/services/pubsubService.ts
```

### 未跟踪文件 (14个)
```
CHANGELOG.md
CODE_QUALITY_ANALYSIS_REPORT.md
COMPREHENSIVE_ANALYSIS_REPORT.md
MVP_DESKTOP_GROUPCHAT_PLAN.md
MVP_RELEASE_PLAN.md
alou-desktop/src/components/common/NetworkStatus.css
alou-desktop/src/components/common/NetworkStatus.jsx
alou-desktop/src/components/common/OnboardingGuide.css
alou-desktop/src/components/common/OnboardingGuide.jsx
alou-desktop/src/components/common/Toast.css
alou-desktop/src/components/common/Toast.jsx
alou-desktop/src/hooks/useAutoResizeTextarea.js
alou-desktop/src/hooks/useNewMessageNotification.js
alou-desktop/src/hooks/useScrollBehavior.js
alou-desktop/src/hooks/useToast.js
alou-desktop/src/types/groupchat.ts
alou-desktop/src/utils/groupchat/
frontend/package-lock.json
```

### 已暂存文件
无

---

## 项目结构

```
alou/
├── .cursor/            # Cursor IDE 配置
├── .vscode/            # VS Code 配置
├── alou-cli/           # CLI 工具 (Rust)
├── alou-desktop/       # 桌面应用 (Tauri + React)
│   ├── src/
│   │   ├── components/
│   │   │   ├── agent/          # 群聊组件
│   │   │   ├── common/         # 通用组件 (新增)
│   │   │   └── AgentChat/      # Agent聊天组件
│   │   ├── hooks/              # 自定义Hooks (新增)
│   │   ├── types/              # TypeScript类型定义
│   │   └── utils/              # 工具函数
│   └── ...
├── alou-edge/          # 边缘计算服务 (Rust)
└── [配置文件]          # 根目录配置文件
```

---

## 近期开发重点

基于工作区变更，当前开发重点为：

1. **群聊功能完善** - GroupChatPanel、GroupChatMessage 等组件持续迭代
2. **新增通用组件** - Toast、NetworkStatus、OnboardingGuide
3. **Hooks增强** - useToast、useScrollBehavior、useAutoResizeTextarea 等
4. **TypeScript支持** - 新增 groupchat.ts 类型定义

---

## 存档时间

- **存档创建**: 2026-02-28 06:56 UTC
- **存档者**: 系统自动存档
- **存档ID**: `archive-20260228-0656`

---

*此文件由自动存档脚本生成，用于记录仓库当前状态快照。*
