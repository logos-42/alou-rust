# name
web-search

# description
网络搜索技能，支持多种搜索引擎和搜索策略

# version
1.0.0

# license
MIT

# author
Alou Team

# category
web

# allowed_tools
- http
- search

# parameters
{
  "type": "object",
  "properties": {
    "query": {
      "type": "string",
      "description": "搜索关键词"
    },
    "engine": {
      "type": "string",
      "description": "搜索引擎",
      "enum": ["google", "bing", "duckduckgo", "baidu"],
      "default": "google"
    },
    "numResults": {
      "type": "number",
      "description": "返回结果数量",
      "default": 10,
      "minimum": 1,
      "maximum": 100
    },
    "options": {
      "type": "object",
      "description": "搜索选项",
      "properties": {
        "language": {
          "type": "string",
          "default": "zh-CN"
        },
        "region": {
          "type": "string",
          "default": "cn"
        },
        "timeRange": {
          "type": "string",
          "description": "时间范围",
          "enum": ["any", "day", "week", "month", "year"]
        },
        "safeSearch": {
          "type": "boolean",
          "default": true
        }
      }
    }
  },
  "required": ["query"]
}

# instructions
## 执行步骤

1. 验证搜索关键词
2. 选择搜索引擎
3. 构建搜索 URL
4. 发送 HTTP 请求
5. 解析搜索结果
6. 返回结构化结果

## 注意事项

- 遵守搜索引擎的 robots.txt
- 控制请求频率，避免被封禁
- 尊重版权和知识产权

## 示例

```typescript
// 基本搜索
const results = await skill.execute({
  query: "TypeScript 教程"
});

// 指定搜索引擎
const googleResults = await skill.execute({
  query: "AI Agent",
  engine: "google",
  numResults: 20
});

// 带选项搜索
const recentResults = await skill.execute({
  query: "Web3 开发",
  engine: "google",
  numResults: 10,
  options: {
    timeRange: "week",
    language: "zh-CN"
  }
});
```

# dependencies
- node >= 18
- node-fetch >= 3

# changelog
## 1.0.0
- 初始版本
- 支持多搜索引擎
- 支持搜索结果过滤
