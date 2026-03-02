---
name: code-refactor
version: 1.0.0
description: "智能代码重构 Skill - 基于 AST 的代码重构"
category: development
author: "Alou Agent"
license: MIT
risk_level: medium
allowed_tools: [bash, python, file_read, file_write]
---

# Code Refactor Skill

智能代码重构工具，基于抽象语法树(AST)实现安全、精准的代码重构。

## 功能特性

### 支持的重构操作

1. **重命名变量** (`rename_variable`)
   - 智能识别变量作用域
   - 避免命名冲突
   - 保持代码语义不变

2. **提取函数** (`extract_function`)
   - 自动识别可提取代码块
   - 生成函数签名
   - 处理参数传递和返回值

3. **内联变量** (`inline_variable`)
   - 将变量替换为其实际值
   - 清理无用变量

4. **优化导入** (`optimize_imports`)
   - 移除未使用的导入
   - 合并重复导入

### 使用示例

**重命名变量**:
```json
{
  "operation": "rename_variable",
  "file": "src/main.py",
  "old_name": "oldVar",
  "new_name": "new_var"
}
```

**提取函数**:
```json
{
  "operation": "extract_function",
  "file": "src/main.py",
  "start_line": 10,
  "end_line": 25,
  "function_name": "new_function"
}
```

### 脚本引用

- `@scripts/main.py` - 主执行脚本，接收 JSON 输入并返回结果
- `@scripts/lib/utils.py` - 工具函数库，包含 AST 操作辅助函数
