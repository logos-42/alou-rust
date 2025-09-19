# Alou Pay 部署指南

本指南将帮助您将 Alou Pay 应用部署到 AWS Ubuntu 24.04 服务器。

## 🖥️ 服务器信息

- **实例ID**: i-0856990788dcf5041
- **公网IP**: 140.179.186.11
- **内网IP**: 172.31.0.76
- **系统**: Ubuntu 24.04.2 LTS
- **架构**: x86_64

## 📋 部署步骤

### 第一步：连接到服务器

```bash
ssh -i alou.pem ubuntu@140.179.186.11
```

### 第二步：设置服务器环境

```bash
# 下载项目代码
git clone <your-repository-url>
cd mcp_client_rust

# 给脚本执行权限
chmod +x deploy/*.sh

# 运行环境设置脚本
./deploy/setup_server.sh

# 重新加载环境变量
source ~/.bashrc
```

### 第三步：配置环境变量

```bash
# 设置 DeepSeek API Key
export DEEPSEEK_API_KEY="your-actual-api-key-here"

# 将环境变量添加到 ~/.bashrc
echo 'export DEEPSEEK_API_KEY="your-actual-api-key-here"' >> ~/.bashrc
```

### 第四步：部署应用

```bash
# 运行部署脚本
./deploy/deploy.sh
```

### 第五步：验证部署

```bash
# 检查后端服务状态
pm2 status

# 检查 Nginx 状态
sudo systemctl status nginx

# 测试 API 端点
curl http://140.179.186.11/api/health

# 测试前端
curl http://140.179.186.11
```

## 🔧 可选配置

### 使用 systemd 服务（替代 PM2）

如果您希望使用 systemd 而不是 PM2 来管理 Rust 后端：

```bash
# 停止 PM2 服务
pm2 stop alou-backend
pm2 delete alou-backend

# 安装 systemd 服务
sudo cp deploy/alou-pay.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable alou-pay
sudo systemctl start alou-pay

# 检查状态
sudo systemctl status alou-pay
```

### SSL 证书配置（可选）

如果您有域名并希望启用 HTTPS：

```bash
# 安装 Certbot
sudo apt install -y certbot python3-certbot-nginx

# 获取 SSL 证书（替换 your-domain.com）
sudo certbot --nginx -d your-domain.com

# 自动续期
sudo crontab -e
# 添加以下行：
# 0 12 * * * /usr/bin/certbot renew --quiet
```

## 📊 监控和日志

### PM2 管理命令

```bash
# 查看所有进程状态
pm2 status

# 查看实时日志
pm2 logs alou-backend

# 重启服务
pm2 restart alou-backend

# 查看进程详情
pm2 show alou-backend

# 监控资源使用
pm2 monit
```

### 系统日志

```bash
# 应用日志
tail -f /var/log/alou-pay/backend.log
tail -f /var/log/alou-pay/backend-error.log

# Nginx 日志
tail -f /var/log/nginx/alou-pay.access.log
tail -f /var/log/nginx/alou-pay.error.log

# 系统服务日志（如果使用 systemd）
sudo journalctl -u alou-pay -f
```

## 🔄 更新部署

当您需要更新应用时：

```bash
# 拉取最新代码
git pull origin main

# 运行更新脚本
./deploy/update.sh
```

## 🚨 故障排除

### 常见问题

#### 1. 后端服务无法启动

```bash
# 检查端口是否被占用
sudo netstat -tlnp | grep :3001

# 检查配置文件
cat /opt/alou-pay/backend/.env

# 查看详细错误日志
pm2 logs alou-backend --lines 50
```

#### 2. Nginx 配置错误

```bash
# 测试 Nginx 配置
sudo nginx -t

# 重新加载配置
sudo systemctl reload nginx

# 检查错误日志
sudo tail -f /var/log/nginx/error.log
```

#### 3. API 请求失败

```bash
# 检查后端是否运行
curl http://localhost:3001/api/health

# 检查防火墙
sudo ufw status

# 检查端口监听
sudo ss -tlnp | grep :3001
```

#### 4. 前端页面无法加载

```bash
# 检查文件权限
ls -la /opt/alou-pay/frontend/

# 检查 Nginx 配置
sudo nginx -t

# 重启 Nginx
sudo systemctl restart nginx
```

### 性能调优

#### 增加系统资源限制

```bash
# 编辑系统限制
sudo nano /etc/security/limits.conf

# 添加以下行：
ubuntu soft nofile 65536
ubuntu hard nofile 65536
```

#### 优化 Nginx

```bash
# 编辑 Nginx 主配置
sudo nano /etc/nginx/nginx.conf

# 调整 worker 进程数和连接数
worker_processes auto;
worker_connections 1024;
```

## 🌐 访问应用

部署完成后，您可以通过以下地址访问应用：

- **前端**: http://140.179.186.11
- **API健康检查**: http://140.179.186.11/api/health
- **聊天API**: http://140.179.186.11/api/chat

## 📞 支持

如果在部署过程中遇到问题，请检查：

1. 服务器日志：`/var/log/alou-pay/`
2. PM2 日志：`pm2 logs`
3. Nginx 日志：`/var/log/nginx/`
4. 系统日志：`sudo journalctl -xe`

## 🔐 安全建议

1. **定期更新系统**：
   ```bash
   sudo apt update && sudo apt upgrade -y
   ```

2. **配置防火墙**：
   ```bash
   sudo ufw status
   sudo ufw enable
   ```

3. **使用强密码和密钥认证**

4. **定期备份重要数据**

5. **监控系统资源使用情况**

6. **考虑使用 HTTPS**（如果有域名）
