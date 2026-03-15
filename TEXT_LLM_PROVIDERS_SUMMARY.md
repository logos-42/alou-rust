# 文本 LLM Provider 完整支持

## 📅 完成日期
2026-03-15

## 🎯 目标
扩展 Alou 平台支持的文本 LLM Provider，提供更多 AI 模型选择。

---

## ✅ 已支持的 Provider（11 个）

### 文本 LLM Provider（10 个）

| Provider | 默认模型 | 官网 | 特点 |
|----------|----------|------|------|
| **DeepSeek** | deepseek-chat | https://deepseek.com | 中国领先，性价比高 |
| **OpenAI** | gpt-4 | https://openai.com | 行业标杆，功能全面 |
| **Claude** | claude-3-opus | https://anthropic.com | 长文本，安全性高 |
| **Kimi** | kimi-k2-turbo | https://kimi.ai | 长文本处理优秀 |
| **OpenRouter** ⭐ | anthropic/claude-3.5-sonnet | https://openrouter.ai | 统一 API，多模型访问 |
| **GLM (智谱)** ⭐ | glm-4 | https://www.zhipuai.cn | 中国领先，多模态 |
| **Gemini** ⭐ | gemini-2.0-flash-exp | https://ai.google.dev | Google 最新，多模态 |
| **MiniMax Text** ⭐ | minimax-01 | https://platform.minimaxi.com | 中国领先，需要 group_id |

### 媒体 Provider（6 个）

| Provider | 能力 | 官网 |
|----------|------|------|
| MiniMax | TTS, Video | https://platform.minimaxi.com |
| Google | Image | https://cloud.google.com |
| Jimeng | Image, Video | https://www.jimeng.pro |
| Haimian | Music | https://www.haimianyinyue.com |
| Seedance | Video | 字节跳动 |
| Seedream | Image | 字节跳动 |

---

## 📝 配置示例

### 1. OpenRouter（推荐）

OpenRouter 是一个统一的 API 网关，支持访问多种 LLM 模型：

```json
{
  "provider": "openrouter",
  "api_key": "your-openrouter-api-key",
  "model": "anthropic/claude-3.5-sonnet",
  "base_url": "https://openrouter.ai/api/v1/chat/completions"
}
```

**支持的模型**（通过 OpenRouter）：
- `anthropic/claude-3.5-sonnet` - Claude 3.5 Sonnet
- `openai/gpt-4o` - GPT-4o
- `google/gemini-pro` - Gemini Pro
- `meta-llama/llama-3-70b-instruct` - Llama 3
- `mistralai/mistral-large` - Mistral Large
- 等等 100+ 模型

### 2. GLM (智谱 AI)

```json
{
  "provider": "glm",
  "api_key": "your-zhipuai-api-key",
  "model": "glm-4",
  "base_url": "https://open.bigmodel.cn/api/paas/v4/chat/completions"
}
```

**支持的模型**：
- `glm-4` - 最新版本
- `glm-3-turbo` - 快速版本
- `cogview-3` - 图片生成

### 3. Gemini (Google)

```json
{
  "provider": "gemini",
  "api_key": "your-google-api-key",
  "model": "gemini-2.0-flash-exp",
  "base_url": "https://generativelanguage.googleapis.com/v1beta/models"
}
```

**支持的模型**：
- `gemini-2.0-flash-exp` - 最新实验版
- `gemini-1.5-pro` - 专业版
- `gemini-1.5-flash` - 快速版

### 4. MiniMax Text

```json
{
  "provider": "minimax",
  "api_key": "your-minimax-api-key",
  "model": "minimax-01",
  "base_url": "https://api.minimaxi.com/v1/text/chatcompletion_v2",
  "config": {
    "group_id": "your-group-id"
  }
}
```

**注意**: MiniMax 文本模型需要 `group_id` 配置。

---

## 🚀 使用方式

### 在设置面板配置

1. 打开设置 → API 配置
2. 点击"+ 添加配置"
3. 选择 Provider（如 OpenRouter）
4. 填写 API Key
5. 选择模型（或自定义）
6. 测试连接
7. 保存

### 在代码中使用

```rust
use crate::agent::ai_client::AiClient;
use crate::agent::config::UserApiConfig;

let config = UserApiConfig {
    provider: "openrouter".to_string(),
    api_key: "your-api-key".to_string(),
    model: Some("anthropic/claude-3.5-sonnet".to_string()),
    base_url: None,
    temperature: 0.7,
    max_tokens: 4096,
    config: None,
};

let client = AiClient::new(&config)?;

let response = client.send_message(messages, Some(tools)).await?;
```

---

## 📊 Provider 对比

| Provider | 价格 | 速度 | 质量 | 工具调用 | 中文支持 |
|----------|------|------|------|----------|----------|
| DeepSeek | ¥ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ✅ | ⭐⭐⭐⭐⭐ |
| OpenAI | $$ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ✅ | ⭐⭐⭐ |
| Claude | $$ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ✅ | ⭐⭐⭐ |
| Kimi | ¥ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ✅ | ⭐⭐⭐⭐⭐ |
| OpenRouter | ¥-$$ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ✅ | ⭐⭐⭐⭐ |
| GLM | ¥ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ✅ | ⭐⭐⭐⭐⭐ |
| Gemini | $ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⚠️ | ⭐⭐⭐ |
| MiniMax | ¥ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ✅ | ⭐⭐⭐⭐⭐ |

**价格说明**：
- ¥ = 便宜
- $$ = 中等
- $$$ = 昂贵

---

## 🎯 推荐场景

### 日常对话
- **DeepSeek** - 性价比高
- **Kimi** - 中文优秀

### 复杂任务
- **Claude 3.5** (via OpenRouter) - 推理能力强
- **GPT-4o** (via OpenRouter) - 功能全面

### 长文本处理
- **Kimi** - 支持超长上下文
- **Claude** - 长文本理解好

### 多模态任务
- **Gemini** - 原生多模态
- **GLM** - 支持图文理解

### 经济实惠
- **DeepSeek** - 价格低廉
- **GLM** - 性价比高

---

## 📁 核心文件

| 文件 | 说明 | 行数 |
|------|------|------|
| `openrouter.rs` | OpenRouter Provider | ~230 |
| `glm.rs` | GLM Provider | ~230 |
| `gemini.rs` | Gemini Provider | ~180 |
| `minimax_text.rs` | MiniMax 文本 Provider | ~230 |
| `ai_client.rs` | AI 客户端（已更新） | ~215 |
| `mod.rs` | Provider 导出 | ~40 |

---

## 🔧 提交历史

```
a9b774c feat: 添加 4 个新的文本 LLM Provider
6a95b98 docs: 添加异步工具调用架构文档
a1fb190 feat: 在 executor.rs 中集成异步工具调用管理器
...
```

---

## 📚 参考链接

- OpenRouter: https://openrouter.ai/docs
- GLM: https://open.bigmodel.cn/dev/api
- Gemini: https://ai.google.dev/docs
- MiniMax: https://platform.minimaxi.com/document

---

实施状态：✅ 已完成
支持的 Provider 总数：11 个（文本 10 + 媒体 6，其中 MiniMax 同时支持文本和媒体）
