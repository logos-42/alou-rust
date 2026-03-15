# 媒体 API 官方文档参考

## 概述

本文档基于各媒体 Provider 的官方 API 文档整理。

---

## 1. MiniMax (智谱 AI)

### 官方平台
- 平台：https://platform.minimaxi.com
- 文档：https://platform.minimaxi.com/docs

### TTS 语音合成 API

**端点**:
```
POST https://api.minimaxi.com/v1/t2a_v2
```

**认证**:
```
Authorization: Bearer <your_api_key>
```

**请求示例**:
```json
{
  "model": "speech-2.8-hd",
  "text": "你好，欢迎使用 MiniMax",
  "voice_setting": {
    "voice_id": "male-qn-qingse",
    "speed": 1,
    "vol": 1,
    "pitch": 0,
    "emotion": "happy"
  },
  "audio_setting": {
    "sample_rate": 32000,
    "bitrate": 128000,
    "format": "mp3",
    "channel": 1
  }
}
```

**响应示例**:
```json
{
  "data": {
    "audio": "<hex 编码的音频>",
    "status": 2
  },
  "extra_info": {
    "audio_length": 9900,
    "audio_sample_rate": 32000,
    "word_count": 52
  },
  "base_resp": {
    "status_code": 0,
    "status_msg": "success"
  }
}
```

**特殊标记**:
- 停顿控制：`你好<#0.5#>欢迎` (停顿 0.5 秒)
- 语气词：`今天是不是很开心呀 (laughs)` (仅 speech-2.8 支持)

---

## 2. Google Imagen (Vertex AI)

### 官方文档
- 文档：https://docs.cloud.google.com/vertex-ai/generative-ai/docs/image/overview

### API 端点

**REST API**:
```
POST https://{LOCATION}-aiplatform.googleapis.com/v1/projects/{PROJECT_ID}/locations/{LOCATION}/publishers/google/models/imagen-3.0-generate-002:predict
```

**Python SDK**:
```python
import vertexai
from vertexai.preview.vision_models import ImageGenerationModel

vertexai.init(project="your-project-id", location="us-central1")
model = ImageGenerationModel.from_pretrained("imagen-3.0-generate-002")

response = model.generate_images(
    prompt="A T-Rex relaxing on a beach",
    number_of_images=4,
    aspect_ratio="16:9",
    safety_filter_level="block_some",
)

for idx, image in enumerate(response.images):
    image.save(f"generated_image_{idx}.png")
```

### 认证方式

**方式 1: Application Default Credentials**
```bash
gcloud auth application-default login
```

**方式 2: API Key**
```
Authorization: Bearer YOUR_API_KEY
```

### 请求参数

```json
{
  "instances": [{"prompt": "A modern office building at sunset"}],
  "parameters": {
    "sampleCount": 4,
    "aspectRatio": "16:9",
    "safetySetting": "block_some",
    "addWatermark": false,
    "negativePrompt": "clutter, mess, dark"
  }
}
```

---

## 3. 海绵音乐 (Haimian Music)

### 官方网站
- 官网：https://www.haimianyinyue.com/
- 开发者平台：https://developer.byteplus.com

### API 端点 (参考)

```
POST https://api.haimianyinyue.com/v1/music/generation
GET  https://api.haimianyinyue.com/v1/music/status?task_id={task_id}
```

### 认证方式
```
Authorization: Bearer YOUR_API_KEY
```

### 请求参数

```json
{
  "model": "haimian-music-v1",
  "prompt": "创作一首抒情的钢琴曲，带有淡淡的忧伤",
  "style": "pop",
  "mood": "sad",
  "duration": 60,
  "with_vocals": true
}
```

### 响应格式

**生成响应**:
```json
{
  "code": 0,
  "msg": "success",
  "data": {
    "task_id": "task_123456789",
    "status": "processing",
    "progress": 0
  }
}
```

**状态查询**:
```json
{
  "code": 0,
  "msg": "success",
  "data": {
    "task_id": "task_123456789",
    "status": "completed",
    "progress": 100,
    "music_url": "https://...",
    "duration": 60.5,
    "format": "mp3"
  }
}
```

---

## 4. 错误码参考

### MiniMax
| status_code | 说明 |
|-------------|------|
| 0 | 成功 |
| 1001 | 参数错误 |
| 1002 | API Key 无效 |
| 1003 | 余额不足 |
| 2001 | 生成失败 |

### Google
| 错误类型 | HTTP 状态码 | 说明 |
|----------|-------------|------|
| INVALID_ARGUMENT | 400 | 参数错误 |
| PERMISSION_DENIED | 403 | 权限不足 |
| RESOURCE_EXHAUSTED | 429 | 配额超限 |
| INTERNAL | 500 | 服务器错误 |

---

更新时间：2026-03-15
