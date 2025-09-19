#!/bin/bash

# Alou服务管理脚本
APP_NAME="alou-pay"
BACKEND_SERVICE="alou-backend"
SERVER_IP="140.179.151.58"

case "$1" in
    start)
        echo "🚀 启动服务..."
        pm2 start ${BACKEND_SERVICE}
        sudo systemctl start nginx
        echo "✅ 服务已启动"
        ;;
    stop)
        echo "🛑 停止服务..."
        pm2 stop ${BACKEND_SERVICE}
        sudo systemctl stop nginx
        echo "✅ 服务已停止"
        ;;
    restart)
        echo "🔄 重启服务..."
        pm2 restart ${BACKEND_SERVICE}
        sudo systemctl restart nginx
        echo "✅ 服务已重启"
        ;;
    status)
        echo "📊 服务状态："
        pm2 status
        echo ""
        sudo systemctl status nginx --no-pager -l
        ;;
    logs)
        echo "📝 查看后端日志："
        pm2 logs ${BACKEND_SERVICE}
        ;;
    nginx-logs)
        echo "📝 查看Nginx日志："
        sudo tail -f /var/log/nginx/alou-pay.access.log
        ;;
    health)
        echo "🏥 健康检查："
        echo "后端直连："
        curl -s http://localhost:3001/api/health | jq . 2>/dev/null || echo "后端API不可用"
        echo ""
        echo "前端代理："
        curl -s http://localhost/api/health | jq . 2>/dev/null || echo "前端代理不可用"
        echo ""
        echo "外部访问："
        curl -s http://${SERVER_IP}/api/health | jq . 2>/dev/null || echo "外部访问不可用"
        ;;
    update)
        echo "🔄 更新应用..."
        git pull
        cargo build --release --bin agent-http-server
        cd frontend && npm run build && cd ..
        pm2 restart ${BACKEND_SERVICE}
        echo "✅ 更新完成"
        ;;
    deploy)
        echo "🚀 重新部署..."
        ./deploy/cloud_deploy.sh
        ;;
    backup)
        echo "💾 备份配置..."
        BACKUP_DIR="/opt/backup/alou-$(date +%Y%m%d-%H%M%S)"
        sudo mkdir -p ${BACKUP_DIR}
        sudo cp -r /opt/alou-pay ${BACKUP_DIR}/
        sudo cp /etc/nginx/sites-available/alou-pay ${BACKUP_DIR}/
        echo "备份完成: ${BACKUP_DIR}"
        ;;
    clean)
        echo "🧹 清理日志..."
        pm2 flush
        sudo truncate -s 0 /var/log/nginx/alou-pay.access.log
        sudo truncate -s 0 /var/log/nginx/alou-pay.error.log
        echo "✅ 日志已清理"
        ;;
    monitor)
        echo "📈 实时监控..."
        pm2 monit
        ;;
    *)
        echo "Alou 服务管理脚本"
        echo ""
        echo "用法: $0 {command}"
        echo ""
        echo "可用命令："
        echo "  start        启动所有服务"
        echo "  stop         停止所有服务"
        echo "  restart      重启所有服务"
        echo "  status       查看服务状态"
        echo "  logs         查看后端日志"
        echo "  nginx-logs   查看Nginx日志"
        echo "  health       健康检查"
        echo "  update       更新应用"
        echo "  deploy       重新部署"
        echo "  backup       备份配置"
        echo "  clean        清理日志"
        echo "  monitor      实时监控"
        echo ""
        echo "示例："
        echo "  $0 status    # 查看服务状态"
        echo "  $0 health    # 检查服务健康状态"
        echo "  $0 logs      # 查看实时日志"
        exit 1
        ;;
esac
