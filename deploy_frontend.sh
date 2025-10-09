#!/bin/bash

# 前端部署脚本
# 用于将构建好的前端文件部署到云服务器

echo "🚀 开始部署前端到云服务器..."

# 检查dist目录是否存在
if [ ! -d "frontend/dist" ]; then
    echo "❌ 错误: frontend/dist 目录不存在，请先运行 npm run build"
    exit 1
fi

# 服务器配置
SERVER_USER="ubuntu"
SERVER_HOST="your-server-ip"  # 请替换为你的服务器IP
SERVER_PATH="/var/www/alou-frontend"

echo "📦 准备部署文件..."

# 创建部署包
tar -czf frontend-dist.tar.gz -C frontend dist

echo "📤 上传文件到服务器..."

# 上传到服务器
scp frontend-dist.tar.gz ${SERVER_USER}@${SERVER_HOST}:/tmp/

echo "🔧 在服务器上部署..."

# 在服务器上执行部署命令
ssh ${SERVER_USER}@${SERVER_HOST} << 'EOF'
    echo "在服务器上开始部署..."
    
    # 创建目录
    sudo mkdir -p /var/www/alou-frontend
    
    # 解压文件
    cd /tmp
    sudo tar -xzf frontend-dist.tar.gz
    
    # 移动文件到目标目录
    sudo rm -rf /var/www/alou-frontend/*
    sudo mv dist/* /var/www/alou-frontend/
    
    # 设置权限
    sudo chown -R www-data:www-data /var/www/alou-frontend
    sudo chmod -R 755 /var/www/alou-frontend
    
    # 清理临时文件
    rm -f /tmp/frontend-dist.tar.gz
    rm -rf /tmp/dist
    
    echo "✅ 前端部署完成！"
    echo "📁 文件位置: /var/www/alou-frontend"
    echo "🌐 访问地址: http://your-server-ip/"
EOF

# 清理本地临时文件
rm -f frontend-dist.tar.gz

echo "🎉 前端部署完成！"
echo "📝 下一步："
echo "1. 配置Nginx反向代理"
echo "2. 启动HTTP服务器: cargo run --bin agent_http_server"
echo "3. 访问: http://your-server-ip/"

