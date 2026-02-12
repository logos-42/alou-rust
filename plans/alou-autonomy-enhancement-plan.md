# Alou 系统能动性增强架构规划

**创建时间**: 2026-02-12
**最后更新**: 2026-02-12
**参考**: https://github.com/logos-42/macopenclaw/tree/master/alou-project

---

## 📋 核心目标

让 Alou AI 具备真正的**自主工作能力**：
- ✅ 自主工作（无需人工持续干预）
- ✅ 循环做事（持续执行任务）
- ✅ 工具调用（使用 Skills 和工具）
- ✅ 记忆管理（短期 + 长期记忆）
- ✅ 主动帮助（发现需要就行动）

---

## 🏗️ 系统架构总览

```
┌─────────────────────────────────────────────────────────────────┐
│                      Alou AI 核心系统                            │
├─────────────────────────────────────────────────────────────────┤
│  ┌───────────────────────────────────────────────────────────┐ │
│  │              Claude Skills 自主执行引擎                     │ │
│  │                                                           │ │
│  │   用户需求 → AI分析 → 选择Skill → 执行Skill → 返回结果     │ │
│  └───────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────────┐ │
│  │ Skills发现  │ │ Skill选择  │ │       工具执行              │ │
│  │ Discover    │ │ AutoSelect │ │       ToolExecutor           │ │
│  └─────────────┘ └─────────────┘ └─────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                    Claude SDK Skills 格式                        │
│              ~/.alou/skills/{skill_name}/SKILL.md               │
└─────────────────────────────────────────────────────────────────┘
```

---

## ✅ Phase 1: 任务队列系统 - 已完成

### 核心功能
- 任务添加、查询、更新
- 优先级排序（Critical > High > Medium > Low）
- 状态管理
- 持久化存储
- 心跳检测

### 代码位置
- **[`task_queue.rs`](alou-desktop/src-tauri/src/tools/task_queue.rs)** - 核心实现
- **[`task_queue_tool.rs`](alou-desktop/src-tauri/src/tools/task_queue_tool.rs)** - Tauri命令

---

## ✅ Phase 2: 记忆管理系统 - 已完成

### 核心功能
- 短期记忆（内存存储）
- 长期记忆（文件持久化）
- IPFS归档支持
- 垃圾回收机制

### 代码位置
- **[`memory_manager.rs`](alou-desktop/src-tauri/src/memory_manager.rs)** - 核心实现

---

## ✅ Phase 3: 心跳机制与自主循环 - 已完成

### 核心功能
- 自主智能体状态管理
- 心跳检测（默认30秒间隔）
- 任务自动执行
- 状态持久化

### 代码位置
- **[`autonomous_agent.rs`](alou-desktop/src-tauri/src/autonomous_agent.rs)** - 自主智能体
- **[`ai_loop.rs`](alou-desktop/src-tauri/src/ai_loop.rs)** - AI循环

---

## ✅ AI自动响应与 Phase 4:群聊功能 - 已完成

### 核心功能
- @提及响应（@alou, @AI, @agent）
- 关键词触发响应
- 响应冷却机制

### 代码位置
- **[`autonomous_executor.rs`](alou-desktop/src-tauri/src/tools/autonomous_executor.rs)** - 自主执行引擎
- **[`autonomous_executor_tool.rs`](alou-desktop/src-tauri/src/tools/autonomous_executor_tool.rs)** - Tauri命令

---

## ✅ Phase 5: Claude Skills 自动选择器 - 🆕 已完成

### 🆕 核心功能：与Claude官方SDK一致

**Claude Skills 格式**：
```
~/.alou/skills/{skill_name}/SKILL.md
```

**SKILL.md 标准格式**：
```markdown
# name
技能名称

# description
技能描述（用于发现）

# version
1.0.0

# license
MIT

# allowed_tools
- filesystem
- search

# parameters
{
  "type": "object",
  "properties": {
    "param1": {
      "type": "string",
      "description": "参数描述"
    }
  },
  "required": ["param1"]
}

# instructions
## 执行步骤

1. 步骤1
2. 步骤2
```

### Claude Skills 服务 ([`agentSkillsService.ts`](alou-desktop/src/services/agentSkillsService.ts))

| 方法 | 功能 |
|------|------|
| `discoverSkills()` | 发现可用Skills（Progressive Disclosure Level 1） |
| `loadSkill()` | 加载Skill完整内容（Progressive Disclosure Level 2） |
| `executeSkill()` | 执行Skill |
| `createSkill()` | 创建新Skill |
| `deleteSkill()` | 删除Skill |
| `listSkills()` | 列出所有Skills |
| `autoExecute()` | AI自动选择并执行Skill |
| `executeSkillChain()` | 执行Skill链 |

### 示例 Skills
| Skill名称 | 功能 |
|-----------|------|
| `readme` | 读取并分析README文件 |
| `analyze_code` | 分析项目代码结构 |

---

## ✅ Phase 6: Claude Skills 自主执行引擎 - 🆕 已完成

### 🆕 核心功能：端到端工作流

**Claude Skills 工作流**：
```
用户输入 → AI分析需求 → 选择最佳Skill → 执行Skill → 返回结果
```

### 代码位置
- **[`autonomousAgentService.ts`](alou-desktop/src/services/autonomousAgentService.ts)** - 自主智能体服务

### 核心方法

| 方法 | 功能 |
|------|------|
| `runClaudeSkillsWorkflow()` | Claude Skills 完整工作流 |
| `runHybridWorkflow()` | Skills + 工具混合工作流 |
| `executeClaudeSkill()` | AI自动选择并执行Skill |
| `executeSkillChain()` | 执行多个Skills |

---

## 🚀 Claude Skills 工作流示例

```
用户: "@alou 请帮我分析项目代码结构"

┌─────────────────────────────────────────────────────────────┐
│ Claude Skills 工作流                                        │
├─────────────────────────────────────────────────────────────┤
│ 1. 发现 Skills                                              │
│    ✅ 发现 5 个Skills: readme, analyze_code, ...          │
├─────────────────────────────────────────────────────────────┤
│ 2. 分析用户意图                                             │
│    🎯 意图: "分析代码结构"                                  │
│    📊 匹配: analyze_code (置信度 95%)                        │
├─────────────────────────────────────────────────────────────┤
│ 3. 选择最佳Skill                                           │
│    ✅ 选择: analyze_code                                    │
│    📝 描述: 分析项目代码结构，生成分析报告                    │
├─────────────────────────────────────────────────────────────┤
│ 4. 加载Skill内容                                           │
│    📖 加载: ~/.alou/skills/analyze_code/SKILL.md           │
├─────────────────────────────────────────────────────────────┤
│ 5. 执行Skill                                               │
│    ⚙️ 执行 analyze_code                                    │
│    📊 结果: {                                             │
│      "total_files": 100,                                   │
│      "by_extension": { ".ts": 50, ".tsx": 30 },           │
│      "modules": ["module1", "module2"]                     │
│    }                                                       │
├─────────────────────────────────────────────────────────────┤
│ 6. 返回结果                                                │
│    ✅ 执行成功 (350ms)                                      │
│    📊 项目包含 100 个文件，50 个 TypeScript 文件           │
└─────────────────────────────────────────────────────────────┘
```

---

## 📁 关键文件索引

### Claude Skills 系统
| 文件 | 描述 |
|------|------|
| [`agentSkillsService.ts`](alou-desktop/src/services/agentSkillsService.ts) | Claude Skills 服务 |
| [`autonomousAgentService.ts`](alou-desktop/src/services/autonomousAgentService.ts) | 自主智能体服务 |

### 工具执行系统
| 文件 | 描述 |
|------|------|
| [`autonomous_executor.rs`](alou-desktop/src-tauri/src/tools/autonomous_executor.rs) | 执行引擎核心 |
| [`autonomous_executor_tool.rs`](alou-desktop/src-tauri/src/tools/autonomous_executor_tool.rs) | Tauri命令 |

### Skills 自动选择
| 文件 | 描述 |
|------|------|
| [`skill_auto_selector.rs`](alou-desktop/src-tauri/src/tools/skill_auto_selector.rs) | 选择器核心 |
| [`skillAutoSelectorService.ts`](alou-desktop/src/services/skillAutoSelectorService.ts) | 前端服务 |

---

## 🎯 使用指南

### 1. 创建 Claude Skill

在 `~/.alou/skills/analyze_code/SKILL.md` 创建：

```markdown
# name
analyze_code

# description
分析项目代码结构，生成分析报告

# version
1.0.0

# allowed_tools
- filesystem
- search

# instructions
## 分析代码结构

1. 扫描指定目录下的所有代码文件
2. 统计文件数量、行数
3. 识别主要模块和依赖关系
4. 生成分析报告
```

### 2. 前端使用

```typescript
import autonomousAgentService from '@/services/autonomousAgentService';

// Claude Skills 工作流
const result = await autonomousAgentService.runClaudeSkillsWorkflow(
  "请帮我分析项目代码结构"
);

console.log(result.message);
// 输出: Claude Skill "analyze_code" 执行成功

// 直接执行Skill
await autonomousAgentService.executeClaudeSkill("分析代码");
```

---

*让 Alou AI 成为真正自主的 AI 助手*
*与 Claude Agent SDK Skills 格式完全一致*
