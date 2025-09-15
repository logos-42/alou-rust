# Alou Rust - 自动化工作流智能体

一个基于Rust和Model Context Protocol (MCP)的智能自动化工作流系统，集成了DeepSeek API，提供强大的上下文感知和工具调用能力。

## 🚀 核心特性

### 智能体架构
- **多模态智能体**：支持文件操作、网络请求、记忆管理等多种工具
- **上下文感知**：智能工作区检测，自动识别项目结构和环境
- **工具发现**：自动发现和集成MCP服务器提供的工具
- **连接池管理**：高效的MCP服务器连接池，支持多服务器并发

### DeepSeek集成
- **API客户端**：完整的DeepSeek API集成
- **智能提示**：基于工作区上下文的动态系统提示生成
- **对话管理**：支持多轮对话和上下文保持
- **工具调用**：AI驱动的智能工具选择和参数生成

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
```json
{
  "mcpServers": {
    "filesystem": {
      "command": "uvx",
      "args": ["mcp-server-filesystem"],
      "directory": "."
    },
    "memory": {
      "command": "uvx", 
      "args": ["mcp-server-memory"],
      "directory": "."
    }
  }
}
```

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
cargo run --bin agent-cli
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
- `McpAgent::initialize()`: 初始化智能体
- `McpAgent::process_input()`: 处理用户输入
- `McpAgent::execute_tool()`: 执行工具调用
- `ConnectionPool::get_connection()`: 获取MCP服务器连接

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
