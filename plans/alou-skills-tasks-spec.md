# Alou Agent Skills & Tasks 规范设计

## 基于人月神话原则的智能体系统架构

> 人月神话核心教义：Brooks法则 - "Adding manpower to a late software project makes it later"。本规范通过赋予Agent自主性和能动性，将"人月"转化为"Agent月"。

---

## 一、架构概览

### 1.1 核心组件分布

```
┌─────────────────────────────────────────────────────────────┐
│                        Frontend (TypeScript)                │
│  alou-desktop/src/types/skills.ts                           │
│  alou-desktop/src/types/tasks.ts                           │
│  alou-desktop/src/services/*                               │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Backend - alou-edge (Rust)               │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Task System (主要实现)                               │   │
│  │  - spec.rs: TaskSpec 定义                           │   │
│  │  - cluster_action.rs: Task, ClusterAction           │   │
│  │  - task_orchestrator.rs: 任务编排引擎                │   │
│  │  - cluster_executor.rs: 集群执行器                  │   │
│  │  - batch_processor.rs: 批处理                        │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Skill System (主要实现)                              │   │
│  │  - skill_loader.rs: 技能加载                         │   │
│  │  - skill_executor.rs: 技能执行                       │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## 二、Skills 规范

### 2.1 文件格式

Skills 存储于 `~/.alou/williw/skills/` 或项目 `./skills/`

### 2.2 SKILL.md 格式 (复用现有类型)

```yaml
---
name: skill-name
description: 技能描述
version: 1.0.0
license: MIT

# 渐进式披露级别 (Progressive Disclosure Level)
pdl:
  level: 1
  reveal_on: [action:execute, intent:analyze]

# 能力声明
capabilities:
  - type: tool
    name: filesystem
    operations: [read, write, list]
  - type: skill
    name: code-analyzer

# 允许的工具
allowed_tools:
  - bash
  - python
  - filesystem

# 依赖
dependencies:
  runtime: [python>=3.8]
  python: [requests, pandas]

# 参数定义 (JSON Schema)
parameters:
  type: object
  properties:
    input:
      type: string
      description: 输入数据
  required: [input]

# 指令
instructions: |
  ## 使用指南
  1. 分析需求
  2. 选择工具
  3. 执行并验证

# 示例
examples:
  - name: 基础分析
    input: {file: data.csv}
    expected: {rows: 1000}

# 元数据
metadata:
  category: data-analysis
  tags: [analytics, ml]
  difficulty: intermediate
```

### 2.3 Skills 类型定义

复用现有文件 [`alou-desktop/src/types/skills.ts`](alou-desktop/src/types/skills.ts):

```typescript
import {
  SkillDefinition,    // 技能定义
  SkillMetadata,      // 技能元数据
  SkillContext,      // 执行上下文
  SkillResult,       // 执行结果
  SkillParameters    // 参数定义
} from '../types/skills';
```

---

## 三、Task 规范 (Rust 实现)

### 3.1 现有 Rust 实现

Task 系统主要在 `alou-edge/src/agent/` 中实现:

| 文件 | 职责 |
|------|------|
| [`spec.rs`](alou-edge/src/agent/spec.rs) | TaskSpec 定义 |
| [`cluster_action.rs`](alou-edge/src/agent/cluster_action.rs) | Task, TaskStatus, ClusterAction |
| [`task_orchestrator.rs`](alou-edge/src/agent/task_orchestrator.rs) | 任务编排引擎 |
| [`cluster_executor.rs`](alou-edge/src/agent/cluster_executor.rs) | 集群执行器 |
| [`batch_processor.rs`](alou-edge/src/agent/batch_processor.rs) | 批处理 |

### 3.2 现有 Task 类型 (Rust)

```rust
// alou-edge/src/agent/cluster_action.rs

pub struct Task {
    pub task_id: String,
    pub task_type: String,
    pub description: String,
    pub input: Value,
    pub output: Option<Value>,
    pub status: TaskStatus,
    pub dependencies: Vec<TaskDependency>,
    pub assigned_agent: Option<AgentAssignment>,
    pub created_at: i64,
    pub started_at: Option<i64>,
    pub completed_at: Option<i64>,
    pub error: Option<String>,
    pub retry_count: u32,
    pub max_retries: u32,
}

pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

pub enum DependencyType {
    Sequential,    // 顺序执行
    Parallel,      // 并行执行
    Conditional { condition: String },
}
```

### 3.3 TASK.md 文件格式

```yaml
---
name: task-name
description: 任务描述
version: 1.0.0
type: parallel | sequential | swarm

# 前置条件
preconditions:
  - condition: "API密钥已配置"
    check: tool: verify_config
    required: true

# 执行步骤
steps:
  - id: step_1
    name: 数据准备
    type: parallel
    parallel_tasks:
      - task: data-clean
        agent: cleaner-1
      - task: data-validate
        agent: validator-1

  - id: step_2
    name: 生成报告
    type: tool
    tool: generate_report
    depends_on: [step_1]
    timeout_ms: 30000
    retry:
      max_attempts: 3
      strategy: exponential
      base_delay_ms: 1000

# Swarm 配置
swarm:
  strategy: broadcast | round_robin | capability_based | consensus
  min_agents: 2
  max_agents: 10
  timeout_ms: 60000
  
  roles:
    - name: coordinator
      responsibility: 任务分发与结果汇总
      required: true
    - name: worker
      responsibility: 执行具体子任务
      required: true
      count: 3

  consensus:
    enabled: true
    threshold: 0.7

# 验证规则
validation:
  rules:
    - id: output_format
      type: output_format
      severity: error
```

---

## 四、Task 执行引擎 (复用现有)

### 4.1 任务编排 (Rust)

[`task_orchestrator.rs`](alou-edge/src/agent/task_orchestrator.rs) 已实现:

```rust
pub enum WorkflowType {
    Sequential,   // 顺序执行
    Parallel,     // 并行执行
    Conditional { condition: String },
}

// 使用方式
let orchestrator = TaskOrchestrator::new();
let workflow = orchestrator.build_workflow(&tasks)?;

// 顺序执行
orchestrator.execute_sequential(&mut tasks, executor).await?;

// 并行执行  
orchestrator.execute_parallel(&mut tasks, executor).await?;

// 条件分支
orchestrator.execute_conditional(&mut cond_task, &mut true_branch, &mut false_branch, executor, evaluator).await?;
```

### 4.2 集群执行 (Rust)

[`cluster_executor.rs`](alou-edge/src/agent/cluster_executor.rs) 已实现:

```rust
// 执行集群行动
let result = cluster_executor
    .execute_cluster_action(action, wallet_address, chain)
    .await?;

// 合并多个 Agent 结果
let merged = cluster_executor.merge_results(&task_results)?;
```

---

## 五、目录结构

### 5.1 全局 Williw 目录

```
~/.alou/williw/
├── skills/                    # 全局技能库
│   ├── _core/                # 核心技能
│   ├── _community/          # 社区技能
│   └── _custom/             # 用户自定义技能
├── tasks/                    # 任务库
│   ├── _templates/          # 任务模板
│   └── _archived/           # 已完成任务
├── agents/                   # Agent 配置
└── config.yaml              # 全局配置
```

### 5.2 项目本地

```
project/
├── skills/
├── tasks/
├── .alou/
└── williw.config.yaml
```

---

## 六、与现有系统集成

### 6.1 现有 Rust 组件

| 组件 | 位置 | 状态 |
|------|------|------|
| TaskSpec | alou-edge/src/agent/spec.rs | ✅ 已有 |
| Task | alou-edge/src/agent/cluster_action.rs | ✅ 已有 |
| TaskOrchestrator | alou-edge/src/agent/task_orchestrator.rs | ✅ 已有 |
| ClusterExecutor | alou-edge/src/agent/cluster_executor.rs | ✅ 已有 |
| BatchProcessor | alou-edge/src/agent/batch_processor.rs | ✅ 已有 |

### 6.2 现有 TypeScript 组件

| 组件 | 位置 | 状态 |
|------|------|------|
| Task types | alou-desktop/src/types/tasks.ts | ✅ 已有 |
| Skill types | alou-desktop/src/types/skills.ts | ✅ 已有 |
| TaskQueue | alou-desktop/src/services/taskQueue.ts | ✅ 已有 |
| AgentCoordinator | alou-desktop/src/services/agentCoordinatorService.ts | ✅ 已有 |

### 6.3 需要实现

1. **SKILL.md 解析器**: 解析 YAML 为 SkillDefinition
2. **TASK.md 解析器**: 解析 YAML 为 TaskSpec
3. **williw CLI 工具**: 全局 skills/tasks 管理
4. **Rust 技能加载**: alou-edge 中实现技能加载器

---

## 七、总结

项目已有完善的 Task 系统实现 (Rust):

- **TaskSpec**: 任务规格定义
- **Task**: 任务实体
- **TaskOrchestrator**: 顺序/并行/条件执行
- **ClusterExecutor**: 多 Agent 协作执行
- **BatchProcessor**: 批量处理

需要补充:

1. **SKILL.md 文件格式** - 技能定义
2. **TASK.md 文件格式** - 任务定义
3. **williw 目录结构** - 全局 skills/tasks
4. **解析器** - 将文件转换为 Rust/TS 类型

---

版本: 1.0.0
维护者: Alou Team
