# Alou Desktop Linux 构建产物

## 📦 版本信息
- **版本**: 0.1.11
- **构建时间**: 2026-03-04 19:31 GMT+8
- **构建环境**: Linux 5.15.0-125-generic x86_64
- **Node.js**: v22.22.0
- **npm**: 10.9.4

## 🏗️ 可用构建

### 1. Alou-Desktop-0.1.11-web.tar.gz (推荐)
**Web部署版本** - 开箱即用

**特点**:
- ✅ 完整的React前端构建
- ✅ 内置Node.js HTTP服务器
- ✅ 一键启动脚本 (`start.sh`)
- ✅ 系统安装脚本 (`install.sh`)
- ✅ 无需编译，直接运行

**使用方法**:
```bash
# 下载并解压
tar -xzf Alou-Desktop-0.1.11-web.tar.gz
cd Alou-Desktop-0.1.11-web

# 启动服务器
./start.sh

# 在浏览器中访问
# http://localhost:3000
```

### 2. Alou-0.1.11-linux.tar.gz
**源代码包** - 完整项目源代码

**特点**:
- ✅ 完整的项目源代码
- ✅ 前端和后端代码
- ✅ 配置文件和文档
- ✅ 需要Node.js开发环境

**使用方法**:
```bash
# 解压
tar -xzf Alou-0.1.11-linux.tar.gz
cd Alou-0.1.11-linux

# 安装依赖
npm install

# 开发模式
npm run dev
# 访问 http://localhost:1420
```

## 🚀 快速开始

### 系统要求
- **操作系统**: Linux (Ubuntu/Debian/CentOS/Fedora)
- **Node.js**: 18.0.0+
- **内存**: 2GB+ RAM
- **存储**: 500MB+ 可用空间

### 5分钟部署
```bash
# 1. 下载Web版本
wget [文件地址]/Alou-Desktop-0.1.11-web.tar.gz

# 2. 解压
tar -xzf Alou-Desktop-0.1.11-web.tar.gz

# 3. 启动
cd Alou-Desktop-0.1.11-web
./start.sh

# 4. 访问
# 打开浏览器: http://localhost:3000
```

## 🔧 功能特性

### 核心功能
- 🤖 **AI智能体协作** - 多Agent协同工作
- 🔗 **Web3集成** - 以太坊/Solana钱包支持
- 🌐 **去中心化群聊** - 基于IPFS PubSub
- 🧠 **DeepSeek/Claude集成** - 智能对话

### 技术架构
- **前端**: React + TypeScript + Vite
- **UI框架**: 自定义组件
- **状态管理**: Zustand
- **HTTP客户端**: Axios
- **Web3**: ethers.js + WalletConnect

## 📋 部署指南

### 开发环境
```bash
# 从源代码开始
tar -xzf Alou-0.1.11-linux.tar.gz
cd Alou-0.1.11-linux
npm install
npm run dev
```

### 生产环境
```bash
# 使用Web版本
tar -xzf Alou-Desktop-0.1.11-web.tar.gz
cd Alou-Desktop-0.1.11-web

# 使用PM2管理 (推荐)
npm install -g pm2
pm2 start start-server.cjs --name "alou-desktop"

# 或使用systemd
sudo cp alou-desktop.service /etc/systemd/system/
sudo systemctl enable alou-desktop
sudo systemctl start alou-desktop
```

## 🐛 故障排除

### 常见问题
1. **端口占用**: 修改 `start-server.cjs` 中的 `PORT` 变量
2. **Node.js版本**: 确保使用Node.js 18+
3. **权限问题**: 给脚本执行权限 `chmod +x *.sh`
4. **依赖缺失**: 运行 `npm install` 安装依赖

### 错误解决
```bash
# 检查Node.js版本
node --version

# 检查端口占用
netstat -tlnp | grep :3000

# 查看日志
tail -f start-server.log
```

## 🔄 更新与维护

### 构建信息
- **构建时间**: 2026-03-04
- **Git提交**: wasm分支最新版本
- **构建者**: 天慕之舞 (Skygaze Dancer) 😜

### 更新计划
- 定期构建新版本
- 添加更多Linux发行版支持
- 优化构建脚本

## 📞 支持与反馈

### 获取帮助
1. 查看项目主README: `../../README.md`
2. 检查控制台输出
3. 参考构建日志

### 报告问题
- GitHub Issues: https://github.com/logos-42/alou-rust/issues
- 提供系统信息和错误日志

## 📄 许可证
MIT License - 详见项目根目录LICENSE文件

---

**构建备注**: 此构建产物基于wasm分支源码，专为Linux系统优化。😜