#!/bin/bash

# 快速服务器部署脚本 - 跳过环境检查
set -e

# 配置变量
APP_NAME="alou-pay"
APP_DIR="/opt/${APP_NAME}"
SERVER_IP="140.179.151.58"
BACKEND_PORT=3001

echo "🚀 开始快速部署..."

# 停止现有服务
echo "🛑 停止现有服务..."
pm2 stop alou-backend 2>/dev/null || true
pm2 delete alou-backend 2>/dev/null || true
sudo systemctl stop nginx 2>/dev/null || true

# 构建应用
echo "🔨 构建应用..."

# 构建后端
echo "📦 构建 Rust 后端..."
source $HOME/.cargo/env 2>/dev/null || true
cargo build --release --bin agent-http-server

# 构建前端
echo "🎨 构建 Vue 前端..."
cd frontend
npm run build
cd ..

# 部署文件
echo "📁 部署文件..."

# 创建目录
sudo mkdir -p ${APP_DIR}/{backend,frontend,logs}
sudo chown -R $USER:$USER ${APP_DIR}

# 部署后端
cp target/release/agent-http-server ${APP_DIR}/backend/
cp agent_config.json ${APP_DIR}/backend/ 2>/dev/null || echo "⚠️ 使用默认agent配置"
cp mcp.json ${APP_DIR}/backend/ 2>/dev/null || echo "⚠️ 使用默认mcp配置"

# 创建环境文件
cat > ${APP_DIR}/backend/.env << EOF
PORT=${BACKEND_PORT}
RUST_LOG=info
DEEPSEEK_API_KEY=${DEEPSEEK_API_KEY:-your-api-key-here}
EOF

# 部署前端
sudo rm -rf ${APP_DIR}/frontend/*
sudo cp -r frontend/dist/* ${APP_DIR}/frontend/
sudo chown -R www-data:www-data ${APP_DIR}/frontend

# 配置PM2
echo "⚙️ 配置PM2..."
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
    log_file: '${APP_DIR}/logs/backend.log',
    error_file: '${APP_DIR}/logs/backend-error.log',
    out_file: '${APP_DIR}/logs/backend-out.log',
    time: true
  }]
};
EOF

# 配置Nginx
echo "🌐 配置Nginx..."
sudo tee /etc/nginx/sites-available/${APP_NAME} > /dev/null << EOF
server {
    listen 80;
    server_name ${SERVER_IP} localhost;
    
    # 前端静态文件
    location / {
        root ${APP_DIR}/frontend;
        try_files \$uri \$uri/ /index.html;
        index index.html;
        
        # 静态资源缓存
        location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2|ttf|eot)$ {
            expires 1y;
            add_header Cache-Control "public, immutable";
        }
    }

    # API代理
    location /api/ {
        proxy_pass http://127.0.0.1:${BACKEND_PORT};
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        
        # CORS
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
}
EOF

# 启用Nginx站点
sudo ln -sf /etc/nginx/sites-available/${APP_NAME} /etc/nginx/sites-enabled/
sudo rm -f /etc/nginx/sites-enabled/default

# 测试Nginx配置
echo "🧪 测试Nginx配置..."
sudo nginx -t

# 启动服务
echo "🚀 启动服务..."
cd ${APP_DIR}
pm2 start ecosystem.config.js
pm2 save

# 重启Nginx
sudo systemctl restart nginx

# 等待服务启动
echo "⏳ 等待服务启动..."
sleep 5

# 验证部署
echo "🧪 验证部署..."

# 检查后端
if curl -s http://localhost:${BACKEND_PORT}/api/health > /dev/null; then
    echo "✅ 后端服务正常"
else
    echo "❌ 后端服务异常"
    echo "查看后端日志:"
    pm2 logs alou-backend --lines 10
fi

# 检查前端代理
if curl -s http://localhost/api/health > /dev/null; then
    echo "✅ 前端代理正常"
else
    echo "❌ 前端代理异常"
    echo "Nginx状态:"
    sudo systemctl status nginx --no-pager -l
fi

# 显示服务状态
echo "📊 服务状态："
pm2 status

echo ""
echo "🎉 部署完成！"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🌐 访问地址:"
echo "   前端: http://${SERVER_IP}"
echo "   API:  http://${SERVER_IP}/api/health"
echo ""
echo "🔧 管理命令:"
echo "   pm2 status                    # 查看服务状态"
echo "   pm2 logs alou-backend         # 查看后端日志"
echo "   pm2 restart alou-backend      # 重启后端"
echo "   sudo systemctl status nginx  # 查看Nginx状态"
echo ""
echo "🧪 测试命令:"
echo "   curl http://${SERVER_IP}/api/health"
echo "   curl http://${SERVER_IP}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
