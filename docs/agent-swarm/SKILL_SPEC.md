# Skill 规范 v2.0

基于《人月神话》模块化原则的 Skill 系统规范

## 设计原则

### 1. 概念完整性
- 统一的元数据格式 (YAML Frontmatter)
- 一致的接口定义
- 标准化的文档结构

### 2. 信息隐藏
- 实现细节封装在 scripts/
- 通过明确的接口暴露能力
- 版本控制保证兼容性

### 3. 正交性
- Skill 之间相互独立
- 可自由组合不产生冲突
- 单一职责原则

## 目录结构

```
~/.config/agents/skills/{skill-name}/
├── SKILL.md              # 主要规范文件 (必需)
├── SPEC.md               # 详细技术规范 (可选)
├── API_REFERENCE.md      # API 参考 (可选)
├── CHANGELOG.md          # 变更日志 (可选)
├── scripts/              # 可执行脚本
│   ├── lib/              # 库文件和模块
│   │   ├── __init__.py
│   │   ├── utils.py
│   │   └── constants.py
│   ├── main.py           # 主入口脚本
│   ├── helper.js         # 辅助脚本
│   └── test_*.py         # 脚本级测试
├── assets/               # 资源文件
│   ├── templates/        # 模板文件
│   ├── configs/          # 配置文件
│   └── data/             # 数据文件
├── tests/                # 测试目录
│   ├── unit/
│   ├── integration/
│   └── fixtures/
└── examples/             # 使用示例
    ├── basic/
    └── advanced/
```

## SKILL.md 规范

### 文件格式

SKILL.md 必须包含 YAML Frontmatter + Markdown 内容：

```markdown
---
# ===== 基本信息 =====
name: skill-name                    # 必需: Skill 标识符 (kebab-case)
version: 1.0.0                      # 必需: 语义化版本

description: |
  简短的描述，说明 Skill 的核心功能。
  可以跨越多行。

description_zh: "中文描述"           # 可选: 本地化描述

# ===== 分类与标签 =====
category: web-development           # 必需: 主分类
tags:                               # 可选: 标签列表
  - frontend
  - react
  - typescript

# ===== 作者信息 =====
author: "Author Name"               # 必需
author_email: "author@example.com"  # 可选
license: MIT                        # 必需
repository: "https://github.com/..." # 可选

# ===== 风险与权限 =====
risk_level: low                     # 必需: low | medium | high
trust_level: verified               # 可选: verified | community | experimental

allowed_tools:                      # 必需: 允许的工具列表
  - bash
  - python
  - node
  - file_read
  - file_write
  - web_search
  - web_fetch

denied_tools:                       # 可选: 明确禁止的工具
  - file_delete
  - system_exec

# ===== 文件系统权限 =====
filesystem:                         # 可选: 详细的文件权限
  read:
    - "${PROJECT_ROOT}/**"
    - "${SKILL_DIR}/**"
  write:
    - "${PROJECT_ROOT}/temp/**"
  deny:
    - "**/.env"
    - "**/secrets/**"
    - "**/.ssh/**"

# ===== 网络权限 =====
network:                            # 可选: 网络访问控制
  allow:
    - "api.github.com"
    - "*.openai.com"
    - "registry.npmjs.org"
  deny:
    - "localhost"
    - "127.0.0.1"
    - "10.0.0.0/8"

# ===== 依赖关系 =====
dependencies:                       # 可选: 依赖的其他 skills
  - name: base-skill
    version: ">=1.0.0 <2.0.0"
  - name: utils-skill
    version: "^1.2.0"

conflicts:                          # 可选: 冲突的 skills
  - old-skill-name

# ===== 资源限制 =====
resources:                          # 可选: 执行资源限制
  max_memory: "512MB"
  max_cpu: 2
  timeout: 300
  max_file_size: "10MB"

# ===== 输入输出 Schema =====
input_schema:                       # 可选: JSON Schema 格式
  type: object
  required: ["target"]
  properties:
    target:
      type: string
      description: "目标文件路径"
    options:
      type: object
      properties:
        recursive:
          type: boolean
          default: false

output_schema:                      # 可选: 输出 Schema
  type: object
  properties:
    success:
      type: boolean
    results:
      type: array
      items:
        type: string

# ===== 示例数据 =====
examples:                           # 可选: 使用示例
  - name: "基本用法"
    input:
      target: "./src"
    output:
      success: true
      count: 42

# ===== 钩子配置 =====
hooks:                              # 可选: 生命周期钩子
  before_exec: "scripts/before.sh"
  after_exec: "scripts/after.sh"
  on_error: "scripts/error.sh"

# ===== 元数据 =====
created_at: "2024-01-01"            # 可选
updated_at: "2024-06-01"            # 可选
min_agent_version: "1.0.0"          # 可选: 最低 Agent 版本
---

# Skill 名称

## 概述

详细描述该 Skill 的功能、适用场景和价值。

## 何时使用

明确列出使用条件：

- ✅ **适用场景**:
  - 场景 1 的描述
  - 场景 2 的描述
  
- ❌ **不适用场景**:
  - 不适用的情况 1
  - 不适用的情况 2

## 前置条件

使用本 Skill 前需要满足的条件：

1. 依赖安装
2. 环境配置
3. 权限要求

## 使用说明

### 方法 1: 通过 Agent 调用

```
@skill-name(input_param="value")
```

### 方法 2: 通过脚本调用

```bash
python3 @scripts/main.py --input '{"key": "value"}'
```

## 参数说明

| 参数名 | 类型 | 必需 | 默认值 | 说明 |
|--------|------|------|--------|------|
| target | string | 是 | - | 目标路径 |
| options | object | 否 | {} | 配置选项 |

## 返回值

成功时返回：
```json
{
  "success": true,
  "data": {...}
}
```

失败时返回：
```json
{
  "success": false,
  "error": {
    "code": "ERROR_CODE",
    "message": "错误描述"
  }
}
```

## 脚本引用

- `@scripts/main.py` - 主执行脚本
- `@scripts/lib/utils.py` - 工具函数
- `@assets/template.txt` - 模板文件

## 最佳实践

1. **建议 1**: 描述
2. **建议 2**: 描述
3. **建议 3**: 描述

## 故障排除

### 常见问题 1

**症状**: ...

**解决方案**: ...

### 常见问题 2

**症状**: ...

**解决方案**: ...

## 变更日志

参见 [CHANGELOG.md](./CHANGELOG.md)

## 许可证

MIT License - 详见 [LICENSE](./LICENSE)
```

## 字段详细说明

### 必需字段

| 字段 | 类型 | 说明 | 示例 |
|------|------|------|------|
| name | string | Skill 标识符，kebab-case | `code-analyzer` |
| version | string | 语义化版本 | `1.0.0` |
| description | string | 一句话描述 | "分析代码质量" |
| category | string | 主分类 | `development` |
| author | string | 作者名 | "John Doe" |
| license | string | 许可证 | MIT |
| risk_level | string | 风险等级 | low/medium/high |
| allowed_tools | array | 允许的工具 | [bash, python] |

### 风险等级定义

| 等级 | 定义 | 示例 |
|------|------|------|
| **low** | 只读操作，无副作用 | 代码分析、文档生成 |
| **medium** | 有限写操作，可撤销 | 文件修改、代码重构 |
| **high** | 破坏性操作，不可撤销 | 删除文件、执行命令 |

### 工具类型

| 工具 | 说明 | 风险 |
|------|------|------|
| bash | 执行 shell 命令 | high |
| python | 执行 Python 脚本 | medium |
| node | 执行 Node.js 脚本 | medium |
| file_read | 读取文件 | low |
| file_write | 写入文件 | medium |
| file_delete | 删除文件 | high |
| web_search | 网络搜索 | low |
| web_fetch | 获取网页内容 | low |
| db_query | 数据库查询 | medium |
| git | Git 操作 | medium |

## 脚本规范

### 脚本目录结构

```
scripts/
├── lib/                      # 库目录
│   ├── __init__.py
│   ├── core.py              # 核心逻辑
│   ├── utils.py             # 工具函数
│   └── constants.py         # 常量定义
├── main.py                  # 主入口 (必需)
├── cli.py                   # 命令行接口
└── test_main.py             # 测试
```

### 主脚本接口

主脚本必须支持以下调用方式：

```bash
# 方式 1: JSON 输入
python3 main.py --input '{"key": "value"}'

# 方式 2: 文件输入
python3 main.py --input-file config.json

# 方式 3: 交互模式
python3 main.py --interactive
```

### 脚本输出格式

脚本必须输出 JSON 到 stdout：

```json
{
  "success": true,
  "data": {
    // 具体返回数据
  },
  "metrics": {
    "duration_ms": 1234,
    "memory_mb": 45
  },
  "artifacts": [
    "/path/to/generated/file"
  ]
}
```

或错误时：

```json
{
  "success": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid input parameter",
    "details": {...}
  }
}
```

## 测试规范

### 测试目录结构

```
tests/
├── unit/                    # 单元测试
│   ├── test_utils.py
│   └── test_core.py
├── integration/             # 集成测试
│   └── test_workflow.py
├── fixtures/                # 测试数据
│   ├── sample_code.py
│   └── expected_output.json
└── conftest.py             # pytest 配置
```

### 测试要求

1. **单元测试覆盖率** > 80%
2. **集成测试** 覆盖主要流程
3. **边界测试** 处理异常情况
4. **性能测试** 确保执行时间合理

## 版本管理

### 语义化版本

遵循 [SemVer](https://semver.org/)：

- **MAJOR**: 不兼容的 API 变更
- **MINOR**: 向后兼容的功能添加
- **PATCH**: 向后兼容的问题修复

### 版本兼容性

```yaml
dependencies:
  - name: other-skill
    version: ">=1.0.0 <2.0.0"  # 兼容 1.x
  - name: utils-skill
    version: "~1.2.0"           # 兼容 1.2.x
```

## 验证工具

提供 Skill 验证工具：

```bash
# 验证 Skill 结构
alou skill validate my-skill

# 测试 Skill 执行
alou skill test my-skill --input '{"test": true}'

# 检查依赖
alou skill check-deps my-skill
```

## 示例 Skill

### 完整示例

参见 [示例 Skill](../examples/skill-template/)

### 最小示例

```markdown
---
name: hello-world
version: 1.0.0
description: "Hello World 示例 Skill"
category: examples
author: "Demo"
license: MIT
risk_level: low
allowed_tools: [bash]
---

# Hello World

简单的示例 Skill。

## 使用

```
@hello-world(name="World")
```

## 脚本

- `@scripts/hello.sh` - 主脚本
```

脚本 `scripts/hello.sh`:

```bash
#!/bin/bash
NAME=${1:-"World"}
echo "{\"success\": true, \"data\": {\"message\": \"Hello, $NAME!\"}}"
```

---

## 附录: 分类列表

| 分类 | 说明 |
|------|------|
| development | 开发工具 |
| testing | 测试相关 |
| deployment | 部署发布 |
| data | 数据处理 |
| web | Web 开发 |
| ai | AI/ML |
| security | 安全 |
| devops | DevOps |
| documentation | 文档 |
| examples | 示例 |
