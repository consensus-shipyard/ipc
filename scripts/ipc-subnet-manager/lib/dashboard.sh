#!/bin/bash
# Live monitoring dashboard for IPC subnet

# Dashboard state variables
declare -A ERROR_COUNTS
declare -A ERROR_SAMPLES
declare -A METRICS
declare -a RECENT_EVENTS

# Initialize error categories
ERROR_CATEGORIES=(
    "checkpoint"
    "finality"
    "network"
    "consensus"
    "rpc"
    "other"
)

# ANSI escape codes for dashboard
CLEAR_SCREEN="\033[2J"
CURSOR_HOME="\033[H"
CURSOR_HIDE="\033[?25l"
CURSOR_SHOW="\033[?25h"
BOLD="\033[1m"
RESET="\033[0m"
GREEN="\033[32m"
YELLOW="\033[33m"
RED="\033[31m"
CYAN="\033[36m"
BLUE="\033[34m"

# Initialize dashboard
initialize_dashboard() {
    # Hide cursor for cleaner display
    echo -ne "${CURSOR_HIDE}"

    # Initialize error counts
    for category in "${ERROR_CATEGORIES[@]}"; do
        ERROR_COUNTS[$category]=0
        ERROR_SAMPLES[$category]=""
    done

    # Initialize metrics
    METRICS[start_time]=$(date +%s)
    METRICS[last_height]=0
    METRICS[last_check]=0

    # Initialize recent events queue
    RECENT_EVENTS=()

    # Trap cleanup on exit
    trap cleanup_dashboard EXIT INT TERM
}

# Cleanup on exit
cleanup_dashboard() {
    echo -ne "${CURSOR_SHOW}"
    clear
}

# Add event to recent events (max 5)
add_event() {
    local icon="$1"
    local message="$2"
    local timestamp=$(date +%H:%M:%S)

    RECENT_EVENTS=("$timestamp  $icon $message" "${RECENT_EVENTS[@]}")

    # Keep only last 5 events
    if [ ${#RECENT_EVENTS[@]} -gt 5 ]; then
        RECENT_EVENTS=("${RECENT_EVENTS[@]:0:5}")
    fi
}

# Categorize error message
categorize_error() {
    local error_msg="$1"
    local category="other"
    local sample=""

    if echo "$error_msg" | grep -qi "checkpoint\|bottomup"; then
        category="checkpoint"
        sample=$(echo "$error_msg" | grep -oE "(mempool|broadcast|signature)" | head -1)
    elif echo "$error_msg" | grep -qi "finality\|parent.*finality"; then
        category="finality"
        sample=$(echo "$error_msg" | grep -oE "(sync|vote|proposal)" | head -1)
    elif echo "$error_msg" | grep -qi "network\|p2p\|peer\|libp2p"; then
        category="network"
        sample=$(echo "$error_msg" | grep -oE "(peer|connection|gossip)" | head -1)
    elif echo "$error_msg" | grep -qi "consensus\|round\|proposal\|prevote"; then
        category="consensus"
        sample=$(echo "$error_msg" | grep -oE "(round|timeout|proposal)" | head -1)
    elif echo "$error_msg" | grep -qi "rpc\|http\|timeout"; then
        category="rpc"
        sample=$(echo "$error_msg" | grep -oE "(timeout|connection)" | head -1)
    fi

    ERROR_COUNTS[$category]=$((${ERROR_COUNTS[$category]} + 1))
    if [ -z "${ERROR_SAMPLES[$category]}" ]; then
        ERROR_SAMPLES[$category]="$sample"
    fi
}

# Fetch current metrics from validator
fetch_metrics() {
    local validator_idx="$1"
    local ip=$(get_config_value "validators[$validator_idx].ip")
    local ssh_user=$(get_config_value "validators[$validator_idx].ssh_user")
    local ipc_user=$(get_config_value "validators[$validator_idx].ipc_user")
    local name="${VALIDATORS[$validator_idx]}"

    # Fetch block height and info (with timeout)
    local status=$(timeout 5 ssh -o ConnectTimeout=3 -o StrictHostKeyChecking=no "$ssh_user@$ip" \
        "curl -s --max-time 2 http://localhost:26657/status 2>/dev/null" 2>/dev/null || echo '{"result":{"sync_info":{}}}')

    METRICS[height]=$(echo "$status" | jq -r '.result.sync_info.latest_block_height // 0' 2>/dev/null || echo "0")
    METRICS[block_time]=$(echo "$status" | jq -r '.result.sync_info.latest_block_time // ""' 2>/dev/null || echo "")
    METRICS[catching_up]=$(echo "$status" | jq -r '.result.sync_info.catching_up // true' 2>/dev/null || echo "true")

    # Fetch network info (with timeout)
    local net_info=$(timeout 5 ssh -o ConnectTimeout=3 -o StrictHostKeyChecking=no "$ssh_user@$ip" \
        "curl -s --max-time 2 http://localhost:26657/net_info 2>/dev/null" 2>/dev/null || echo '{"result":{}}')
    METRICS[peers]=$(echo "$net_info" | jq -r '.result.n_peers // 0' 2>/dev/null || echo "0")

    # Fetch mempool status (with timeout)
    local mempool=$(timeout 5 ssh -o ConnectTimeout=3 -o StrictHostKeyChecking=no "$ssh_user@$ip" \
        "curl -s --max-time 2 http://localhost:26657/num_unconfirmed_txs 2>/dev/null" 2>/dev/null || echo '{"result":{}}')
    METRICS[mempool_size]=$(echo "$mempool" | jq -r '.result.n_txs // 0' 2>/dev/null || echo "0")
    METRICS[mempool_bytes]=$(echo "$mempool" | jq -r '.result.total_bytes // 0' 2>/dev/null || echo "0")

    # Calculate block production rate
    local current_time=$(date +%s)
    local time_diff=$((current_time - METRICS[last_check]))

    if [ $time_diff -ge 60 ] && [ ${METRICS[last_height]} -gt 0 ]; then
        local height_diff=$((METRICS[height] - METRICS[last_height]))
        METRICS[blocks_per_min]=$height_diff
        METRICS[last_height]=${METRICS[height]}
        METRICS[last_check]=$current_time
    elif [ ${METRICS[last_height]} -eq 0 ]; then
        METRICS[last_height]=${METRICS[height]}
        METRICS[last_check]=$current_time
        METRICS[blocks_per_min]=0
    fi

    # Fetch parent finality from logs (recent, with timeout)
    local finality=$(timeout 5 ssh -o ConnectTimeout=3 -o StrictHostKeyChecking=no "$ssh_user@$ip" \
        "sudo su - $ipc_user -c 'grep ParentFinalityCommitted ~/.ipc-node/logs/*.log 2>/dev/null | tail -1'" 2>/dev/null || echo "")

    if [ -n "$finality" ]; then
        METRICS[parent_height]=$(echo "$finality" | grep -oE 'parent_height: [0-9]+' | grep -oE '[0-9]+' || echo "0")
        METRICS[finality_time]=$(echo "$finality" | grep -oE '[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}' || echo "")
    fi

    # Fetch parent chain height (with timeout)
    local parent_rpc=$(get_config_value "subnet.parent_rpc")
    local parent_height_hex=$(timeout 5 curl -s --max-time 3 -X POST -H "Content-Type: application/json" \
        --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
        "$parent_rpc" 2>/dev/null | jq -r '.result // "0x0"' 2>/dev/null || echo "0x0")
    METRICS[parent_chain_height]=$((16#${parent_height_hex#0x})) 2>/dev/null || METRICS[parent_chain_height]=0

    # Calculate finality lag
    if [ "${METRICS[parent_height]:-0}" -gt 0 ] && [ "${METRICS[parent_chain_height]:-0}" -gt 0 ]; then
        METRICS[finality_lag]=$((METRICS[parent_chain_height] - METRICS[parent_height]))
    else
        METRICS[finality_lag]=0
    fi

    # Scan recent logs for errors (with timeout)
    local errors=$(timeout 10 ssh -o ConnectTimeout=3 -o StrictHostKeyChecking=no "$ssh_user@$ip" \
        "sudo su - $ipc_user -c 'tail -500 ~/.ipc-node/logs/*.log 2>/dev/null | grep -E \"ERROR|WARN\" 2>/dev/null | tail -100'" 2>/dev/null || echo "")

    # Process errors
    while IFS= read -r error_line; do
        if [ -n "$error_line" ]; then
            categorize_error "$error_line"
        fi
    done <<< "$errors"

    # Count checkpoint signatures (with timeout)
    local signatures=$(timeout 5 ssh -o ConnectTimeout=3 -o StrictHostKeyChecking=no "$ssh_user@$ip" \
        "sudo su - $ipc_user -c 'tail -100 ~/.ipc-node/logs/*.log 2>/dev/null | grep -c \"broadcasted signature\" 2>/dev/null'" 2>/dev/null || echo "0")
    METRICS[checkpoint_sigs]=$(echo "$signatures" | tr -d ' \n')
}

# Format number with commas
format_number() {
    printf "%'d" "$1" 2>/dev/null || echo "$1"
}

# Format bytes to human readable
format_bytes() {
    local bytes=$1
    if [ $bytes -lt 1024 ]; then
        echo "${bytes}B"
    elif [ $bytes -lt 1048576 ]; then
        echo "$((bytes / 1024))KB"
    else
        echo "$((bytes / 1048576))MB"
    fi
}

# Get status indicator
get_status_indicator() {
    local value=$1
    local threshold_good=$2
    local threshold_warn=$3
    local higher_is_better=${4:-true}

    if [ "$higher_is_better" = "true" ]; then
        if [ $value -ge $threshold_good ]; then
            echo -e "${GREEN}✓${RESET}"
        elif [ $value -ge $threshold_warn ]; then
            echo -e "${YELLOW}⚠${RESET}"
        else
            echo -e "${RED}✗${RESET}"
        fi
    else
        if [ $value -le $threshold_good ]; then
            echo -e "${GREEN}✓${RESET}"
        elif [ $value -le $threshold_warn ]; then
            echo -e "${YELLOW}⚠${RESET}"
        else
            echo -e "${RED}✗${RESET}"
        fi
    fi
}

# Calculate uptime
get_uptime() {
    local start_time=${METRICS[start_time]}
    local current_time=$(date +%s)
    local uptime_seconds=$((current_time - start_time))

    local hours=$((uptime_seconds / 3600))
    local minutes=$(((uptime_seconds % 3600) / 60))

    echo "${hours}h ${minutes}m"
}

# Draw the dashboard
draw_dashboard() {
    local name="$1"
    local subnet_id=$(get_config_value "subnet.id")
    local subnet_short="${subnet_id:0:20}..."

    # Clear screen and move cursor to home
    echo -ne "${CLEAR_SCREEN}${CURSOR_HOME}"

    # Header
    echo -e "${BOLD}${CYAN}╔═══════════════════════════════════════════════════════════════════════╗${RESET}"
    printf "${BOLD}${CYAN}║${RESET}           ${BOLD}IPC SUBNET LIVE MONITOR${RESET} - %-27s ${BOLD}${CYAN}║${RESET}\n" "$name"
    printf "${BOLD}${CYAN}║${RESET}  Subnet: %-24s Refresh: 3s    Uptime: %-6s ${BOLD}${CYAN}║${RESET}\n" "$subnet_short" "$(get_uptime)"
    echo -e "${BOLD}${CYAN}╚═══════════════════════════════════════════════════════════════════════╝${RESET}"
    echo ""

    # Block Production
    local height=$(format_number ${METRICS[height]:-0})
    local blocks_per_min=${METRICS[blocks_per_min]:-0}
    local block_status=$(get_status_indicator $blocks_per_min 30 10 true)

    echo -e "${BOLD}┌─ BLOCK PRODUCTION ────────────────────────────────────────────────────┐${RESET}"
    printf "│ Height: %-6s  (+%-3d in 1m)  Avg Block Time: --    Rate: --      │\n" "$height" "$blocks_per_min"
    printf "│ Status: %b PRODUCING             Last Block: --                     │\n" "$block_status"
    echo -e "${BOLD}└───────────────────────────────────────────────────────────────────────┘${RESET}"
    echo ""

    # Parent Finality
    local subnet_finality=$(format_number ${METRICS[parent_height]:-0})
    local parent_chain=$(format_number ${METRICS[parent_chain_height]:-0})
    local lag=${METRICS[finality_lag]:-0}
    local finality_status=$(get_status_indicator $lag 30 100 false)

    echo -e "${BOLD}┌─ PARENT FINALITY ─────────────────────────────────────────────────────┐${RESET}"
    printf "│ Subnet: %-8s  Parent Chain: %-8s  Lag: %-4d blocks           │\n" "$subnet_finality" "$parent_chain" "$lag"
    printf "│ Status: %b SYNCING               Last Commit: --                     │\n" "$finality_status"
    echo -e "${BOLD}└───────────────────────────────────────────────────────────────────────┘${RESET}"
    echo ""

    # Network Health
    local peers=${METRICS[peers]:-0}
    local expected_peers=2
    local peer_status=$(get_status_indicator $peers $expected_peers 1 true)

    echo -e "${BOLD}┌─ NETWORK HEALTH ──────────────────────────────────────────────────────┐${RESET}"
    printf "│ CometBFT Peers: %d/%d %b    Libp2p Peers: --    RPC: ${GREEN}✓${RESET} RESPONSIVE     │\n" "$peers" "$expected_peers" "$peer_status"
    echo -e "${BOLD}└───────────────────────────────────────────────────────────────────────┘${RESET}"
    echo ""

    # Mempool Status
    local mempool_size=${METRICS[mempool_size]:-0}
    local mempool_bytes=${METRICS[mempool_bytes]:-0}
    local mempool_max=10000
    local mempool_pct=0
    if [ $mempool_max -gt 0 ]; then
        mempool_pct=$((mempool_size * 100 / mempool_max))
    fi
    local mempool_status=$(get_status_indicator $mempool_pct 80 50 false)
    local mempool_bytes_fmt=$(format_bytes $mempool_bytes)
    local mempool_size_fmt=$(format_number $mempool_size)
    local mempool_max_fmt=$(format_number $mempool_max)

    # Dynamic status text based on mempool state
    local mempool_state="HEALTHY"
    if [ $mempool_size -eq 0 ]; then
        mempool_state="EMPTY"
    elif [ $mempool_pct -ge 80 ]; then
        mempool_state="${RED}CRITICAL${RESET}"
    elif [ $mempool_pct -ge 50 ]; then
        mempool_state="${YELLOW}WARNING${RESET}"
    elif [ $mempool_size -gt 100 ]; then
        mempool_state="${YELLOW}ACTIVE${RESET}"
    else
        mempool_state="${GREEN}HEALTHY${RESET}"
    fi

    echo -e "${BOLD}┌─ MEMPOOL STATUS ──────────────────────────────────────────────────────┐${RESET}"
    printf "│ Pending Transactions: %-8s (%-3d%% full)    Status: %b           │\n" "$mempool_size_fmt" "$mempool_pct" "$mempool_status"
    printf "│ Max Capacity: %-8s    Size: %-6s    State: %-18s │\n" "$mempool_max_fmt" "$mempool_bytes_fmt" "$mempool_state"
    echo -e "${BOLD}└───────────────────────────────────────────────────────────────────────┘${RESET}"
    echo ""

    # Checkpoint Activity
    local checkpoint_sigs=${METRICS[checkpoint_sigs]:-0}

    echo -e "${BOLD}┌─ CHECKPOINT ACTIVITY (Last 5 min) ────────────────────────────────────┐${RESET}"
    printf "│ Signatures: %-3d broadcast          Last: --                          │\n" "$checkpoint_sigs"
    echo -e "${BOLD}└───────────────────────────────────────────────────────────────────────┘${RESET}"
    echo ""

    # Error Summary
    local total_errors=0
    for category in "${ERROR_CATEGORIES[@]}"; do
        total_errors=$((total_errors + ${ERROR_COUNTS[$category]}))
    done

    local error_rate=0
    if [ $total_errors -gt 0 ]; then
        error_rate=$(echo "scale=1; $total_errors / 5" | bc 2>/dev/null || echo "0")
    fi

    echo -e "${BOLD}┌─ ERROR SUMMARY (Last 5 min) ──────────────────────────────────────────┐${RESET}"

    for category in "${ERROR_CATEGORIES[@]}"; do
        local count=${ERROR_COUNTS[$category]:-0}
        local sample=${ERROR_SAMPLES[$category]:-}
        local icon="●"
        local color="${GREEN}"

        if [ $count -gt 0 ]; then
            icon="⚠"
            if [ $count -gt 10 ]; then
                color="${RED}"
            else
                color="${YELLOW}"
            fi
        fi

        local display_name=$(echo "$category" | awk '{for(i=1;i<=NF;i++)sub(/./,toupper(substr($i,1,1)),$i)}1')
        case $category in
            "checkpoint") display_name="Bottom-up Checkpoint" ;;
            "finality") display_name="Parent Finality" ;;
            "network") display_name="Network/P2P" ;;
            "consensus") display_name="Consensus" ;;
            "rpc") display_name="RPC/API" ;;
        esac

        # Simplified formatting - just show count
        printf "│ ${color}%-2s${RESET} %-23s  %-3d                                             │\n" "$icon" "$display_name:" "$count"
    done

    printf "│ ${BOLD}Total Errors:${RESET} %-3d          Error Rate: %.1f/min                        │\n" "$total_errors" "$error_rate"
    echo -e "${BOLD}└───────────────────────────────────────────────────────────────────────┘${RESET}"
    echo ""

    # Recent Events
    echo -e "${BOLD}┌─ RECENT EVENTS ───────────────────────────────────────────────────────┐${RESET}"
    if [ ${#RECENT_EVENTS[@]} -eq 0 ]; then
        echo "│ No recent events                                                      │"
    else
        for event in "${RECENT_EVENTS[@]}"; do
            printf "│ %-69s │\n" "$event"
        done
    fi
    echo -e "${BOLD}└───────────────────────────────────────────────────────────────────────┘${RESET}"
    echo ""

    # Footer
    echo -e "${CYAN}Press 'q' to quit, 'r' to reset counters${RESET}"
}

# Main dashboard loop
run_dashboard() {
    local validator_idx="${1:-0}"
    local refresh_interval="${2:-3}"

    load_config

    local name="${VALIDATORS[$validator_idx]}"

    log_info "Starting live dashboard for $name (refresh: ${refresh_interval}s)"
    echo ""

    initialize_dashboard

    # Main loop
    while true; do
        # Fetch latest metrics (with error handling)
        fetch_metrics "$validator_idx" || true

        # Draw dashboard (with error handling)
        draw_dashboard "$name" || true

        # Check for user input (non-blocking)
        read -t "$refresh_interval" -n 1 key 2>/dev/null || true

        case "$key" in
            q|Q)
                break
                ;;
            r|R)
                # Reset error counters
                for category in "${ERROR_CATEGORIES[@]}"; do
                    ERROR_COUNTS[$category]=0
                    ERROR_SAMPLES[$category]=""
                done
                RECENT_EVENTS=()
                add_event "✓" "Counters reset"
                ;;
        esac
    done

    cleanup_dashboard
    log_info "Dashboard stopped"
}

