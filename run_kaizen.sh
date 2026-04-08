#!/bin/bash
# Kaizen 自进化循环快速启动脚本
# 使用方法: ./run_kaizen.sh [evolution|research|status|log|stop]

set -e

# 颜色
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${CYAN}╔═══════════════════════════════════════╗${NC}"
echo -e "${CYAN}║   🔄 Kaizen 自进化循环快速启动        ║${NC}"
echo -e "${CYAN}╚═══════════════════════════════════════╝${NC}"
echo

# 检查 LLM API Key
if [ -z "$LLM_API_KEY" ]; then
    echo -e "${YELLOW}⚠️  未检测到 LLM_API_KEY 环境变量${NC}"
    echo
    
    # 尝试加载 .env 文件
    if [ -f ".env" ]; then
        echo -e "${GREEN}✓ 找到 .env 文件，加载配置...${NC}"
        export $(cat .env | grep -v '^#' | xargs)
    else
        echo -e "${RED}❌ 请先设置 LLM API Key:${NC}"
        echo "   方式 1: export LLM_API_KEY=your_key"
        echo "   方式 2: 创建 .env 文件包含 LLM_API_KEY=your_key"
        exit 1
    fi
fi

# 默认命令
COMMAND=${1:-help}
shift || true

case "$COMMAND" in
    evolution)
        echo -e "${GREEN}🚀 启动进化引擎...${NC}"
        echo
        cargo run --release -- kaizen evolution "$@"
        ;;
    
    research)
        echo -e "${GREEN}🔬 启动自动研究 (Karpathy 模式)...${NC}"
        echo
        cargo run --release -- kaizen research "$@"
        ;;
    
    status)
        echo -e "${GREEN}📊 查看 Kaizen 循环状态...${NC}"
        echo
        cargo run --release -- kaizen status
        ;;
    
    log)
        LINES=${1:-20}
        echo -e "${GREEN}📝 查看实验日志 (最近 $LINES 行)...${NC}"
        echo
        cargo run --release -- kaizen log $LINES
        ;;
    
    stop)
        echo -e "${YELLOW}🛑 停止 Kaizen 循环...${NC}"
        echo
        cargo run --release -- kaizen stop
        ;;
    
    help|*)
        echo -e "${CYAN}用法: ./run_kaizen.sh [命令] [选项]${NC}"
        echo
        echo -e "${CYAN}命令:${NC}"
        echo "  evolution [选项]    启动进化引擎"
        echo "  research [选项]     启动自动研究"
        echo "  status              查看状态"
        echo "  log [行数]          查看日志 (默认 20 行)"
        echo "  stop                停止循环"
        echo
        echo -e "${CYAN}进化引擎选项:${NC}"
        echo "  --iterations <N>    迭代次数 (默认: 5)"
        echo "  --task <描述>       任务描述"
        echo
        echo -e "${CYAN}自动研究选项:${NC}"
        echo "  --iterations <N>    迭代次数 (默认: 5)"
        echo "  --auto-push         自动推送到 GitHub"
        echo "  --strict            严格模式"
        echo "  --no-dry-run        禁用安全模式"
        echo "  --target <文件>     目标文件"
        echo
        echo -e "${CYAN}示例:${NC}"
        echo "  ./run_kaizen.sh evolution --iterations 10 --task \"优化性能\""
        echo "  ./run_kaizen.sh research --auto-push --no-dry-run"
        echo "  ./run_kaizen.sh status"
        echo "  ./run_kaizen.sh log 50"
        ;;
esac
