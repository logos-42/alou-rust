# Seedance API 测试指南

## API 存储位置

### 1. 前端存储（localStorage）
- **键名**: `user_api_key`, `ai_provider`, `ai_model`
- **位置**: 浏览器 localStorage
- **用途**: 备份和快速访问

### 2. Tauri 加密存储（主要）
- **文件路径**: `~/Library/Application Support/alou/agent_config.json` (macOS)
- **加密方式**: AES-256-GCM
- **密钥生成**: 基于机器信息（hostname, username）+ 固定盐值
- **配置结构**:
```json
{
  "user_apis": [
    {
      "id": "seedance-1",
      "provider": "seedance",
      "api_key": "加密存储",
      "base_url": "https://api.302.ai/doubao",
      "model": "seedance-1.0-pro",
      "is_active": true
    }
  ],
  "workers_api": { ... },
  "execution_strategy": "LocalOnly",
  "default_provider": "seedance"
}
```

### 3. 媒体 Provider 配置
- **当前状态**: `MediaApiConfig::load()` 返回默认配置（TODO 状态）
- **需要实现**: 从文件加载媒体 Provider 配置
- **配置位置**: 应与 `agent_config.json` 相同或相邻

## Seedance Provider 实现状态

### ✅ 已完成
1. **Provider 实现**: `alou-desktop/src-tauri/src/agent/providers/seedance/mod.rs`
   - ✅ 创建视频任务（异步）
   - ✅ 查询任务状态
   - ✅ 等待任务完成
   - ✅ 完整的错误处理

2. **Provider 注册**: 
   - ✅ `registry.rs` - 注册到 ProviderRegistry
   - ✅ `media_factory.rs` - 注册到 MediaProviderFactory
   - ✅ `mod.rs` - 模块导出

3. **前端 UI**:
   - ✅ `ApiConfigModal.tsx` - Provider 选择列表
   - ✅ 特殊字段配置（Base URL）
   - ✅ 能力标签显示（video）

### ❌ 未完成
1. **媒体配置存储**: `media_config.rs` 的 `load()` 方法还是 TODO 状态
2. **前端媒体配置保存**: 没有 UI 保存媒体 Provider 配置
3. **API 密钥传递**: 前端配置的 API Key 没有传递到媒体 Provider

## 测试 Seedance API

### 方法 1: 使用 Rust 单元测试

在 `alou-desktop/src-tauri/src/agent/providers/seedance/mod.rs` 中已有测试：

```bash
cd alou-desktop/src-tauri

# 设置环境变量
export SEEDANCE_API_KEY="your_api_key_here"

# 运行测试（需要添加 #[tokio::test]）
cargo test --package alou-desktop --lib agent::providers::seedance::tests::test_create_video_task -- --nocapture
```

### 方法 2: 创建集成测试脚本

创建文件：`alou-desktop/src-tauri/tests/test_seedance_api.rs`

```rust
use alou_desktop::agent::providers::seedance::SeedanceProvider;
use alou_desktop::agent::media_config::ProviderConfig;
use alou_desktop::agent::providers::media_provider::{MediaProvider, VideoOptions};

#[tokio::test]
async fn test_seedance_video_generation() {
    // 配置 Provider
    let config = ProviderConfig {
        name: "seedance".to_string(),
        api_key: std::env::var("SEEDANCE_API_KEY")
            .expect("请设置环境变量 SEEDANCE_API_KEY"),
        base_url: Some("https://api.302.ai/doubao".to_string()),
        model: Some("seedance-1.0-pro".to_string()),
        enabled: true,
        capabilities: vec![],
        config: None,
    };

    // 创建 Provider
    let provider = SeedanceProvider::new(&config).expect("创建 Provider 失败");

    // 创建视频任务
    let options = VideoOptions {
        prompt: "女孩抱着狐狸，女孩睁开眼，温柔地看向镜头".to_string(),
        duration: Some(10.0),
        duration_secs: Some(10),
        model: Some("seedance-1.0-pro".to_string()),
        resolution: Some("720p".to_string()),
    };

    let task = provider.generate_video(options).await.expect("创建任务失败");
    println!("任务创建成功，task_id: {}", task.task_id);

    // 轮询任务状态
    let mut attempts = 0;
    let max_attempts = 60; // 最多等待 5 分钟

    while attempts < max_attempts {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        
        let status = provider.get_task_status(&task.task_id).await.expect("查询状态失败");
        println!("状态：{:?}, 进度：{}%", status.status, status.progress);

        match status.status {
            alou_desktop::agent::providers::media_provider::TaskStatus::Completed => {
                println!("视频生成完成！URL: {:?}", status.result.unwrap().url);
                return;
            }
            alou_desktop::agent::providers::media_provider::TaskStatus::Failed => {
                panic!("视频生成失败：{}", status.error.unwrap());
            }
            _ => {}
        }

        attempts += 1;
    }

    panic!("视频生成超时");
}
```

运行测试：
```bash
cd alou-desktop/src-tauri
export SEEDANCE_API_KEY="your_302ai_api_key"
cargo test --test test_seedance_api -- --nocapture
```

### 方法 3: 使用 curl 直接测试 API

```bash
# 1. 创建视频任务
curl -X POST "https://api.302.ai/doubao/doubao-seedance" \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "seedance-1.0-pro",
    "content": [
      {
        "type": "text",
        "text": "女孩抱着狐狸，女孩睁开眼，温柔地看向镜头"
      }
    ],
    "resolution": "720p",
    "ratio": "16:9",
    "duration": 10,
    "generate_audio": true,
    "service_tier": "default"
  }'

# 响应：{"id": "cgt-20250605180428-cfjt9"}

# 2. 查询任务状态
curl -X GET "https://api.302.ai/doubao/doubao-seedance/cgt-20250605180428-cfjt9" \
  -H "Authorization: Bearer YOUR_API_KEY"

# 响应示例：
# {
#   "id": "cgt-20250605180428-cfjt9",
#   "status": "succeeded",
#   "video": {
#     "play_addr": "https://...",
#     "cover": "https://...",
#     "duration": 10,
#     "width": 1280,
#     "height": 720
#   },
#   "progress": 100
# }
```

## 当前功能验证清单

### ✅ 现有功能（正常）
1. **文本 LLM Provider** - DeepSeek/OpenAI/Claude/Kimi
2. **API 配置存储** - Tauri 加密存储
3. **前端配置 UI** - ApiConfigModal 组件
4. **Agent 执行** - 使用配置的 API Key

### ⚠️ 媒体 Provider 功能（需要完善）
1. **Provider 实现** - ✅ Seedance/Seedream/MiniMax/Google/Jimeng/Haimian
2. **Provider 注册** - ✅ 已注册到 Registry
3. **配置存储** - ❌ `MediaApiConfig::load()` 返回空配置
4. **前端配置** - ❌ 没有保存媒体 Provider 配置的 UI
5. **API 调用** - ⚠️ 需要配置后才能调用

## 完善媒体 Provider 配置的步骤

### 1. 实现 MediaApiConfig 的加载和保存

```rust
// alou-desktop/src-tauri/src/agent/media_config.rs

impl MediaApiConfig {
    fn config_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("alou");
        path.push("media_config.json");
        path
    }

    pub fn load() -> Result<Self, String> {
        let config_path = Self::config_path();
        
        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("读取配置文件失败：{}", e))?;

        let config: MediaApiConfig = serde_json::from_str(&content)
            .map_err(|e| format!("解析配置文件失败：{}", e))?;

        Ok(config)
    }

    pub fn save(&self) -> Result<(), String> {
        let config_path = Self::config_path();
        
        // 确保目录存在
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建目录失败：{}", e))?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("序列化配置失败：{}", e))?;

        std::fs::write(&config_path, content)
            .map_err(|e| format!("写入配置文件失败：{}", e))?;

        Ok(())
    }
}
```

### 2. 添加 Tauri 命令

```rust
// alou-desktop/src-tauri/src/agent/commands.rs

use crate::agent::media_config::MediaApiConfig;

#[tauri::command]
pub async fn get_media_config() -> Result<MediaApiConfig, String> {
    MediaApiConfig::load()
}

#[tauri::command]
pub async fn update_media_config(config: MediaApiConfig) -> Result<(), String> {
    config.save()
}
```

### 3. 前端添加媒体配置 UI

在 `ApiConfigModal.tsx` 中添加媒体 Provider 配置标签页，允许用户为每个 Provider 输入 API Key。

## 快速测试现有功能

```bash
# 1. 检查 Provider 注册
cd alou-desktop/src-tauri
cargo test --package alou-desktop --lib agent::providers::media_factory::tests

# 2. 检查配置结构
cargo test --package alou-desktop --lib agent::media_config::tests

# 3. 构建并运行应用
cargo tauri dev

# 4. 在浏览器开发者工具中查看
console.log(localStorage.getItem('user_api_key'))
console.log(localStorage.getItem('ai_provider'))
```

## 总结

✅ **Seedance Provider 实现完成** - 代码已就绪
✅ **Provider 已注册** - 可以在系统中使用
⚠️ **配置存储需要完善** - `MediaApiConfig::load()` 需要实现文件读写
⚠️ **前端需要配置 UI** - 需要添加媒体 Provider 的 API Key 配置界面

**建议下一步**:
1. 实现 `MediaApiConfig::load()` 和 `save()` 方法
2. 添加 Tauri 命令支持媒体配置
3. 在前端 `ApiConfigModal` 中添加媒体 Provider 配置标签页
4. 测试完整的 API 调用流程
