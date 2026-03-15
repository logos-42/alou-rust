# 即梦 Seedance/Seedream Provider 实现完成报告

## 概述

已完成即梦 (Jimeng) Seedance 1.0 和 Seedream API 的 Provider 实现，并注册到系统的 Provider 列表中。

## 官方 API 文档参考

### Seedance 1.0 (视频生成)
- **官方文档**: https://doc.302.ai/305249446e0
- **API 端点**: `https://api.302.ai/doubao/doubao-seedance`
- **支持模型**:
  - `seedance-1.0-pro`: 标准版，2.2 PTC/1M tokens
  - `seedance-1.0-pro-fast`: 快速版，0.6 PTC/1M tokens
  - `seedance-1.0-lite`: 精简版，1.5 PTC/1M tokens
- **功能**: 文生视频、图片生视频
- **参数**:
  - `content`: 文字或图片内容数组
  - `resolution`: 分辨率 (720p, 1080p)
  - `ratio`: 宽高比 (16:9, 9:16, adaptive)
  - `duration`: 时长 (5 或 10 秒)
  - `generate_audio`: 是否生成音频
  - `service_tier`: 服务等级 (default/flex)

### Seedream (图片生成)
- **官方文档**: https://apidoc.deerapi.com/seedream-%E5%9B%BE%E5%83%8F%E7%94%9F%E6%88%90-331149260e0
- **API 端点**: `https://api.deerapi.com/v1/images/generations`
- **支持模型**:
  - `doubao-seedream-4-0-250828`: Seedream 4.0
  - `doubao-seedream-4-5-251128`: Seedream 4.5
  - `doubao-seedream-5-0-260128`: Seedream 5.0
  - `doubao-seedream-3-0-t2i-250415`: Seedream 3.0 (text-to-image)
- **功能**: 文生图片、多参考图生图、组图生成
- **参数**:
  - `prompt`: 文字描述
  - `image`: 参考图 URL 数组
  - `size`: 输出尺寸 (2K)
  - `n`: 生成数量（参考图 + 生成图 ≤ 15）
  - `watermark`: 是否添加水印

## 实现文件

### 新增文件

1. **`alou-desktop/src-tauri/src/agent/providers/seedance/mod.rs`**
   - SeedanceProvider 实现
   - 支持视频生成（异步任务）
   - 支持任务状态查询
   - 支持模型选择

2. **`alou-desktop/src-tauri/src/agent/providers/seedream/mod.rs`**
   - SeedreamProvider 实现
   - 支持图片生成（同步）
   - 支持多参考图
   - 支持组图生成

### 修改文件

1. **`alou-desktop/src-tauri/src/agent/providers/mod.rs`**
   - 添加 seedance 和 seedream 模块导出
   - 导出 SeedanceProvider 和 SeedreamProvider

2. **`alou-desktop/src-tauri/src/agent/providers/registry.rs`**
   - 在 `create_media_provider` 中添加 seedance 和 seedream 分支

3. **`alou-desktop/src-tauri/src/agent/providers/media_factory.rs`**
   - 在 `create_provider` 中添加 seedance 和 seedream 分支
   - 在 `list_providers` 中添加两个 Provider 的信息
   - 在 `get_provider_by_capability` 中更新推荐逻辑
   - 更新单元测试

4. **`alou-desktop/src-tauri/src/agent/media_config.rs`**
   - 重构配置结构，使用 HashMap 存储 Provider 配置
   - 添加 Capability 枚举
   - 添加辅助方法

5. **`alou-desktop/src-tauri/src/agent/providers/media_provider.rs`**
   - 更新 MediaProvider trait 定义
   - 添加完整的媒体类型和选项定义

## 所有 Provider 注册列表

### AI Provider (文本对话)
- ✅ DeepSeek
- ✅ OpenAI
- ✅ Claude
- ✅ Kimi

### Media Provider (媒体生成)
- ✅ MiniMax (TTS, 视频)
- ✅ Google (Imagen 图片)
- ✅ Jimeng (即梦基础版 - 图片，视频)
- ✅ Haimian (海绵音乐)
- ✅ **Seedance (即梦视频 1.0)** - 新增
- ✅ **Seedream (即梦图片)** - 新增

## Provider 能力对比

| Provider | 图片 | 音频 | 视频 | 音乐 | 备注 |
|----------|------|------|------|------|------|
| MiniMax | ❌ | ✅ (TTS) | ✅ | ❌ | 中文优化 |
| Google | ✅ | ❌ | ❌ | ❌ | 高质量 |
| Jimeng | ✅ | ❌ | ✅ | ❌ | 基础版 |
| Haimian | ❌ | ✅ | ❌ | ✅ | 音乐生成 |
| Seedance | ❌ | ❌ | ✅ | ❌ | 即梦视频 1.0 |
| Seedream | ✅ | ❌ | ❌ | ❌ | 即梦图片 |

## 使用方法

### 配置 Provider

```rust
use crate::agent::media_config::{MediaApiConfig, ProviderConfig, Capability};

let mut config = MediaApiConfig::default();

// 配置 Seedance
config.providers.insert("seedance".to_string(), ProviderConfig {
    name: "seedance".to_string(),
    api_key: "your_api_key".to_string(),
    base_url: Some("https://api.302.ai/doubao".to_string()),
    model: Some("seedance-1.0-pro".to_string()),
    enabled: true,
    capabilities: vec![Capability::Video],
    config: None,
});

// 配置 Seedream
config.providers.insert("seedream".to_string(), ProviderConfig {
    name: "seedream".to_string(),
    api_key: "your_api_key".to_string(),
    base_url: Some("https://api.deerapi.com".to_string()),
    model: Some("doubao-seedream-5-0-260128".to_string()),
    enabled: true,
    capabilities: vec![Capability::Image],
    config: None,
});
```

### 使用 ProviderRegistry

```rust
let registry = ProviderRegistry::new(&config)?;

// 获取 Seedance Provider
let seedance = registry.get_media_provider("seedance")
    .expect("Seedance provider not found");

// 获取 Seedream Provider
let seedream = registry.get_media_provider("seedream")
    .expect("Seedream provider not found");

// 根据能力获取 Provider
let video_provider = registry.get_provider_by_capability(Capability::Video);
let image_provider = registry.get_provider_by_capability(Capability::Image);
```

### 使用 MediaProviderFactory

```rust
// 创建 Provider
let provider = MediaProviderFactory::create_provider("seedance", &config)?;

// 列出所有 Provider
let providers = MediaProviderFactory::list_providers();

// 根据能力获取推荐 Provider
let recommended = MediaProviderFactory::get_provider_by_capability("video_cn");
// 返回：Some("seedance")
```

## 单元测试

运行测试：

```bash
cd alou-desktop/src-tauri
cargo test --package alou-desktop --lib agent::providers::media_factory::tests
cargo test --package alou-desktop --lib agent::media_config::tests
```

## 注意事项

1. **API Key 配置**: 需要分别从 302.AI 和 DeerAPI 获取 API Key
2. **异步任务**: Seedance 视频生成是异步的，需要轮询任务状态
3. **文件存储**: 生成的媒体文件会自动保存到本地存储
4. **错误处理**: 所有 API 调用都有完整的错误处理
5. **日志记录**: 所有操作都有详细的日志记录

## 总结

✅ 已完成所有 Provider 的实现和注册
✅ 代码符合项目规范和架构设计
✅ 包含完整的单元测试
✅ 文档齐全，易于使用

所有 Provider 现在都可以在系统中正常使用。
