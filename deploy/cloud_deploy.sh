#!/bin/bash

# 云服务器部署脚本 - Alou智能助手
set -e

# 配置变量
APP_NAME="alou-pay"
APP_DIR="/opt/${APP_NAME}"
SERVER_IP="140.179.151.58"  # 你的云服务器IP
BACKEND_PORT=3001

echo "🚀 开始云服务器部署..."

# 检查环境
check_environment() {
    echo "🔍 检查部署环境..."
    
    # 检查必要的命令
    command -v cargo >/dev/null 2>&1 || { echo "❌ 需要安装 Rust"; exit 1; }
    command -v npm >/dev/null 2>&1 || { echo "❌ 需要安装 Node.js"; exit 1; }
    command -v nginx >/dev/null 2>&1 || { echo "❌ 需要安装 Nginx"; exit 1; }
    command -v pm2 >/dev/null 2>&1 || { echo "❌ 需要安装 PM2"; exit 1; }
    
    echo "✅ 环境检查通过"
}

# 停止现有服务
stop_services() {
    echo "🛑 停止现有服务..."
    pm2 stop alou-backend 2>/dev/null || true
    pm2 delete alou-backend 2>/dev/null || true
    sudo systemctl stop nginx 2>/dev/null || true
}

# 构建应用
build_application() {
    echo "🔨 构建应用..."
    
    # 检查项目根目录
    if [ ! -f "Cargo.toml" ] || [ ! -d "frontend" ]; then
        echo "❌ 请在项目根目录运行此脚本"
        exit 1
    fi
    
    # 构建后端
    echo "📦 构建 Rust 后端..."
    source $HOME/.cargo/env 2>/dev/null || true
    cargo build --release --bin agent-http-server
    
    # 构建前端
    echo "🎨 构建 Vue 前端..."
    cd frontend
    npm ci --production=false
    npm run build
    cd ..
}

# 部署文件
deploy_files() {
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
}

# 配置服务
configure_services() {
    echo "⚙️ 配置服务..."
    
    # PM2配置
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
    time: true,
    env: {
      NODE_ENV: 'production'
    }
  }]
};
EOF

    # Nginx配置
    sudo tee /etc/nginx/sites-available/${APP_NAME} > /dev/null << EOF
server {
    listen 80;
    server_name ${SERVER_IP} localhost;
    
    # 安全配置
    server_tokens off;
    client_max_body_size 10M;
    
    # Gzip压缩
    gzip on;
    gzip_vary on;
    gzip_min_length 1024;
    gzip_types text/plain text/css text/xml text/javascript application/javascript application/xml+rss application/json;

    # 前端静态文件
    location / {
        root ${APP_DIR}/frontend;
        try_files \$uri \$uri/ /index.html;
        index index.html;
        
        # 静态资源缓存
        location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2|ttf|eot)$ {
            expires 1y;
            add_header Cache-Control "public, immutable";
            access_log off;
        }
        
        # HTML不缓存
        location ~* \.html$ {
            add_header Cache-Control "no-cache, no-store, must-revalidate";
            add_header Pragma "no-cache";
            add_header Expires "0";
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
        proxy_cache_bypass \$http_upgrade;
        
        # 超时配置
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
        
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

    # 安全头部
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header Referrer-Policy "no-referrer-when-downgrade" always;
    add_header Content-Security-Policy "default-src 'self' http: https: data: blob: 'unsafe-inline' 'unsafe-eval'" always;

    # 日志
    access_log /var/log/nginx/alou-pay.access.log;
    error_log /var/log/nginx/alou-pay.error.log;
}
EOF
    
    # 启用站点
    sudo ln -sf /etc/nginx/sites-available/${APP_NAME} /etc/nginx/sites-enabled/
    sudo rm -f /etc/nginx/sites-enabled/default
}

# 启动服务
start_services() {
    echo "🚀 启动服务..."
    
    # 测试Nginx配置
    sudo nginx -t
    
    # 启动后端
    cd ${APP_DIR}
    pm2 start ecosystem.config.js
    pm2 save
    
    # 启动Nginx
    sudo systemctl restart nginx
    sudo systemctl enable nginx
}

# 验证部署
verify_deployment() {
    echo "🧪 验证部署..."
    
    # 等待服务启动
    sleep 5
    
    # 检查后端
    if curl -s http://localhost:${BACKEND_PORT}/api/health > /dev/null; then
        echo "✅ 后端服务正常"
    else
        echo "❌ 后端服务异常"
        pm2 logs alou-backend --lines 10
    fi
    
    # 检查前端
    if curl -s http://localhost/api/health > /dev/null; then
        echo "✅ 前端代理正常"
    else
        echo "❌ 前端代理异常"
        sudo nginx -t
    fi
    
    # 显示状态
    echo "📊 服务状态："
    pm2 status
    sudo systemctl status nginx --no-pager -l
}

# 显示部署信息
show_deployment_info() {
    echo ""
    echo "🎉 部署完成！"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🌐 访问地址:"
    echo "   前端: http://${SERVER_IP}"
    echo "   API:  http://${SERVER_IP}/api/health"
    echo ""
    echo "📁 文件位置:"
    echo "   应用目录: ${APP_DIR}"
    echo "   日志目录: ${APP_DIR}/logs"
    echo "   Nginx配置: /etc/nginx/sites-available/${APP_NAME}"
    echo ""
    echo "🔧 管理命令:"
    echo "   pm2 status                    # 查看服务状态"
    echo "   pm2 logs alou-backend         # 查看后端日志"
    echo "   pm2 restart alou-backend      # 重启后端"
    echo "   sudo systemctl status nginx  # 查看Nginx状态"
    echo "   sudo systemctl restart nginx # 重启Nginx"
    echo ""
    echo "🧪 测试命令:"
    echo "   curl http://${SERVER_IP}/api/health"
    echo "   curl http://${SERVER_IP}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}

# 主执行流程
main() {
    check_environment
    stop_services
    build_application
    deploy_files
    configure_services
    start_services
    verify_deployment
    show_deployment_info
}

# 执行部署
main "$@"
