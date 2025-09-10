# Alou3 Rust - 高性能AI智能体

这是Alou3项目的Rust语言重新实现版本，提供更快的运行速度和更好的性能。该项目是一个在终端运行的AI智能体，支持MCP工具集成和DeepSeek API。

## 🚀 特性

- **高性能**: 使用Rust语言实现，提供更快的启动速度和运行性能
- **MCP工具支持**: 完整的MCP (Model Context Protocol) 工具集成
- **DeepSeek API**: 支持DeepSeek AI模型的API调用
- **异步处理**: 基于Tokio的异步运行时，支持高并发
- **内存安全**: Rust的内存安全保证，避免常见的内存错误
- **跨平台**: 支持Windows、macOS和Linux
- **命令行界面**: 友好的CLI界面，支持交互式聊天

## 📋 系统要求

- Rust 1.70+ 
- 网络连接（用于API调用）
- DeepSeek API密钥

## 🛠️ 安装

### 1. 克隆项目

```bash
git clone <repository-url>
cd alou3-rust
```

### 2. 安装依赖

```bash
cargo build --release
```

### 3. 配置环境变量

创建 `.env` 文件或设置环境变量：

```bash
# 必需
export DEEPSEEK_API_KEY=your_deepseek_api_key_here

# 可选
export DEEPSEEK_API_ENDPOINT=https://api.deepseek.com/v1
export ALOU_DEBUG=false
export ALOU_WORKSPACE_DIRS=/path/to/workspace1,/path/to/workspace2
```

## 🎯 使用方法

### 基本用法

```bash
# 启动交互式聊天模式
cargo run

# 执行单个命令
cargo run -- exec "读取文件 /path/to/file.txt"

# 列出可用工具
cargo run -- tools

# 测试MCP连接
cargo run -- test

# 初始化配置
cargo run -- init
```

### 命令行选项

```bash
# 启用调试模式
cargo run -- --debug

# 启用详细输出
cargo run -- --verbose

# 指定工作目录
cargo run -- --workdir /path/to/project

# 指定配置文件
cargo run -- --config /path/to/config.json
```

### 交互式命令

在聊天模式下，您可以使用以下命令：

- `help` 或 `h` - 显示帮助信息
- `tools` 或 `t` - 列出所有可用工具
- `clear` 或 `c` - 清屏
- `exit`、`quit` 或 `q` - 退出程序

## 🔧 配置

### MCP配置 (mcp.json)

项目使用 `mcp.json` 文件配置MCP服务器：

```json
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["@modelcontextprotocol/server-filesystem", "/Users"],
      "timeout": 30000,
      "trust": true
    },
    "memory": {
      "command": "npx",
      "args": ["@modelcontextprotocol/server-memory"],
      "timeout": 30000,
      "trust": true
    }
  }
}
```

### 环境变量

| 变量名 | 必需 | 默认值 | 描述 |
|--------|------|--------|------|
| `DEEPSEEK_API_KEY` | 是 | - | DeepSeek API密钥 |
| `DEEPSEEK_API_ENDPOINT` | 否 | `https://api.deepseek.com/v1` | DeepSeek API端点 |
| `ALOU_DEBUG` | 否 | `false` | 启用调试模式 |
| `ALOU_WORKSPACE_DIRS` | 否 | 当前目录 | 工作区目录列表 |

## 🏗️ 项目结构

```
alou3-rust/
├── src/
│   ├── main.rs              # 主入口文件
│   ├── lib.rs               # 库入口文件
│   ├── types.rs             # 类型定义
│   ├── tools.rs             # 工具trait和实现
│   ├── workspace_context.rs # 工作区上下文
│   ├── mcp_config.rs        # MCP配置管理
│   ├── mcp_tool.rs          # MCP工具实现
│   ├── mcp_client.rs        # MCP客户端
│   ├── tool_registry.rs     # 工具注册表
│   ├── deepseek_client.rs   # DeepSeek客户端
│   └── agent.rs             # 智能体实现
├── Cargo.toml               # 项目配置
└── README.md                # 项目说明
```

## 🔌 工具集成

### 内置工具

- **文件操作**: 读取、写入、搜索文件
- **代码分析**: 分析代码结构、查找问题
- **系统命令**: 执行shell命令
- **网络请求**: 获取网页内容、API调用
- **内存管理**: 保存和检索重要信息

### MCP工具

通过MCP协议集成的工具：

- **filesystem**: 文件系统操作
- **memory**: 内存管理
- **fetch**: 网络请求
- **其他**: 根据配置自动发现

## 🚀 性能优化

### 编译优化

```bash
# 发布版本（推荐）
cargo build --release

# 启用所有优化
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

### 运行时优化

- 使用 `--release` 标志运行
- 设置合适的超时时间
- 启用连接池
- 使用异步I/O

## 🧪 测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_name

# 运行集成测试
cargo test --test integration_tests
```

## 📊 基准测试

```bash
# 运行基准测试
cargo bench

# 性能分析
cargo install cargo-flamegraph
cargo flamegraph --bin alou3-rust
```

## 🐛 故障排除

### 常见问题

1. **API密钥错误**
   ```
   错误: DeepSeek API key is required
   解决: 设置 DEEPSEEK_API_KEY 环境变量
   ```

2. **MCP服务器连接失败**
   ```
   错误: Error connecting to MCP server 'filesystem'
   解决: 检查MCP服务器是否正确安装和配置
   ```

3. **工具发现失败**
   ```
   错误: No tools found
   解决: 检查mcp.json配置文件
   ```

### 调试模式

启用调试模式获取详细日志：

```bash
cargo run -- --debug --verbose
```

## 🤝 贡献

欢迎贡献代码！请遵循以下步骤：

1. Fork 项目
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 打开 Pull Request

## 📄 许可证

本项目采用 Apache-2.0 许可证。详见 [LICENSE](LICENSE) 文件。

## 🙏 致谢

- 原Alou3 TypeScript项目
- Rust社区
- MCP协议规范
- DeepSeek团队

## 📞 支持

如果您遇到问题或有建议，请：

1. 查看 [Issues](https://github.com/your-repo/issues)
2. 创建新的Issue
3. 联系维护者

---

**注意**: 这是Alou3项目的Rust重新实现版本，专注于性能和稳定性。如果您需要更多功能，请参考原TypeScript版本。
