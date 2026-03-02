# name
filesystem

# description
文件系统操作技能，支持读取、写入、删除、移动文件和目录

# version
1.0.0

# license
MIT

# author
Alou Team

# category
file

# allowed_tools
- filesystem

# parameters
{
  "type": "object",
  "properties": {
    "path": {
      "type": "string",
      "description": "文件或目录路径"
    },
    "operation": {
      "type": "string",
      "description": "操作类型",
      "enum": ["read", "write", "delete", "move", "copy", "list", "exists", "mkdir"]
    },
    "content": {
      "type": "string",
      "description": "写入的内容（write 操作需要）"
    },
    "destination": {
      "type": "string",
      "description": "目标路径（move/copy 操作需要）"
    },
    "options": {
      "type": "object",
      "description": "可选配置",
      "properties": {
        "encoding": {
          "type": "string",
          "default": "utf-8"
        },
        "recursive": {
          "type": "boolean",
          "default": false
        }
      }
    }
  },
  "required": ["path", "operation"]
}

# instructions
## 执行步骤

1. 验证路径合法性
2. 检查权限
3. 执行指定操作
4. 返回操作结果

## 注意事项

- 只能访问用户目录下的文件
- 写操作前请确认路径存在
- 删除操作不可恢复，请谨慎使用

## 示例

```typescript
// 读取文件
const result = await skill.execute({
  path: "~/Documents/test.txt",
  operation: "read"
});

// 写入文件
await skill.execute({
  path: "~/Documents/test.txt",
  operation: "write",
  content: "Hello World"
});

// 列出目录
const files = await skill.execute({
  path: "~/Documents",
  operation: "list"
});
```

# dependencies
- node >= 18

# changelog
## 1.0.0
- 初始版本
- 支持基本文件操作
