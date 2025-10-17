#!/bin/bash
# Color output utilities

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Logging functions
log_error() {
    echo -e "${RED}[ERROR]${NC} $*" >&2
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*"
}

log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_check() {
    local status="$1"
    shift
    if [ "$status" = "ok" ]; then
        echo -e "${GREEN}[✓]${NC} $*"
    else
        echo -e "${RED}[✗]${NC} $*"
    fi
}

log_header() {
    echo ""
    echo -e "${BOLD}${CYAN}========================================${NC}"
    echo -e "${BOLD}${CYAN}  $*${NC}"
    echo -e "${BOLD}${CYAN}========================================${NC}"
    echo ""
}

log_section() {
    echo ""
    echo -e "${BOLD}>>> $*${NC}"
    echo ""
}

log_subsection() {
    echo -e "${CYAN}  -- $*${NC}"
}

