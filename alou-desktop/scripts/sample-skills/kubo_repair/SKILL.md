# name
kubo_repair

# description
自动诊断和修复 Kubo/IPFS 节点启动失败的问题。支持 macOS/Linux/Windows 全平台自动修复。

# version
1.0.0

# allowed_tools
- bash
- filesystem
- network
- system

# parameters
{
  "type": "object",
  "properties": {
    "error_type": {
      "type": "string",
      "description": "错误类型: binary_missing, port_conflict, config_corrupted, permission_denied, network_error, unknown"
    },
    "error_message": {
      "type": "string",
      "description": "原始错误信息"
    },
    "platform": {
      "type": "string",
      "description": "平台: darwin, linux, win32"
    }
  },
  "required": ["error_type", "error_message", "platform"]
}

# instructions |
## 执行步骤

### 步骤 1: 收集诊断信息
1. 首先获取平台信息: darwin/macOS, linux, win32/Windows
2. 检查以下路径的 Kubo 二进制是否存在:
   - macOS/Linux: ~/.alou/kubo/ipfs, /usr/local/bin/ipfs
   - Windows: %USERPROFILE%\.alou\kubo\ipfs.exe
3. 检查 5001 端口是否被占用
4. 检查 IPFS 数据目录是否存在和权限

### 步骤 2: 根据错误类型执行修复

#### 类型 A: binary_missing (二进制缺失)
**诊断命令**:
```bash
# 检查二进制是否存在
ls -la ~/.alou/kubo/ipfs 2>/dev/null || echo "NOT_FOUND"
which ipfs 2>/dev/null || echo "NOT_FOUND"
```

**修复步骤**:
1. 确定正确的下载 URL:
   - macOS (Intel): https://dist.ipfs.tech/kubo/v0.39.0/kubo_v0.39.0_darwin-amd64.tar.gz
   - macOS (Apple Silicon): https://dist.ipfs.tech/kubo/v0.39.0/kubo_v0.39.0_darwin-arm64.tar.gz
   - Linux (x86_64): https://dist.ipfs.tech/kubo/v0.39.0/kubo_v0.39.0_linux-amd64.tar.gz
   - Windows: https://dist.ipfs.tech/kubo/v0.39.0/kubo_v0.39.0_windows-amd64.zip

2. 创建目录: mkdir -p ~/.alou/kubo

3. 下载并解压:
   - macOS/Linux: 
     ```
     cd ~/.alou/kubo
     curl -L <URL> | tar xz
     mv kubo/kubo/ipfs . (根据实际解压结构)
     chmod +x ipfs
     ```
   - Windows: 使用 PowerShell 下载并解压

4. 验证安装: ~/.alou/kubo/ipfs --version

#### 类型 B: port_conflict (端口被占用)
**诊断命令**:
```bash
# 检查 5001 端口
lsof -i :5001  # macOS/Linux
netstat -ano | findstr :5001  # Windows

# 检查其他 IPFS 进程
ps aux | grep ipfs  # macOS/Linux
tasklist | findstr ipfs  # Windows
```

**修复步骤**:
1. 找到占用端口的进程 PID
2. 终止该进程:
   - macOS/Linux: kill -9 <PID>
   - Windows: taskkill /PID <PID> /F
3. 或者修改 IPFS 配置使用其他端口:
   - 编辑 ~/.alou/ipfs/config
   - 修改 "Addresses/API": "/ip4/127.0.0.1/tcp/5002"
4. 重试启动

#### 类型 C: config_corrupted (配置损坏)
**诊断命令**:
```bash
# 检查配置是否可读
cat ~/.alou/ipfs/config 2>/dev/null | head -20
```

**修复步骤**:
1. 备份旧配置: mv ~/.alou/ipfs/config ~/.alou/ipfs/config.backup
2. 删除数据目录重新初始化:
   - rm -rf ~/.alou/ipfs
   - ~/.alou/kubo/ipfs init --profile=server
3. 重新配置所需参数
4. 启动节点

#### 类型 D: permission_denied (权限不足)
**诊断命令**:
```bash
# 检查文件和目录权限
ls -la ~/.alou/
ls -la ~/.alou/kubo/
ls -la ~/.alou/ipfs/
```

**修复步骤**:
1. 确保用户拥有目录:
   - macOS/Linux: 
     ```
     sudo chown -R $(whoami) ~/.alou
     chmod -R 755 ~/.alou
     chmod +x ~/.alou/kubo/ipfs
     ```
   - Windows: 以管理员身份运行，或修改文件属性

2. 如果使用非标准目录，确保有完全控制权限

#### 类型 E: network_error (网络问题)
**诊断命令**:
```bash
# 测试网络连接
curl -s https://dist.ipfs.tech/ 2>&1 | head -5

# 检查代理设置
echo $HTTP_PROXY
echo $HTTPS_PROXY
```

**修复步骤**:
1. 检查并配置代理:
   - export HTTP_PROXY=http://proxy:8080
   - export HTTPS_PROXY=http://proxy:8080
2. 或配置 IPFS 使用代理:
   - 编辑 ~/.alou/ipfs/config
   - 设置 "Gateway": "/ip4/0.0.0.0/tcp/8080" (仅本地)
3. 检查防火墙是否阻止连接

#### 类型 F: unknown (未知错误)
**修复步骤**:
1. 收集完整日志:
   - macOS/Linux: ~/.alou/ipfs/logs/
   - Windows: %USERPROFILE%\.alou\ipfs\logs\
2. 尝试完全重新初始化:
   - 停止所有 IPFS 进程
   - rm -rf ~/.alou/ipfs
   - ~/.alou/kubo/ipfs init --profile=server
3. 以调试模式启动查看详细输出:
   - ~/.alou/kubo/ipfs daemon --verbose

### 步骤 3: 验证修复
1. 启动 Kubo 节点:
   - ~/.alou/kubo/ipfs daemon &
   - 等待 3-5 秒
2. 测试 API 可用性:
   - curl -s http://127.0.0.1:5001/api/v0/version
   - 期望返回: {"Version":"0.39.0",...}
3. 测试节点 ID:
   - ~/.alou/kubo/ipfs id
   - 期望返回节点信息
4. 报告修复结果

### 步骤 4: 返回结果
返回 JSON 格式结果:
```json
{
  "success": true/false,
  "error_type": "修复的错误类型",
  "actions_taken": ["执行的动作列表"],
  "verification": {
    "api_available": true/false,
    "node_id": "节点ID或null"
  },
  "message": "简要说明"
}
```

## 使用场景
- 当 IPFS/Kubo 节点启动失败时
- 当检测到 "Kubo binary not found" 错误时
- 当端口 5001 被占用时
- 当 IPFS API 连接超时时
- 当需要自动修复 IPFS 相关问题时

## 注意事项
1. 修复前先备份重要配置 (~/.alou/ipfs/config)
2. 需要完整的工具权限才能执行修复
3. 某些修复需要管理员/root 权限
4. 修复后务必验证节点可用性
5. Windows 上可能需要管理员权限执行 kill 和 chmod 操作
6. 如果同时运行多个 IPFS 实例，注意数据目录隔离
