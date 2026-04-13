#!/bin/bash
# Alou-cli Continuous Improvement Loop
# Runs kaizen evolution in a continuous loop with configurable intervals
# Usage: ./alou_loop.sh [iterations] [interval_seconds]

set -e

# Colors
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
MAX_ITERATIONS=${1:-0}  # 0 = unlimited
CHECK_INTERVAL=${2:-60} # 60 seconds default
LOG_FILE="alou_loop.log"
ITERATION=0
SUCCESS_COUNT=0
FAIL_COUNT=0

# Logging
log() {
    local ts=$(date '+%Y-%m-%d %H:%M:%S')
    echo -e "${ts} $1" | tee -a "$LOG_FILE"
}

# Cleanup on Ctrl+C
cleanup() {
    echo -e "\n${YELLOW}╔═══════════════════════════════════════╗${NC}"
    echo -e "${YELLOW}║   🛑 Alou Loop Stopped                ║${NC}"
    echo -e "${YELLOW}╚═══════════════════════════════════════╝${NC}"
    log "${BLUE}📊 Session Summary:${NC}"
    log "   Total iterations: $ITERATION"
    log "   ${GREEN}✓ Successful: $SUCCESS_COUNT${NC}"
    log "   ${RED}✗ Failed: $FAIL_COUNT${NC}"
    exit 0
}

trap cleanup SIGINT SIGTERM

# Header
echo -e "${CYAN}╔═══════════════════════════════════════╗${NC}"
echo -e "${CYAN}║   🔄 Alou-CLI Continuous Loop         ║${NC}"
echo -e "${CYAN}╚═══════════════════════════════════════╝${NC}"
echo
log "${GREEN}🚀 Starting continuous improvement loop${NC}"
log "   Interval: ${CHECK_INTERVAL}s | Max iterations: ${MAX_ITERATIONS:-∞}"
echo

# Main loop
while true; do
    ITERATION=$((ITERATION + 1))
    
    # Check iteration limit
    if [ "$MAX_ITERATIONS" -gt 0 ] && [ "$ITERATION" -gt "$MAX_ITERATIONS" ]; then
        log "${GREEN}✅ Completed $MAX_ITERATIONS iterations${NC}"
        break
    fi
    
    echo -e "\n${CYAN}━━━ Iteration $ITERATION ━━━${NC}"
    log "${BLUE}🔍 Running kaizen evolution...${NC}"
    
    # Run kaizen evolution
    if cargo run --release --manifest-path alou-cli/Cargo.toml -- kaizen evolution \
        --iterations 1 \
        --task "Auto-improve code quality and fix issues" 2>&1 | tee -a "$LOG_FILE"; then
        
        SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
        log "${GREEN}✅ Iteration $ITERATION completed successfully${NC}"
    else
        FAIL_COUNT=$((FAIL_COUNT + 1))
        log "${RED}❌ Iteration $ITERATION failed${NC}"
    fi
    
    # Show next iteration countdown
    if [ "$MAX_ITERATIONS" -eq 0 ] || [ "$ITERATION" -lt "$MAX_ITERATIONS" ]; then
        log "${YELLOW}⏳ Next iteration in ${CHECK_INTERVAL}s... (Ctrl+C to stop)${NC}"
        sleep "$CHECK_INTERVAL"
    fi
done

echo -e "\n${GREEN}╔═══════════════════════════════════════╗${NC}"
echo -e "${GREEN}║   ✓ Loop Completed Successfully       ║${NC}"
echo -e "${GREEN}╚═══════════════════════════════════════╝${NC}"
log "${BLUE}📊 Final Summary:${NC}"
log "   Total iterations: $ITERATION"
log "   ${GREEN}✓ Successful: $SUCCESS_COUNT${NC}"
log "   ${RED}✗ Failed: $FAIL_COUNT${NC}"
