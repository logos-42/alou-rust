#!/bin/bash

# Alou Pay 服务器环境设置脚本
# 适用于 Ubuntu 24.04 LTS

set -e

echo "🚀 开始设置 Alou Pay 服务器环境..."

# 更新系统包
echo "📦 更新系统包..."
sudo apt update && sudo apt upgrade -y

# 安装基础工具
echo "🔧 安装基础工具..."
sudo apt install -y curl wget git build-essential pkg-config libssl-dev unzip

# 安装 Rust
echo "🦀 安装 Rust..."
if ! command -v rustc &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
    rustup default stable
    echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
else
    echo "Rust 已安装，跳过..."
fi

export DEEPSEEK_API_KEY="sk-69244d21fb1f4481bd4b36ed9bb59b18"
echo 'export DEEPSEEK_API_KEY="sk-69244d21fb1f4481bd4b36ed9bb59b18"' >> ~/.bashrc


# 安装 Node.js (使用 NodeSource 仓库)
echo "📦 安装 Node.js..."
if ! command -v node &> /dev/null; then
    curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
    sudo apt-get install -y nodejs
else
    echo "Node.js 已安装，跳过..."
fi

# 安装 Nginx
echo "🌐 安装 Nginx..."
if ! command -v nginx &> /dev/null; then
    sudo apt install -y nginx
    sudo systemctl enable nginx
else
    echo "Nginx 已安装，跳过..."
fi

# 安装 PM2 (用于进程管理)
echo "⚡ 安装 PM2..."
if ! command -v pm2 &> /dev/null; then
    sudo npm install -g pm2
    pm2 startup
    sudo env PATH=$PATH:/usr/bin /usr/lib/node_modules/pm2/bin/pm2 startup systemd -u $USER --hp $HOME
else
    echo "PM2 已安装，跳过..."
fi

# 创建应用目录
echo "📁 创建应用目录..."
sudo mkdir -p /opt/alou-pay
sudo chown $USER:$USER /opt/alou-pay

# 安装 uvx (用于 MCP 服务器)
echo "🔧 安装 uvx..."
if ! command -v uvx &> /dev/null; then
    curl -LsSf https://astral.sh/uv/install.sh | sh
    source $HOME/.cargo/env
    echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
else
    echo "uvx 已安装，跳过..."
fi

# 配置防火墙
echo "🔥 配置防火墙..."
sudo ufw allow ssh
sudo ufw allow 80
sudo ufw allow 443
sudo ufw allow 3001  # Rust 后端端口
sudo ufw --force enable

# 创建日志目录
echo "📝 创建日志目录..."
sudo mkdir -p /var/log/alou-pay
sudo chown $USER:$USER /var/log/alou-pay

# 显示版本信息
echo "✅ 环境设置完成！版本信息："
echo "Rust: $(rustc --version)"
echo "Node.js: $(node --version)"
echo "NPM: $(npm --version)"
echo "Nginx: $(nginx -v 2>&1)"
echo "PM2: $(pm2 --version)"

echo "🎉 服务器环境设置完成！"
echo "📍 应用目录: /opt/alou-pay"
echo "📍 日志目录: /var/log/alou-pay"
echo "🔄 请运行 'source ~/.bashrc' 或重新登录以应用环境变量"
