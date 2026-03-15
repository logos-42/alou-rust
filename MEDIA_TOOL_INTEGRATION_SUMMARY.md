# 媒体 API 工具集成 - 完整实施说明

本文档说明了将媒体生成 API 集成到工具系统的完整修改内容。

## ✅ 已完成的工作

### 1. 创建媒体工具实现
**文件**: `alou-desktop/src-tauri/src/agent_runtime/media_tools.rs` (已创建)

包含 4 个媒体工具：
- `GenerateImageTool` - 图片生成
- `GenerateAudioTool` - 语音合成
- `GenerateVideoTool` - 视频生成（异步）
- `GetVideoStatusTool` - 查询视频状态

### 2. 更新 ToolBus 注册媒体工具
**文件**: `alou-desktop/src-tauri/src/agent_runtime/tool_bus.rs` (已修改)

添加了 `register_media_tools()` 方法

### 3. 更新 AgentRuntime 初始化
**文件**: `alou-desktop/src-tauri/src/agent_runtime/mod.rs` (已修改)

- 添加 `provider_registry` 字段
- 在 `new()` 中初始化 `ProviderRegistry` 并注册媒体工具

### 4. 更新 executor.rs 的默认工具列表
**文件**: `alou-desktop/src-tauri/src/agent/executor.rs` (需要手动修改)

需要在 `get_default_tools()` 中添加 4 个媒体工具，在 `get_tool_parameters()` 中添加参数定义

---

## 🔧 需要手动完成的修改

### 修改 1: executor.rs - 添加媒体工具到默认列表

**位置**: `alou-desktop/src-tauri/src/agent/executor.rs`

在 `get_default_tools()` 函数的 `vec![...]` 末尾添加：

```rust
// 媒体生成工具
AiTool {
    name: "generate_image".to_string(),
    description: "根据文字描述生成图片，支持风景、人物、艺术创作等。当用户要求生成图片、绘制图像、创建插画时使用此工具。".to_string(),
    parameters: serde_json::json!({
        "type": "object",
        "properties": {
            "prompt": { "type": "string", "description": "图片描述，详细描述要生成的图片内容" },
            "width": { "type": "integer", "description": "图片宽度（像素），默认 1024", "default": 1024 },
            "height": { "type": "integer", "description": "图片高度（像素），默认 1024", "default": 1024 },
            "provider": { "type": "string", "enum": ["google", "jimeng"], "description": "图片生成 Provider", "default": "google" }
        },
        "required": ["prompt"]
    })
},
AiTool {
    name: "generate_audio".to_string(),
    description: "将文字转换为语音（TTS），支持多种音色和语言。当用户要求朗读文本、生成语音、创建音频时使用此工具。".to_string(),
    parameters: serde_json::json!({
        "type": "object",
        "properties": {
            "text": { "type": "string", "description": "要转换为语音的文字内容" },
            "voice_id": { "type": "string", "description": "音色 ID（可选）" },
            "provider": { "type": "string", "enum": ["minimax"], "description": "语音合成 Provider", "default": "minimax" }
        },
        "required": ["text"]
    })
},
AiTool {
    name: "generate_video".to_string(),
    description: "根据文字描述生成视频，支持动画、实景等风格。注意：视频生成是异步任务。当用户要求生成视频时使用此工具。".to_string(),
    parameters: serde_json::json!({
        "type": "object",
        "properties": {
            "prompt": { "type": "string", "description": "视频描述" },
            "duration": { "type": "integer", "description": "视频时长（秒）", "default": 5 },
            "provider": { "type": "string", "enum": ["minimax", "jimeng"], "description": "视频生成 Provider", "default": "minimax" }
        },
        "required": ["prompt"]
    })
},
AiTool {
    name: "get_video_status".to_string(),
    description: "查询异步视频生成任务的状态。".to_string(),
    parameters: serde_json::json!({
        "type": "object",
        "properties": {
            "task_id": { "type": "string", "description": "任务 ID" },
            "provider": { "type": "string", "enum": ["minimax", "jimeng"], "description": "Provider" }
        },
        "required": ["task_id", "provider"]
    })
},
```

### 修改 2: executor.rs - 添加媒体工具参数定义

**位置**: `alou-desktop/src-tauri/src/agent/executor.rs`

在 `get_tool_parameters()` 函数的 `match` 语句中添加：

```rust
"generate_image" => serde_json::json!({
    "type": "object",
    "properties": {
        "prompt": { "type": "string", "description": "图片描述" },
        "width": { "type": "integer", "default": 1024 },
        "height": { "type": "integer", "default": 1024 },
        "provider": { "type": "string", "enum": ["google", "jimeng"], "default": "google" }
    },
    "required": ["prompt"]
}),
"generate_audio" => serde_json::json!({
    "type": "object",
    "properties": {
        "text": { "type": "string", "description": "文字内容" },
        "voice_id": { "type": "string" },
        "provider": { "type": "string", "enum": ["minimax"], "default": "minimax" }
    },
    "required": ["text"]
}),
"generate_video" => serde_json::json!({
    "type": "object",
    "properties": {
        "prompt": { "type": "string", "description": "视频描述" },
        "duration": { "type": "integer", "default": 5 },
        "provider": { "type": "string", "enum": ["minimax", "jimeng"], "default": "minimax" }
    },
    "required": ["prompt"]
}),
"get_video_status" => serde_json::json!({
    "type": "object",
    "properties": {
        "task_id": { "type": "string" },
        "provider": { "type": "string", "enum": ["minimax", "jimeng"] }
    },
    "required": ["task_id", "provider"]
}),
```

### 修改 3: 前端 useAsyncTaskPolling.ts - 添加媒体工具执行逻辑

**位置**: `alou-desktop/src/components/AgentChat/hooks/useAsyncTaskPolling.ts`

在 `executeToolCallsAndSubmitResults` 函数中，在工具执行循环内添加判断：

```typescript
// 检查是否是媒体工具
const isMediaTool = ['generate_image', 'generate_audio', 'generate_video', 'get_video_status'].includes(toolCall.tool)

if (isMediaTool) {
  // 使用 HTTP API 执行媒体工具
  const response = await apiClient.post('/api/media/generate', {
    tool: toolCall.tool,
    args: toolCall.arguments,
    timeout: 300000
  })
  
  result = {
    tool: toolCall.tool,
    success: response.data.success || false,
    result: response.data.result || response.data,
    error: response.data.error,
    arguments: toolCall.arguments,
    timestamp: Date.now(),
    tool_call_id: toolCall.id,
  }
} else {
  // 使用本地 Tauri 命令执行普通工具（现有逻辑）
  // ...
}
```

### 修改 4: 添加媒体 API 路由（如果需要）

**位置**: `alou-desktop/src-tauri/src/tool_api.rs` 或新建 `media_api.rs`

添加 `/api/media/generate` 路由：

```rust
#[derive(Deserialize)]
struct MediaGenerateRequest {
    tool: String,
    args: serde_json::Value,
    timeout: Option<u64>,
}

async fn generate_media(
    AxumState(state): AxumState<Arc<Mutex<Option<ApiState>>>>,
    Json(payload): Json<MediaGenerateRequest>,
) -> Json<serde_json::Value> {
    // 获取 ProviderRegistry
    // 调用对应的媒体工具
    // 返回结果
}
```

### 修改 5: 前端媒体播放器组件（可选）

**位置**: `alou-desktop/src/components/AgentChat/`

创建 `MediaMessageViewer.tsx` 组件，用于显示图片/音频/视频消息。

---

## 🧪 测试流程

1. **配置媒体 Provider**
   - 打开设置 → API 配置
   - 添加 Google/Jimeng/MiniMax 配置
   - 测试连接

2. **测试图片生成**
   ```
   用户：帮我生成一张日落的图片
   → LLM 应返回 tool_call: generate_image
   → 前端执行工具
   → 显示生成的图片
   ```

3. **测试语音合成**
   ```
   用户：把这句话读出来：你好世界
   → LLM 应返回 tool_call: generate_audio
   → 前端执行工具
   → 播放音频
   ```

4. **测试视频生成**
   ```
   用户：生成一个猫咪玩耍的视频
   → LLM 应返回 tool_call: generate_video
   → 前端执行工具（异步）
   → 轮询状态
   → 播放视频
   ```

---

## 📊 架构数据流

```
用户："生成一张日落的图片"
  │
  ▼
前端发送消息到 LLM
POST /api/chat/completions
messages: [{role: "user", content: "生成一张日落的图片"}]
tools: [generate_image, generate_audio, ...]
  │
  ▼
LLM 理解意图，返回 tool_call
{
  "choices": [{
    "message": {
      "role": "assistant",
      "content": "",
      "tool_calls": [{
        "id": "call_img_001",
        "type": "function",
        "function": {
          "name": "generate_image",
          "arguments": "{\"prompt\":\"美丽的日落风景\"}"
        }
      }]
    }
  }]
}
  │
  ▼
前端 useAsyncTaskPolling 识别媒体工具
executeToolCallsAndSubmitResults()
  │
  ▼
调用 HTTP API
POST /api/media/generate
{
  "tool": "generate_image",
  "args": {"prompt": "美丽的日落风景"}
}
  │
  ▼
后端调用 ProviderRegistry
→ GoogleProvider.generate_image()
→ 保存到本地 ~/Alou/media/image_xxx.png
→ 返回 file_path
  │
  ▼
前端提交结果给 LLM
POST /api/chat/completions
messages: [{
  "role": "tool",
  "content": "图片已生成：/path/to/image.png",
  "tool_call_id": "call_img_001"
}]
  │
  ▼
LLM 生成最终回复
"我已经为你生成了一张日落图片"
  │
  ▼
前端显示图片并回复用户
```

---

## 📝 下一步

1. 完成 executor.rs 的手动修改
2. 完成前端 useAsyncTaskPolling.ts 的修改
3. 添加媒体 API 路由（如果需要）
4. 测试端到端流程
5. 添加媒体播放器组件

---

实施日期：2026-03-15
状态：核心代码已完成，需要手动集成
