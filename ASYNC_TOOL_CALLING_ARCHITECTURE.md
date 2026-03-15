# 异步工具调用架构 - 完整实施文档

## 完成日期
2026-03-15

## 目标
实现异步工具调用系统，让 AI 可以在工具执行期间继续做其他事情。

---

## 架构设计

### 核心组件

```
RalphLoopExecutor
  ├─ ai_client: Arc<AiClient>
  ├─ task_manager: Arc<TaskManager>
  ├─ tool_bridge: Arc<ToolBridge>
  └─ async_tool_manager: Option<Arc<AsyncToolManager>> ← 新增

AsyncToolManager
  ├─ tasks: Arc<RwLock<HashMap<task_id, AsyncToolTask>>>
  ├─ tool_executor: Arc<dyn ToolExecutor>
  └─ polling_tasks: Arc<Mutex<Vec<task_id>>>
       └─ 后台轮询 (tokio::spawn)
```

---

## 工具分类

### 同步工具（立即执行并等待）
- filesystem, bash, search, git_helper, agent_document

### 异步工具（创建任务，后台轮询）
- generate_image, generate_video, generate_music, generate_audio

---

## 使用方式

### 1. 创建执行器

```rust
let async_tool_manager = Arc::new(AsyncToolManager::new(tool_executor));

let executor = RalphLoopExecutor::new(...)
    .with_app_handle(app_handle)
    .with_async_tool_manager(async_tool_manager);
```

### 2. 创建异步任务

```rust
let task_id = async_tool_manager.create_task(
    "generate_image".to_string(),
    args,
    true,  // requires_polling = true
).await;

// AI 可以继续做其他事情...
```

### 3. 查询状态

```rust
if let Some(task) = async_tool_manager.get_task_status(&task_id).await {
    match task.status {
        AsyncToolStatus::Completed => println!("完成"),
        AsyncToolStatus::Running => println!("{}%", task.progress as u32),
        AsyncToolStatus::Failed => println!("失败"),
        _ => {}
    }
}
```

---

## 核心文件

| 文件 | 说明 |
|------|------|
| async_tool_manager.rs | 异步工具管理器 (~280 行) |
| executor.rs | 已集成异步工具调用 |
| providers/registry.rs | 6 个 Provider 注册 |

---

## 提交历史（9 次）

```
a1fb190 feat: 在 executor.rs 中集成异步工具调用管理器
15b7e2c feat: 实现异步工具调用管理器
412b3c7 fix: 在 Provider Registry 中添加海绵音乐注册
bb6da40 docs: 添加媒体 API 官方文档参考
2b6d62e feat: 添加海绵音乐 Provider 支持
dbe3f1a feat: 优化媒体 API 实现
007b5a8 chore: 完整存档媒体 API 工具集成项目
5f73b52 feat: 添加媒体 Provider 实现和基础设施
9948f6e feat: 完成媒体 API 工具集成
```

---

实施状态：✅ 核心架构完成
