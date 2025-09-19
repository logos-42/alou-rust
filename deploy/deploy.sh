#!/bin/bash

# Alou Pay 部署脚本
# 用于部署 Rust 后端和 Vue 前端到 Ubuntu 服务器

set -e

# 配置变量
APP_NAME="alou-pay"
APP_DIR="/opt/${APP_NAME}"
BACKEND_PORT=3001
FRONTEND_PORT=80
DOMAIN="140.179.105.56" # 您的服务器IP

echo "🚀 开始部署 ${APP_NAME}..."

# 检查是否在项目根目录
if [ ! -f "Cargo.toml" ] || [ ! -d "frontend" ]; then
    echo "❌ 请在项目根目录运行此脚本"
    exit 1
fi

# 停止现有服务
echo "🛑 停止现有服务..."
pm2 stop alou-backend 2>/dev/null || true
pm2 delete alou-backend 2>/dev/null || true
sudo systemctl stop nginx 2>/dev/null || true

# 创建应用目录
echo "📁 准备应用目录..."
sudo mkdir -p ${APP_DIR}/{backend,frontend}
sudo chown -R $USER:$USER ${APP_DIR}

# 构建 Rust 后端
echo "🦀 构建 Rust 后端..."
source $HOME/.cargo/env
cargo build --release --bin agent-http-server

# 复制后端文件
echo "📦 部署 Rust 后端..."
cp target/release/agent-http-server ${APP_DIR}/backend/
cp agent_config.json ${APP_DIR}/backend/ 2>/dev/null || echo "⚠️ agent_config.json 不存在，将使用默认配置"
cp mcp.json ${APP_DIR}/backend/ 2>/dev/null || echo "⚠️ mcp.json 不存在，将使用默认配置"

# 创建后端环境文件
echo "🔧 创建后端环境配置..."
cat > ${APP_DIR}/backend/.env << EOF
PORT=${BACKEND_PORT}
RUST_LOG=info
DEEPSEEK_API_KEY=${DEEPSEEK_API_KEY:-your-api-key-here}
EOF

# 构建 Vue 前端
echo "🎨 构建 Vue 前端..."
cd frontend
npm ci
npm run build
cd ..

# 部署前端文件
echo "📦 部署 Vue 前端..."
sudo rm -rf ${APP_DIR}/frontend/*
sudo cp -r frontend/dist/* ${APP_DIR}/frontend/

# 创建 PM2 配置
echo "⚡ 配置 PM2..."
cat > ${APP_DIR}/ecosystem.config.js << EOF
module.exports = {
  apps: [{
    name: 'alou-backend',
    script: '${APP_DIR}/backend/agent-http-server',
    cwd: '${APP_DIR}/backend',
    env_file: '${APP_DIR}/backend/.env',
    instances: 1,
    exec_mode: 'fork',
    autorestart: true,
    watch: false,
    max_memory_restart: '1G',
    log_file: '/var/log/alou-pay/backend.log',
    error_file: '/var/log/alou-pay/backend-error.log',
    out_file: '/var/log/alou-pay/backend-out.log',
    time: true
  }]
};
EOF

# 配置 Nginx
echo "🌐 配置 Nginx..."
sudo tee /etc/nginx/sites-available/${APP_NAME} > /dev/null << EOF
server {
    listen 80;
    server_name ${DOMAIN} localhost;

    # 前端静态文件
    location / {
        root ${APP_DIR}/frontend;
        try_files \$uri \$uri/ /index.html;
        index index.html;
        
        # 缓存静态资源
        location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2|ttf|eot)$ {
            expires 1y;
            add_header Cache-Control "public, immutable";
        }
    }

    # API 代理到 Rust 后端
    location /api/ {
        proxy_pass http://127.0.0.1:${BACKEND_PORT};
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        proxy_cache_bypass \$http_upgrade;
        
        # CORS 头部
        add_header 'Access-Control-Allow-Origin' '*' always;
        add_header 'Access-Control-Allow-Methods' 'GET, POST, OPTIONS' always;
        add_header 'Access-Control-Allow-Headers' 'Content-Type, Authorization' always;
        
        if (\$request_method = 'OPTIONS') {
            return 204;
        }
    }

    # 健康检查
    location /health {
        proxy_pass http://127.0.0.1:${BACKEND_PORT}/api/health;
        access_log off;
    }

    # 安全头部
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header Referrer-Policy "no-referrer-when-downgrade" always;
    add_header Content-Security-Policy "default-src 'self' http: https: data: blob: 'unsafe-inline'" always;
}
EOF

# 启用 Nginx 站点
sudo ln -sf /etc/nginx/sites-available/${APP_NAME} /etc/nginx/sites-enabled/
sudo rm -f /etc/nginx/sites-enabled/default

# 测试 Nginx 配置
echo "🧪 测试 Nginx 配置..."
sudo nginx -t

# 启动服务
echo "🚀 启动服务..."
cd ${APP_DIR}
pm2 start ecosystem.config.js
pm2 save

# 重启 Nginx
sudo systemctl restart nginx

# 显示状态
echo "📊 服务状态："
pm2 status
sudo systemctl status nginx --no-pager -l

# 显示部署信息
echo ""
echo "🎉 部署完成！"
echo "📍 应用目录: ${APP_DIR}"
echo "🌐 前端访问: http://${DOMAIN}"
echo "🔌 后端API: http://${DOMAIN}/api/health"
echo "📝 后端日志: /var/log/alou-pay/"
echo ""
echo "🔧 管理命令:"
echo "  pm2 status              # 查看后端状态"
echo "  pm2 logs alou-backend   # 查看后端日志"
echo "  pm2 restart alou-backend # 重启后端"
echo "  sudo systemctl status nginx # 查看 Nginx 状态"
echo "  sudo nginx -t           # 测试 Nginx 配置"
echo ""
echo "🧪 测试部署:"
echo "  curl http://${DOMAIN}/api/health"
echo "  curl http://${DOMAIN}"
