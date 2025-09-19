# 🚀 Alou云服务器部署指南

## 前置条件

### 服务器要求
- Ubuntu 20.04+ 或 CentOS 7+
- 至少 2GB RAM
- 至少 20GB 磁盘空间
- 开放端口：80, 443, 3001

### 需要安装的软件
```bash
# 更新系统
sudo apt update && sudo apt upgrade -y

# 安装基础工具
sudo apt install -y curl wget git build-essential

# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 安装 Node.js 20
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs

# 安装 PM2
sudo npm install -g pm2

# 安装 Nginx
sudo apt install -y nginx

# 安装 jq (用于JSON处理)
sudo apt install -y jq
```

## 部署步骤

### 1. 克隆项目
```bash
git clone <your-repo-url> alou-pay
cd alou-pay
```

### 2. 设置环境变量
```bash
export DEEPSEEK_API_KEY="your-actual-deepseek-api-key"
```

### 3. 执行部署
```bash
# 给脚本执行权限
chmod +x deploy/cloud_deploy.sh deploy/manage.sh

# 运行部署脚本
./deploy/cloud_deploy.sh
```

## 服务管理

### 使用管理脚本
```bash
# 查看服务状态
./deploy/manage.sh status

# 健康检查
./deploy/manage.sh health

# 查看日志
./deploy/manage.sh logs

# 重启服务
./deploy/manage.sh restart

# 更新应用
./deploy/manage.sh update
```

### 手动管理命令
```bash
# PM2 命令
pm2 status                    # 查看服务状态
pm2 logs alou-backend         # 查看后端日志
pm2 restart alou-backend      # 重启后端
pm2 stop alou-backend         # 停止后端
pm2 delete alou-backend       # 删除后端服务

# Nginx 命令
sudo systemctl status nginx   # 查看Nginx状态
sudo systemctl restart nginx  # 重启Nginx
sudo nginx -t                 # 测试Nginx配置
```

## 访问应用

部署完成后，可以通过以下地址访问：

- **前端页面**: `http://YOUR_SERVER_IP`
- **API健康检查**: `http://YOUR_SERVER_IP/api/health`
- **API测试**: `http://YOUR_SERVER_IP/api/test`

## 目录结构

```
/opt/alou-pay/
├── backend/
│   ├── agent_http_server     # Rust后端可执行文件
│   ├── agent_config.json     # 智能体配置
│   ├── mcp.json             # MCP服务器配置
│   └── .env                 # 环境变量
├── frontend/                # Vue前端构建文件
├── logs/                    # 应用日志
└── ecosystem.config.js      # PM2配置
```

## 配置文件

### Nginx配置位置
- 配置文件: `/etc/nginx/sites-available/alou-pay`
- 日志文件: `/var/log/nginx/alou-pay.*.log`

### PM2配置
- 配置文件: `/opt/alou-pay/ecosystem.config.js`
- 应用日志: `/opt/alou-pay/logs/`

## 故障排除

### 1. 端口被占用
```bash
# 查看端口占用
sudo netstat -tlnp | grep :3001
sudo netstat -tlnp | grep :80

# 杀死占用进程
sudo kill -9 <PID>
```

### 2. 权限问题
```bash
# 修复文件权限
sudo chown -R $USER:$USER /opt/alou-pay
sudo chown -R www-data:www-data /opt/alou-pay/frontend
```

### 3. 服务不启动
```bash
# 查看详细日志
pm2 logs alou-backend --lines 50
sudo journalctl -u nginx -f

# 检查配置
sudo nginx -t
pm2 describe alou-backend
```

### 4. API连接失败
```bash
# 测试后端直连
curl http://localhost:3001/api/health

# 测试Nginx代理
curl http://localhost/api/health

# 检查防火墙
sudo ufw status
sudo iptables -L
```

### 5. 前端页面空白
```bash
# 检查前端文件
ls -la /opt/alou-pay/frontend/

# 检查Nginx错误日志
sudo tail -f /var/log/nginx/alou-pay.error.log

# 重新构建前端
cd frontend && npm run build
sudo cp -r dist/* /opt/alou-pay/frontend/
```

## 性能优化

### 1. 启用HTTPS (可选)
```bash
# 安装 Certbot
sudo apt install certbot python3-certbot-nginx

# 获取SSL证书
sudo certbot --nginx -d your-domain.com

# 自动续期
sudo crontab -e
# 添加: 0 12 * * * /usr/bin/certbot renew --quiet
```

### 2. 设置防火墙
```bash
sudo ufw allow ssh
sudo ufw allow http
sudo ufw allow https
sudo ufw enable
```

### 3. 监控设置
```bash
# PM2监控
pm2 install pm2-logrotate  # 日志轮转
pm2 startup               # 开机启动
pm2 save                  # 保存配置
```

## 备份策略

### 自动备份脚本
```bash
#!/bin/bash
# /opt/backup_alou.sh

DATE=$(date +%Y%m%d-%H%M%S)
BACKUP_DIR="/opt/backup/alou-${DATE}"

mkdir -p ${BACKUP_DIR}
cp -r /opt/alou-pay ${BACKUP_DIR}/
cp /etc/nginx/sites-available/alou-pay ${BACKUP_DIR}/

# 保留最近7天的备份
find /opt/backup -name "alou-*" -mtime +7 -exec rm -rf {} \;
```

### 定时备份
```bash
# 添加到crontab
sudo crontab -e
# 添加: 0 2 * * * /opt/backup_alou.sh
```

## 更新流程

### 应用更新
```bash
cd alou-pay
git pull
./deploy/manage.sh update
```

### 系统更新
```bash
sudo apt update && sudo apt upgrade -y
sudo systemctl restart nginx
./deploy/manage.sh restart
```
