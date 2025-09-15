# Alou Rust - 自动化工作流智能体

一个基于Rust和Model Context Protocol (MCP)的智能自动化工作流系统，集成了DeepSeek API，提供强大的上下文感知和工具调用能力。

## ✨ 最新更新

- **✅ 工具调用修复**：完全修复了智能体工具调用解析和执行逻辑
- **✅ 后台加载优化**：实现了静默的后台MCP服务器加载，提升启动速度
- **✅ Windows兼容性**：优化了Windows系统下的MCP服务器配置
- **✅ 智能目录访问**：支持多目录配置和智能路径解析

## 🚀 核心特性

### 智能体架构
- **多模态智能体**：支持文件操作、网络请求、记忆管理等多种工具
- **上下文感知**：智能工作区检测，自动识别项目结构和环境
- **工具发现**：自动发现和集成MCP服务器提供的工具
- **连接池管理**：高效的MCP服务器连接池，支持多服务器并发
- **后台加载**：静默的后台MCP服务器加载，提升用户体验
- **智能工具调用**：完整的工具调用解析和执行机制

### DeepSeek集成
- **API客户端**：完整的DeepSeek API集成
- **智能提示**：基于工作区上下文的动态系统提示生成
- **对话管理**：支持多轮对话和上下文保持
- **工具调用**：AI驱动的智能工具选择和参数生成
- **响应解析**：自动解析DeepSeek响应中的工具调用
- **错误处理**：智能的错误处理和重试机制

### 工作流自动化
- **任务分解**：自动将复杂任务分解为可执行步骤
- **并行执行**：支持多工具并行调用
- **错误恢复**：智能错误处理和重试机制
- **状态管理**：完整的智能体状态跟踪和恢复

## 🏗️ 架构设计

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   DeepSeek API  │    │   MCP Agent      │    │  MCP Servers    │
│                 │◄──►│                  │◄──►│                 │
│ • Chat API      │    │ • Context Mgmt   │    │ • Filesystem    │
│ • Tool Calling  │    │ • Tool Discovery │    │ • Memory        │
│ • Streaming     │    │ • Workflow Exec  │    │ • Network       │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │
                                ▼
                       ┌──────────────────┐
                       │  Connection Pool │
                       │                  │
                       │ • Multi-server   │
                       │ • Health Check   │
                       │ • Auto-reconnect │
                       └──────────────────┘
```

## 📦 核心组件

### 智能体 (Agent)
```rust
use mcp_client_rs::agent::{McpAgent, AgentConfig, DeepSeekConfig};

let config = AgentConfig {
    deepseek: DeepSeekConfig {
        base_url: "https://api.deepseek.com".to_string(),
        api_key: "your-api-key".to_string(),
        model: "deepseek-chat".to_string(),
        max_tokens: 4000,
        temperature: 0.7,
    },
    behavior: BehaviorConfig {
        max_retries: 3,
        timeout_seconds: 30,
        verbose_logging: true,
        tool_strategy: ToolStrategy::Auto,
    },
    workspace: WorkspaceConfig {
        directories: vec![".".to_string()],
        smart_detection: true,
        exclude_patterns: vec!["target".to_string()],
    },
};

let mut agent = McpAgent::new(config).await?;
agent.initialize().await?;
let response = agent.process_input("分析当前项目结构并生成报告").await?;
```

### 连接池 (ConnectionPool)
```rust
use mcp_client_rs::connection_pool::{ConnectionPool, McpServerConfig};

let pool = ConnectionPool::new();

// 注册MCP服务器
pool.register_server("filesystem".to_string(), McpServerConfig {
    command: "uvx".to_string(),
    args: vec!["mcp-server-filesystem".to_string()],
    directory: Some(".".to_string()),
    env: None,
}).await;

// 获取连接
let client = pool.get_connection("filesystem").await?;
```

### 工作区上下文 (WorkspaceContext)
```rust
use mcp_client_rs::workspace_context::WorkspaceContextFactory;

// 智能检测工作区
let context = WorkspaceContextFactory::create_smart();

// 自定义工作区
let context = WorkspaceContextFactory::create_custom(vec![
    PathBuf::from("/path/to/project1"),
    PathBuf::from("/path/to/project2"),
]);
```

## 🛠️ 使用方法

### 基本使用
```rust
use mcp_client_rs::agent::{McpAgent, AgentConfig, DeepSeekConfig, BehaviorConfig, WorkspaceConfig, ToolStrategy};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    // 创建智能体配置
    let config = AgentConfig {
        deepseek: DeepSeekConfig {
            base_url: "https://api.deepseek.com".to_string(),
            api_key: std::env::var("DEEPSEEK_API_KEY")?,
            model: "deepseek-chat".to_string(),
            max_tokens: 4000,
            temperature: 0.7,
        },
        behavior: BehaviorConfig {
            max_retries: 3,
            timeout_seconds: 30,
            verbose_logging: true,
            tool_strategy: ToolStrategy::Auto,
        },
        workspace: WorkspaceConfig {
            directories: vec![".".to_string()],
            smart_detection: true,
            exclude_patterns: vec!["target".to_string(), "node_modules".to_string()],
        },
    };
    
    // 创建并初始化智能体
    let mut agent = McpAgent::new(config).await?;
    agent.initialize().await?;
    
    // 处理用户输入
    let response = agent.process_input("帮我分析这个Rust项目的代码结构").await?;
    println!("智能体响应: {}", response);
    
    Ok(())
}
```

### 高级配置
```rust
// 自定义工具策略
let config = AgentConfig {
    // ... 其他配置
    behavior: BehaviorConfig {
        tool_strategy: ToolStrategy::Priority(vec![
            "filesystem".to_string(),
            "memory".to_string(),
            "network".to_string(),
        ]),
        // ... 其他配置
    },
    // ... 其他配置
};
```

## 🔧 环境配置

### 环境变量
```bash
# DeepSeek API配置
export DEEPSEEK_API_KEY="your-api-key-here"
export DEEPSEEK_BASE_URL="https://api.deepseek.com"  # 可选，默认值

# 工作区配置
export ALOU_WORKSPACE_DIRS="/path/to/project1,/path/to/project2"  # 可选

# 日志配置
export RUST_LOG="info"  # debug, info, warn, error
```

### MCP服务器配置
创建 `mcp.json` 配置文件：

#### Linux/macOS 配置
```json
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": [
        "-y",
        "@modelcontextprotocol/server-filesystem",
        "/path/to/allowed/directory"
      ]
    },
    "memory": {
      "command": "npx",
      "args": [
        "-y",
        "@modelcontextprotocol/server-memory"
      ]
    }
  }
}
```

#### Windows 配置
```json
{
  "mcpServers": {
    "filesystem": {
      "command": "npx.cmd",
      "args": [
        "-y",
        "@modelcontextprotocol/server-filesystem",
        "C:\\",
        "D:\\",
        "E:\\"
      ]
    },
    "memory": {
      "command": "npx.cmd",
      "args": [
        "-y",
        "@modelcontextprotocol/server-memory"
      ]
    }
  }
}
```

> **注意**：Windows 系统需要使用 `npx.cmd` 而不是 `npx`，这是因为 Windows 上的 `npx` 是 PowerShell 脚本，而 MCP 服务器需要直接可执行的批处理文件。

## 🚀 快速开始

1. **克隆项目**
```bash
git clone https://github.com/your-username/alou-rust.git
cd alou-rust
```

2. **安装依赖**
```bash
cargo build
```

3. **配置环境**
```bash
export DEEPSEEK_API_KEY="your-api-key"
```

4. **运行示例**
```bash
# 交互式聊天模式
cargo run --bin agent-cli chat

# 测试模式
cargo run --bin agent-cli test --message "列出当前目录内容"

# 静默模式（减少日志输出）
cargo run --bin agent-cli chat --quiet

# 清洁模式（完全隐藏技术日志）
cargo run --bin agent-cli chat --clean
```

## 🎯 功能演示

### 基本工具调用
```bash
# 列出目录内容
cargo run --bin agent-cli test --message "列出当前目录的内容"

# 访问特定目录
cargo run --bin agent-cli test --message "列出D:\\AI\\Alou2目录的内容"

# 文件操作
cargo run --bin agent-cli test --message "读取README.md文件的内容"
```

### 智能推理和问题解决
智能体能够：
- 自动发现目录名称大小写问题
- 智能推荐正确的路径
- 提供格式化的输出结果
- 处理多步骤任务

### 后台加载优化
- 启动时静默加载 MCP 服务器
- 用户首次输入时自动完成工具发现
- 支持多种日志级别（正常、静默、清洁模式）

## 🔧 故障排除

### 常见问题

#### 1. MCP 服务器启动失败
**错误信息**：`Failed to spawn process error=program not found`

**解决方案**：
- **Windows 用户**：确保使用 `npx.cmd` 而不是 `npx`
- **检查 Node.js 安装**：运行 `node --version` 和 `npm --version`
- **检查 PATH 环境变量**：确保 npm 全局包路径在 PATH 中

#### 2. 工具调用不执行
**症状**：智能体显示工具调用 JSON 但不执行

**解决方案**：
- 确保 DeepSeek API 密钥正确配置
- 检查网络连接
- 查看日志输出中的错误信息

#### 3. 目录访问权限问题
**错误信息**：`Access denied` 或目录不在允许列表中

**解决方案**：
- 在 `mcp.json` 中添加允许访问的目录路径
- 确保路径格式正确（Windows 使用 `\\`，Linux/macOS 使用 `/`）
- 检查文件系统权限

#### 4. 启动速度慢
**症状**：应用启动时间过长

**解决方案**：
- 使用 `--quiet` 或 `--clean` 模式减少日志输出
- 后台加载功能会自动优化启动速度
- 检查网络连接和 MCP 服务器响应时间

### 调试模式

启用详细日志输出：
```bash
export RUST_LOG=debug
cargo run --bin agent-cli chat
```

## 📚 API文档

### 核心类型

- `McpAgent`: 主要的智能体实现
- `AgentConfig`: 智能体配置结构
- `ConnectionPool`: MCP服务器连接池
- `WorkspaceContext`: 工作区上下文管理
- `ToolInfo`: 工具信息结构

### 主要方法

- `McpAgent::new()`: 创建新的智能体实例
- `McpAgent::initialize()`: 初始化智能体（包含后台MCP服务器加载）
- `McpAgent::process_input()`: 处理用户输入（支持工具调用解析和执行）
- `McpAgent::execute_tool()`: 执行工具调用
- `McpAgent::discover_tools_silent()`: 静默发现可用工具
- `ConnectionPool::get_connection()`: 获取MCP服务器连接
- `ConnectionPool::list_registered_servers()`: 列出已注册的服务器

## Contributing

Contributions are welcome! Please open an issue or submit a PR if you have improvements, bug fixes, or new features to propose.

1. Fork the repo
2. Create a new branch
3. Add your changes and tests
4. Submit a Pull Request

Please @ darinkishore in the PR if you do send one over.

### Credits

- [MCP Rust SDK](https://github.com/Derek-X-Wang/mcp-rust-sdk)
- [AIchat](https://github.com/sigoden/aichat/tree/main)

### License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
