# Agent Swarm 使用示例

本文档提供使用 `alou-cli` Swarm 系统的完整示例，帮助开发者快速上手 Agent 协调和任务管理。

## 目录

- [快速开始](#快速开始)
- [创建任务](#创建任务)
- [并行执行示例](#并行执行示例)
- [Agent Swarm 示例](#agent-swarm-示例)
- [事件订阅](#事件订阅)
- [完整示例](#完整示例)

---

## 快速开始

### 1. 启动 Swarm 协调器

```rust
use alou_cli::swarm::{SwarmCoordinator, CoordinatorConfig, LoadBalancingStrategy};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用默认配置创建协调器
    let coordinator = SwarmCoordinator::default();
    
    // 启动协调器
    coordinator.start().await?;
    println!("Swarm 协调器已启动");
    
    // 应用逻辑...
    
    // 停止协调器
    coordinator.stop().await?;
    println!("Swarm 协调器已停止");
    
    Ok(())
}
```

### 2. 自定义配置

```rust
use alou_cli::swarm::{SwarmCoordinator, CoordinatorConfig, LoadBalancingStrategy};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建自定义配置
    let config = CoordinatorConfig {
        max_concurrent_tasks: 20,        // 最大并发任务数
        queue_capacity: 2000,            // 任务队列容量
        default_timeout_ms: 600_000,     // 默认超时时间（10分钟）
        heartbeat_interval_ms: 10_000,   // 心跳间隔（10秒）
        enable_preemption: true,         // 启用任务抢占
        load_balancing_strategy: LoadBalancingStrategy::LeastLoaded,
    };
    
    let coordinator = SwarmCoordinator::new(config);
    coordinator.start().await?;
    
    println!("协调器配置:");
    println!("  - 最大并发任务: 20");
    println!("  - 负载均衡策略: LeastLoaded");
    
    // ... 应用逻辑
    
    coordinator.stop().await?;
    Ok(())
}
```

---

## 创建任务

### 1. 基础任务创建

```rust
use alou_cli::swarm::{
    SwarmCoordinator, TaskStep, TaskCreateOptions, 
    TaskPriority, ExecutionMode
};

async fn create_simple_task(coordinator: &SwarmCoordinator) -> Result<(), String> {
    // 定义任务步骤
    let steps = vec![
        TaskStep {
            id: "step-1".to_string(),
            name: "数据收集".to_string(),
            description: Some("从 API 获取数据".to_string()),
            action: "fetch_data".to_string(),
            parameters: Some({
                let mut params = std::collections::HashMap::new();
                params.insert("url".to_string(), serde_json::json!("https://api.example.com/data"));
                params.insert("method".to_string(), serde_json::json!("GET"));
                params
            }),
            depends_on: None,
            condition: None,
            parallel: false,
            optional: false,
            retry_count: Some(3),
            status: None,
            result: None,
            error: None,
            started_at: None,
            completed_at: None,
        },
        TaskStep {
            id: "step-2".to_string(),
            name: "数据处理".to_string(),
            description: Some("处理收集到的数据".to_string()),
            action: "process_data".to_string(),
            parameters: None,
            depends_on: Some(vec!["step-1".to_string()]),
            condition: None,
            parallel: false,
            optional: false,
            retry_count: Some(2),
            status: None,
            result: None,
            error: None,
            started_at: None,
            completed_at: None,
        },
    ];
    
    // 创建任务选项
    let options = TaskCreateOptions {
        execution_mode: Some(ExecutionMode::Sequential),
        priority: Some(TaskPriority::High),
        max_agents: Some(1),
        timeout: Some(300_000), // 5分钟
        required_capabilities: Some(vec!["data_processing".to_string()]),
        metadata: None,
        auto_execute: true, // 创建后立即执行
    };
    
    // 创建任务
    let task = coordinator
        .create_task("数据同步任务", steps, options)
        .await?;
    
    println!("任务已创建: {}", task.id);
    println!("任务状态: {:?}", task.status);
    
    Ok(())
}
```

### 2. 提交任务并等待结果

```rust
use alou_cli::swarm::{SwarmCoordinator, TaskStep, TaskCreateOptions, TaskStatus};
use tokio::time::{sleep, Duration};

async fn submit_and_wait(
    coordinator: &SwarmCoordinator,
    task_id: &str
) -> Result<(), String> {
    // 手动入队（如果 auto_execute 为 false）
    coordinator.enqueue_task(task_id.to_string()).await?;
    println!("任务 {} 已入队", task_id);
    
    // 等待任务完成
    loop {
        sleep(Duration::from_millis(500)).await;
        
        if let Some(task) = coordinator.get_task(task_id).await {
            match task.status {
                TaskStatus::Completed => {
                    println!("任务完成!");
                    if let Some(result) = task.result {
                        println!("结果: {:?}", result.output);
                        if let Some(metrics) = result.metrics {
                            println!("执行时长: {}ms", metrics.duration);
                        }
                    }
                    break;
                }
                TaskStatus::Failed | TaskStatus::Timeout => {
                    println!("任务失败!");
                    if let Some(error) = task.error {
                        println!("错误: {}", error.message);
                    }
                    break;
                }
                _ => {
                    println!("任务状态: {:?}", task.status);
                }
            }
        }
    }
    
    Ok(())
}
```

### 3. 取消任务

```rust
use alou_cli::swarm::{SwarmCoordinator, TaskStatus};

async fn cancel_running_task(
    coordinator: &SwarmCoordinator,
    task_id: &str
) -> Result<(), String> {
    // 获取任务状态
    if let Some(task) = coordinator.get_task(task_id).await {
        if !task.status.is_terminal() {
            coordinator.cancel_task(task_id, "用户取消").await?;
            println!("任务 {} 已取消", task_id);
        } else {
            println!("任务已完成或已失败，无法取消");
        }
    } else {
        println!("任务不存在");
    }
    
    Ok(())
}
```

---

## 并行执行示例

### 1. 多个独立任务并行执行

```rust
use alou_cli::swarm::{
    SwarmCoordinator, TaskStep, TaskCreateOptions, 
    TaskPriority, ExecutionMode
};
use std::collections::HashMap;

async fn execute_parallel_tasks(coordinator: &SwarmCoordinator) -> Result<(), String> {
    let mut task_ids = Vec::new();
    
    // 创建多个独立的并行任务
    for i in 0..5 {
        let steps = vec![
            TaskStep {
                id: format!("analyze-{}", i),
                name: format!("分析文件 {}", i),
                description: Some(format!("分析第 {} 个文件", i)),
                action: "analyze_file".to_string(),
                parameters: Some({
                    let mut params = HashMap::new();
                    params.insert("file_id".to_string(), serde_json::json!(i));
                    params
                }),
                depends_on: None,
                condition: None,
                parallel: true, // 可以并行执行
                optional: false,
                retry_count: Some(2),
                status: None,
                result: None,
                error: None,
                started_at: None,
                completed_at: None,
            },
        ];
        
        let options = TaskCreateOptions {
            execution_mode: Some(ExecutionMode::Parallel),
            priority: Some(TaskPriority::Medium),
            auto_execute: true,
            ..Default::default()
        };
        
        let task = coordinator
            .create_task(format!("分析任务 {}", i), steps, options)
            .await?;
        
        task_ids.push(task.id);
    }
    
    println!("已创建 {} 个并行任务", task_ids.len());
    
    // 等待所有任务完成
    for task_id in &task_ids {
        wait_for_task(coordinator, task_id).await?;
    }
    
    Ok(())
}

async fn wait_for_task(
    coordinator: &SwarmCoordinator,
    task_id: &str
) -> Result<(), String> {
    use tokio::time::{sleep, Duration};
    
    loop {
        if let Some(task) = coordinator.get_task(task_id).await {
            if task.status.is_terminal() {
                println!("任务 {} 完成: {:?}", task_id, task.status);
                break;
            }
        }
        sleep(Duration::from_millis(100)).await;
    }
    
    Ok(())
}
```

### 2. 批量任务处理

```rust
use alou_cli::swarm::{
    SwarmCoordinator, Task, TaskStep, TaskCreateOptions, TaskStatus
};

async fn batch_processing(
    coordinator: &SwarmCoordinator,
    items: Vec<String>
) -> Result<Vec<Task>, String> {
    let mut tasks = Vec::new();
    
    // 为每个项目创建任务
    for (idx, item) in items.iter().enumerate() {
        let steps = vec![
            TaskStep {
                id: format!("process-{}", idx),
                name: format!("处理项目: {}", item),
                description: Some(format!("处理项目 {}", item)),
                action: "process_item".to_string(),
                parameters: Some({
                    let mut params = std::collections::HashMap::new();
                    params.insert("item".to_string(), serde_json::json!(item));
                    params.insert("index".to_string(), serde_json::json!(idx));
                    params
                }),
                depends_on: None,
                condition: None,
                parallel: true,
                optional: false,
                retry_count: Some(3),
                status: None,
                result: None,
                error: None,
                started_at: None,
                completed_at: None,
            },
        ];
        
        let options = TaskCreateOptions {
            execution_mode: Some(ExecutionMode::Parallel),
            priority: Some(TaskPriority::Medium),
            auto_execute: true,
            ..Default::default()
        };
        
        let task = coordinator
            .create_task(format!("批量处理任务 {}", idx), steps, options)
            .await?;
        
        tasks.push(task);
    }
    
    println!("批量创建了 {} 个任务", tasks.len());
    
    // 等待所有任务完成
    let mut completed = 0;
    let mut failed = 0;
    
    loop {
        let all_terminal = tasks.iter().all(|t| {
            if let Some(task) = coordinator.get_task(&t.id).as_ref() {
                task.status.is_terminal()
            } else {
                false
            }
        });
        
        if all_terminal {
            break;
        }
        
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    // 统计结果
    for task in &tasks {
        if let Some(t) = coordinator.get_task(&task.id).await {
            match t.status {
                TaskStatus::Completed => completed += 1,
                TaskStatus::Failed => failed += 1,
                _ => {}
            }
        }
    }
    
    println!("批量处理完成: {} 成功, {} 失败", completed, failed);
    
    Ok(tasks)
}
```

---

## Agent Swarm 示例

### 1. 注册 Agent

```rust
use alou_cli::swarm::{SwarmCoordinator, AgentInfo, AgentStatus};

async fn register_agents(coordinator: &SwarmCoordinator) -> Result<(), String> {
    // 注册代码分析 Agent
    let code_analyzer = AgentInfo {
        id: "agent-code-analyzer".to_string(),
        name: "代码分析器".to_string(),
        display_name: Some("Code Analyzer".to_string()),
        description: Some("专门分析代码质量和结构".to_string()),
        capabilities: vec![
            "code_analysis".to_string(),
            "static_analysis".to_string(),
            "security_scan".to_string(),
        ],
        status: AgentStatus::Idle,
        current_load: 0.0,
        specialization: Some("rust".to_string()),
        performance_score: Some(0.95),
        current_task_id: None,
        completed_tasks: 0,
        failed_tasks: 0,
    };
    
    // 注册文档生成 Agent
    let doc_generator = AgentInfo {
        id: "agent-doc-generator".to_string(),
        name: "文档生成器".to_string(),
        display_name: Some("Doc Generator".to_string()),
        description: Some("根据代码生成文档".to_string()),
        capabilities: vec![
            "documentation".to_string(),
            "markdown".to_string(),
            "api_doc".to_string(),
        ],
        status: AgentStatus::Idle,
        current_load: 0.0,
        specialization: Some("documentation".to_string()),
        performance_score: Some(0.90),
        current_task_id: None,
        completed_tasks: 0,
        failed_tasks: 0,
    };
    
    // 注册测试 Agent
    let test_agent = AgentInfo {
        id: "agent-test-runner".to_string(),
        name: "测试执行器".to_string(),
        display_name: Some("Test Runner".to_string()),
        description: Some("执行自动化测试".to_string()),
        capabilities: vec![
            "testing".to_string(),
            "unit_test".to_string(),
            "integration_test".to_string(),
        ],
        status: AgentStatus::Idle,
        current_load: 0.0,
        specialization: Some("testing".to_string()),
        performance_score: Some(0.88),
        current_task_id: None,
        completed_tasks: 0,
        failed_tasks: 0,
    };
    
    coordinator.register_agent(code_analyzer).await?;
    coordinator.register_agent(doc_generator).await?;
    coordinator.register_agent(test_agent).await?;
    
    println!("已注册 3 个 Agent");
    
    Ok(())
}
```

### 2. 创建 Swarm

```rust
use alou_cli::swarm::{
    SwarmCoordinator, AgentSwarm, SwarmCoordination, 
    SwarmCoordinationMode
};

async fn create_development_swarm(
    coordinator: &SwarmCoordinator
) -> Result<AgentSwarm, String> {
    // 定义 Swarm 成员
    let member_ids = vec![
        "agent-code-analyzer".to_string(),
        "agent-doc-generator".to_string(),
        "agent-test-runner".to_string(),
    ];
    
    // 配置 Swarm 协作模式
    let coordination = SwarmCoordination {
        mode: SwarmCoordinationMode::LeaderFollower,
        communication_protocol: "message-bus".to_string(),
        consensus_required: true,
        leader_id: Some("agent-code-analyzer".to_string()),
        decision_strategy: Some("leader-decides".to_string()),
    };
    
    // 创建 Swarm
    let swarm = coordinator
        .create_swarm(
            "开发团队 Swarm",
            member_ids,
            coordination,
        )
        .await?;
    
    println!("Swarm 创建成功!");
    println!("  ID: {}", swarm.id);
    println!("  名称: {}", swarm.name);
    println!("  成员数: {}", swarm.members.len());
    println!("  协作模式: {:?}", swarm.coordination.mode);
    
    Ok(swarm)
}
```

### 3. 分配任务给 Swarm

```rust
use alou_cli::swarm::{
    SwarmCoordinator, TaskStep, TaskCreateOptions, 
    ExecutionMode, TaskPriority
};

async fn assign_task_to_swarm(
    coordinator: &SwarmCoordinator,
    swarm_id: &str
) -> Result<(), String> {
    // 创建一个需要 Swarm 协作的复杂任务
    let steps = vec![
        // 代码分析步骤
        TaskStep {
            id: "analyze".to_string(),
            name: "代码分析".to_string(),
            description: Some("分析代码结构和质量".to_string()),
            action: "analyze_code".to_string(),
            parameters: Some({
                let mut params = std::collections::HashMap::new();
                params.insert("target".to_string(), serde_json::json!("src/main.rs"));
                params
            }),
            depends_on: None,
            condition: None,
            parallel: false,
            optional: false,
            retry_count: Some(2),
            status: None,
            result: None,
            error: None,
            started_at: None,
            completed_at: None,
        },
        // 测试步骤（依赖于分析）
        TaskStep {
            id: "test".to_string(),
            name: "运行测试".to_string(),
            description: Some("执行单元测试和集成测试".to_string()),
            action: "run_tests".to_string(),
            parameters: None,
            depends_on: Some(vec!["analyze".to_string()]),
            condition: None,
            parallel: false,
            optional: false,
            retry_count: Some(3),
            status: None,
            result: None,
            error: None,
            started_at: None,
            completed_at: None,
        },
        // 文档生成步骤（依赖于分析，可与测试并行）
        TaskStep {
            id: "docs".to_string(),
            name: "生成文档".to_string(),
            description: Some("根据分析结果生成文档".to_string()),
            action: "generate_docs".to_string(),
            parameters: None,
            depends_on: Some(vec!["analyze".to_string()]),
            condition: None,
            parallel: true, // 可以与测试并行
            optional: true,  // 文档生成失败不影响整体
            retry_count: Some(1),
            status: None,
            result: None,
            error: None,
            started_at: None,
            completed_at: None,
        },
    ];
    
    let options = TaskCreateOptions {
        execution_mode: Some(ExecutionMode::Swarm),
        priority: Some(TaskPriority::High),
        max_agents: Some(3), // Swarm 中的 3 个 Agent
        required_capabilities: Some(vec![
            "code_analysis".to_string(),
            "testing".to_string(),
            "documentation".to_string(),
        ]),
        auto_execute: true,
        ..Default::default()
    };
    
    let task = coordinator
        .create_task("完整代码审查", steps, options)
        .await?;
    
    println!("任务 {} 已分配给 Swarm {}", task.id, swarm_id);
    
    Ok(())
}
```

### 4. 解散 Swarm

```rust
use alou_cli::swarm::SwarmCoordinator;

async fn disband_swarm(
    coordinator: &SwarmCoordinator,
    swarm_id: &str
) -> Result<(), String> {
    coordinator.disband_swarm(swarm_id, "任务完成，解散 Swarm").await?;
    println!("Swarm {} 已解散", swarm_id);
    Ok(())
}
```

---

## 事件订阅

### 1. 基本事件监听

```rust
use alou_cli::swarm::{SwarmCoordinator, SwarmEvent};
use tokio::sync::mpsc;

async fn subscribe_to_events(coordinator: &SwarmCoordinator) -> Result<(), String> {
    // 订阅事件
    let mut event_receiver = coordinator.subscribe_events().await;
    
    println!("开始监听 Swarm 事件...");
    
    // 在单独的任务中处理事件
    tokio::spawn(async move {
        while let Some(event) = event_receiver.recv().await {
            match &event {
                SwarmEvent::TaskCreated { task_id, timestamp } => {
                    println!("[{}] 任务创建: {}", timestamp, task_id);
                }
                SwarmEvent::TaskStarted { task_id, executor_id, timestamp } => {
                    println!("[{}] 任务开始: {} (执行器: {})", 
                        timestamp, task_id, executor_id);
                }
                SwarmEvent::TaskProgress { task_id, progress, message } => {
                    let msg = message.as_deref().unwrap_or("");
                    println!("[进度] {}: {:.1}% - {}", task_id, progress, msg);
                }
                SwarmEvent::TaskCompleted { task_id, result, timestamp } => {
                    println!("[{}] 任务完成: {} - {}", 
                        timestamp, task_id, 
                        result.summary.as_deref().unwrap_or("完成"));
                }
                SwarmEvent::TaskFailed { task_id, error, timestamp } => {
                    println!("[{}] 任务失败: {} - {}", 
                        timestamp, task_id, error.message);
                }
                SwarmEvent::AgentJoined { agent_id, agent_info } => {
                    println!("Agent 加入: {} ({:?})", 
                        agent_id, agent_info.status);
                }
                SwarmEvent::AgentLeft { agent_id, reason } => {
                    println!("Agent 离开: {} - {}", agent_id, reason);
                }
                SwarmEvent::SwarmFormed { swarm_id, member_ids } => {
                    println!("Swarm 形成: {} (成员: {:?})", swarm_id, member_ids);
                }
                SwarmEvent::SwarmDisbanded { swarm_id, reason } => {
                    println!("Swarm 解散: {} - {}", swarm_id, reason);
                }
                _ => {
                    println!("收到事件: {:?}", event);
                }
            }
        }
    });
    
    Ok(())
}
```

### 2. 进度追踪器

```rust
use alou_cli::swarm::{SwarmCoordinator, SwarmEvent, TaskStatus};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

#[derive(Debug, Clone)]
struct TaskProgress {
    task_id: String,
    current_progress: f32,
    status: TaskStatus,
}

async fn track_task_progress(
    coordinator: &SwarmCoordinator,
    target_task_ids: Vec<String>
) -> Result<(), String> {
    let progress_map: Arc<RwLock<HashMap<String, TaskProgress>>> = 
        Arc::new(RwLock::new(HashMap::new()));
    
    // 初始化进度追踪
    {
        let mut map = progress_map.write().await;
        for task_id in &target_task_ids {
            map.insert(task_id.clone(), TaskProgress {
                task_id: task_id.clone(),
                current_progress: 0.0,
                status: TaskStatus::Pending,
            });
        }
    }
    
    let mut event_receiver = coordinator.subscribe_events().await;
    let map_clone = progress_map.clone();
    let target_ids = target_task_ids.clone();
    
    // 处理事件
    tokio::spawn(async move {
        while let Some(event) = event_receiver.recv().await {
            match event {
                SwarmEvent::TaskProgress { task_id, progress, .. } => {
                    if target_ids.contains(&task_id) {
                        let mut map = map_clone.write().await;
                        if let Some(p) = map.get_mut(&task_id) {
                            p.current_progress = progress;
                        }
                        
                        // 显示进度条
                        let bar_length = 30;
                        let filled = ((progress / 100.0) * bar_length as f32) as usize;
                        let bar: String = std::iter::repeat("█")
                            .take(filled)
                            .chain(std::iter::repeat("░")
                                .take(bar_length - filled))
                            .collect();
                        
                        println!("{} [{}] {:.1}%", task_id, bar, progress);
                    }
                }
                SwarmEvent::TaskCompleted { task_id, .. } => {
                    if target_ids.contains(&task_id) {
                        println!("✓ 任务 {} 完成!", task_id);
                    }
                }
                SwarmEvent::TaskFailed { task_id, error, .. } => {
                    if target_ids.contains(&task_id) {
                        println!("✗ 任务 {} 失败: {}", task_id, error.message);
                    }
                }
                _ => {}
            }
        }
    });
    
    Ok(())
}
```

### 3. 实时状态监控

```rust
use alou_cli::swarm::{SwarmCoordinator, TaskStatus};
use tokio::time::{interval, Duration};

async fn monitor_system_status(coordinator: &SwarmCoordinator) -> Result<(), String> {
    let mut ticker = interval(Duration::from_secs(5));
    
    println!("开始系统状态监控...");
    println!("{:-^50}", "");
    
    loop {
        ticker.tick().await;
        
        // 获取所有任务
        let all_tasks = coordinator.list_tasks().await;
        
        // 按状态统计
        let mut stats = HashMap::new();
        for status in [
            TaskStatus::Pending,
            TaskStatus::Queued,
            TaskStatus::InProgress,
            TaskStatus::Completed,
            TaskStatus::Failed,
        ] {
            let count = all_tasks.iter()
                .filter(|t| t.status == status)
                .count();
            stats.insert(format!("{:?}", status), count);
        }
        
        // 获取队列统计
        let queue_stats = coordinator.get_queue_stats().await;
        
        // 获取所有 Agent
        let agents = coordinator.list_agents().await;
        let online_agents = agents.iter()
            .filter(|a| a.status == AgentStatus::Online || a.status == AgentStatus::Idle)
            .count();
        
        // 显示状态
        println!("\n[{}] 系统状态", chrono::Local::now().format("%H:%M:%S"));
        println!("  任务统计:");
        for (status, count) in &stats {
            println!("    {}: {}", status, count);
        }
        println!("  队列长度: {}", queue_stats.total);
        println!("  Agent: {}/{}", online_agents, agents.len());
        println!("{:-^50}", "");
    }
}
```

---

## 完整示例

### 综合示例：代码审查工作流

```rust
use alou_cli::swarm::{
    SwarmCoordinator, CoordinatorConfig, LoadBalancingStrategy,
    AgentInfo, AgentStatus, AgentSwarm, SwarmCoordination, SwarmCoordinationMode,
    Task, TaskStep, TaskCreateOptions, TaskPriority, ExecutionMode,
    SwarmEvent, TaskStatus
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 初始化协调器
    println!("=== 初始化 Swarm 协调器 ===");
    let config = CoordinatorConfig {
        max_concurrent_tasks: 10,
        queue_capacity: 100,
        default_timeout_ms: 300_000,
        heartbeat_interval_ms: 5_000,
        enable_preemption: false,
        load_balancing_strategy: LoadBalancingStrategy::CapabilityMatch,
    };
    
    let coordinator = SwarmCoordinator::new(config);
    coordinator.start().await?;
    println!("协调器已启动\n");
    
    // 2. 注册 Agents
    println!("=== 注册 Agents ===");
    let agents = vec![
        AgentInfo {
            id: "analyzer-1".to_string(),
            name: "代码分析器".to_string(),
            display_name: Some("Code Analyzer".to_string()),
            description: Some("分析代码质量和安全性".to_string()),
            capabilities: vec!["code_analysis".to_string(), "security_scan".to_string()],
            status: AgentStatus::Idle,
            current_load: 0.0,
            specialization: Some("rust".to_string()),
            performance_score: Some(0.95),
            current_task_id: None,
            completed_tasks: 0,
            failed_tasks: 0,
        },
        AgentInfo {
            id: "tester-1".to_string(),
            name: "测试执行器".to_string(),
            display_name: Some("Test Runner".to_string()),
            description: Some("执行各类测试".to_string()),
            capabilities: vec!["testing".to_string(), "unit_test".to_string()],
            status: AgentStatus::Idle,
            current_load: 0.0,
            specialization: Some("testing".to_string()),
            performance_score: Some(0.90),
            current_task_id: None,
            completed_tasks: 0,
            failed_tasks: 0,
        },
        AgentInfo {
            id: "docs-1".to_string(),
            name: "文档生成器".to_string(),
            display_name: Some("Doc Generator".to_string()),
            description: Some("生成项目文档".to_string()),
            capabilities: vec!["documentation".to_string(), "markdown".to_string()],
            status: AgentStatus::Idle,
            current_load: 0.0,
            specialization: Some("documentation".to_string()),
            performance_score: Some(0.88),
            current_task_id: None,
            completed_tasks: 0,
            failed_tasks: 0,
        },
    ];
    
    for agent in &agents {
        coordinator.register_agent(agent.clone()).await?;
        println!("  注册: {} ({:?})", agent.name, agent.capabilities);
    }
    println!();
    
    // 3. 创建 Swarm
    println!("=== 创建代码审查 Swarm ===");
    let swarm = coordinator.create_swarm(
        "代码审查团队",
        agents.iter().map(|a| a.id.clone()).collect(),
        SwarmCoordination {
            mode: SwarmCoordinationMode::LeaderFollower,
            communication_protocol: "message-bus".to_string(),
            consensus_required: false,
            leader_id: Some("analyzer-1".to_string()),
            decision_strategy: Some("capability-match".to_string()),
        },
    ).await?;
    println!("Swarm 创建成功: {}\n", swarm.id);
    
    // 4. 启动事件监听
    println!("=== 启动事件监听 ===");
    let mut event_receiver = coordinator.subscribe_events().await;
    
    tokio::spawn(async move {
        while let Some(event) = event_receiver.recv().await {
            match event {
                SwarmEvent::TaskProgress { task_id, progress, message } => {
                    let msg = message.as_deref().unwrap_or("执行中...");
                    println!("  [{}] {:.0}% - {}", task_id, progress, msg);
                }
                SwarmEvent::TaskCompleted { task_id, result, .. } => {
                    println!("  ✓ 任务完成: {}", task_id);
                    if let Some(summary) = result.summary {
                        println!("    摘要: {}", summary);
                    }
                }
                SwarmEvent::TaskFailed { task_id, error, .. } => {
                    println!("  ✗ 任务失败: {} - {}", task_id, error.message);
                }
                _ => {}
            }
        }
    });
    
    // 5. 创建复杂任务
    println!("\n=== 创建代码审查任务 ===");
    let steps = vec![
        TaskStep {
            id: "security-scan".to_string(),
            name: "安全扫描".to_string(),
            description: Some("扫描潜在安全漏洞".to_string()),
            action: "security_scan".to_string(),
            parameters: Some({
                let mut p = HashMap::new();
                p.insert("severity".to_string(), serde_json::json!("high"));
                p
            }),
            depends_on: None,
            condition: None,
            parallel: false,
            optional: false,
            retry_count: Some(2),
            status: None,
            result: None,
            error: None,
            started_at: None,
            completed_at: None,
        },
        TaskStep {
            id: "unit-tests".to_string(),
            name: "单元测试".to_string(),
            description: Some("运行单元测试套件".to_string()),
            action: "run_tests".to_string(),
            parameters: Some({
                let mut p = HashMap::new();
                p.insert("type".to_string(), serde_json::json!("unit"));
                p
            }),
            depends_on: None,
            condition: None,
            parallel: true, // 可以与安全扫描并行
            optional: false,
            retry_count: Some(1),
            status: None,
            result: None,
            error: None,
            started_at: None,
            completed_at: None,
        },
        TaskStep {
            id: "generate-report".to_string(),
            name: "生成报告".to_string(),
            description: Some("整合结果生成审查报告".to_string()),
            action: "generate_report".to_string(),
            parameters: None,
            depends_on: Some(vec!["security-scan".to_string(), "unit-tests".to_string()]),
            condition: None,
            parallel: false,
            optional: true,
            retry_count: Some(1),
            status: None,
            result: None,
            error: None,
            started_at: None,
            completed_at: None,
        },
    ];
    
    let options = TaskCreateOptions {
        execution_mode: Some(ExecutionMode::Swarm),
        priority: Some(TaskPriority::High),
        max_agents: Some(3),
        timeout: Some(600_000),
        required_capabilities: Some(vec![
            "code_analysis".to_string(),
            "testing".to_string(),
            "documentation".to_string(),
        ]),
        metadata: None,
        auto_execute: true,
    };
    
    let task = coordinator
        .create_task("代码审查任务", steps, options)
        .await?;
    
    println!("任务创建成功: {}", task.id);
    println!("执行模式: {:?}", task.execution_mode);
    println!("优先级: {:?}\n", task.priority);
    
    // 6. 等待任务完成
    println!("=== 等待任务执行 ===");
    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        if let Some(t) = coordinator.get_task(&task.id).await {
            if t.status.is_terminal() {
                println!("\n任务最终状态: {:?}", t.status);
                
                if let Some(result) = t.result {
                    println!("成功: {}", result.success);
                    if let Some(metrics) = result.metrics {
                        println!("执行时长: {}ms", metrics.duration);
                        println!("完成步骤: {}", metrics.steps_completed);
                    }
                }
                
                break;
            }
        }
    }
    
    // 7. 清理
    println!("\n=== 清理 ===");
    coordinator.disband_swarm(&swarm.id, "任务完成").await?;
    println!("Swarm 已解散");
    
    coordinator.stop().await?;
    println!("协调器已停止");
    
    Ok(())
}
```

---

## 更多资源

- [架构文档](./ARCHITECTURE.md) - 了解 Swarm 系统的整体架构
- [任务规范](./TASK_SPEC.md) - 详细了解 Task 数据结构和规范
- [技能规范](./SKILL_SPEC.md) - 了解 Agent Skill 系统

## 常见问题

### Q: 如何设置任务超时？

```rust
let options = TaskCreateOptions {
    timeout: Some(300_000), // 5分钟，单位为毫秒
    ..Default::default()
};
```

### Q: 如何处理任务失败？

```rust
// 检查任务状态
if let Some(task) = coordinator.get_task(task_id).await {
    if task.status == TaskStatus::Failed {
        if let Some(error) = task.error {
            if error.recoverable {
                // 可以重试
                coordinator.enqueue_task(task_id.to_string()).await?;
            }
        }
    }
}
```

### Q: 如何限制并发任务数？

在 `CoordinatorConfig` 中设置 `max_concurrent_tasks`：

```rust
let config = CoordinatorConfig {
    max_concurrent_tasks: 5, // 最多同时执行 5 个任务
    ..Default::default()
};
```
