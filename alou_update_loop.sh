#!/bin/bash
# Alou-cli Auto-Update Loop
# This script runs alou-cli in a loop to automatically check and apply code updates
# Usage: ./alou_update_loop.sh [options]

set -e

# Colors
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Configuration
CHECK_INTERVAL=${CHECK_INTERVAL:-300}  # 5 minutes default
MAX_ITERATIONS=${MAX_ITERATIONS:-0}    # 0 = unlimited
LOG_FILE="${LOG_FILE:-alou_update_loop.log}"
ALOU_CLI_PATH="${ALOU_CLI_PATH:-./alou-cli}"

# Counters
iteration=0
updates_applied=0
errors=0

# Logging function
log() {
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo -e "${timestamp} - $1" | tee -a "$LOG_FILE"
}

# Error handler
error_handler() {
    errors=$((errors + 1))
    log "${RED}❌ Error occurred in iteration $iteration${NC}"
    log "${YELLOW}⏳ Waiting ${CHECK_INTERVAL}s before retry...${NC}"
    sleep "$CHECK_INTERVAL"
}

# Graceful shutdown
cleanup() {
    log "${YELLOW}🛑 Shutting down alou-cli update loop...${NC}"
    log "${GREEN}📊 Final Stats:${NC}"
    log "   - Total iterations: $iteration"
    log "   - Updates applied: $updates_applied"
    log "   - Errors encountered: $errors"
    exit 0
}

trap cleanup SIGINT SIGTERM

# Header
echo -e "${CYAN}╔═══════════════════════════════════════╗${NC}"
echo -e "${CYAN}║   🔄 Alou-CLI Auto-Update Loop        ║${NC}"
echo -e "${CYAN}╚═══════════════════════════════════════╝${NC}"
echo
log "${GREEN}🚀 Starting alou-cli update loop${NC}"
log "   Check interval: ${CHECK_INTERVAL}s"
log "   Max iterations: ${MAX_ITERATIONS} (${MAX_ITERATIONS:-0:-1:-unlimited})"
log "   Log file: $LOG_FILE"
echo

# Main loop
while true; do
    iteration=$((iteration + 1))
    
    # Check max iterations
    if [ "$MAX_ITERATIONS" -gt 0 ] && [ "$iteration" -gt "$MAX_ITERATIONS" ]; then
        log "${GREEN}✅ Reached maximum iterations ($MAX_ITERATIONS)${NC}"
        break
    fi
    
    log "${CYAN}🔄 Iteration $iteration${NC}"
    
    # Run alou-cli update check
    if [ -d "$ALOU_CLI_PATH" ]; then
        cd "$ALOU_CLI_PATH"
        
        # Check for updates or run analysis
        if cargo run -- check 2>&1 | tee -a "$LOG_FILE"; then
            log "${GREEN}✅ Check completed successfully${NC}"
            
            # Apply updates if available
            if cargo run -- apply 2>&1 | tee -a "$LOG_FILE"; then
                updates_applied=$((updates_applied + 1))
                log "${GREEN}✅ Updates applied successfully${NC}"
            else
                log "${YELLOW}⚠️  No updates to apply or apply failed${NC}"
            fi
        else
            error_handler
        fi
        
        cd - > /dev/null
    else
        log "${RED}❌ Alou-cli not found at: $ALOU_CLI_PATH${NC}"
        error_handler
    fi
    
    # Wait for next check
    log "${YELLOW}⏳ Next check in ${CHECK_INTERVAL}s...${NC}"
    sleep "$CHECK_INTERVAL" &
    wait $!
done

log "${GREEN}✅ Update loop completed${NC}"
