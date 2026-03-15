# 海绵音乐 (Haimian Music) Provider 集成文档

## 📝 概述

海绵音乐是字节跳动推出的 AI 音乐生成平台，支持通过文字描述生成个性化音乐作品。

**官方网站**: https://www.haimianyinyue.com/

**开发者平台**: https://developer.byteplus.com

---

## 🎯 功能特点

- **AI 音乐生成**: 通过文字描述生成完整音乐作品
- **多种音乐风格**: 支持流行、电子、古典、民谣等多种风格
- **人声处理**: 支持带人声的音乐生成
- **歌词支持**: 可指定歌词内容
- **时长控制**: 支持 30 秒 -3 分钟的音乐时长
- **异步任务**: 音乐生成是异步任务，需要轮询状态

---

## 🔧 配置方法

在设置面板 → API 配置 中添加海绵音乐配置：

```json
{
  "provider": "haimian",
  "api_key": "your-haimian-api-key",
  "base_url": null,
  "enabled": true,
  "capabilities": ["music"]
}
```

**配置说明**:
- `api_key`: 从字节跳动开发者平台获取
- `base_url`: 可选，如果有自定义 API 域名
- `capabilities`: 标记为 `["music"]` 表示支持音乐生成

---

## 📡 API 端点

```
# 音乐生成（异步）
POST https://api.haimianyinyue.com/v1/music/generation

# 状态查询
GET https://api.haimianyinyue.com/v1/music/status?task_id={task_id}

# 音乐下载
GET https://api.haimianyinyue.com/v1/music/download/{task_id}
```

---

## 📤 请求参数

### 音乐生成请求

```json
{
  "model": "haimian-music-v1",
  "prompt": "创作一首抒情的钢琴曲，带有淡淡的忧伤",
  "style": "pop",
  "mood": "sad",
  "duration": 60,
  "with_vocals": true,
  "lyrics": "可选的歌词内容",
  "genre": "华语流行",
  "tempo": 80
}
```

**参数说明**:
| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| model | string | 是 | 模型版本 |
| prompt | string | 是 | 音乐描述/提示词 |
| style | string | 否 | 音乐风格 (pop, rock, electronic, classical 等) |
| mood | string | 否 | 情绪 (happy, sad, energetic, calm 等) |
| duration | u32 | 否 | 时长（秒），默认 60 |
| with_vocals | bool | 否 | 是否需要人声 |
| lyrics | string | 否 | 歌词内容 |
| genre | string | 否 | 音乐流派 |
| tempo | u32 | 否 | 速度/BPM |

---

## 📥 响应格式

### 音乐生成响应

```json
{
  "code": 0,
  "msg": "success",
  "data": {
    "task_id": "task_123456789",
    "status": "processing",
    "progress": 0
  },
  "request_id": "req_987654321"
}
```

### 状态查询响应

```json
{
  "code": 0,
  "msg": "success",
  "data": {
    "task_id": "task_123456789",
    "status": "completed",
    "progress": 100,
    "music_url": "https://...",
    "download_url": "https://...",
    "duration": 60.5,
    "format": "mp3"
  }
}
```

**状态说明**:
- `processing`: 生成中
- `completed`: 已完成
- `failed`: 失败

---

## 💻 使用示例

### Rust 代码调用

```rust
use crate::agent::providers::haimian::HaimianMusicProvider;
use crate::agent::media_config::ProviderConfig;
use crate::agent::providers::media_provider::AudioOptions;

// 配置
let config = ProviderConfig {
    api_key: "your-api-key".to_string(),
    base_url: None,
    model: None,
    enabled: true,
    capabilities: vec!["music".to_string()],
    config: None,
};

// 创建 Provider
let provider = HaimianMusicProvider::new(&config)?;

// 生成音乐
let options = AudioOptions {
    text: "创作一首抒情的钢琴曲，带有淡淡的忧伤".to_string(),
    voice_id: None,
    model: Some("pop".to_string()),
    speed: None,
    pitch: None,
    volume: None,
};

let output = provider.generate_audio(options).await?;
println!("音乐生成成功：{}", output.file_path.unwrap());
```

### 前端调用

```typescript
// 在聊天中使用
用户："帮我创作一首抒情的钢琴曲"

// LLM 识别工具调用
tool_call: {
  name: "generate_audio",
  arguments: {
    text: "创作一首抒情的钢琴曲",
    provider: "haimian",
    model: "pop"
  }
}

// 前端执行工具调用
const result = await apiClient.post('/api/media/generate', {
  tool: 'generate_audio',
  args: {
    text: "创作一首抒情的钢琴曲",
    provider: "haimian"
  }
});

// 显示音乐播放器
<MediaMessageViewer 
  type="audio" 
  filePath={result.file_path}
  metadata={{ duration: 60, format: 'mp3' }}
/>
```

---

## 🎵 支持的音乐风格

- **流行 (Pop)**: 华语流行、粤语流行、K-Pop
- **电子 (Electronic)**: House, Techno, EDM
- **古典 (Classical)**: 钢琴曲、弦乐四重奏、交响乐
- **民谣 (Folk)**: 校园民谣、城市民谣
- **摇滚 (Rock)**: 流行摇滚、独立摇滚
- **轻音乐 (Ambient)**: 放松音乐、背景音乐
- **爵士 (Jazz)**: 平滑爵士、摇摆爵士
- **R&B**: 当代 R&B、灵魂乐

---

## ⚠️ 注意事项

1. **API Key 获取**: 需要从字节跳动开发者平台申请
2. **异步任务**: 音乐生成是异步任务，需要轮询状态（已自动处理）
3. **时长限制**: 单次生成最长支持 180 秒（3 分钟）
4. **并发限制**: 根据 API 套餐不同，并发限制可能不同
5. **文件格式**: 默认输出 MP3 格式，支持 WAV 格式（需指定）

---

## 🔍 错误码说明

| 错误码 | 说明 |
|--------|------|
| 0 | 成功 |
| 1001 | 参数错误 |
| 1002 | API Key 无效 |
| 1003 | 余额不足 |
| 2001 | 生成失败 |
| 2002 | 超时 |
| 3001 | 内容违规 |

---

## 📚 参考链接

- 官方网站：https://www.haimianyinyue.com/
- 开发者平台：https://developer.byteplus.com
- API 文档：https://developer.byteplus.com/haimian

---

实施日期：2026-03-15
状态：✅ 已完成
