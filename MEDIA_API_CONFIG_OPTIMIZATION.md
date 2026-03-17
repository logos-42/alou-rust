# Alou 媒体 API 配置优化方案

## 📊 当前架构

### 工作流程
```
用户请求 → LLM AI → ToolBus → 媒体工具 → ProviderRegistry → 媒体 Provider → API
         ↓
     RalphLoop (agent_actor) → 多轮循环/并发执行
```

### 核心组件

1. **ToolBus** (`src-tauri/src/agent_runtime/tool_bus.rs`)
   - 注册媒体工具：`generate_image`, `generate_audio`, `generate_video`
   - 工具执行并发支持

2. **ProviderRegistry** (`src-tauri/src/agent/providers/registry.rs`)
   - 启动时从 `media_config.json` 加载所有启用的 Provider
   - 运行时根据名称提供 Provider 实例
   - 支持多 Provider 并存

3. **媒体工具** (`src-tauri/src/agent_runtime/media_tools.rs`)
   - 调用时指定 `provider` 参数
   - LLM 根据提示词和上下文自动选择 Provider

## ✅ 优化配置

### 1. API 配置理念变更

**旧模式：** 单一激活配置
- 用户需要手动设置"当前使用"的 API
- 其他配置被忽略

**新模式：** 多配置并存，智能路由
- 所有配置的 API 都可用
- LLM 根据任务类型自动选择
- 支持同一任务多次调用不同 Provider

### 2. 配置存储

#### 文本 LLM 配置 (`agent_config.json`)
```json
{
  "user_apis": [
    {
      "id": "api_xxx",
      "provider": "deepseek",
      "api_key": "sk-xxx",
      "model": "deepseek-v3.2",
      "is_active": false  // 已废弃，保留兼容性
    },
    {
      "id": "api_xxx",
      "provider": "claude",
      "api_key": "sk-xxx",
      "model": "claude-sonnet-4-6-20260218",
      "is_active": false
    }
  ],
  "default_provider": "deepseek"
}
```

#### 媒体 API 配置 (`media_config.json`)
```json
{
  "providers": {
    "seedance": {
      "name": "seedance",
      "api_key": "ark-xxx",
      "base_url": null,
      "model": "doubao-seedance-1.0-pro",
      "enabled": true,
      "capabilities": ["video"]
    },
    "google": {
      "name": "google",
      "api_key": "xxx",
      "base_url": null,
      "model": "imagen-4",
      "enabled": true,
      "capabilities": ["image"]
    },
    "suno": {
      "name": "suno",
      "api_key": "xxx",
      "base_url": null,
      "model": "suno-v5",
      "enabled": true,
      "capabilities": ["music"]
    }
  }
}
```

### 3. 工具调用示例

#### LLM 自动选择 Provider
```javascript
// 用户请求："帮我生成一个视频，内容是日落时分的海滩"
// LLM 分析后调用：
{
  "tool": "generate_video",
  "args": {
    "prompt": "日落时分的海滩，海浪轻拍沙滩，金色阳光洒满海面",
    "provider": "seedance",  // LLM 根据配置和任务选择
    "duration": 5
  }
}

// 工具执行：
// 1. ToolBus 路由到 GenerateVideoTool
// 2. 从 ProviderRegistry 获取 seedance Provider
// 3. 调用火山方舟 API 生成视频
```

#### 多轮调用示例
```javascript
// 用户请求："帮我制作一个宣传视频，先写脚本，再生成图片，最后生成视频"
// LLM 多轮调用：

// 第 1 轮：文本生成（使用 deepseek）
{
  "tool": "text_completion",
  "args": {"prompt": "写一个海滩度假宣传脚本"}
}

// 第 2 轮：图片生成（使用 google）
{
  "tool": "generate_image",
  "args": {
    "prompt": "热带海滩度假村，蓝天白云，棕榈树",
    "provider": "google"
  }
}

// 第 3 轮：背景音乐（使用 suno）
{
  "tool": "generate_audio",
  "args": {
    "text": "轻松愉快的海滩度假音乐",
    "provider": "suno"
  }
}

// 第 4 轮：视频生成（使用 seedance）
{
  "tool": "generate_video",
  "args": {
    "prompt": "根据脚本生成视频",
    "provider": "seedance",
    "duration": 10
  }
}
```

### 4. 配置 UI 优化

#### 文本 LLM 配置
- 支持添加多个 Provider
- 移除"激活"按钮，改为"启用/禁用"开关
- 显示提示："所有配置的 Provider 都会被使用，LLM 根据任务自动选择"

#### 媒体 API 配置
- 按能力分组显示：
  - 📷 图片生成：google, jimeng, seedream
  - 🎵 音乐生成：suno, haimian
  - 🎬 视频生成：seedance, jimeng
- 每个 Provider 显示其支持的能力标签
- 启用/禁用开关控制是否在 ToolBus 中注册

### 5. ProviderRegistry 优化

#### 启动时加载
```rust
// src-tauri/src/agent_runtime/mod.rs
let media_config = MediaApiConfig::load().unwrap_or_else(|_| MediaApiConfig::default());
let provider_registry = Arc::new(
    ProviderRegistry::new(&media_config)
        .unwrap_or_else(|e| {
            log::warn!("ProviderRegistry 创建失败：{}, 使用空配置", e);
            ProviderRegistry::new(&MediaApiConfig::default()).unwrap()
        })
);

// 注册媒体工具到 ToolBus
let mut tool_bus = ToolBus::new();
tool_bus.register_media_tools(provider_registry.clone());
```

#### 运行时复用
```rust
// src-tauri/src/agent_runtime/media_tools.rs
let provider = self.provider_registry
    .get_media_provider(provider_name)
    .ok_or_else(|| format!("媒体 Provider '{}' 不存在或未启用", provider_name))?;

// 检查能力
if !provider.supported_types().contains(&MediaType::Image) {
    return Err(format!("Provider '{}' 不支持图片生成", provider_name));
}
```

## 🔧 待优化项

### 1. 前端配置 UI
- [ ] 移除"激活"按钮
- [ ] 添加"启用/禁用"开关
- [ ] 按能力分组显示媒体 Provider
- [ ] 显示每个 Provider 的使用示例

### 2. 后端配置验证
- [ ] 保存时测试 API 连接
- [ ] 支持批量导入配置
- [ ] 配置变更时热更新 ProviderRegistry

### 3. LLM 提示词优化
- [ ] 在 system prompt 中说明可用的 Provider
- [ ] 提供 Provider 选择指南
- [ ] 支持用户指定 Provider 偏好

## 📝 总结

**核心思想：** 所有配置的 API 都是"活跃"的，LLM 根据任务需求自动选择最合适的 Provider。

**优势：**
1. 无需手动切换 API
2. 支持多 Provider 协作
3. 提高任务执行效率
4. 降低用户配置复杂度

**配置建议：**
- 文本 LLM：配置 2-3 个不同厂商的 API（如 deepseek + claude）
- 图片生成：配置 google + jimeng（备用）
- 视频生成：配置 seedance
- 音乐生成：配置 suno + haimian（备用）
