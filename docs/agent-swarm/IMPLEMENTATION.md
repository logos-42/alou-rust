# Agent Swarm 实现指南

本文档描述如何实现和使用基于《人月神话》原则的 Agent Swarm 系统。

## 架构概述

```
┌─────────────────────────────────────────────────────────────────┐
│                      Agent Swarm System                          │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │   Skill      │  │    Task      │  │      Swarm           │  │
│  │   Registry   │  │   Planner    │  │   Coordinator        │  │
│  │  (技能目录)   │  │  (任务规划)   │  │   (协调调度)          │  │
│  └──────┬───────┘  └──────┬───────┘  └──────────┬───────────┘  │
│         │                 │                      │              │
│         └─────────────────┼──────────────────────┘              │
│                           ▼                                     │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              Task Executor (任务执行器)                   │   │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐    │   │
│  │  │ Local   │  │ Remote  │  │ Skill   │  │  Bash   │    │   │
│  │  │Executor │  │Executor │  │Runner   │  │ Runner  │    │   │
│  │  └─────────┘  └─────────┘  └─────────┘  └─────────┘    │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

## 核心组件

### 1. Skill Registry (技能注册表)

存储和管理可复用的技能。

```rust
use alou_cli::swarm::SkillRegistry;

let registry = SkillRegistry::new();
registry.load_from_dir("~/.config/agents/skills").await?;

// 执行技能
let result = registry.execute("code-analyzer", json!({
    "target": "./src"
})).await?;
```

### 2. Task Planner (任务规划器)

将目标分解为可执行的任务计划。

```rust
use alou_cli::swarm::{TaskPlanner, PlanningContext};

let planner = TaskPlanner::default();
let context = PlanningContext::default();

let plan = planner.plan("重构用户认证模块", &context)?;
println!("预计执行时间: {}ms", plan.estimated_duration_ms);
```

### 3. Swarm Coordinator (Swarm 协调器)

管理任务队列、分配 Agent、监控执行。

```rust
use alou_cli::swarm::{SwarmCoordinator, CoordinatorConfig};

let config = CoordinatorConfig {
    max_concurrent_tasks: 10,
    default_timeout_ms: 300_000,
    ..Default::default()
};

let coordinator = SwarmCoordinator::new(config);
coordinator.start().await?;
```

### 4. Task Executor (任务执行器)

实际执行任务，支持多种执行方式。

```rust
use alou_cli::swarm::{LocalExecutor, TaskExecutor};

let executor = LocalExecutor::default_with_registry(registry);
let result = executor.execute(&task).await?;
```

## 快速开始

### 安装依赖

在 `Cargo.toml` 中添加：

```toml
[dependencies]
alou-cli = { path = "../alou-cli" }
tokio = { version = "1", features = ["full"] }
serde_json = "1.0"
```

### 基本示例

```rust
use alou_cli::swarm::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 创建并启动协调器
    let coordinator = SwarmCoordinator::default();
    coordinator.start().await?;
    
    // 2. 注册 Agent
    let agent = AgentInfo {
        id: "agent-001".to_string(),
        name: "Code Analyzer".to_string(),
        capabilities: vec!["code_analysis".to_string(), "rust".to_string()],
        status: AgentStatus::Idle,
        current_load: 0.0,
        ..Default::default()
    };
    coordinator.register_agent(agent).await?;
    
    // 3. 创建任务
    let steps = vec![TaskStep {
        id: "step-1".to_string(),
        name: "分析代码".to_string(),
        action: "skill".to_string(),
        parameters: Some({
            let mut params = std::collections::HashMap::new();
            params.insert("skill".to_string(), json!("code-analyzer"));
            params.insert("target".to_string(), json!("./src"));
            params
        }),
        ..Default::default()
    }];
    
    let options = TaskCreateOptions {
        execution_mode: Some(ExecutionMode::Sequential),
        priority: Some(TaskPriority::High),
        auto_execute: true,
        ..Default::default()
    };
    
    let task = coordinator.create_task("代码分析", steps, options).await?;
    println!("创建任务: {}", task.id);
    
    // 4. 等待任务完成
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        
        if let Some(t) = coordinator.get_task(&task.id).await {
            match t.status {
                TaskStatus::Completed => {
                    println!("任务完成!");
                    if let Some(result) = t.result {
                        println!("结果: {:?}", result.output);
                    }
                    break;
                }
                TaskStatus::Failed | TaskStatus::Timeout => {
                    println!("任务失败: {:?}", t.error);
                    break;
                }
                _ => println!("任务状态: {:?}", t.status),
            }
        }
    }
    
    // 5. 停止协调器
    coordinator.stop().await?;
    Ok(())
}
```

## 并行执行模式

### 模式 1: 独立任务并行

```rust
// 创建多个独立的并行任务
let task_ids = vec!["task-a", "task-b", "task-c"];
let mut handles = vec![];

for id in &task_ids {
    let coord = coordinator.clone();
    let handle = tokio::spawn(async move {
        let steps = vec![/* ... */];
        let options = TaskCreateOptions {
            execution_mode: Some(ExecutionMode::Parallel),
            ..Default::default()
        };
        coord.create_task(id.to_string(), steps, options).await
    });
    handles.push(handle);
}

// 等待所有任务
for handle in handles {
    handle.await??;
}
```

### 模式 2: 流水线执行

```rust
// 阶段 1: 并行分析
let analyze_tasks = vec![
    create_analysis_task("代码质量"),
    create_analysis_task("安全扫描"),
    create_analysis_task("性能分析"),
];

// 阶段 2: 生成报告（依赖阶段1）
let report_task = create_report_task(&analyze_tasks);

// 阶段 3: 自动修复（可选，依赖报告）
let fix_task = create_fix_task(&report_task);
```

### 模式 3: Map-Reduce

```rust
use alou_cli::swarm::planner::ExecutionStrategy;

// Map: 分片处理
let files = vec!["a.rs", "b.rs", "c.rs"];
let mut map_tasks = vec![];

for file in files {
    map_tasks.push(TaskStep {
        action: "analyze_file".to_string(),
        parameters: Some({
            let mut params = HashMap::new();
            params.insert("file".to_string(), json!(file));
            params
        }),
        parallel: true,
        ..Default::default()
    });
}

// Reduce: 聚合结果
let reduce_task = TaskStep {
    action: "aggregate_results".to_string(),
    depends_on: Some(map_tasks.iter().map(|t| t.id.clone()).collect()),
    ..Default::default()
};
```

## Agent Swarm 协作

### 创建 Swarm

```rust
// 注册多个 Agent
let agents = vec![
    AgentInfo {
        id: "rust-expert".to_string(),
        name: "Rust Expert".to_string(),
        capabilities: vec!["rust".to_string(), "systems".to_string()],
        specialization: Some("systems".to_string()),
        ..Default::default()
    },
    AgentInfo {
        id: "frontend-expert".to_string(),
        name: "Frontend Expert".to_string(),
        capabilities: vec!["typescript".to_string(), "react".to_string()],
        specialization: Some("frontend".to_string()),
        ..Default::default()
    },
    AgentInfo {
        id: "security-expert".to_string(),
        name: "Security Expert".to_string(),
        capabilities: vec!["security".to_string(), "audit".to_string()],
        specialization: Some("security".to_string()),
        ..Default::default()
    },
];

for agent in &agents {
    coordinator.register_agent(agent.clone()).await?;
}

// 创建 Swarm
let swarm = coordinator.create_swarm(
    "Full Stack Team",
    agents.iter().map(|a| a.id.clone()).collect(),
    SwarmCoordination {
        mode: SwarmCoordinationMode::LeaderFollower,
        communication_protocol: "message-bus".to_string(),
        consensus_required: false,
        leader_id: Some("rust-expert".to_string()),
        decision_strategy: Some("leader-decides".to_string()),
    }
).await?;
```

### Swarm 任务分配

```rust
// 根据能力分配任务给 Swarm 成员
let task = Task {
    required_capabilities: Some(vec!["security".to_string()]),
    ..Default::default()
};

// 协调器会自动选择能力匹配的 Agent
let executor_id = coordinator.select_executor(&task).await;
assert_eq!(executor_id, Some("security-expert".to_string()));
```

## 事件监听

```rust
use alou_cli::swarm::coordinator::SwarmEvent;

// 订阅事件
let mut events = coordinator.subscribe_events().await;

while let Some(event) = events.recv().await {
    match event {
        SwarmEvent::TaskStarted { task_id, executor_id, timestamp } => {
            println!("[{}] 任务 {} 开始执行 (执行器: {})", 
                timestamp, task_id, executor_id);
        }
        SwarmEvent::TaskProgress { task_id, progress, message } => {
            println!("[{}] 进度: {}% - {:?}", task_id, progress, message);
        }
        SwarmEvent::TaskCompleted { task_id, result, timestamp } => {
            println!("[{}] 任务 {} 完成!", timestamp, task_id);
        }
        SwarmEvent::AgentStatusChanged { agent_id, old_status, new_status } => {
            println!("Agent {}: {:?} -> {:?}", agent_id, old_status, new_status);
        }
        _ => {}
    }
}
```

## 错误处理

```rust
match coordinator.execute(&task).await {
    Ok(result) => {
        println!("成功: {:?}", result);
    }
    Err(TaskError { code, message, recoverable, .. }) => {
        if recoverable {
            println!("可恢复错误 [{}]: {}", code, message);
            // 重试逻辑
        } else {
            println!("不可恢复错误 [{}]: {}", code, message);
            // 回滚或清理
        }
    }
}
```

## 性能优化

### 1. 调整并发度

```rust
let config = CoordinatorConfig {
    max_concurrent_tasks: 20,  // 根据 CPU 核心数调整
    ..Default::default()
};
```

### 2. 使用批量提交

```rust
// 批量创建任务而不是逐个创建
let tasks = vec![task1, task2, task3];
coordinator.create_tasks_batch(tasks).await?;
```

### 3. 启用检查点

```rust
let options = TaskCreateOptions {
    metadata: Some(TaskMetadata {
        checkpoint_enabled: true,
        ..Default::default()
    }),
    ..Default::default()
};
```

## 调试技巧

### 1. 查看任务队列

```rust
let stats = coordinator.get_queue_stats().await;
println!("队列统计: {:?}", stats);
```

### 2. 任务依赖可视化

```rust
let plan = planner.plan("重构项目", &context)?;
println!("关键路径: {:?}", plan.dependency_graph.critical_path);
println!("可并行组: {:?}", plan.dependency_graph.parallel_groups);
```

### 3. 性能分析

```rust
let task = coordinator.get_task(&task_id).await.unwrap();
if let Some(metrics) = task.result.as_ref().and_then(|r| r.metrics.as_ref()) {
    println!("执行时长: {}ms", metrics.duration);
    println!("Token 使用: {}", metrics.tokens_used.unwrap_or(0));
}
```

## 完整示例

参见：
- [示例代码](./examples/)
- [USAGE_EXAMPLES.md](./USAGE_EXAMPLES.md)
- [Task Plan 示例](./examples/task-plan-example.yaml)

## 进阶主题

### 自定义执行器

```rust
use async_trait::async_trait;

struct MyCustomExecutor {
    id: String,
}

#[async_trait]
impl TaskExecutor for MyCustomExecutor {
    fn id(&self) -> &str { &self.id }
    fn capabilities(&self) -> &[String] { &["custom".to_string()] }
    
    async fn execute(&self, task: &Task) -> Result<TaskResult, TaskError> {
        // 自定义执行逻辑
        Ok(TaskResult {
            success: true,
            output: json!({"custom": true}),
            ..Default::default()
        })
    }
    
    // ... 其他方法
}
```

### 自定义规划策略

```rust
struct MyCustomPlanner;

impl TaskPlanner {
    fn custom_strategy(&self, tasks: &[Task]) -> ExecutionStrategy {
        // 自定义策略逻辑
        if tasks.len() > 10 {
            ExecutionStrategy::Swarm
        } else {
            ExecutionStrategy::Parallel
        }
    }
}
```

## 故障排除

| 问题 | 可能原因 | 解决方案 |
|------|----------|----------|
| 任务长时间等待 | 没有可用的 Agent | 注册更多 Agent 或检查能力匹配 |
| 任务超时 | 任务复杂度过高 | 增加 timeout 或拆分任务 |
| 依赖任务不执行 | 前置任务失败 | 检查前置任务错误或使用 optional 依赖 |
| 内存不足 | 并发任务过多 | 减少 max_concurrent_tasks |

## 相关文档

- [ARCHITECTURE.md](./ARCHITECTURE.md) - 架构设计
- [SKILL_SPEC.md](./SKILL_SPEC.md) - Skill 规范
- [TASK_SPEC.md](./TASK_SPEC.md) - Task 规范
- [USAGE_EXAMPLES.md](./USAGE_EXAMPLES.md) - 使用示例
