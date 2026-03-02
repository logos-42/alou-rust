# Agent Swarm 系统

基于《人月神话》原则的 AI Agent 分布式协作系统

> **"向进度落后的项目增加人手，只会使进度更加落后"**  
> —— Fred Brooks, 《人月神话》

## 核心理念

本系统通过以下方式解决"人月神话"的困境：

1. **减少沟通成本** - 清晰的任务边界和接口契约
2. **概念完整性** - 统一的架构设计和规范
3. **外科手术式团队** - Chief Agent + 多个独立 Task Agent
4. **模块化设计** - 可复用的 Skill 和独立的 Task

## 系统架构

```
┌─────────────────────────────────────────────────────────────────┐
│                      Agent Swarm System                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   Chief Agent (协调者)                                           │
│        │                                                         │
│        ├──▶ Task Planner (任务规划) ──▶ 生成执行计划              │
│        │                                                         │
│        ├──▶ Swarm Coordinator (Swarm 协调) ──▶ 调度任务          │
│        │              │                                          │
│        │              ├──▶ Local Executor (本地执行)              │
│        │              ├──▶ Remote Executor (远程执行)             │
│        │              └──▶ Skill Runner (Skill 执行)             │
│        │                                                         │
│        └──▶ Skill Registry (技能注册表) ──▶ 管理可复用能力        │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## 快速开始

```rust
use alou_cli::swarm::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 启动 Swarm 协调器
    let coordinator = SwarmCoordinator::default();
    coordinator.start().await?;
    
    // 2. 创建并行任务
    let steps = vec![
        TaskStep {
            id: "analyze".to_string(),
            name: "代码分析".to_string(),
            action: "skill".to_string(),
            parameters: Some({
                let mut params = HashMap::new();
                params.insert("skill".to_string(), serde_json::json!("code-analyzer"));
                params.insert("target".to_string(), serde_json::json!("./src"));
                params
            }),
            ..Default::default()
        }
    ];
    
    let options = TaskCreateOptions {
        execution_mode: Some(ExecutionMode::Parallel),
        priority: Some(TaskPriority::High),
        auto_execute: true,
        ..Default::default()
    };
    
    let task = coordinator.create_task("代码分析", steps, options).await?;
    
    // 3. 等待完成
    loop {
        if let Some(t) = coordinator.get_task(&task.id).await {
            if t.status.is_terminal() {
                println!("任务完成: {:?}", t.result);
                break;
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
    
    coordinator.stop().await?;
    Ok(())
}
```

## 核心概念

### Skill (技能)

**Skill** 是全局可复用的能力单元，存储在 `~/.config/agents/skills/`。

```yaml
# SKILL.md
---
name: code-analyzer
version: 1.0.0
description: "代码分析 Skill"
category: development
risk_level: medium
allowed_tools: [bash, python, file_read]
---

## 使用

```python
python3 @scripts/main.py --input '{"target": "./src"}'
```
```

### Task (任务)

**Task** 是一次性的执行单元，支持并行和分布式执行。

```yaml
# task.yaml
version: "2.0"
task:
  id: "task-001"
  name: "代码分析"
  skill:
    name: "code-analyzer"
  input:
    params:
      target: "./src"
  config:
    timeout: 300
    retries: 2
    parallelizable: true
```

### Agent Swarm

**Swarm** 是一组协作的 Agent，共同完成复杂任务。

```rust
// 创建 Swarm
let swarm = coordinator.create_swarm(
    "Dev Team",
    vec!["rust-expert", "frontend-expert", "security-expert"],
    SwarmCoordination {
        mode: SwarmCoordinationMode::LeaderFollower,
        leader_id: Some("rust-expert".to_string()),
        ..Default::default()
    }
).await?;
```

## 文档索引

| 文档 | 内容 |
|------|------|
| [ARCHITECTURE.md](./ARCHITECTURE.md) | 系统架构设计，基于人月神话的原则 |
| [SKILL_SPEC.md](./SKILL_SPEC.md) | Skill 规范定义（v2.0） |
| [TASK_SPEC.md](./TASK_SPEC.md) | Task 规范定义（v2.0） |
| [IMPLEMENTATION.md](./IMPLEMENTATION.md) | 实现指南和 API 文档 |
| [USAGE_EXAMPLES.md](./USAGE_EXAMPLES.md) | 详细使用示例 |

## 示例

### 并行代码审查

```yaml
# examples/task-plan-example.yaml
plan:
  objective: "全面代码审查与重构"
  tasks:
    - id: "analyze-code-quality"
      name: "代码质量分析"
      skill: { name: "code-analyzer" }
      config:
        timeout: 180
        parallelizable: true

    - id: "analyze-security"
      name: "安全扫描"
      skill: { name: "security-scanner" }
      config:
        timeout: 300
        parallelizable: true

    - id: "generate-report"
      name: "生成报告"
      skill: { name: "report-generator" }
      dependsOn:
        - "analyze-code-quality"
        - "analyze-security"
      config:
        parallelizable: false
```

### Agent 协作

```
阶段 1: 并行分析
├─ Agent A: 代码质量分析
├─ Agent B: 安全漏洞扫描
└─ Agent C: 性能分析

阶段 2: 报告生成（依赖阶段1）
└─ Agent D: 整合结果

阶段 3: 自动修复（可选）
└─ Agent E: 应用修复
```

## 特性

- ✅ **并行执行** - 无依赖任务同时执行
- ✅ **依赖管理** - 自动解析任务依赖关系
- ✅ **负载均衡** - 智能分配任务给 Agent
- ✅ **进度追踪** - 实时任务状态和进度
- ✅ **事件驱动** - 完整的任务生命周期事件
- ✅ **错误恢复** - 可配置的重试和回退策略
- ✅ **资源限制** - 内存、CPU、超时控制
- ✅ **检查点** - 支持任务恢复和断点续传

## 安装

### 从源码构建

```bash
cd alou-cli
cargo build --release
```

### 添加依赖

```toml
[dependencies]
alou-cli = { path = "../alou-cli" }
```

## 目录结构

```
docs/agent-swarm/
├── README.md                 # 本文件
├── ARCHITECTURE.md           # 架构设计
├── SKILL_SPEC.md             # Skill 规范
├── TASK_SPEC.md              # Task 规范
├── IMPLEMENTATION.md         # 实现指南
├── USAGE_EXAMPLES.md         # 使用示例
└── examples/
    └── task-plan-example.yaml # 示例 Task Plan

alou-cli/src/swarm/
├── mod.rs                    # 模块导出
├── types.rs                  # 核心类型定义
├── coordinator.rs            # Swarm 协调器
├── executor.rs               # 任务执行器
├── planner.rs                # 任务规划器
└── skill_registry.rs         # 技能注册表

test-skills/
└── code-refactor/            # 示例 Skill
    ├── SKILL.md
    └── scripts/
        ├── main.py
        └── lib/
            └── utils.py
```

## 贡献

欢迎贡献！请遵循以下步骤：

1. 阅读 [SKILL_SPEC.md](./SKILL_SPEC.md) 和 [TASK_SPEC.md](./TASK_SPEC.md)
2. 确保代码符合项目风格
3. 添加测试用例
4. 提交 PR

## 许可证

MIT License - 详见项目根目录 LICENSE 文件

## 引用

> Brooks, F. P. (1975). *The Mythical Man-Month: Essays on Software Engineering*. Addison-Wesley.
