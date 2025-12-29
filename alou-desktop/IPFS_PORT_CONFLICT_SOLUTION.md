# IPFS 端口冲突解决方案

## 问题描述
Alou 桌面应用启动时检测到端口 5001 已被占用，这可能是由于：
- IPFS Desktop 正在运行
- 其他 IPFS 守护进程
- 其他应用程序占用了该端口

## 解决方案

### 方案1：停止其他 IPFS 实例（推荐）

#### Windows 系统：
1. 打开任务管理器（Ctrl+Shift+Esc）
2. 查找并结束以下进程：
   - IPFS Desktop
   - ipfs.exe
   - go-ipfs
3. 或者在命令提示符中运行：
   ```cmd
   netstat -ano | findstr :5001
   taskkill /PID <PID> /F
   ```

#### macOS/Linux 系统：
```bash
# 查找占用端口的进程
lsof -i :5001

# 停止进程
kill -9 <PID>
```

### 方案2：修改 Alou 使用的 IPFS 端口

#### 方法A：通过环境变量
1. 创建 `.env.local` 文件在项目根目录：
   ```
   VITE_IPFS_API_URL=http://127.0.0.1:5002
   VITE_IPFS_GATEWAY_URL=http://127.0.0.1:8081
   ```

2. 重启应用

#### 方法B：修改代码配置
修改以下文件中的默认端口：
- `src/services/agentService.js`
- `src/services/agentResolverService.js`
- `src/services/pubsubService.js`

将默认端口从 5001 改为其他可用端口（如 5002）。

### 方案3：配置 IPFS 使用不同端口

如果希望同时运行多个 IPFS 实例：

1. 为 Alou 创建独立的 IPFS 配置：
   ```bash
   # 创建新的 IPFS 仓库
   ipfs init --profile=test --empty-repo
   
   # 修改配置文件中的端口
   ipfs config Addresses.API /ip4/127.0.0.1/tcp/5002
   ipfs config Addresses.Gateway /ip4/127.0.0.1/tcp/8081
   ipfs config Addresses.Swarm '["/ip4/0.0.0.0/tcp/4002", "/ip6/::/tcp/4002", "/ip4/0.0.0.0/udp/4002/quic", "/ip6/::/udp/4002/quic"]'
   ```

2. 启动新的 IPFS 实例：
   ```bash
   ipfs daemon --enable-pubsub-experiment
   ```

3. 在 Alou 中配置使用新端口

### 方案4：使用 Docker 运行 IPFS

```bash
# 拉取 IPFS 镜像
docker pull ipfs/kubo:latest

# 运行容器（使用不同端口）
docker run -d \
  --name alou-ipfs \
  -p 5002:5001 \
  -p 8081:8080 \
  -p 4002:4001 \
  ipfs/kubo:latest
```

然后在 Alou 中配置：
```
VITE_IPFS_API_URL=http://127.0.0.1:5002
VITE_IPFS_GATEWAY_URL=http://127.0.0.1:8081
```

## 快速检查脚本

创建一个 `check-ports.js` 文件：

```javascript
const net = require('net');

const ports = [5001, 5002, 5003, 8080, 8081, 8082];

function checkPort(port) {
  return new Promise((resolve) => {
    const server = net.createServer();
    server.once('error', () => {
      resolve({ port, status: '占用' });
    });
    server.once('listening', () => {
      server.close();
      resolve({ port, status: '空闲' });
    });
    server.listen(port);
  });
}

async function checkAllPorts() {
  console.log('检查端口状态...');
  for (const port of ports) {
    const result = await checkPort(port);
    console.log(`端口 ${port}: ${result.status}`);
  }
}

checkAllPorts();
```

运行：`node check-ports.js`

## 推荐方案

对于大多数用户，推荐**方案1**（停止其他 IPFS 实例）或**方案2A**（通过环境变量修改端口）。

如果需要在开发时同时运行多个 IPFS 实例，使用**方案3**（配置不同端口）。

## 验证解决方案

解决后，重启 Alou 应用，应该不再看到端口冲突警告。
