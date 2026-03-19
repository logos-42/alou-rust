# 媒体生成闭环问题分析报告

## ✅ 已修复：本地存档集成

### 修改内容

**修改文件**: `alou-desktop/src-tauri/src/media_api.rs`

**实现方案**: 在 Media API 层集成自动存档功能

1. **新增 `MediaArchiveManager` 结构** (第 20-95 行):
   - 存档目录：`~/.config/alou/media_archives/`
   - 方法：`archive()`, `get()`, `list()`
   - 使用 JSON 文件格式存储存档

2. **修改所有媒体生成函数**:
   - `execute_generate_image()` - 图片生成后自动存档
   - `execute_generate_audio()` - 音频生成后自动存档  
   - `execute_generate_video()` - 视频完成后自动存档
   - `execute_get_video_status()` - 查询状态时如果完成也存档

3. **新增 API 路由**:
   - `GET /api/media/archive/list` - 列出所有存档（最多 100 条）
   - `GET /api/media/archive/get/:archive_id` - 获取存档详情

4. **存档 ID 格式**:
   - 图片：`media:image:{timestamp}`
   - 音频：`media:audio:{timestamp}`
   - 视频：`media:video:{timestamp}` 和 `media:video:task:{task_id}`

### 返回结果示例

```json
{
  "success": true,
  "media_type": "image",
  "provider": "google",
  "file_path": "/path/to/image.png",
  "url": "...",
  "metadata": {...},
  "archive": {
    "archive_id": "media:image:1710763200",
    "status": "saved"
  }
}
```

### 存档文件结构

```
~/.config/alou/media_archives/
├── media_image_1710763200.json
├── media_audio_1710763300.json
└── media_video_1710763400.json
```

每个存档文件包含：
- `type`: 媒体类型
- `file_path`: 本地文件路径
- `url`: 访问 URL
- `metadata`: 元数据
- `created_at`: 创建时间

---

## 📊 当前系统架构

### 组件概览

```
┌─────────────────────────────────────────────────────────────────┐
│                         Agent Layer                              │
│  (agent_runtime/agent_supervisor.rs, agent_actor.rs)            │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Tool Facade 层                              │
│  (tools/facade.rs - 统一入口)                                    │
│  - 优先查询 ToolBus (媒体工具)                                   │
│  - Fallback 到 ToolRegistry (核心工具)                            │
└─────────────────────────────────────────────────────────────────┘
                              │
            ┌─────────────────┴─────────────────┐
            ▼                                   ▼
┌───────────────────────┐          ┌───────────────────────────────┐
│      ToolBus          │          │       ToolRegistry            │
│  (agent_runtime/      │          │  (tools/registry.rs)          │
│   tool_bus.rs)        │          │  - DashMap 无锁并发           │
│  - 媒体工具           │          │  - 核心工具 (文件系统、搜索等) │
│  - generate_image     │          │                               │
│  - generate_audio     │          │                               │
│  - generate_video     │          │                               │
│  - get_video_status   │          │                               │
└───────────────────────┘          └───────────────────────────────┘
            │
            ▼
┌─────────────────────────────────────────────────────────────────┐
│                    MediaProvider 层                              │
│  (agent/providers/media_provider.rs)                            │
│  - google (Imagen)                                              │
│  - jimeng (即梦)                                                │
│  - minimax (音频/视频)                                          │
└─────────────────────────────────────────────────────────────────┘
```

### 独立组件（未集成）

```
┌─────────────────────────────────────────────────────────────────┐
│                    Media API Server                             │
│  (media_api.rs)                                                 │
│  - HTTP API: /api/media/generate                                │
│  - HTTP API: /api/media/tools                                   │
│  - 启动位置：agent_runtime/commands.rs:166                      │
│  - 状态：✅ 独立运行，但未与工具调用链路集成                      │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                    IPFS Archive Tool                            │
│  (tools/ipfs_archive.rs)                                        │
│  - 工具 ID: ipfs_archive                                        │
│  - 动作：archive, retrieve, publish_ipns                        │
│  - 状态：✅ 已注册到 ToolRegistry，但未自动调用                   │
└─────────────────────────────────────────────────────────────────┘
```

## 🔴 问题分析

### 1. 媒体 API 孤岛

**问题**：`media_api.rs` 提供了完整的 HTTP API，但 Agent 调用工具时**不经过 API**

**调用链路对比**：

```
❌ 当前实际调用链路:
Agent → ToolFacade → ToolBus → media_tools.rs → MediaProvider
       (media_api.rs 被绕过)

✅ 期望的调用链路:
Agent → ToolFacade → media_api.rs → MediaProvider
                              ↓
                      ipfs_archive.rs (自动存档)
```

**代码证据**：

- `media_api.rs:452-476` - `start_media_api_server()` 在后台运行 HTTP 服务器
- `agent_runtime/commands.rs:166` - API 服务器启动，但没有与 ToolBus/ToolRegistry 集成
- `media_tools.rs:46-113` - `GenerateImageTool::execute()` 直接调用 `ProviderRegistry`

### 2. IPFS 存档未自动化

**问题**：媒体生成后**不会自动存档**到 IPFS，需要手动调用

**当前流程**：
```
1. Agent 调用 generate_image
2. MediaProvider 生成图片 → file_path: "/path/to/image.png"
3. 返回结果给 Agent
4. ❌ 没有自动调用 ipfs_archive
```

**期望流程**：
```
1. Agent 调用 generate_image
2. MediaProvider 生成图片 → file_path: "/path/to/image.png"
3. ✅ 自动调用 ipfs_archive.archive() → CID: "Qm..."
4. 返回结果给 Agent (包含 file_path + IPFS CID)
```

### 3. 工具调用分散

**问题**：媒体工具注册在 `ToolBus`，核心工具在 `ToolRegistry`，管理分散

```rust
// ToolBus (agent_runtime/tool_bus.rs:31-42)
pub fn register_media_tools(&self, provider_registry: Arc<ProviderRegistry>) {
    self.register_tool("generate_image", Box::new(GenerateImageTool::new(...)));
    self.register_tool("generate_audio", Box::new(GenerateAudioTool::new(...)));
    // ...
}

// ToolRegistry (tools/registry.rs)
pub async fn register(&mut self, tool: Arc<dyn ToolExecutor>) -> Result<(), ToolError>
```

## 🔧 修复方案

### 方案 A：修改 Media API 集成存档（推荐）

**修改文件**：`media_api.rs`

**实现步骤**：

1. 在 `execute_generate_image` 等函数中添加存档逻辑
2. 返回结果包含 IPFS CID

```rust
// media_api.rs:120-140 (修改后)
async fn execute_generate_image(...) -> Result<serde_json::Value, String> {
    // ... 原有生成逻辑 ...
    
    match provider.generate_image(options).await {
        Ok(output) => {
            // ✅ 新增：自动存档到 IPFS
            let ipfs_cid = archive_to_ipfs(&output.file_path).await?;
            
            Ok(serde_json::json!({
                "success": true,
                "media_type": "image",
                "provider": provider_name,
                "file_path": output.file_path,
                "url": output.url,
                "ipfs_cid": ipfs_cid,  // ✅ 新增
                "metadata": { ... }
            }))
        }
        Err(e) => Err(...)
    }
}
```

### 方案 B：修改 Media Tools 集成存档

**修改文件**：`tools/media_tools.rs`

**实现步骤**：

1. 在 `GenerateImageTool::execute` 等函数中添加存档调用
2. 使用 `ToolRegistry::execute("ipfs_archive", ...)` 调用存档工具

```rust
// media_tools.rs:50-80 (修改后)
async fn execute(&self, args: Value, context: &ToolContext) -> Result<Value, String> {
    // ... 原有生成逻辑 ...
    
    match provider.generate_image(options).await {
        Ok(output) => {
            // ✅ 新增：调用 IPFS 存档工具
            let archive_args = json!({
                "action": "archive",
                "archive_type": "custom_file",
                "name": format!("Generated Image {}", chrono::Utc::now()),
                "data": {
                    "file_path": &output.file_path,
                    "prompt": &options.prompt,
                }
            });
            
            let archive_result = context.execute_tool("ipfs_archive", archive_args).await?;
            let ipfs_cid = archive_result["cid"].as_str().unwrap_or("");
            
            Ok(json!({
                "success": true,
                "file_path": output.file_path,
                "ipfs_cid": ipfs_cid,
                ...
            }))
        }
        Err(e) => Err(...)
    }
}
```

### 方案 C：创建媒体工作流工具

**新建文件**：`tools/media_workflow.rs`

**实现步骤**：

1. 创建新的工作流工具，串联生成和存档
2. Agent 调用工作流工具而非单个工具

```rust
pub struct MediaWorkflowTool {
    registry: Arc<ToolRegistry>,
}

#[async_trait]
impl Tool for MediaWorkflowTool {
    fn name(&self) -> &str { "media_workflow" }
    
    async fn execute(&self, args: Value, context: &ToolContext) -> Result<Value, String> {
        // 1. 调用媒体生成工具
        let generate_result = context.execute_tool("generate_image", args.clone()).await?;
        
        // 2. 提取 file_path
        let file_path = generate_result["file_path"].as_str()
            .ok_or("No file_path in generate result")?;
        
        // 3. 调用 IPFS 存档工具
        let archive_args = json!({
            "action": "archive",
            "archive_type": "media",
            "name": format!("Media {}", chrono::Utc::now()),
            "data": { "file_path": file_path }
        });
        let archive_result = context.execute_tool("ipfs_archive", archive_args).await?;
        
        // 4. 返回完整结果
        Ok(json!({
            "success": true,
            "file_path": file_path,
            "ipfs_cid": archive_result["cid"],
            "workflow": "media_generation_with_archive"
        }))
    }
}
```

## 📋 建议实施步骤

### 立即可行的测试

1. **测试当前 API 可用性**：
   ```bash
   ./test_media_loop.sh
   ```

2. **验证调用链路**：
   - 检查 `media_api_port` 文件确认 API 端口
   - 使用 curl 测试 `/api/media/generate`
   - 检查日志确认调用路径

3. **手动测试存档**：
   ```bash
   curl -X POST http://127.0.0.1:PORT/api/tool/execute \
     -H "Content-Type: application/json" \
     -d '{"tool_id": "ipfs_archive", "args": {"action": "archive", ...}}'
   ```

### 长期修复计划

| 优先级 | 任务 | 预计工作量 |
|--------|------|------------|
| P0 | 方案 A/B 实现自动存档 | 2-3 小时 |
| P1 | 统一工具注册管理 | 4-6 小时 |
| P2 | 创建工作流工具 | 2-3 小时 |
| P3 | 添加监控和日志 | 1-2 小时 |

## 🎯 结论

**核心问题**：媒体生成 API 和 IPFS 存档工具都是**独立的功能孤岛**，没有形成闭环。

**推荐方案**：**方案 A**（修改 Media API 集成存档），原因：
- 最小改动范围
- 保持 API 独立性
- 易于测试和调试
- 不影响现有 ToolBus/ToolRegistry 架构

**下一步**：您想让我实施哪个方案？或者先运行测试脚本验证当前状态？
