#!/bin/bash
# Execution abstraction layer for local and remote execution

# Execute command on a validator (local or remote)
# Usage: exec_on_host <validator_idx> <command>
exec_on_host() {
    local validator_idx="$1"
    shift
    local cmd="$*"

    if is_local_mode; then
        local_exec "$validator_idx" "$cmd"
    else
        local ip=$(get_config_value "validators[$validator_idx].ip")
        local ssh_user=$(get_config_value "validators[$validator_idx].ssh_user")
        local ipc_user=$(get_config_value "validators[$validator_idx].ipc_user")
        ssh_exec "$ip" "$ssh_user" "$ipc_user" "$cmd"
    fi
}

# Execute command directly on validator (remote mode wrapper)
# Usage: exec_on_host_direct <validator_idx> <command>
exec_on_host_direct() {
    local validator_idx="$1"
    shift
    local cmd="$*"

    if is_local_mode; then
        local_exec "$validator_idx" "$cmd"
    else
        local ip=$(get_config_value "validators[$validator_idx].ip")
        local ssh_user=$(get_config_value "validators[$validator_idx].ssh_user")
        local ipc_user=$(get_config_value "validators[$validator_idx].ipc_user")
        ssh_exec_direct "$ip" "$ssh_user" "$ipc_user" "$cmd"
    fi
}

# Execute command locally
# Usage: local_exec <validator_idx> <command>
local_exec() {
    local validator_idx="$1"
    shift
    local cmd="$*"

    if [ "$DRY_RUN" = true ]; then
        log_info "[DRY-RUN] Would execute locally: $cmd"
        return 0
    fi

    # Execute command in a subshell with proper environment
    eval "$cmd" 2>&1
}

# Test connectivity to validator
# Usage: test_connectivity <validator_idx>
test_connectivity() {
    local validator_idx="$1"

    if is_local_mode; then
        # Local mode: just check if we can execute commands
        return 0
    else
        local ip=$(get_config_value "validators[$validator_idx].ip")
        local ssh_user=$(get_config_value "validators[$validator_idx].ssh_user")
        test_ssh "$ip" "$ssh_user"
    fi
}

# Copy file to validator
# Usage: copy_to_host <validator_idx> <local_file> <remote_path>
copy_to_host() {
    local validator_idx="$1"
    local local_file="$2"
    local remote_path="$3"

    if is_local_mode; then
        if [ "$DRY_RUN" = true ]; then
            log_info "[DRY-RUN] Would copy $local_file to $remote_path"
            return 0
        fi

        # Expand tilde in remote path
        remote_path="${remote_path/#\~/$HOME}"

        # Create directory if it doesn't exist
        local dir=$(dirname "$remote_path")
        mkdir -p "$dir"

        # Copy file
        cp "$local_file" "$remote_path"
    else
        local ip=$(get_config_value "validators[$validator_idx].ip")
        local ssh_user=$(get_config_value "validators[$validator_idx].ssh_user")
        local ipc_user=$(get_config_value "validators[$validator_idx].ipc_user")
        scp_to_host "$ip" "$ssh_user" "$ipc_user" "$local_file" "$remote_path"
    fi
}

# Copy file from validator
# Usage: copy_from_host <validator_idx> <remote_path> <local_file>
copy_from_host() {
    local validator_idx="$1"
    local remote_path="$2"
    local local_file="$3"

    if is_local_mode; then
        if [ "$DRY_RUN" = true ]; then
            log_info "[DRY-RUN] Would copy $remote_path to $local_file"
            return 0
        fi

        # Expand tilde in remote path
        remote_path="${remote_path/#\~/$HOME}"

        # Copy file
        cp "$remote_path" "$local_file"
    else
        local ip=$(get_config_value "validators[$validator_idx].ip")
        local ssh_user=$(get_config_value "validators[$validator_idx].ssh_user")
        local ipc_user=$(get_config_value "validators[$validator_idx].ipc_user")
        scp_from_host "$ip" "$ssh_user" "$ipc_user" "$remote_path" "$local_file"
    fi
}

# Check if process is running on validator
# Usage: check_process_running <validator_idx> <process_pattern>
check_process_running() {
    local validator_idx="$1"
    local process_pattern="$2"

    if is_local_mode; then
        if [ "$DRY_RUN" = true ]; then
            return 0
        fi
        pgrep -f "$process_pattern" > /dev/null 2>&1
    else
        local ip=$(get_config_value "validators[$validator_idx].ip")
        local ssh_user=$(get_config_value "validators[$validator_idx].ssh_user")
        ssh_check_process "$ip" "$ssh_user" "$process_pattern"
    fi
}

# Kill process on validator
# Usage: kill_process <validator_idx> <process_pattern>
kill_process() {
    local validator_idx="$1"
    local process_pattern="$2"

    if is_local_mode; then
        if [ "$DRY_RUN" = true ]; then
            log_info "[DRY-RUN] Would kill process: $process_pattern"
            return 0
        fi
        pkill -f "$process_pattern" 2>/dev/null || true
    else
        local ip=$(get_config_value "validators[$validator_idx].ip")
        local ssh_user=$(get_config_value "validators[$validator_idx].ssh_user")
        local ipc_user=$(get_config_value "validators[$validator_idx].ipc_user")
        ssh_exec "$ip" "$ssh_user" "$ipc_user" "pkill -f '$process_pattern' || true"
    fi
}

# Get node home directory for a validator
# Usage: get_node_home <validator_idx>
get_node_home() {
    local validator_idx="$1"

    if is_local_mode; then
        # In local mode, each validator gets its own subdirectory
        local node_home_base=$(get_config_value "paths.node_home_base")
        local name="${VALIDATORS[$validator_idx]}"
        echo "${node_home_base}/${name}"
    else
        # In remote mode, use the configured node_home
        get_config_value "paths.node_home"
    fi
}

