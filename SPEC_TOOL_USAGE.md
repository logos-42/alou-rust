# Spec Tool 使用指南

## 概述

Spec Tool 是一个用于管理规格文档的工具，已集成到 Alou 的工具生态系统中。AI Agent（如 Ralph Loop）可以像调用其他工具一样调用 Spec Tool。

## 架构设计

```
┌─────────────────────────────────────────────────────────────┐
│                      AI Agent (Ralph Loop)                   │
│                    调用统一的 ToolExecutor                   │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│              Rust SpecTool (ToolExecutor 接口)               │
│  - 统一的工具调用接口                                        │
│  - 参数验证                                                  │
│  - 超时控制                                                  │
│  - 错误处理                                                  │
└────────────────────┬────────────────────────────────────────┘
                     │ (通过 Node.js 子进程调用)
                     ▼
┌─────────────────────────────────────────────────────────────┐
│            spec-agent.js (TypeScript/Node.js)               │
│  - 文件系统操作 (~/.alou/specs/)                            │
│  - 模板处理 (~/.spec-workflow/templates/)                   │
│  - 验证逻辑                                                  │
│  - 导出/导入                                                 │
└─────────────────────────────────────────────────────────────┘
```

## 工具信息

- **工具 ID**: `spec`
- **工具名称**: `Spec Tool`
- **分类**: `Development`
- **优先级**: `Medium`
- **依赖**: Node.js (>=18.0.0)

## 支持的操作

| 操作 | 说明 | 必需参数 | 可选参数 |
|------|------|----------|----------|
| `create` | 创建新的规格文档 | `operation`, `spec_type`, `spec_data` | `template_id` |
| `update` | 更新规格文档 | `operation`, `spec_id`, `spec_data` | - |
| `get` | 获取规格文档 | `operation`, `spec_id` | - |
| `delete` | 删除规格文档 | `operation`, `spec_id` | - |
| `list` | 列出所有规格文档 | `operation` | `spec_type` |
| `validate` | 验证规格文档 | `operation`, `spec_id` 或 `spec_data` | - |
| `generate_from_template` | 从模板生成规格 | `operation`, `template_id`, `spec_data` | - |
| `export` | 导出规格文档 | `operation`, `spec_id`, `output_format` | - |
| `import` | 导入规格文档 | `operation`, `spec_data`, `spec_type` | - |

## 规格类型

| 类型 | 说明 | 模板文件 |
|------|------|----------|
| `product` | 产品需求规格 | `product-template.md` |
| `technical` | 技术规格 | `tech-template.md` |
| `design` | 设计规格 | `design-template.md` |
| `api` | API 规格 | - |
| `user_story` | 用户故事 | - |
| `tasks` | 任务规格 | `tasks-template.md` |
| `structure` | 结构规格 | `structure-template.md` |

## 使用示例

### 1. 创建产品规格文档

```json
{
  "operation": "create",
  "spec_type": "product",
  "spec_data": {
    "title": "AI 群聊功能需求",
    "content": "本项目旨在实现一个支持多个 AI 智能体同时在线聊天的群聊系统...",
    "target_users": "需要多 AI 协作的用户",
    "core_features": "智能体管理、消息同步、协作决策"
  },
  "template_id": "product"
}
```

**返回结果**:
```json
{
  "success": true,
  "data": {
    "operation": "create",
    "spec": {
      "id": "abc123...",
      "spec_type": "product",
      "title": "AI 群聊功能需求",
      "content": "...",
      "metadata": {
        "version": "1.0.0",
        "status": "draft",
        "tags": [],
        "template_id": "product"
      },
      "created_at": "2026-03-07T10:00:00Z",
      "updated_at": "2026-03-07T10:00:00Z"
    }
  },
  "execution_time_ms": 150,
  "output": "Spec 操作 'create' 执行成功"
}
```

### 2. 获取规格文档

```json
{
  "operation": "get",
  "spec_id": "abc123..."
}
```

### 3. 列出所有产品规格

```json
{
  "operation": "list",
  "spec_type": "product"
}
```

### 4. 验证规格文档

```json
{
  "operation": "validate",
  "spec_id": "abc123..."
}
```

**返回结果**:
```json
{
  "success": true,
  "data": {
    "operation": "validate",
    "validation_result": {
      "is_valid": true,
      "errors": [],
      "warnings": ["规格内容较为简短，可能需要补充更多细节"],
      "suggestions": ["建议添加标签以便于分类和搜索"]
    }
  }
}
```

### 5. 导出规格为 Markdown

```json
{
  "operation": "export",
  "spec_id": "abc123...",
  "output_format": "md"
}
```

### 6. 更新规格文档

```json
{
  "operation": "update",
  "spec_id": "abc123...",
  "spec_data": {
    "title": "AI 群聊功能需求 v2",
    "content": "更新后的内容...",
    "metadata": {
      "version": "2.0.0",
      "status": "review"
    }
  }
}
```

### 7. 从模板生成规格

```json
{
  "operation": "generate_from_template",
  "template_id": "tech",
  "spec_data": {
    "system_architecture": "微服务架构",
    "tech_stack": "React + Node.js + PostgreSQL",
    "database_design": "关系型数据库设计..."
  }
}
```

## 在 Ralph Loop 中使用

```rust
// 在 Ralph Loop 或其他 AI Agent 中调用
use serde_json::json;

let result = executor.execute_tool("spec", json!({
    "operation": "create",
    "spec_type": "product",
    "spec_data": {
        "title": "新产品需求",
        "content": "需求描述...",
        "target_users": "目标用户群体"
    },
    "template_id": "product"
}), &context).await?;

if result.success {
    let spec_id = result.data["spec"]["id"].as_str().unwrap();
    println!("创建规格成功，ID: {}", spec_id);
}
```

## 在 TypeScript/JavaScript 中使用

```typescript
// 通过 Tauri invoke 调用
import { invoke } from '@tauri-apps/api/core'

const result = await invoke('execute_tool', {
  toolId: 'spec',
  args: JSON.stringify({
    operation: 'create',
    spec_type: 'product',
    spec_data: {
      title: '新产品需求',
      content: '需求描述...'
    },
    template_id: 'product'
  })
})

console.log('创建规格成功:', result)
```

## 错误处理

Spec Tool 可能返回以下错误：

| 错误类型 | 说明 | 解决方案 |
|----------|------|----------|
| `InvalidArguments` | 参数缺失或格式错误 | 检查必需参数是否完整 |
| `ExecutionFailed` | Node.js 脚本执行失败 | 检查 Node.js 是否安装，脚本是否存在 |
| `Timeout` | 执行超时（30 秒） | 检查系统负载，重试操作 |
| `ToolUnavailable` | 工具不可用 | 检查工具是否正确注册 |

## 文件存储位置

- **规格文件**: `~/.alou/specs/{spec_id}.json`
- **模板文件**: `~/.spec-workflow/templates/`
- **导出文件**: `~/.alou/exports/{spec_id}_{timestamp}.{format}`

## 最佳实践

1. **使用模板**: 创建规格时尽量使用模板，确保格式统一
2. **及时验证**: 创建或更新后调用 `validate` 检查规格质量
3. **版本管理**: 更新规格时更新 `metadata.version`
4. **添加标签**: 为规格添加标签便于分类和搜索
5. **定期导出**: 重要规格定期导出备份

## 帮助信息

在 Rust 代码中获取帮助：

```rust
let spec_tool = SpecTool::new();
let help_text = spec_tool.help();
println!("{}", help_text);
```

## 开发说明

### 添加新的规格类型

1. 在 `spec_tool.rs` 中添加新的 `SpecType` 枚举值
2. 在 `spec-agent.js` 中添加对应的验证逻辑
3. （可选）创建对应的模板文件

### 添加新的操作

1. 在 `spec_tool.rs` 中添加新的 `SpecOperation` 枚举值
2. 在 `spec-agent.js` 的 `handleSpecRequest` 中添加处理逻辑
3. 在 `validate_args` 中添加参数验证

## 相关文件

- Rust 实现：`alou-desktop/src-tauri/src/tools/spec_tool.rs`
- Node.js 脚本：`alou-desktop/scripts/spec-agent.js`
- 工具注册：`alou-desktop/src-tauri/src/bridges/tool_bridge.rs`
- 模板目录：`.spec-workflow/templates/`
