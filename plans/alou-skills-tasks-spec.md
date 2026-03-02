# Alou Agent Skills & Tasks 规范设计

## 基于人月神话原则的智能体系统架构

> 人月神话核心教义：Brooks法则 - "Adding manpower to a late software project makes it later"。本规范通过赋予Agent自主性和能动性，实现真正的敏捷开发。

---

## 一、完整架构概览

### 1.1 核心组件分布 (已实现)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        alou-desktop/src-tauri/src/agent/                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │  swarm.rs (513行) - 多Agent协作                                        │  │
│  │  - AgentInstance: Agent实例                                          │  │
│  │  - AgentRole: Coordinator/Worker/Specialist                          │  │
│  │  - TaskPlan: 任务分解计划                                             │  │
│  │  - Subtask: 子任务(含依赖关系)                                        │  │
│  │  - SwarmCoordinator: 任务编排与执行                                    │  │
│  │  - SwarmEvent: 事件通知                                               │  │
│  └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │  autonomy.rs (19543字符) - Agent自主性框架                              │  │
│  │  - AutonomyConfig: 自主性配置                                         │  │
│  │  - auto_discover_skills: 自动发现技能                                 │  │
│  │  - auto_select_tools: 自动选择工具                                    │  │
│  │  - auto_decompose_tasks: 自动分解复杂任务                             │  │
│  │  - auto_coordinate_swarm: 自动协调Swarm                              │  │
│  │  - autonomy_level: 0.0(手动) -> 1.0(完全自主)                        │  │
│  │  - ConfirmationRule: 敏感操作确认规则                                 │  │
│  └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │  task.rs - 任务管理                                                   │  │
│  │  executor.rs (51142字符) - 任务执行器                                 │  │
│  │  memory.rs - Agent记忆                                               │  │
│  │  role.rs - Agent角色定义                                             │  │
│  └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                        alou-edge/src/agent/ (Rust)                          │
├─────────────────────────────────────────────────────────────────────────────┤
│  spec.rs: TaskSpec 定义                                                     │
│  cluster_action.rs: Task, TaskStatus, ClusterAction                        │
│  task_orchestrator.rs: 顺序/并行/条件执行                                    │
│  cluster_executor.rs: 集群执行器                                            │
│  batch_processor.rs: 批处理                                                 │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                        alou-desktop/src/ (TypeScript)                       │
├─────────────────────────────────────────────────────────────────────────────┤
│  types/skills.ts: SkillDefinition, SkillMetadata, SkillContext             │
│  types/tasks.ts: Task, ExecutionMode, AgentSwarm, SwarmCoordination       │
│  services/taskQueue.ts: 任务队列                                            │
│  services/agentCoordinatorService.ts: Agent协调服务                          │
│  services/autonomousAgentService.ts: 自主执行服务                            │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 二、已实现的核心功能

### 2.1 Swarm 多Agent协作 (swarm.rs)

```rust
// Agent角色
pub enum AgentRole {
    Coordinator,  // 领导者 - 任务分解
    Worker,      // 执行者 - 执行子任务
    Specialist(String), // 专家角色
}

// Agent状态
pub enum AgentStatus {
    Idle,
    Busy(String),   // 执行中的任务ID
    Offline,
    Error(String),
}

// 任务计划
pub struct TaskPlan {
    pub task_id: String,
    pub subtasks: Vec<Subtask>,
    pub dependencies: TaskDependencyGraph,
    pub assigned_agents: HashMap<String, String>,
}

// 子任务
pub struct Subtask {
    pub id: String,
    pub parent_task_id: String,
    pub description: String,
    pub assigned_agent: Option<String>,
    pub dependencies: Vec<String>,
    pub estimated_duration_secs: u64,
    pub priority: Priority,
    pub status: TaskStatus,
}
```

### 2.2 Agent自主性框架 (autonomy.rs)

```rust
// 自主性配置
pub struct AutonomyConfig {
    pub auto_discover_skills: bool,      // 自动发现技能
    pub auto_select_tools: bool,        // 自动选择工具
    pub auto_decompose_tasks: bool,     // 自动分解任务
    pub auto_coordinate_swarm: bool,     // 自动协调Swarm
    pub max_autonomous_iterations: u32,  // 最大自主迭代
    pub autonomy_level: f32,             // 自主级别 0.0-1.0
}

// 预设模式
impl AutonomyConfig {
    pub fn fully_autonomous() -> Self { ... }  // 完全自主
    pub fn conservative() -> Self { ... }       // 保守模式
}
```

---

## 三、Skills 规范

### 3.1 SKILL.md 格式

```yaml
---
name: skill-name
description: 技能描述
version: 1.0.0
license: MIT

# 渐进式披露级别
pdl:
  level: 1
  reveal_on: [action:execute, intent:analyze]

# 能力声明
capabilities:
  - type: tool
    name: filesystem
    operations: [read, write, list]

# 允许的工具
allowed_tools:
  - bash
  - python
  - filesystem

# 依赖
dependencies:
  runtime: [python>=3.8]
  python: [requests]

# 参数定义
parameters:
  type: object
  properties:
    input:
      type: string
  required: [input]

# 指令
instructions: |
  1. 分析需求
  2. 选择工具
  3. 执行并验证
```

### 3.2 复用现有类型

```typescript
import {
  SkillDefinition,
  SkillMetadata,
  SkillContext,
  SkillResult,
} from '../types/skills';
```

---

## 四、Task 规范

### 4.1 TASK.md 格式

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
      - task: data-validate

  - id: step_2
    name: 生成报告
    type: tool
    tool: generate_report
    depends_on: [step_1]
    timeout_ms: 30000
    retry:
      max_attempts: 3

# Swarm配置
swarm:
  strategy: broadcast | round_robin | capability_based
  min_agents: 2
  max_agents: 10

  roles:
    - name: coordinator
      required: true
    - name: worker
      required: true
      count: 3
```

---

## 五、目录结构

### 5.1 全局 Williw 目录

```
~/.alou/williw/
├── skills/                    # 技能库
│   ├── _core/
│   ├── _community/
│   └── _custom/
├── tasks/                    # 任务库
│   ├── _templates/
│   └── _archived/
├── agents/                   # Agent配置
└── config.yaml              # 全局配置
```

---

## 六、现有实现总结

| 组件 | 位置 | 状态 |
|------|------|------|
| SwarmCoordinator | alou-desktop/src-tauri/src/agent/swarm.rs | ✅ 已实现 |
| AutonomyConfig | alou-desktop/src-tauri/src/agent/autonomy.rs | ✅ 已实现 |
| TaskManager | alou-desktop/src-tauri/src/agent/task.rs | ✅ 已实现 |
| RalphLoopExecutor | alou-desktop/src-tauri/src/agent/executor.rs | ✅ 已实现 |
| TaskOrchestrator | alou-edge/src/agent/task_orchestrator.rs | ✅ 已实现 |
| ClusterExecutor | alou-edge/src/agent/cluster_executor.rs | ✅ 已实现 |

### 人月神话原则实现

1. **渐进式披露 (Progressive Disclosure)**: PDL级别控制信息展示
2. **自治权 (Autonomy)**: autonomy_level 控制Agent自主程度
3. **并行性 (Parallelism)**: TaskPlan + Subtask 支持并行执行
4. **可组合性 (Composability)**: Swarm协调多个Agent协作

---

## 七、需要补充的部分

1. **williw 目录结构**: 创建 `~/.alou/williw/`
2. **SKILL.md 解析器**: 将YAML解析为SkillDefinition
3. **TASK.md 解析器**: 将YAML解析为TaskSpec
4. **技能文件示例**: 创建示例skills目录

---

版本: 1.0.0
维护者: Alou Team
